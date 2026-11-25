//! One authoritative duel per local connection. The native renderer receives read-only
//! snapshots in process. All player orders cross the TCP connection in input order.
use super::{
    geometry::Point,
    simulation::{Game, PLAYER},
};
use crate::characters::roster::Hero;
use std::{
    io::{self, Read, Write},
    net::{Shutdown, TcpListener, TcpStream},
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

const STEP: Duration = Duration::from_nanos(16_666_667);
const PACKET_SIZE: usize = 10;

#[derive(Clone, Copy)]
pub(super) enum Order {
    Cast(u8, Point),
    Move(Point),
    Attack(Point),
    Stop,
}

impl Order {
    // Fixed-size frame: opcode, slot, world X and Y as little-endian floats.
    fn encode(self) -> [u8; PACKET_SIZE] {
        let (op, slot, point) = match self {
            Self::Cast(slot, p) => (1, slot, p),
            Self::Move(p) => (2, 0, p),
            Self::Attack(p) => (3, 0, p),
            Self::Stop => (4, 0, Point::default()),
        };
        let mut bytes = [0; PACKET_SIZE];
        bytes[0] = op;
        bytes[1] = slot;
        bytes[2..6].copy_from_slice(&point.x.to_le_bytes());
        bytes[6..].copy_from_slice(&point.y.to_le_bytes());
        bytes
    }

    fn apply(bytes: &[u8; PACKET_SIZE], game: &mut Game) {
        let point = Point::at(
            f32::from_le_bytes(bytes[2..6].try_into().unwrap()),
            f32::from_le_bytes(bytes[6..].try_into().unwrap()),
        );
        if !point.x.is_finite() || !point.y.is_finite() {
            return;
        }
        // Preserve off-lane aim directions while rejecting nonsensical coordinates.
        if point.x.abs() > 100_000. || point.y.abs() > 100_000. {
            return;
        }
        match (bytes[0], bytes[1]) {
            (1, slot @ 0..=3) => game.cast(PLAYER, u32::from(slot), point),
            (2, 0) => game.command(point),
            (3, 0) => game.attack_unit_near(point),
            (4, 0) => game.stop_player(),
            _ => {}
        }
    }
}

pub(super) struct MatchServer {
    sender: TcpStream,
    latest: Arc<Mutex<Game>>,
    worker: Option<JoinHandle<()>>,
    failed: bool,
}

impl MatchServer {
    pub(super) fn start(hero: Hero) -> io::Result<Self> {
        let listener = TcpListener::bind(("127.0.0.1", 0))?;
        let sender = TcpStream::connect(listener.local_addr()?)?;
        let endpoint = sender.local_addr()?;
        let receiver = loop {
            let (socket, peer) = listener.accept()?;
            if peer == endpoint {
                break socket;
            }
        };
        drop(listener);
        sender.set_nodelay(true)?;
        sender.set_write_timeout(Some(Duration::from_millis(100)))?;
        receiver.set_nonblocking(true)?;
        let game = Game::with_hero(hero);
        let latest = Arc::new(Mutex::new(game.clone()));
        let published = Arc::clone(&latest);
        let worker = thread::Builder::new()
            .name("fenrae-match".into())
            .spawn(move || serve(receiver, game, published))?;
        Ok(Self {
            sender,
            latest,
            worker: Some(worker),
            failed: false,
        })
    }

    pub(super) fn snapshot(&self) -> Game {
        self.latest.lock().unwrap().clone()
    }

    pub(super) fn send(&mut self, order: Order) {
        if !self.failed && self.sender.write_all(&order.encode()).is_err() {
            self.failed = true;
            let _ = self.sender.shutdown(Shutdown::Both);
        }
    }

    pub(super) fn disconnected(&self) -> bool {
        self.failed
            || self
                .worker
                .as_ref()
                .is_some_and(|worker| worker.is_finished())
    }
}

impl Drop for MatchServer {
    fn drop(&mut self) {
        let _ = self.sender.shutdown(Shutdown::Both);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

fn serve(mut socket: TcpStream, mut game: Game, latest: Arc<Mutex<Game>>) {
    let mut frame = [0; PACKET_SIZE];
    let mut used = 0;
    let mut next = Instant::now() + STEP;
    loop {
        // Bound work per tick and retain partial TCP frames between ticks.
        for _ in 0..256 {
            match socket.read(&mut frame[used..]) {
                Ok(0) => return,
                Ok(n) => {
                    used += n;
                    if used == PACKET_SIZE {
                        Order::apply(&frame, &mut game);
                        used = 0;
                    }
                }
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => break,
                Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
                Err(_) => return,
            }
        }
        let now = Instant::now();
        for _ in 0..6 {
            if now < next {
                break;
            }
            game.step(1. / 60.);
            next += STEP;
        }
        if next <= now {
            next = now + STEP;
        }
        latest.lock().unwrap().clone_from(&game);
        thread::sleep(next.saturating_duration_since(Instant::now()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn server_handles_click_once_and_checks_its_own_cooldown() {
        let mut server = MatchServer::start(Hero::Deafult).unwrap();
        let origin = server.snapshot().units[PLAYER].position;
        let cast = Order::Cast(1, origin + Point::at(-200., 0.));
        assert_eq!(server.snapshot().units[PLAYER].cooldown(1), 0.);
        server.send(cast);
        server.send(cast);
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            let game = server.snapshot();
            if game.units[PLAYER].cooldown(1) > 0. {
                assert!(
                    (game.units[PLAYER].position.x
                        - origin.x
                        - 101.)
                        .abs()
                        < 0.1
                );
                break;
            }
            assert!(Instant::now() < deadline);
            thread::sleep(Duration::from_millis(2));
        }
    }
}
