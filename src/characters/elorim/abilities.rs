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

/// Iskra Żaru. Wystrzeliwuje świetlistą kulę w kierunku kursora.
fn q(game: &mut Game, source: usize, point: Point) -> bool {
    let origin = game.units[source].position;
    let direction = game.cast_direction(source, point);
    game.launch(
        source,
        direction,
        game.magic(source, 17., 0.80),
        SkillHit::default(),
        520.,
        0.72,
    );
    game.fx(
        source,
        Shape::Beam,
        origin,
        origin + direction * 49.,
        8.,
        0.25,
    );
    true
}

/// Krąg Koncentracji. Tworzy krąg zadający okresowe obrażenia i spowalniający wrogów.
fn w(game: &mut Game, source: usize, point: Point) -> bool {
    let at = game.aim_within(source, point, 305.);
    game.place_zone(
        source,
        at,
        84.,
        0.2,
        3.1,
        0.55,
        ZoneAction::Rain {
            damage: game.magic(source, 4., 0.19),
            slow: 0.3,
            marked_bonus: 0.,
        },
    );
    true
}

/// Przemieszczenie Światła. Przeskakuje do przodu i zostawia wybuch w miejscu startu.
fn e(game: &mut Game, source: usize, point: Point) -> bool {
    let origin = game.units[source].position;
    game.dash_to(source, game.aim_within(source, point, 118.));
    game.fx(source, Shape::Burst, origin, origin, 60., 0.48);
    for target in game.enemies(source, origin, 60.) {
        game.skill_hit(source, target, game.magic(source, 14., 0.60), true);
    }
    true
}

/// Nowa Zorza. Przywołuje filar światła, który po chwili wybucha i ogłusza.
fn r(game: &mut Game, source: usize, point: Point) -> bool {
    let at = game.aim_within(source, point, 305.);
    game.place_zone(source, at, 95., 0.88, 0.95, 1., ZoneAction::DawnNova);
    true
}
