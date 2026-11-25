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
    pub(super) slow: Timed,
    pub(super) frenzy: Timed,
    pub(super) longshot: Timed,
    pub(super) hidden: f32,
    pub(super) revealed: f32,
    pub(super) tags: [f32; 2],
}

impl Status {
    pub fn concealed(self) -> bool { self.hidden > 0. && self.revealed <= 0. }
    pub fn restricted(self) -> bool { self.slow.value() > 0. }
    pub fn marked(self) -> bool { self.revealed > 0. || self.tags.iter().any(|&time| time > 0.) }
    pub fn tick(&mut self, dt: f32) {
        for timer in [&mut self.slow, &mut self.frenzy, &mut self.longshot] { timer.tick(dt); }
        for timer in self.tags.iter_mut().chain([&mut self.hidden, &mut self.revealed]) {
            *timer = (*timer - dt).max(0.);
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Tone {
    Eclipse,
    Solar,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Shape {
    Beam,
    Dash,
    Mark,
    Burst,
    Rain,
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
