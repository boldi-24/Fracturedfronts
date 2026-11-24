use super::Stats;
use super::roster::Profile;

pub(crate) const SOLIVANA: Profile = Profile {
    name: "Solviana",
    role: "Wojowniczka · front",
    solar: true,
    stats: Stats {
        health: 235.,
        ad: 34.,
        ap: 6.,
        armor: 25.,
        magic_resist: 18.,
        attack_speed: 1.28,
        speed: 145.,
        reach: 49.,
        omnivamp: 0.065,
    },
    passive: ("Żar Słońca", "Trzy ładunki żaru wybuchają i spowalniają."),
    abilities: [
        "Promienne Cięcie",
        "Tarcza Świtu",
        "Skok Ognia",
        "Gniew Słońca",
    ],
    cooldowns: [6., 12., 14., 90.],
    descriptions: [
        "Cięcie w linii; żar skraca odnowienie.",
        "Tarcza, redukcja obrażeń i leczenie z ataków.",
        "Doskok, cios i spowolnienie celu.",
        "Wybuch odrzuca wrogów i wzmacnia obrażenia.",
    ],
};
