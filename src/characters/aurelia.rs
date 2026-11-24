use super::Stats;
use super::roster::Profile;

pub(crate) const AURELIA: Profile = Profile {
    name: "Aurelia",
    role: "Łuczniczka · carry",
    solar: true,
    stats: Stats {
        health: 170.,
        ad: 29.,
        ap: 6.,
        armor: 13.,
        magic_resist: 17.,
        attack_speed: 1.3,
        speed: 150.,
        reach: 153.,
        omnivamp: 0.045,
    },
    passive: ("Światlisty Fokus", "Powtarzane ataki ładują leczący strzał."),
    abilities: [
        "Podwójna Strzała",
        "Skrzydła Prędkości",
        "Solarna Pułapka",
        "Deszcz Świtu",
    ],
    cooldowns: [6., 16., 14., 100.],
    descriptions: [
        "Dwie strzały, druga nakładuje ładunek.",
        "Więcej ruchu i ataków, przebicie pancerza.",
        "Pułapka spowalnia i ujawnia wrogów.",
        "Opóźniony grad strzał ze spowolnieniem.",
    ],
};
