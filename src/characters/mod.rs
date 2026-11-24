pub(crate) mod aelor;
pub(crate) mod elorim;
pub(crate) mod arxen;
pub(crate) mod aurelia;
pub(crate) mod helvara;
pub(crate) mod kaelith;
pub(crate) mod luness;
pub(crate) mod nyxalune;
pub(crate) mod selune;
pub(crate) mod solviana;
pub(crate) mod veyrion;
pub(crate) mod zyrean;
pub(crate) mod nera;
pub(crate) mod taran;
pub(crate) mod roster;

pub(crate) struct Stats {
    pub(crate) health: f32,
    pub(crate) speed: f32,
    pub(crate) reach: f32,
    pub(crate) ad: f32,
    pub(crate) ap: f32,
    pub(crate) armor: f32,
    pub(crate) magic_resist: f32,
    pub(crate) attack_speed: f32,
    pub(crate) omnivamp: f32,
}

pub(crate) struct Dash {
    pub(crate) distance: f32,
    pub(crate) cooldown: f32,
    pub(crate) shield: f32,
}
