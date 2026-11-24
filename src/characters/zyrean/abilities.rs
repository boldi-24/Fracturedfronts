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

/// Błysk Ostrza. Cięcie w linii; znak skraca odnowienie.
fn q(game: &mut Game, source: usize, point: Point) -> bool {
    let origin = game.units[source].position;
    let direction = game.cast_direction(source, point);
    let range = 118.;
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
        game.units[source].q = game.units[source].q * 0.5;
    }
    for target in targets {
        game.skill_hit(source, target, damage, true);
    }
    true
}

/// Zasłona Światła. Ukrycie, szybszy ruch i mocniejszy atak.
fn w(game: &mut Game, source: usize, point: Point) -> bool {
    let origin = game.units[source].position;
    let direction = game.cast_direction(source, point);
    let status = &mut game.units[source].status;
    status.hidden = 2.5;
    status.empowered = 2.5;
    status.haste.set(0.4, 2.5);
    game.fx(
        source,
        Shape::Dash,
        origin,
        origin + direction * 35.,
        27.,
        0.5,
    );
    true
}

/// Skok Promienia. Doskok do celu; trzy znaki dają ogłuszenie.
fn e(game: &mut Game, source: usize, point: Point) -> bool {
    let Some(target) = game.enemy_target(source, point, 204., false) else {
        return false;
    };
    let marked = game.stacks(source, target) >= 3;
    game.dash_enemy(source, target);
    let damage = game.physical(source, 13., 0.75);
    game.skill_hit(source, target, damage, true);
    if marked {
        game.weaken(target, (0., 0.), 1.);
    }
    true
}

/// Egzekucja Świtu. Seria cięć; zabójstwo skraca odnowienie.
fn r(game: &mut Game, source: usize, point: Point) -> bool {
    let origin = game.units[source].position;
    let Some(target) = game.enemy_target(source, point, 245., false) else {
        return false;
    };
    game.dash_enemy(source, target);
    let position = game.units[target].position;
    game.fx(source, Shape::Phoenix, origin, position, 69., 0.88);
    game.place_zone(
        source,
        position,
        51.,
        0.08,
        0.84,
        0.19,
        ZoneAction::Flurry { target },
    );
    true
}
