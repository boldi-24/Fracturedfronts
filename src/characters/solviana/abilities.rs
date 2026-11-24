use super::*;

pub(super) fn cast(game: &mut Game, source: usize, slot: u32, point: Point) -> bool {
    match slot {
        0 => q(game, source, point),
        1 => e(game, source, point),
        2 => w(game, source, point),
        3 => r(game, source, point),
        _ => false,
    }
}

/// Promienne Cięcie. Cięcie w linii; żar skraca odnowienie.
fn q(game: &mut Game, source: usize, point: Point) -> bool {
    let origin = game.units[source].position;
    let direction = game.cast_direction(source, point);
    let range = 144.;
    let damage = game.physical(source, 13., 0.79);
    let targets = game.line_targets(source, direction, range, 19.);
    game.fx(
        source,
        Shape::Arc,
        origin,
        origin + direction * range,
        range * 0.67,
        0.41,
    );
    if targets
        .iter()
        .any(|&target| game.stacks(source, target) > 0)
    {
        game.units[source].q = game.units[source].q - 2.;
    }
    for target in targets {
        game.skill_hit(source, target, damage, true);
    }
    true
}

/// Tarcza Świtu. Tarcza, redukcja obrażeń i leczenie z ataków.
fn w(game: &mut Game, source: usize, _point: Point) -> bool {
    game.shield(source, source, 37., 4.);
    game.units[source].status.guard.set(0.25, 4.);
    game.units[source].status.sustain.set(0.15, 4.);
    true
}

/// Skok Ognia. Doskok, cios i spowolnienie celu.
fn e(game: &mut Game, source: usize, point: Point) -> bool {
    let Some(target) = game.enemy_target(source, point, 204., false) else {
        return false;
    };

    game.dash_enemy(source, target);
    let damage = game.physical(source, 13., 0.75);
    game.skill_hit(source, target, damage, true);
    game.weaken(target, (0.40, 2.), 0.);
    true
}

/// Gniew Słońca. Wybuch odrzuca wrogów i wzmacnia obrażenia.
fn r(game: &mut Game, source: usize, _point: Point) -> bool {
    let origin = game.units[source].position;
    let damage = game.physical(source, 33., 0.9);
    game.fx(source, Shape::Burst, origin, origin, 121., 0.78);
    for target in game.enemies(source, origin, 108.) {
        game.skill_hit(source, target, damage, true);
        game.knockback(origin, target, 62.);
    }
    game.units[source].status.power.set(0.30, 6.);
    true
}
