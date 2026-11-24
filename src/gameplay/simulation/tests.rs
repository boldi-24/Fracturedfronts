use super::*;

fn near(actual: f32, expected: f32) {
    assert!((actual - expected).abs() < 0.001, "{actual} != {expected}");
}

#[test]
fn physical_and_magic_damage_use_their_own_resistance() {
    let stats = Some(&taran::STATS);
    near(
        Damage::Physical(100.).after_resistance(stats),
        10000. / 124.,
    );
    near(Damage::Magic(100.).after_resistance(stats), 10000. / 116.);
    near(Damage::Magic(100.).after_resistance(None), 100.);
}

#[test]
fn omnivamp_uses_health_lost_and_does_not_heal_from_towers() {
    let mut game = Game::new(false);
    game.units[PLAYER].health = 100.;
    game.units[BOT].health = 3.;
    game.hurt(PLAYER, BOT, Damage::Physical(100.));
    near(game.units[PLAYER].health, 100. + 3. * taran::STATS.omnivamp);
    near(game.units[BOT].respawn, 5.);

    let health = game.units[PLAYER].health;
    game.hurt(PLAYER, RED_TOWER, Damage::Physical(100.));
    near(game.units[PLAYER].health, health);
}

#[test]
fn shield_and_armor_both_reduce_damage() {
    let mut game = Game::new(false);
    game.units[PLAYER].shield = taran::E.shield;
    game.hurt(BOT, PLAYER, Damage::Physical(100.));
    near(
        game.units[PLAYER].health,
        taran::STATS.health - (10000. / 124.) * 0.45,
    );
}

#[test]
fn hero_attacks_use_ad_and_the_configured_attack_speed() {
    let mut game = Game::new(true);
    game.strike(PLAYER, BOT);
    let after_first = taran::STATS.health - nera::STATS.ad * (100. / 124.);
    near(game.units[BOT].health, after_first);
    game.strike(PLAYER, BOT);
    near(game.units[BOT].health, after_first);
    game.units[PLAYER].update(nera::STATS.attack_speed.recip());
    game.strike(PLAYER, BOT);
    assert!(game.units[BOT].health < after_first);
}

#[test]
fn nera_s_projectile_uses_ap_and_magic_defense() {
    let mut game = Game::new(true);
    game.units[PLAYER].position = Point::at(500., 280.);
    game.units[BOT].position = Point::at(560., 280.);
    game.cast(PLAYER, 0, game.units[BOT].position);
    game.units[BOT].q = 1.;
    game.units[BOT].attack = 1.;
    game.step(0.05);
    let raw = nera::Q.base_damage + nera::STATS.ap * nera::Q.ap_ratio;
    near(
        game.units[BOT].health,
        taran::STATS.health - raw * (100. / 116.),
    );
}

#[test]
fn taran_s_cone_uses_ad_and_physical_defense() {
    let mut game = Game::new(false);
    game.units[PLAYER].position = Point::at(500., 280.);
    game.units[BOT].position = Point::at(560., 280.);
    game.cast(PLAYER, 0, game.units[BOT].position);
    let raw = taran::Q.base_damage + taran::STATS.ad * taran::Q.ad_ratio;
    near(
        game.units[BOT].health,
        nera::STATS.health - raw * (100. / 112.),
    );
}

