use super::Stats;
use super::roster::Profile;

pub(crate) const KAELITH: Profile = Profile {
    name: "Kaelith",
    role: "Asasynka · szybki atak",
    solar: false,
    stats: Stats {
        health: 181.,
        ad: 34.,
        ap: 4.,
        armor: 16.,
        magic_resist: 16.,
        attack_speed: 1.38,
        speed: 157.,
        reach: 44.,
        omnivamp: 0.055,
    },
    passive: ("Znamię Feniksa", "Ataki i umiejętności nakładają wybuchające znamiona."),
    abilities: [
        "Podwójne Ostrza",
        "Krok Cienia",
        "Ostrze Zaćmienia",
        "Zstąpienie Feniksa",
    ],
    cooldowns: [6., 12., 10., 80.],
    descriptions: [
        "Dwa cięcia; drugie korzysta ze znamienia.",
        "Ukrycie, szybszy ruch i atak z zaskoczenia.",
        "Doskok do celu; znamię skraca odnowienie.",
        "Opóźniony skok kończący osłabione cele.",
    ],
};
