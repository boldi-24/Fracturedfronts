use super::{Stats, roster::Profile};

pub(crate) const ELORIM: Profile = Profile {
    name: "Elorim",
    role: "Mag Świtu · mag obszarowy",
    solar: true,
    stats: Stats {
        health: 181.,
        ad: 21.,
        ap: 34.,
        armor: 12.,
        magic_resist: 18.,
        attack_speed: 1.08,
        speed: 142.,
        reach: 143.,
        omnivamp: 0.045,
    },
    passive: (
        "Rezonans Światła",
        "Trafienie zaklęciem nakłada świetlisty ładunek. Po chwili ładunek wybucha.",
    ),
    abilities: [
        "Iskra Żaru",
        "Krąg Koncentracji",
        "Przemieszczenie Światła",
        "Nowa Zorza",
    ],
    cooldowns: [5., 11., 12., 68.],
    descriptions: [
        "Wystrzeliwuje świetlistą kulę w kierunku kursora.",
        "Tworzy krąg zadający okresowe obrażenia i spowalniający wrogów.",
        "Przeskakuje do przodu i zostawia wybuch w miejscu startu.",
        "Przywołuje filar światła, który po chwili wybucha i ogłusza.",
    ],
};