#[test]
fn every_card_casts_all_four_abilities_and_produces_visible_effects() {
    for hero in Hero::ALL.into_iter().skip(2) {
        for slot in 0..4 {
            let mut game = Game::with_hero(hero);
            game.units[PLAYER].position = Point::at(480., 280.);
            game.units[BOT].position = Point::at(540., 280.);
            game.units[PLAYER].health = hero.stats().health * 0.19;
            game.cast(PLAYER, slot, game.units[BOT].position);
            let player = game.units[PLAYER];
            let times = hero.profile().unwrap().cooldowns;
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
fn luna_support_chain_and_selune_revive_have_usable_minion_targets() {
    let mut game = Game::with_hero(Hero::Luness);
    let first = FIRST_MINION;
    let second = FIRST_MINION + 1;
    game.units[PLAYER].position = Point::at(280., 280.);
    game.units[first].health = 10.;
    game.units[second].health = 10.;
    game.cast(PLAYER, 0, game.units[first].position);
    assert!(game.units[first].health > 10.);
    assert!(game.units[second].health > 10.);
    assert!(game.units[first].status.light > 0.);

    let mut game = Game::with_hero(Hero::Selune);
    game.units[PLAYER].position = Point::at(280., 280.);
    let point = game.units[first].position;
    game.hurt(BOT, first, Damage::Magic(100.));
    game.cast(PLAYER, 3, point);
    near(game.units[first].health, Kind::Minion.health() * 0.5);
    assert!(game.units[first].status.barrier > 0.);
}

#[test]
fn offense_defense_and_shadows_produce_distinct_passives() {
    let mut aelor = Game::with_hero(Hero::Aelor);
    for _ in 0..3 {
        aelor.hurt(BOT, PLAYER, Damage::Physical(20.));
    }
    assert_eq!(aelor.units[PLAYER].status.stacks, 3);
    let health = aelor.units[PLAYER].health;
    aelor.strike(PLAYER, BOT);
    assert_eq!(aelor.units[PLAYER].status.stacks, 0);
    assert!(aelor.units[PLAYER].health > health);

    let mut arxen = Game::with_hero(Hero::Arxen);
    arxen.units[PLAYER].position = Point::at(480., 280.);
    arxen.units[BOT].position = Point::at(580., 280.);
    arxen.cast(PLAYER, 2, arxen.units[BOT].position);
    arxen.units[BOT].status.hidden = 2.;
    assert!(!arxen.units[BOT].status.concealed());
    arxen.cast(PLAYER, 1, arxen.units[BOT].position);
    assert!(arxen.units[PLAYER].position.x < 480.);
    assert!(arxen.units[PLAYER].status.longshot.value() > 0.);

    let mut kaelith = Game::with_hero(Hero::Kaelith);
    kaelith.cast(PLAYER, 2, Point::at(450., 280.));
    assert!(kaelith.units[PLAYER].status.concealed());
    kaelith.strike(PLAYER, BOT);
    assert!(!kaelith.units[PLAYER].status.concealed());
}

#[test]
fn structures_unlock_in_order_and_only_nexus_destruction_ends_match() {
    for (side, attacker, result) in [(Side::Red, PLAYER, 1), (Side::Blue, BOT, 2)] {
        let mut game = Game::new(false);
        let nexus = side.nexus();
        let towers = side.towers();
        for (i, tower) in towers.into_iter().enumerate() {
            assert_eq!(game.objective(side), tower);
            game.hurt(attacker, nexus, Damage::Physical(10000.));
            near(game.units[nexus].health, Kind::Nexus.health());
            for &protected in &towers[i + 1..] {
                game.hurt(attacker, protected, Damage::Physical(10000.));
                near(game.units[protected].health, Kind::Tower.health());
            }
            game.hurt(attacker, tower, Damage::Physical(10000.));
            assert!(!game.units[tower].alive());
            assert_eq!(game.result, 0);
        }
        assert_eq!(game.objective(side), nexus);
        game.hurt(attacker, nexus, Damage::Physical(10000.));
        assert_eq!(game.result, result);
    }
}

#[test]
fn tower_and_minion_projectiles_fly_before_applying_damage() {
    let mut game = Game::new(false);
    let red_minion = FIRST_MINION + MINIONS_PER_SIDE;
    let tower_pos = game.units[BLUE_TOWER].position;
    game.units[red_minion].position = tower_pos + Point::at(80., 0.);
    for (source, from_tower) in [(BLUE_TOWER, true), (FIRST_MINION + 2, false)] {
        game.units[source].position = tower_pos;
        game.units[red_minion].health = Kind::Minion.health();
        game.strike(source, red_minion);
        near(game.units[red_minion].health, Kind::Minion.health());
        game.update_bolts(0.08);
        let bolt = game.bolts.first().unwrap();
        assert_eq!(bolt.from_tower, from_tower);
        assert!(bolt.position.x > tower_pos.x && bolt.position.x < tower_pos.x + 80.);
        game.update_bolts(0.6);
        assert!(game.bolts.is_empty());
        assert!(game.units[red_minion].health < Kind::Minion.health());
    }
}

#[test]
fn idle_player_match_advances_minions_and_bot_to_enemy_nexus() {
    let mut game = Game::new(false);
    for _ in 0..(20 * 60 * 20) {
        game.step(0.05);
        if game.result != 0 {
            break;
        }
    }
    assert_eq!(
        game.result, 2,
        "bot should finish its siege in a 20-minute unattended duel"
    );
    assert!(
        Side::Blue
            .towers()
            .iter()
            .all(|&tower| !game.units[tower].alive())
    );
}

#[test]
fn match_draws_at_the_twenty_minute_limit() {
    let mut game = Game::new(false);
    game.result = 0;
    game.elapsed = MATCH_LENGTH;
    game.step(0.05);
    assert_eq!(game.result, RESULT_DRAW);
}

#[test]
fn movement_and_aim_work_on_both_ends_of_the_bridge() {
    let mut game = Game::new(false);
    let point = Side::Red.home();
    game.command(point);
    near(game.destination.x, point.x);
    assert!(point.x > 3000.);
    let edge = Point::at(WORLD_WIDTH + 500., -25.).lane();
    near(edge.x, WORLD_WIDTH - 28.);
    near(edge.y, super::super::geometry::LANE_MIN_Y);
    game.units[PLAYER].position = Point::at(2400., LANE_CENTER);
    game.cast(PLAYER, 1, Point::at(2550., LANE_CENTER));
    assert!(game.units[PLAYER].position.x > 2400.);
    assert!(game.units[PLAYER].cast_animation > 0.);
}

#[test]
fn elorim_nova_waits_then_stuns_and_triggers_one_delayed_resonance() {
    let mut game = Game::with_hero(Hero::Elorim);
    let origin = Point::at(2200., LANE_CENTER);
    let impact = Point::at(2360., LANE_CENTER);
    game.units[PLAYER].position = origin;
    game.units[BOT].position = impact;
    let hp = game.units[BOT].health;
    game.cast(PLAYER, 3, impact);
    assert!(
        !game.effects
            .iter()
            .any(|e| e.shape == Shape::Mark && e.to.distance(impact) < 1.)
    );
    game.update_zones(0.8);
    near(game.units[BOT].health, hp);
    near(game.units[BOT].status.stun, 0.);
    game.update_zones(0.1);
    let after_nova = game.units[BOT].health;
    assert!(after_nova < hp && game.units[BOT].status.stun > 1.);
    assert!(game.zones.is_empty());
    game.update_effects(BOT, 0.6);
    near(game.units[BOT].health, after_nova);
    game.update_effects(BOT, 0.14);
    let after_resonance = game.units[BOT].health;
    assert!(after_resonance < after_nova);
    game.update_effects(BOT, 0.8);
    near(game.units[BOT].health, after_resonance);
}

#[test]
fn elorim_nova_misses_targets_that_escape_the_warning_ring() {
    let mut game = Game::with_hero(Hero::Elorim);
    let aim = Point::at(2420., LANE_CENTER);
    game.units[PLAYER].position = Point::at(2230., LANE_CENTER);
    game.units[BOT].position = aim;
    game.cast(PLAYER, 3, aim);
    game.units[BOT].position = aim + Point::at(170., 0.);
    game.update_zones(0.90);
    near(game.units[BOT].health, game.units[BOT].kind.health());
    near(game.units[BOT].status.stun, 0.);
}

#[test]
fn towers_warn_a_locked_target_and_cancel_if_it_leaves() {
    let mut game = Game::new(false);
    let tower = BLUE_TOWER;
    let point = game.units[tower].position;
    game.units[BOT].position = point + Point::at(60., 0.);
    game.tower_update(tower, 0.05);
    assert_eq!(game.tower_aim(tower), Some((BOT, 0.)));
    game.tower_update(tower, 0.35);
    near(game.tower_aim(tower).unwrap().1, 0.5);
    game.units[BOT].position = point + Point::at(250., 0.);
    game.tower_update(tower, 0.4);
    assert!(game.tower_aim(tower).is_none() && game.bolts.is_empty());
    game.units[BOT].position = point;
    game.tower_update(tower, 0.05);
    game.tower_update(tower, TOWER_WINDUP);
    assert!(game.tower_aim(tower).is_none());
    assert_eq!(game.bolts.len(), 1);
}

#[test]
fn waves_contain_frontline_and_ranged_units_with_distinct_attacks() {
    let mut game = Game::new(false);
    let melee = FIRST_MINION;
    let ranged = FIRST_MINION + 2;
    assert!(!game.units[melee].minion_ranged && game.units[ranged].minion_ranged);
    assert!(game.units[melee].reach() < game.units[ranged].reach());
    let victim = FIRST_MINION + MINIONS_PER_SIDE;
    game.strike(melee, victim);
    assert!(game.units[victim].health < Kind::Minion.health() && game.bolts.is_empty());
    game.strike(ranged, victim);
    assert_eq!(game.bolts.len(), 1);
}

#[test]
fn lane_duel_reaches_a_result_with_continuous_player_commands() {
    for hero in [Hero::Taran, Hero::Aurelia, Hero::Elorim] {
        let mut game = Game::with_hero(hero);
        for frame in 0..=24010 {
            if frame % 10 == 0 && game.units[PLAYER].alive() {
                let p = game.units[PLAYER];
                let objective = game.objective(Side::Red);
                let target = game.enemy_in(PLAYER, 205., false).unwrap_or(objective);
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
    let mut game = Game::with_hero(Hero::Taran);
    let point = Point::at(700., LANE_CENTER);
    game.units[FIRST_MINION] = Unit::new(Kind::Minion, Side::Red, point);
    game.units[BOT].position = Point::at(450., LANE_CENTER);
    game.attack_unit_near(point);
    assert_eq!(game.target, Some(FIRST_MINION));
    game.player_update(0.05);
    assert!(game.units[PLAYER].moving);
    game.stop_player();
    let stopped = game.units[PLAYER].position;
    game.player_update(0.05);
    near(stopped.distance(game.units[PLAYER].position), 0.);
    assert!(game.target.is_none() && !game.units[PLAYER].moving);
    game.units[FIRST_MINION].status.hidden = 1.;
    game.attack_unit_near(point);
    assert_eq!(game.target, Some(BOT));
}
