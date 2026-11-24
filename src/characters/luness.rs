use super::Stats;
use super::roster::Profile;

pub(crate) const LUNESS: Profile = Profile {
    name: "Luness",
    role: "Uzdrowiciel · wsparcie",
    solar: true,
    stats: Stats {
        health: 193.,
        ad: 23.,
        ap: 32.,
        armor: 17.,
        magic_resist: 25.,
        attack_speed: 1.11,
        speed: 145.,
        reach: 137.,
        omnivamp: 0.03,
    },
    passive: ("Promień Życia", "Uzdrawiani sojusznicy odzyskują zdrowie przy ataku."),
    abilities: [
        "Uzdrawiający Strumień",
        "Tarcza Świtu",
        "Krąg Ożywienia",
        "Odrodzenie Światła",
    ],
    cooldowns: [6., 12., 14., 120.],
    descriptions: [
        "Leczenie odbija się do dwóch sojuszników.",
        "Tarcza kończy się leczącym wybuchem.",
        "Strefa oczyszcza, leczy i przyspiesza.",
        "Obszarowy zastrzyk zdrowia i odnowa.",
    ],
};
