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

/// Rozpłatanie Półksiężyca. Cięcie w stożku i spowolnienie.
fn q(game: &mut Game, source: usize, point: Point) -> bool {
    let direction = game.cast_direction(source, point);
    let damage = game.physical(source, 14., 0.83);
    for target in game.fan(source, direction, 103., 0.30) {
        game.skill_hit(source, target, damage, false);
        game.weaken(target, (0.30, 2.), 0.);
    }
    true
}

/// Tarcza Zaćmienia. Tarcza i zwiększony pancerz.
fn w(game: &mut Game, source: usize, _point: Point) -> bool {
    game.shield(source, source, 43., 4.);
    let status = &mut game.units[source].status;
    status.armor.set(0.20, 4.);
    status.guard.set(0.14, 4.);
    true
}

/// Skok Feniksa. Skok z obszarowym podrzutem.
fn e(game: &mut Game, source: usize, point: Point) -> bool {
    let origin = game.units[source].position;
    let landing = game.aim_within(source, point, 164.);
    game.dash_to(source, landing);
    game.fx(source, Shape::Phoenix, origin, landing, 69., 0.59);
    let damage = game.physical(source, 12., 0.72);
    for target in game.enemies(source, landing, 67.) {
        game.skill_hit(source, target, damage, false);
        game.weaken(target, (0., 0.), 0.75);
    }
    true
}

/// Gniew Zaćmienia. Osiem sekund wzmocnienia i ognisty ślad.
fn r(game: &mut Game, source: usize, _point: Point) -> bool {
    let origin = game.units[source].position;
    let status = &mut game.units[source].status;
    status.power.set(0.28, 8.);
    status.control_resist.set(0.50, 8.);
    status.aelor_fire = 8.;
    game.fx(source, Shape::Phoenix, origin, origin, 91., 0.82);
    true
}
