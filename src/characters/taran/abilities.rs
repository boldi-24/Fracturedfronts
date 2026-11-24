use super::*;

/// Q — Cięcie Kuźni zadaje obrażenia fizyczne w stożku.
/// E — Żelazny Krok przesuwa Tarana i daje mu osłonę.
pub(super) fn cast(game: &mut Game, source: usize, slot: u32, point: Point) {
    let unit = game.units[source];
    let aim = point - unit.position;
    if game.result != 0 || !unit.alive() || unit.status.stun > 0. || aim.length() < 1. {
        return;
    }
    let direction = aim.direction();
    match slot {
        0 if unit.q <= 0. => {
            game.units[source].q = taran::Q.cooldown;
            game.mark(
                unit.position,
                unit.position + direction * taran::Q.visual_range,
                2,
                0.38,
            );
            for target in 0..UNIT_COUNT {
                let other = game.units[target];
                let delta = other.position - unit.position;
                let facing = delta.x * direction.x + delta.y * direction.y;
                if other.alive()
                    && other.side != unit.side
                    && !other.kind.structure()
                    && delta.length() <= taran::Q.reach + other.kind.radius()
                    && facing >= delta.length() * taran::Q.facing_threshold
                {
                    let damage = taran::Q.base_damage + taran::STATS.ad * taran::Q.ad_ratio;
                    game.hurt(source, target, Damage::Physical(damage));
                }
            }
        }
        1 if unit.e <= 0. => {
            let dash = &taran::E;
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
