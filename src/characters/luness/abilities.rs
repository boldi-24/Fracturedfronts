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

/// Uzdrawiający Strumień. Leczenie odbija się do dwóch sojuszników.
fn q(game: &mut Game, source: usize, point: Point) -> bool {
    let amount = 26. + Hero::Luness.stats().ap * 0.40;
    game.chain_heal(source, point, amount);
    true
}

/// Tarcza Świtu. Tarcza kończy się leczącym wybuchem.
fn w(game: &mut Game, source: usize, point: Point) -> bool {
    let target = game.ally_target(source, point);
    game.shield(source, target, 33. + Hero::Luness.stats().ap * 0.35, 4.);
    game.units[target].status.burst = 4.;
    true
}

/// Krąg Ożywienia. Strefa oczyszcza, leczy i przyspiesza.
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
            haste: 0.20,
            barrier: 0.,
            cleanse: true,
        },
    );
    true
}

/// Odrodzenie Światła. Obszarowy zastrzyk zdrowia i odnowa.
fn r(game: &mut Game, source: usize, _point: Point) -> bool {
    let origin = game.units[source].position;
    let allies = game.allies(source, origin, 207.);
    for ally in allies {
        game.heal(ally, 48. + Hero::Luness.stats().ap * 0.40);
        game.grace(source, ally);
        game.units[ally].status.guard.set(0.25, 3.);
        game.units[ally].status.renewal.set(10., 3.);
    }
    game.draw_effect(Shape::Phoenix, Tone::Heal, origin, origin, 194., 1.0);
    true
}
