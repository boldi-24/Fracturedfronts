use super::{Dash, Stats};

pub(crate) const NAME: &str = "Taran z Kuźni";

pub(crate) const STATS: Stats = Stats {
    health: 245.,
    speed: 136.,
    reach: 42.,
    ad: 32.,
    ap: 0.,
    armor: 24.,
    magic_resist: 16.,
    attack_speed: 1. / 0.74,
    omnivamp: 0.08,
};

pub(crate) struct Cone {
    pub(crate) cooldown: f32,
    pub(crate) visual_range: f32,
    pub(crate) reach: f32,
    pub(crate) facing_threshold: f32,
    pub(crate) base_damage: f32,
    pub(crate) ad_ratio: f32,
}

pub(crate) const Q: Cone = Cone {
    cooldown: 5.,
    visual_range: 91.,
    reach: 94.,
    facing_threshold: 0.32,
    base_damage: 20.,
    ad_ratio: 1.,
};

pub(crate) const E: Dash = Dash {
    distance: 78.,
    cooldown: 8.,
    shield: 2.1,
};
