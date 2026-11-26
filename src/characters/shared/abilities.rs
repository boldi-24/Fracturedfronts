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

/// Księżycowe Strzały. Trzy strzały w wachlarzu.
fn q(game: &mut Game, source: usize, point: Point) -> bool {
    let origin = game.units[source].position;
    let direction = game.cast_direction(source, point);
    let damage = game.physical(source, 20., 1.);
    game.projectile_group = game.projectile_group.wrapping_add(1).max(1);
    let hit = SkillHit {
        group: game.projectile_group,
        repeated: 0.60,
        ..SkillHit::default()
    };
    for angle in [-0.21_f32, 0., 0.21] {
        let (sin, cos) = angle.sin_cos();
        let aim = Point::at(
            direction.x * cos - direction.y * sin,
            direction.x * sin + direction.y * cos,
        );
        game.launch(source, aim, damage, hit, 600., 2.);
        game.fx(source, Shape::Beam, origin, origin + aim * 78., 5., 0.32);
    }
    true
}

/// Znak Łowcy. Oznacza i ujawnia bohatera.
fn w(game: &mut Game, source: usize, point: Point) -> bool {
    let origin = game.units[source].position;
    let Some(target) = game.enemy_target(source, point, 325., true) else {
        return false;
    };
    game.mark_tag(source, target, 6.);
    game.units[target].status.revealed = 6.;
    let position = game.units[target].position;
    game.fx(source, Shape::Mark, origin, position, 33., 1.5);
    true
}

/// Cień Przewietrzenia. Odskok, iluzja i wzmocniony strzał.
fn e(game: &mut Game, source: usize, point: Point) -> bool {
    let origin = game.units[source].position;
    let direction = game.cast_direction(source, point);
    game.dash_to(source, (origin - direction * 101.).lane());
    game.units[source].status.longshot.set(150., 4.);
    game.fx(source, Shape::Phoenix, origin, origin, 25., 1.0);
    true
}

/// Deszcz Zaćmienia. Deszcz strzał silniejszy na oznaczonym celu.
fn r(game: &mut Game, source: usize, point: Point) -> bool {
    let at = game.aim_within(source, point, 305.);
    let damage = game.physical(source, 5., 0.23);
    game.place_zone(
        source,
        at,
        106.,
        0.35,
        3.35,
        0.5,
        ZoneAction::Rain {
            damage,
            slow: 0.30,
            marked_bonus: 0.50,
        },
    );
    true
}
