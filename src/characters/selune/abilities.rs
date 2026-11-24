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

/// Księżycowe Ukojenie. Leczy cel i do dwóch rannych sojuszników.
fn q(game: &mut Game, source: usize, point: Point) -> bool {
    let amount = 26. + Hero::Selune.stats().ap * 0.40;
    game.chain_heal(source, point, amount);
    true
}

/// Tarcza Zmierzchu. Tarcza i zwiększona odporność magiczna.
fn w(game: &mut Game, source: usize, point: Point) -> bool {
    let target = game.ally_target(source, point);
    game.shield(source, target, 33. + Hero::Selune.stats().ap * 0.35, 4.);
    game.units[target].status.magic.set(0.20, 4.);
    true
}

/// Płomień Odnowy. Leczące płatki zdejmują efekt kontroli.
fn e(game: &mut Game, source: usize, point: Point) -> bool {
    let at = game.aim_within(source, point, 305.);
    game.place_zone(
        source,
        at,
        99.,
        0.12,
        5.12,
        0.5,
        ZoneAction::Renewal {
            healing: 0.022,
            guard: 0.,
            haste: 0.,
            barrier: 0.,
            cleanse: true,
        },
    );
    true
}

/// Odrodzenie Feniksa. Wskrzesza jednostkę albo ratuje ciężko ranny cel.
fn r(game: &mut Game, source: usize, point: Point) -> bool {
    let origin = game.units[source].position;
    let side = game.units[source].side;
    let dead = game
        .units
        .iter()
        .enumerate()
        .filter(|(_, unit)| {
            !unit.alive()
                && unit.status.fallen > 0.
                && !unit.kind.structure()
                && unit.side == side
                && unit.position.distance(origin) <= 275.
                && unit.position.distance(point) <= 62.
        })
        .min_by(|(_, a), (_, b)| {
            a.position
                .distance(point)
                .total_cmp(&b.position.distance(point))
        })
        .map(|(i, _)| i);
    if let Some(target) = dead {
        let unit = game.units[target];
        game.units[target] = Unit::new(unit.kind, unit.side, unit.position);
        game.units[target].health *= 0.50;
        game.shield(source, target, 33., 3.);
        game.fx(
            source,
            Shape::Phoenix,
            unit.position,
            unit.position,
            74.,
            1.0,
        );
    } else {
        let target = game.ally_target(source, point);
        if game.units[target].health >= game.units[target].kind.health() * 0.25 {
            return false;
        }
        game.heal(target, 67.);
        game.grace(source, target);
        game.units[target].status.invulnerable = 1.5;
        let position = game.units[target].position;
        game.fx(source, Shape::Phoenix, origin, position, 69., 0.85);
    }
    true
}
