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

/// Uderzenie Zaćmienia. Stożek spowalnia i oznacza cele.
fn q(game: &mut Game, source: usize, point: Point) -> bool {
    let direction = game.cast_direction(source, point);
    let damage = game.physical(source, 13., 0.65);
    for target in game.fan(source, direction, 105., 0.32) {
        game.skill_hit(source, target, damage, false);
        game.weaken(target, (0.4, 2.5), 0.);
        game.mark_tag(source, target, 5.);
        let position = game.units[target].position;
        game.fx(source, Shape::Mark, position, position, 25., 0.65);
    }
    true
}

/// Niezłomna Tarcza. Duża tarcza i zwrot obrażeń na oznaczonych.
fn w(game: &mut Game, source: usize, _point: Point) -> bool {
    game.shield(source, source, Hero::Veyrion.stats().health * 0.22, 4.);
    game.units[source].status.guard.set(0.25, 4.);
    game.units[source].status.reflect = 4.;
    true
}

/// Łańcuchy Feniksa. Przyciąga wroga; znamię daje ogłuszenie.
fn e(game: &mut Game, source: usize, point: Point) -> bool {
    let origin = game.units[source].position;
    let direction = game.cast_direction(source, point);
    if let Some(target) = game
        .line_targets(source, direction, 229., 13.)
        .first()
        .copied()
    {
        let stun = if game.tagged(source, target) { 1. } else { 0. };
        game.pull(source, target, stun);
        let damage = game.physical(source, 15., 0.64);
        game.skill_hit(source, target, damage, false);
    } else {
        game.fx(
            source,
            Shape::Beam,
            origin,
            origin + direction * 229.,
            11.,
            0.42,
        );
    }
    true
}

/// Bastion Feniksa. Odrzuca wrogów, osłania i odnawia sojuszników.
fn r(game: &mut Game, source: usize, _point: Point) -> bool {
    let origin = game.units[source].position;
    for target in game.enemies(source, origin, 135.) {
        game.knockback(origin, target, 69.);
    }
    for ally in game.allies(source, origin, 145.) {
        game.shield(source, ally, 29., 1.);
    }
    game.place_zone(
        source,
        origin,
        145.,
        0.12,
        5.12,
        0.55,
        ZoneAction::Renewal {
            healing: 0.014,
            guard: 0.30,
            haste: 0.,
            barrier: 24.,
            cleanse: false,
        },
    );
    game.fx(source, Shape::Phoenix, origin, origin, 149., 0.85);
    true
}
