use super::Stats;
use super::roster::Profile;

pub(crate) const VEYRION: Profile = Profile {
    name: "Veyrion",
    role: "Tank · inicjator",
    solar: false,
    stats: Stats {
        health: 269.,
        ad: 26.,
        ap: 12.,
        armor: 38.,
        magic_resist: 35.,
        attack_speed: 1.03,
        speed: 131.,
        reach: 45.,
        omnivamp: 0.025,
    },
    passive: ("Nieugięty Strażnik", "Ciosy bohatera budują odporności i awaryjną tarczę."),
    abilities: [
        "Uderzenie Zaćmienia",
        "Niezłomna Tarcza",
        "Łańcuchy Feniksa",
        "Bastion Feniksa",
    ],
    cooldowns: [8., 14., 12., 120.],
    descriptions: [
        "Stożek spowalnia i oznacza cele.",
        "Duża tarcza i zwrot obrażeń na oznaczonych.",
        "Przyciąga wroga; znamię daje ogłuszenie.",
        "Odrzuca wrogów, osłania i odnawia sojuszników.",
    ],
};
