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

/// Srebrna Koma. Kometa uderza i spowalnia pierwszego wroga.
fn q(game: &mut Game, source: usize, point: Point) -> bool {
    let origin = game.units[source].position;
    let direction = game.cast_direction(source, point);
    let damage = game.magic(source, 18., 0.78);
    game.launch(
        source,
        direction,
        damage,
        SkillHit {
            slow: (0.32, 1.7),
            ..SkillHit::default()
        },
        490.,
        0.75,
    );
    game.fx(
        source,
        Shape::Beam,
        origin,
        origin + direction * 70.,
        7.,
        0.37,
    );
    true
}

/// Płaszcz Księżyca. Tarcza i trzy sekundy szybszego ruchu.
fn w(game: &mut Game, source: usize, _point: Point) -> bool {
    game.shield(source, source, 30. + Hero::Nyxalune.stats().ap * 0.30, 3.);
    game.units[source].status.haste.set(0.25, 3.);
    true
}

/// Cień Feniksa. Feniks przebija linię i obniża odporność magiczną.
fn e(game: &mut Game, source: usize, point: Point) -> bool {
    let origin = game.units[source].position;
    let direction = game.cast_direction(source, point);
    let damage = game.magic(source, 11., 0.65);
    game.launch(
        source,
        direction,
        damage,
        SkillHit {
            pierce: true,
            weaken: 0.25,
            ..SkillHit::default()
        },
        555.,
        0.68,
    );
    game.fx(
        source,
        Shape::Phoenix,
        origin,
        origin + direction * 288.,
        53.,
        0.61,
    );
    true
}

/// Zaćmienie Odrodzenia. Strefa z tarczami, spowolnieniem i wybuchem.
fn r(game: &mut Game, source: usize, point: Point) -> bool {
    let at = game.aim_within(source, point, 305.);
    let damage = game.magic(source, 6., 0.15);
    for ally in game.allies(source, at, 126.) {
        game.shield(source, ally, 26., 1.);
    }
    game.place_zone(
        source,
        at,
        126.,
        0.2,
        4.2,
        0.65,
        ZoneAction::Rain {
            damage,
            slow: 0.36,
            marked_bonus: 0.,
        },
    );
    true
}
