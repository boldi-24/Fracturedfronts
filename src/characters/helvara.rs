use super::Stats;
use super::roster::Profile;

pub(crate) const HELVARA: Profile = Profile {
    name: "Helvara",
    role: "Tank · wsparcie",
    solar: true,
    stats: Stats {
        health: 265.,
        ad: 26.,
        ap: 13.,
        armor: 38.,
        magic_resist: 34.,
        attack_speed: 1.04,
        speed: 132.,
        reach: 43.,
        omnivamp: 0.025,
    },
    passive: ("Nieugięta Wola", "Otrzymane ciosy zwiększają odporności."),
    abilities: [
        "Słoneczne Uderzenie",
        "Mur Światła",
        "Przyciągnięcie Blasku",
        "Bastion Świtu",
    ],
    cooldowns: [8., 14., 12., 120.],
    descriptions: [
        "Cięcie tarczą, spowolnienie i ładunek obrony.",
        "Chroni siebie i pobliskich sojuszników.",
        "Łańcuch przyciąga i ogłusza.",
        "Pięć sekund obrony i leczenia w strefie.",
    ],
};
