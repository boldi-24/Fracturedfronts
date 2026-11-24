use super::Stats;
use super::roster::Profile;

pub(crate) const ARXEN: Profile = Profile {
    name: "Arxen",
    role: "Łucznik · carry",
    solar: false,
    stats: Stats {
        health: 174.,
        ad: 30.,
        ap: 2.,
        armor: 14.,
        magic_resist: 15.,
        attack_speed: 1.31,
        speed: 148.,
        reach: 157.,
        omnivamp: 0.045,
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
    cooldowns: [6., 12., 14., 90.],
    descriptions: [
        "Trzy strzały w wachlarzu.",
        "Oznacza i ujawnia bohatera.",
        "Odskok, iluzja i wzmocniony strzał.",
        "Deszcz strzał silniejszy na oznaczonym celu.",
    ],
};
