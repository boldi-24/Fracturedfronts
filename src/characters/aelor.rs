use super::Stats;
use super::roster::Profile;

pub(crate) const AELOR: Profile = Profile {
    name: "Aelor",
    role: "Wojownik · front",
    solar: false,
    stats: Stats {
        health: 242.,
        ad: 33.,
        ap: 8.,
        armor: 26.,
        magic_resist: 18.,
        attack_speed: 1.26,
        speed: 139.,
        reach: 46.,
        omnivamp: 0.075,
    },
    passive: (
        "Feniksowa Wytrwałość",
        "Obrażenia ładują kolejny atak: bonus i leczenie.",
    ),
    abilities: [
        "Rozpłatanie Półksiężyca",
        "Tarcza Zaćmienia",
        "Skok Feniksa",
        "Gniew Zaćmienia",
    ],
    cooldowns: [7., 14., 12., 90.],
    descriptions: [
        "Cięcie w stożku i spowolnienie.",
        "Tarcza i zwiększony pancerz.",
        "Skok z obszarowym podrzutem.",
        "Osiem sekund wzmocnienia i ognisty ślad.",
    ],
};
