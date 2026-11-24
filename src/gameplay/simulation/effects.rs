use super::{Point, UNIT_COUNT};

#[derive(Clone, Copy, Default)]
pub(super) struct Timed {
    amount: f32,
    time: f32,
}

impl Timed {
    pub(super) fn set(&mut self, amount: f32, time: f32) {
        if self.time <= 0. || amount >= self.amount {
            *self = Self { amount, time };
        }
    }

    pub(super) fn value(self) -> f32 {
        if self.time > 0. { self.amount } else { 0. }
    }

    fn tick(&mut self, delta: f32) {
        self.time = (self.time - delta).max(0.);
    }
}

#[derive(Clone, Copy, Default)]
pub(super) struct Trace {
    pub count: u8,
    time: f32,
    target: Option<usize>,
}

impl Trace {
    pub fn add(&mut self, target: usize, duration: f32, max: u8) -> u8 {
        if self.target != Some(target) || self.time <= 0. {
            self.count = 0;
        }
        self.target = Some(target);
        self.count = (self.count + 1).min(max);
        self.time = duration;
        self.count
    }

    pub fn count(self, target: usize) -> u8 {
        if self.time > 0. && self.target == Some(target) {
            self.count
        } else {
            0
        }
    }

    pub fn tick(&mut self, dt: f32) {
        self.time = (self.time - dt).max(0.);
        if self.time == 0. {
            self.count = 0;
        }
    }
}

#[derive(Clone, Copy, Default)]
pub(crate) struct Status {
    pub(super) armor: Timed,
    pub(super) magic: Timed,
    pub(super) weaken_magic: Timed,
    pub(super) guard: Timed,
    pub(super) slow: Timed,
    pub(super) haste: Timed,
    pub(super) frenzy: Timed,
    pub(super) penetration: Timed,
    pub(super) power: Timed,
    pub(super) sustain: Timed,
    pub(super) longshot: Timed,
    pub(super) control_resist: Timed,
    pub(super) renewal: Timed,
    pub(super) barrier: f32,
    pub(super) barrier_time: f32,
    pub(super) burst: f32,
    pub(super) reflect: f32,
    pub(super) hidden: f32,
    pub(super) revealed: f32,
    pub(super) stun: f32,
    pub(super) empowered: f32,
    pub(super) invulnerable: f32,
    pub(super) light: f32,
    pub(super) fallen: f32,
    pub(super) stacks: u8,
    pub(super) aelor_fire: f32,
    pub(super) trail_emit: f32,
    pub(super) resonance: [f32; 2],
    pub(super) tags: [f32; 2],
    pub(super) marks: [Trace; 2],
}

impl Status {
    pub fn concealed(self) -> bool {
        self.hidden > 0. && self.revealed <= 0.
    }
    pub fn protected(self) -> bool {
        self.barrier > 0. || self.guard.value() > 0. || self.invulnerable > 0.
    }
    pub fn restricted(self) -> bool {
        self.stun > 0. || self.slow.value() > 0.
    }
    pub fn marked(self) -> bool {
        self.revealed > 0.
            || self.light > 0.
            || self.stacks > 0
            || self.tags.iter().any(|time| *time > 0.)
            || self.marks.iter().any(|mark| mark.count > 0)
    }
    pub fn tick(&mut self, dt: f32) {
        for timer in [
            &mut self.armor,
            &mut self.magic,
            &mut self.weaken_magic,
            &mut self.guard,
            &mut self.slow,
            &mut self.haste,
            &mut self.frenzy,
            &mut self.penetration,
            &mut self.power,
            &mut self.sustain,
            &mut self.longshot,
            &mut self.control_resist,
            &mut self.renewal,
        ] {
            timer.tick(dt);
        }
        for timer in [
            &mut self.barrier_time,
            &mut self.burst,
            &mut self.reflect,
            &mut self.hidden,
            &mut self.revealed,
            &mut self.stun,
            &mut self.empowered,
            &mut self.invulnerable,
            &mut self.light,
            &mut self.fallen,
            &mut self.aelor_fire,
            &mut self.trail_emit,
        ] {
            *timer = (*timer - dt).max(0.);
        }
        for timer in &mut self.tags {
            *timer = (*timer - dt).max(0.);
        }
        for mark in &mut self.marks {
            mark.tick(dt);
        }
        if self.barrier_time <= 0. {
            self.barrier = 0.;
        }
    }

    pub(super) fn absorb(&mut self, damage: f32) -> f32 {
        let blocked = self.barrier.min(damage);
        self.barrier -= blocked;
        blocked
    }

    pub(super) fn clear_one(&mut self) {
        if self.stun > 0. {
            self.stun = 0.;
        } else {
            self.slow = Timed::default();
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Tone {
    Eclipse,
    Solar,
    Heal,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Shape {
    Arc,
    Beam,
    Dash,
    Shield,
    Mark,
    Burst,
    Rain,
    Trap,
    Phoenix,
}

#[derive(Clone)]
pub(crate) struct Effect {
    pub from: Point,
    pub to: Point,
    pub radius: f32,
    pub shape: Shape,
    pub tone: Tone,
    pub t: f32,
    pub duration: f32,
}

const _: () = assert!(UNIT_COUNT <= u64::BITS as usize);
