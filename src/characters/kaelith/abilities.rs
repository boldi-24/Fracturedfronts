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

/// Podwójne Ostrza. Dwa cięcia; drugie korzysta ze znamienia.
fn q(game: &mut Game, source: usize, point: Point) -> bool {
    let origin = game.units[source].position;
    let direction = game.cast_direction(source, point);
    let targets = game.fan(source, direction, 83., 0.48);
    let hit = game.physical(source, 9., 0.48);
    for target in targets {
        let bonus = if game.stacks(source, target) > 0 {
            1.35
        } else {
            1.
        };
        game.skill_hit(source, target, hit, true);
        game.skill_hit(source, target, Game::scale_damage(hit, bonus), true);
    }
    game.fx(
        source,
        Shape::Arc,
        origin + direction * 8.,
        origin + direction * 59.,
        66.,
        0.65,
    );
    true
}

/// Krok Cienia. Ukrycie, szybszy ruch i atak z zaskoczenia.
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

/// Ostrze Zaćmienia. Doskok do celu; znamię skraca odnowienie.
fn e(game: &mut Game, source: usize, point: Point) -> bool {
    let Some(target) = game.enemy_target(source, point, 185., false) else {
        return false;
    };
    if game.stacks(source, target) > 0 {
        game.units[source].e *= 0.50;
    }
    game.dash_enemy(source, target);
    let damage = game.physical(source, 13., 0.80);
    game.skill_hit(source, target, damage, true);
    game.weaken(target, (0.30, 1.5), 0.);
    true
}

/// Zstąpienie Feniksa. Opóźniony skok kończący osłabione cele.
fn r(game: &mut Game, source: usize, point: Point) -> bool {
    let at = game.aim_within(source, point, 305.);
    game.units[source].status.hidden = 0.75;
    game.place_zone(source, at, 100., 0.75, 0.82, 0., ZoneAction::Dive);
    true
}
