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

/// Podwójna Strzała. Dwie strzały, druga nakładuje ładunek.
fn q(game: &mut Game, source: usize, point: Point) -> bool {
    let origin = game.units[source].position;
    let direction = game.cast_direction(source, point);
    let damage = game.physical(source, 7., 0.82);
    game.launch(
        source,
        direction,
        damage,
        SkillHit {
            stack: false,
            ..SkillHit::default()
        },
        535.,
        0.65,
    );
    game.launch(
        source,
        direction,
        Game::scale_damage(damage, 0.50),
        SkillHit::default(),
        455.,
        0.78,
    );
    game.fx(
        source,
        Shape::Beam,
        origin,
        origin + direction * 108.,
        12.,
        0.34,
    );
    true
}

/// Skrzydła Prędkości. Więcej ruchu i ataków, przebicie pancerza.
fn w(game: &mut Game, source: usize, _point: Point) -> bool {
    let origin = game.units[source].position;
    let status = &mut game.units[source].status;
    status.frenzy.set(0.40, 4.);
    status.haste.set(0.25, 4.);
    status.penetration.set(0.15, 4.);
    game.fx(source, Shape::Phoenix, origin, origin, 63., 0.73);
    true
}

/// Solarna Pułapka. Pułapka spowalnia i ujawnia wrogów.
fn e(game: &mut Game, source: usize, point: Point) -> bool {
    let at = game.aim_within(source, point, 305.);
    game.place_zone(source, at, 42., 0.45, 8., 0.11, ZoneAction::Trap);
    true
}

/// Deszcz Świtu. Opóźniony grad strzał ze spowolnieniem.
fn r(game: &mut Game, source: usize, point: Point) -> bool {
    let at = game.aim_within(source, point, 305.);
    let damage = game.physical(source, 6., 0.25);
    game.place_zone(
        source,
        at,
        112.,
        0.55,
        3.55,
        0.5,
        ZoneAction::Rain {
            damage,
            slow: 0.50,
            marked_bonus: 0.,
        },
    );
    true
}
