use std::ops::{Add, Mul, Sub};

pub(crate) const WORLD_WIDTH: f32 = 3600.;
pub(crate) const WORLD_HEIGHT: f32 = 560.;
pub(crate) const LANE_CENTER: f32 = WORLD_HEIGHT * 0.5;
pub(crate) const LANE_MIN_Y: f32 = 90.;
pub(crate) const LANE_MAX_Y: f32 = 470.;

#[derive(Clone, Copy, Default)]
pub(crate) struct Point {
    pub(crate) x: f32,
    pub(crate) y: f32,
}

impl Point {
    pub(crate) const fn at(x: f32, y: f32) -> Self {
        Self { x, y }
    }
    pub(super) fn length(self) -> f32 {
        self.x.hypot(self.y)
    }
    pub(super) fn distance(self, other: Self) -> f32 {
        (self - other).length()
    }
    pub(super) fn direction(self) -> Self {
        self * self.length().max(0.001).recip()
    }
    pub(super) fn lane(self) -> Self {
        Self::at(
            self.x.clamp(28., WORLD_WIDTH - 28.),
            self.y.clamp(LANE_MIN_Y, LANE_MAX_Y),
        )
    }
}

impl Add for Point {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self::at(self.x + other.x, self.y + other.y)
    }
}

impl Sub for Point {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self::at(self.x - other.x, self.y - other.y)
    }
}

impl Mul<f32> for Point {
    type Output = Self;
    fn mul(self, factor: f32) -> Self {
        Self::at(self.x * factor, self.y * factor)
    }
}
