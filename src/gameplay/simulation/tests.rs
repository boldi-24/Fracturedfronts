use super::*;

fn near(actual: f32, expected: f32) {
    assert!((actual - expected).abs() < 0.001, "{actual} != {expected}");
}

#[test]
fn every_card_casts_all_four_abilities_and_produces_visible_effects() {
    for hero in Hero::ALL {
        for slot in 0..4 {
            let mut game = Game::with_hero(hero);
            game.units[PLAYER].position = Point::at(480., 280.);
            game.units[BOT].position = Point::at(540., 280.);
            game.units[PLAYER].health = hero.stats().health * 0.19;
            game.cast(PLAYER, slot, game.units[BOT].position);
            let player = game.units[PLAYER];
            let times = hero.profile().cooldowns;
            assert!(
                player.cooldown(slot) > 0.
                    && player.cooldown(slot) <= times[[0, 2, 1, 3][slot as usize]],
                "{hero:?} slot {slot} did not activate"
            );
            assert_eq!(player.cast_slot, slot);
            assert!(player.cast_animation > 0.);
            let mut world_effect = !game.effects.is_empty();
            for _ in 0..60 {
                game.step(1. / 60.);
                world_effect |= !game.effects.is_empty();
            }
            assert!(
                world_effect || !game.zones.is_empty(),
                "{hero:?} slot {slot} did not generate a visual effect"
            );
            for _ in 0..180 {
                game.step(1. / 60.);
            }
            assert!(
                game.units
                    .iter()
                    .all(|unit| unit.health.is_finite() && unit.health >= 0.),
                "{hero:?} slot {slot} produced invalid health"
            );
        }
    }
}

#[test]
fn either_nexus_can_be_destroyed_from_the_start() {
    for (side, attacker, result) in [(Side::Red, PLAYER, 1), (Side::Blue, BOT, 2)] {
        let mut game = Game::with_hero(Hero::One);
        assert_eq!(game.units.len(), 4);
        game.hurt(attacker, side.nexus(), Damage::Physical(10000.));
        assert_eq!(game.result, result);
    }
}

#[test]
fn match_draws_at_the_twenty_minute_limit() {
    let mut game = Game::with_hero(Hero::One);
    game.result = 0;
    game.elapsed = MATCH_LENGTH;
    game.step(0.05);
    assert_eq!(game.result, RESULT_DRAW);
}

#[test]
fn movement_and_aim_work_on_both_ends_of_the_bridge() {
    let mut game = Game::with_hero(Hero::One);
    let point = Side::Red.home();
    game.command(point);
    near(game.destination.x, point.x);
    assert!(point.x > WORLD_WIDTH / 2.);
    let edge = Point::at(WORLD_WIDTH + 500., -25.).lane();
    near(edge.x, WORLD_WIDTH - 28.);
    near(edge.y, super::super::geometry::LANE_MIN_Y);
    game.units[PLAYER].position = Point::at(840., LANE_CENTER);
    game.cast(PLAYER, 1, Point::at(990., LANE_CENTER));
    assert!(game.units[PLAYER].position.x < 840.);
    assert!(game.units[PLAYER].cast_animation > 0.);
}

#[test]
fn lane_duel_reaches_a_result_with_continuous_player_commands() {
    for hero in Hero::ALL {
        let mut game = Game::with_hero(hero);
        for frame in 0..=24010 {
            if frame % 10 == 0 && game.units[PLAYER].alive() {
                let p = game.units[PLAYER];
                let objective = Side::Red.nexus();
                let target = if game.units[BOT].alive() && p.position.distance(game.units[BOT].position) <= 205. { BOT } else { objective };
                let aim = game.units[target].position;
                if p.health < p.kind.health() * 0.3 {
                    game.command(Side::Blue.home());
                } else {
                    game.command(aim);
                    if p.position.distance(aim) < 155. { game.cast(PLAYER, 0, aim); }
                }
            }
            game.step(0.05);
            if game.result != 0 { break; }
        }
        eprintln!("{hero:?}: {:.1} s, result {}", game.elapsed, game.result);
        assert_ne!(game.result, 0);
    }
}

#[test]
fn attack_order_selects_by_cursor_then_stop_cancels_pursuit() {
    let mut game = Game::with_hero(Hero::One);
    let point = Point::at(700., LANE_CENTER);
    game.units[BOT].position = Point::at(450., LANE_CENTER);
    game.attack_unit_near(point);
    assert_eq!(game.target, Some(BOT));
    game.player_update(0.05);
    assert!(game.units[PLAYER].moving);
    game.stop_player();
    let stopped = game.units[PLAYER].position;
    game.player_update(0.05);
    near(stopped.distance(game.units[PLAYER].position), 0.);
    assert!(game.target.is_none() && !game.units[PLAYER].moving);
    game.units[BOT].status.hidden = 1.;
    game.attack_unit_near(point);
    assert!(game.target.is_none());
}
