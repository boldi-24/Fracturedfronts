use super::Stats;
use super::roster::Profile;

pub(crate) const ZYRAEN: Profile = Profile {
    name: "Zyraen",
    role: "Asasyn · egzekucja",
    solar: true,
    stats: Stats {
        health: 185.,
        ad: 35.,
        ap: 5.,
        armor: 16.,
        magic_resist: 17.,
        attack_speed: 1.32,
        speed: 155.,
        reach: 46.,
        omnivamp: 0.055,
    },
    passive: ("Znak Świtu", "Umiejętności nakładają znaki, atak zużywa je i leczy."),
    abilities: [
        "Błysk Ostrza",
        "Zasłona Światła",
        "Skok Promienia",
        "Egzekucja Świtu",
    ],
    cooldowns: [6., 14., 12., 80.],
    descriptions: [
        "Cięcie w linii; znak skraca odnowienie.",
        "Ukrycie, szybszy ruch i mocniejszy atak.",
        "Doskok do celu; trzy znaki dają ogłuszenie.",
        "Seria cięć; zabójstwo skraca odnowienie.",
    ],
};
