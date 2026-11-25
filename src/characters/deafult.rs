use super::Stats;
use super::roster::Profile;

pub(crate) const PROFILE: Profile = Profile {
    name: "Deafult",
    role: "deafult · terrorist",
    solar: false,
    stats: Stats {
        health: 174.,
        ad: 100.,
        armor: 14.,
        speed: 148.,
        reach: 43.,
    },
    passive: (
        "Precyzja Fenrae",
        "Seria strzałów do tego samego celu wzmacnia ataki.",
    ),
    abilities: [
        "Księżycowe Strzały",
        "Znak Łowcy",
        "Cień Przewietrzenia",
        "Deszcz Zaćmienia",
    ],
    cooldowns: [1., 12., 14., 90.],
    descriptions: [
        "Trzy strzały w wachlarzu.",
        "Oznacza i ujawnia bohatera.",
        "Odskok, iluzja i wzmocniony strzał.",
        "Deszcz strzał silniejszy na oznaczonym celu.",
    ],
};
