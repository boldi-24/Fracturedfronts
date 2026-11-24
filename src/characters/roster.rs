use super::Stats;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Hero {
    Taran,
    Nera,
    Aelor,
    Arxen,
    Aurelia,
    Helvara,
    Kaelith,
    Luness,
    Nyxalune,
    Selune,
    Solviana,
    Veyrion,
    Zyraen,
    Elorim,
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

const CARTS: [Profile; 12] = [
    super::aelor::AELOR,
    super::arxen::ARXEN,
    super::aurelia::AURELIA,
    super::helvara::HELVARA,
    super::kaelith::KAELITH,
    super::luness::LUNESS,
    super::nyxalune::NYXALUNE,
    super::selune::SELUNE,
    super::solviana::SOLIVANA,
    super::veyrion::VEYRION,
    super::zyrean::ZYRAEN,
    super::elorim::ELORIM,
];

impl Hero {
    pub(crate) const ALL: [Self; 14] = [
        Self::Taran,
        Self::Nera,
        Self::Aelor,
        Self::Arxen,
        Self::Aurelia,
        Self::Helvara,
        Self::Kaelith,
        Self::Luness,
        Self::Nyxalune,
        Self::Selune,
        Self::Solviana,
        Self::Veyrion,
        Self::Zyraen,
        Self::Elorim,
    ];

    pub(crate) fn profile(self) -> Option<&'static Profile> {
        self.portrait_index().map(|index| &CARTS[index])
    }

    pub(crate) fn portrait_index(self) -> Option<usize> {
        (self as usize).checked_sub(2)
    }

    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Taran => super::taran::NAME,
            Self::Nera => super::nera::NAME,
            _ => self.profile().expect("roster profile").name,
        }
    }

    pub(crate) fn stats(self) -> &'static Stats {
        match self {
            Self::Taran => &super::taran::STATS,
            Self::Nera => &super::nera::STATS,
            _ => &self.profile().expect("roster profile").stats,
        }
    }

    pub(crate) fn solar(self) -> bool {
        self.profile().is_some_and(|card| card.solar)
    }

    pub(crate) fn ordered_by_name() -> Vec<Self> {
        let mut ordered: Vec<Self> = Self::ALL.iter().copied().collect();
        ordered.sort_by_key(|hero| hero.name());
        ordered
    }
}
