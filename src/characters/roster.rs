use super::Stats;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Hero {
    One,
    Two,
}

pub(crate) struct Profile {
    pub name: &'static str,
    pub role: &'static str,
    pub solar: bool,
    pub stats: Stats,
    pub passive: (&'static str, &'static str),
    pub abilities: [&'static str; 4],
    pub cooldowns: [f32; 4],
    pub descriptions: [&'static str; 4],
}
 
impl Hero {
    pub(crate) const ALL: [Self; 2] = [Self::One, Self::Two];

    pub(crate) fn profile(self) -> &'static Profile { match self { Self::One => &super::shared::ONE, Self::Two => &super::shared::TWO } }
    pub(crate) fn name(self) -> &'static str { self.profile().name }
    pub(crate) fn stats(self) -> &'static Stats { &self.profile().stats }
    pub(crate) fn solar(self) -> bool { self.profile().solar }
}
