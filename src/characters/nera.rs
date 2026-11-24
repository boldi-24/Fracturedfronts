use super::{Dash, Stats};

pub(crate) const NAME: &str = "Nera z Mgły";

pub(crate) const STATS: Stats = Stats {
    health: 170.,
    speed: 150.,
    reach: 146.,
    ad: 23.,
    ap: 28.,
    armor: 12.,
    magic_resist: 20.,
    attack_speed: 1. / 0.78,
    omnivamp: 0.05,
};

pub(crate) struct Projectile {
    pub(crate) cooldown: f32,
    pub(crate) spawn_offset: f32,
    pub(crate) speed: f32,
    pub(crate) base_damage: f32,
    pub(crate) ap_ratio: f32,
    pub(crate) lifetime: f32,
}

pub(crate) const Q: Projectile = Projectile {
    cooldown: 4.,
    spawn_offset: 23.,
    speed: 440.,
    base_damage: 13.,
    ap_ratio: 1.,
    lifetime: 0.78,
};

pub(crate) const E: Dash = Dash {
    distance: 137.,
    cooldown: 7.,
    shield: 0.,
};
