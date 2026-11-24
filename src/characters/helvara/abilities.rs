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

/// Słoneczne Uderzenie. Cięcie tarczą, spowolnienie i ładunek obrony.
fn q(game: &mut Game, source: usize, point: Point) -> bool {
    let direction = game.cast_direction(source, point);
    let damage = game.physical(source, 17., 0.66);
    let targets = game.fan(source, direction, 96., 0.28);
    if !targets.is_empty() {
        game.units[source].status.stacks = (game.units[source].status.stacks + 1).min(5);
    }
    for target in targets {
        game.skill_hit(source, target, damage, false);
        game.weaken(target, (0.4, 1.5), 0.);
    }
    true
}

/// Mur Światła. Chroni siebie i pobliskich sojuszników.
fn w(game: &mut Game, source: usize, _point: Point) -> bool {
    let origin = game.units[source].position;
    game.shield(source, source, 32., 4.);
    game.units[source].status.guard.set(0.30, 4.);
    for ally in game.allies(source, origin, 140.) {
        if ally != source {
            game.shield(source, ally, 25., 4.);
        }
    }
    true
}

/// Przyciągnięcie Blasku. Łańcuch przyciąga i ogłusza.
fn e(game: &mut Game, source: usize, point: Point) -> bool {
    let origin = game.units[source].position;
    let direction = game.cast_direction(source, point);
    if let Some(target) = game
        .line_targets(source, direction, 235., 12.)
        .first()
        .copied()
    {
        game.pull(source, target, 1.);
    } else {
        game.fx(
            source,
            Shape::Beam,
            origin,
            origin + direction * 235.,
            12.,
            0.46,
        );
    }
    true
}

/// Bastion Świtu. Pięć sekund obrony i leczenia w strefie.
fn r(game: &mut Game, source: usize, _point: Point) -> bool {
    let origin = game.units[source].position;
    game.units[source].status.armor.set(0.40, 5.);
    game.units[source].status.magic.set(0.40, 5.);
    game.place_zone(
        source,
        origin,
        140.,
        0.12,
        5.12,
        0.5,
        ZoneAction::Renewal {
            healing: 0.015,
            guard: 0.,
            haste: 0.,
            barrier: 0.,
            cleanse: false,
        },
    );
    true
}
