use super::*;

/// Q — Igła Mgły wystrzeliwuje pocisk z obrażeniami magicznymi.
/// E — Mglisty Skok przesuwa Nerę w kierunku celu.
pub(super) fn cast(game: &mut Game, source: usize, slot: u32, point: Point) {
    let unit = game.units[source];
    let aim = point - unit.position;
    if game.result != 0 || !unit.alive() || unit.status.stun > 0. || aim.length() < 1. {
        return;
    }
    let direction = aim.direction();
    match slot {
        0 if unit.q <= 0. => {
            game.units[source].q = nera::Q.cooldown;
            game.shots.push(Shot {
                side: unit.side,
                position: unit.position + direction * nera::Q.spawn_offset,
                velocity: direction * nera::Q.speed,
                source,
                damage: Damage::Magic(nera::Q.base_damage + nera::STATS.ap * nera::Q.ap_ratio),
                remaining: nera::Q.lifetime,
                skill: None,
                pierced: 0,
            });
        }
        1 if unit.e <= 0. => {
            let dash = &nera::E;
            let endpoint = (unit.position + direction * dash.distance.min(aim.length())).lane();
            game.units[source].e = dash.cooldown;
            game.units[source].shield = dash.shield;
            game.units[source].position = endpoint;
            game.mark(unit.position, endpoint, 3, 0.34);
            if source == PLAYER {
                game.target = None;
                game.destination = endpoint;
            }
        }
        _ => {}
    }
}
