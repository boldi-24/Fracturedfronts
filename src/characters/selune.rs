use super::Stats;
use super::roster::Profile;

pub(crate) const SELUNE: Profile = Profile {
    name: "Selune",
    role: "Uzdrowicielka · wsparcie",
    solar: false,
    stats: Stats {
        health: 199.,
        ad: 23.,
        ap: 30.,
        armor: 18.,
        magic_resist: 27.,
        attack_speed: 1.1,
        speed: 145.,
        reach: 136.,
        omnivamp: 0.035,
    },
    passive: ("Łaska Fenrae", "Leczenie lub tarcza uruchamia odnowę i ochronę."),
    abilities: [
        "Księżycowe Ukojenie",
        "Tarcza Zmierzchu",
        "Płomień Odnowy",
        "Odrodzenie Feniksa",
    ],
    cooldowns: [6., 12., 14., 100.],
    descriptions: [
        "Leczy cel i do dwóch rannych sojuszników.",
        "Tarcza i zwiększona odporność magiczna.",
        "Leczące płatki zdejmują efekt kontroli.",
        "Wskrzesza jednostkę albo ratuje ciężko ranny cel.",
    ],
};
