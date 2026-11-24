use super::Stats;
use super::roster::Profile;

pub(crate) const NYXALUNE: Profile = Profile {
    name: "Nyxalune",
    role: "Mag · kontrola obszaru",
    solar: false,
    stats: Stats {
        health: 182.,
        ad: 22.,
        ap: 36.,
        armor: 15.,
        magic_resist: 23.,
        attack_speed: 1.13,
        speed: 146.,
        reach: 144.,
        omnivamp: 0.035,
    },
    passive: ("Księżycowy Znak", "Trzecia umiejętność na celu daje obrażenia magiczne."),
    abilities: [
        "Srebrna Koma",
        "Płaszcz Księżyca",
        "Cień Feniksa",
        "Zaćmienie Odrodzenia",
    ],
    cooldowns: [7., 12., 14., 90.],
    descriptions: [
        "Kometa uderza i spowalnia pierwszego wroga.",
        "Tarcza i trzy sekundy szybszego ruchu.",
        "Feniks przebija linię i obniża odporność magiczną.",
        "Strefa z tarczami, spowolnieniem i wybuchem.",
    ],
};
