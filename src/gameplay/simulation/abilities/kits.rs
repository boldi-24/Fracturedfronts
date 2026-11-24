use super::*;

impl Unit {
    pub(crate) fn cooldown(self, slot: u32) -> f32 {
        match slot {
            0 => self.q,
            1 => self.e,
            2 => self.w,
            3 => self.r,
            _ => 0.,
        }
    }

    fn set_cooldown(&mut self, slot: u32, time: f32) {
        match slot {
            0 => self.q = time,
            1 => self.e = time,
            2 => self.w = time,
            3 => self.r = time,
            _ => {}
        }
    }
}

impl Game {
    pub(in crate::gameplay::simulation) fn featured_cast(
        &mut self,
        source: usize,
        hero: Hero,
        slot: u32,
        point: Point,
    ) {
        let unit = self.units[source];
        if self.result != 0
            || !unit.alive()
            || unit.status.stun > 0.
            || slot > 3
            || unit.cooldown(slot) > 0.
        {
            return;
        }
        let profile = hero.profile().expect("featured champion kit");
        let index = [0, 2, 1, 3][slot as usize];
        self.units[source].set_cooldown(slot, profile.cooldowns[index]);
        if !self.apply_kit(source, hero, slot, point) {
            self.units[source].set_cooldown(slot, 0.);
        }
    }

    fn apply_kit(&mut self, source: usize, hero: Hero, slot: u32, point: Point) -> bool {
        use Hero::*;
        let origin = self.units[source].position;
        let delta = point - origin;
        let direction = if delta.length() < 1. {
            Point::at(self.units[source].side.advance(), 0.)
        } else {
            delta.direction()
        };
        let at = self.aim_within(source, point, 305.);
        let physical = |game: &Game, base, ratio| game.physical(source, base, ratio);
        let magic = |game: &Game, base, ratio| game.magic(source, base, ratio);

        match (hero, slot) {
            (Elorim, 0) => {
                self.launch(
                    source,
                    direction,
                    magic(self, 17., 0.80),
                    SkillHit::default(),
                    520.,
                    0.72,
                );
                self.fx(
                    source,
                    Shape::Beam,
                    origin,
                    origin + direction * 49.,
                    8.,
                    0.25,
                );
            }
            (Elorim, 2) => self.place_zone(
                source,
                at,
                84.,
                0.2,
                3.1,
                0.55,
                ZoneAction::Rain {
                    damage: magic(self, 4., 0.19),
                    slow: 0.3,
                    marked_bonus: 0.,
                },
            ),
            (Elorim, 1) => {
                self.dash_to(source, self.aim_within(source, point, 118.));
                self.fx(source, Shape::Burst, origin, origin, 60., 0.48);
                for target in self.enemies(source, origin, 60.) {
                    self.skill_hit(source, target, magic(self, 14., 0.60), true);
                }
            }
            (Elorim, 3) => self.place_zone(source, at, 95., 0.88, 0.95, 1., ZoneAction::DawnNova),
            (Aelor, 0) => {
                let damage = physical(self, 14., 0.83);
                for target in self.fan(source, direction, 103., 0.30) {
                    self.skill_hit(source, target, damage, false);
                    self.weaken(target, (0.30, 2.), 0.);
                }
            }
            (Aelor, 2) => {
                self.shield(source, source, 43., 4.);
                let status = &mut self.units[source].status;
                status.armor.set(0.20, 4.);
                status.guard.set(0.14, 4.);
            }
            (Aelor, 1) => {
                let landing = self.aim_within(source, point, 164.);
                self.dash_to(source, landing);
                self.fx(source, Shape::Phoenix, origin, landing, 69., 0.59);
                let damage = physical(self, 12., 0.72);
                for target in self.enemies(source, landing, 67.) {
                    self.skill_hit(source, target, damage, false);
                    self.weaken(target, (0., 0.), 0.75);
                }
            }
            (Aelor, 3) => {
                let status = &mut self.units[source].status;
                status.power.set(0.28, 8.);
                status.control_resist.set(0.50, 8.);
                status.aelor_fire = 8.;
                self.fx(source, Shape::Phoenix, origin, origin, 91., 0.82);
            }
            (Arxen, 0) => {
                let damage = physical(self, 5., 0.62);
                self.projectile_group = self.projectile_group.wrapping_add(1).max(1);
                let hit = SkillHit {
                    group: self.projectile_group,
                    repeated: 0.60,
                    ..SkillHit::default()
                };
                for angle in [-0.21_f32, 0., 0.21] {
                    let (sin, cos) = angle.sin_cos();
                    let aim = Point::at(
                        direction.x * cos - direction.y * sin,
                        direction.x * sin + direction.y * cos,
                    );
                    self.launch(source, aim, damage, hit, 565., 0.63);
                    self.fx(source, Shape::Beam, origin, origin + aim * 78., 5., 0.32);
                }
            }
            (Arxen, 2) => {
                let Some(target) = self.enemy_target(source, point, 325., true) else {
                    return false;
                };
                self.mark_tag(source, target, 6.);
                self.units[target].status.revealed = 6.;
                let position = self.units[target].position;
                self.fx(source, Shape::Mark, origin, position, 33., 1.5);
            }
            (Arxen, 1) => {
                self.dash_to(source, (origin - direction * 101.).lane());
                self.units[source].status.longshot.set(150., 4.);
                self.fx(source, Shape::Phoenix, origin, origin, 25., 1.0);
            }
            (Arxen, 3) => {
                let damage = physical(self, 5., 0.23);
                self.place_zone(
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
            }
            (Aurelia, 0) => {
                let damage = physical(self, 7., 0.82);
                self.launch(
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
                self.launch(
                    source,
                    direction,
                    Self::scale_damage(damage, 0.50),
                    SkillHit::default(),
                    455.,
                    0.78,
                );
                self.fx(
                    source,
                    Shape::Beam,
                    origin,
                    origin + direction * 108.,
                    12.,
                    0.34,
                );
            }
            (Aurelia, 2) => {
                let status = &mut self.units[source].status;
                status.frenzy.set(0.40, 4.);
                status.haste.set(0.25, 4.);
                status.penetration.set(0.15, 4.);
                self.fx(source, Shape::Phoenix, origin, origin, 63., 0.73);
            }
            (Aurelia, 1) => self.place_zone(source, at, 42., 0.45, 8., 0.11, ZoneAction::Trap),
            (Aurelia, 3) => {
                let damage = physical(self, 6., 0.25);
                self.place_zone(
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
            }
            (Helvara, 0) => {
                let damage = physical(self, 17., 0.66);
                let targets = self.fan(source, direction, 96., 0.28);
                if !targets.is_empty() {
                    self.units[source].status.stacks =
                        (self.units[source].status.stacks + 1).min(5);
                }
                for target in targets {
                    self.skill_hit(source, target, damage, false);
                    self.weaken(target, (0.4, 1.5), 0.);
                }
            }
            (Helvara, 2) => {
                self.shield(source, source, 32., 4.);
                self.units[source].status.guard.set(0.30, 4.);
                for ally in self.allies(source, origin, 140.) {
                    if ally != source {
                        self.shield(source, ally, 25., 4.);
                    }
                }
            }
            (Helvara, 1) => {
                if let Some(target) = self
                    .line_targets(source, direction, 235., 12.)
                    .first()
                    .copied()
                {
                    self.pull(source, target, 1.);
                } else {
                    self.fx(
                        source,
                        Shape::Beam,
                        origin,
                        origin + direction * 235.,
                        12.,
                        0.46,
                    );
                }
            }
            (Helvara, 3) => {
                self.units[source].status.armor.set(0.40, 5.);
                self.units[source].status.magic.set(0.40, 5.);
                self.place_zone(
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
            }
            (Kaelith, 0) => {
                let targets = self.fan(source, direction, 83., 0.48);
                let hit = physical(self, 9., 0.48);
                for target in targets {
                    let bonus = if self.stacks(source, target) > 0 {
                        1.35
                    } else {
                        1.
                    };
                    self.skill_hit(source, target, hit, true);
                    self.skill_hit(source, target, Self::scale_damage(hit, bonus), true);
                }
                self.fx(
                    source,
                    Shape::Arc,
                    origin + direction * 8.,
                    origin + direction * 59.,
                    66.,
                    0.65,
                );
            }
            (Kaelith | Zyraen, 2) => {
                let status = &mut self.units[source].status;
                status.hidden = 2.5;
                status.empowered = 2.5;
                status.haste.set(0.4, 2.5);
                self.fx(
                    source,
                    Shape::Dash,
                    origin,
                    origin + direction * 35.,
                    27.,
                    0.5,
                );
            }
            (Kaelith, 1) => {
                let Some(target) = self.enemy_target(source, point, 185., false) else {
                    return false;
                };
                if self.stacks(source, target) > 0 {
                    self.units[source].e *= 0.50;
                }
                self.dash_enemy(source, target);
                let damage = physical(self, 13., 0.80);
                self.skill_hit(source, target, damage, true);
                self.weaken(target, (0.30, 1.5), 0.);
            }
            (Kaelith, 3) => {
                self.units[source].status.hidden = 0.75;
                self.place_zone(source, at, 100., 0.75, 0.82, 0., ZoneAction::Dive);
            }
            (Luness | Selune, 0) => {
                let amount = 26. + hero.stats().ap * 0.40;
                self.chain_heal(source, point, amount);
            }
            (Luness | Selune, 2) => {
                let target = self.ally_target(source, point);
                self.shield(source, target, 33. + hero.stats().ap * 0.35, 4.);
                if hero == Luness {
                    self.units[target].status.burst = 4.;
                } else {
                    self.units[target].status.magic.set(0.20, 4.);
                }
            }
            (Luness | Selune, 1) => {
                self.place_zone(
                    source,
                    at,
                    99.,
                    0.12,
                    5.12,
                    0.5,
                    ZoneAction::Renewal {
                        healing: 0.022,
                        guard: 0.,
                        haste: if hero == Luness { 0.20 } else { 0. },
                        barrier: 0.,
                        cleanse: true,
                    },
                );
            }
            (Luness, 3) => {
                let allies = self.allies(source, origin, 207.);
                for ally in allies {
                    self.heal(ally, 48. + hero.stats().ap * 0.40);
                    self.grace(source, ally);
                    self.units[ally].status.guard.set(0.25, 3.);
                    self.units[ally].status.renewal.set(10., 3.);
                }
                self.draw_effect(Shape::Phoenix, Tone::Heal, origin, origin, 194., 1.0);
            }
            (Nyxalune, 0) => {
                let damage = magic(self, 18., 0.78);
                self.launch(
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
                self.fx(
                    source,
                    Shape::Beam,
                    origin,
                    origin + direction * 70.,
                    7.,
                    0.37,
                );
            }
            (Nyxalune, 2) => {
                self.shield(source, source, 30. + hero.stats().ap * 0.30, 3.);
                self.units[source].status.haste.set(0.25, 3.);
            }
            (Nyxalune, 1) => {
                let damage = magic(self, 11., 0.65);
                self.launch(
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
                self.fx(
                    source,
                    Shape::Phoenix,
                    origin,
                    origin + direction * 288.,
                    53.,
                    0.61,
                );
            }
            (Nyxalune, 3) => {
                let damage = magic(self, 6., 0.15);
                for ally in self.allies(source, at, 126.) {
                    self.shield(source, ally, 26., 1.);
                }
                self.place_zone(
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
            }
            (Selune, 3) => {
                let side = self.units[source].side;
                let dead = self
                    .units
                    .iter()
                    .enumerate()
                    .filter(|(_, unit)| {
                        !unit.alive()
                            && unit.status.fallen > 0.
                            && !unit.kind.structure()
                            && unit.side == side
                            && unit.position.distance(origin) <= 275.
                            && unit.position.distance(point) <= 62.
                    })
                    .min_by(|(_, a), (_, b)| {
                        a.position
                            .distance(point)
                            .total_cmp(&b.position.distance(point))
                    })
                    .map(|(i, _)| i);
                if let Some(target) = dead {
                    let unit = self.units[target];
                    self.units[target] = Unit::new(unit.kind, unit.side, unit.position);
                    self.units[target].health *= 0.50;
                    self.shield(source, target, 33., 3.);
                    self.fx(
                        source,
                        Shape::Phoenix,
                        unit.position,
                        unit.position,
                        74.,
                        1.0,
                    );
                } else {
                    let target = self.ally_target(source, point);
                    if self.units[target].health >= self.units[target].kind.health() * 0.25 {
                        return false;
                    }
                    self.heal(target, 67.);
                    self.grace(source, target);
                    self.units[target].status.invulnerable = 1.5;
                    let position = self.units[target].position;
                    self.fx(source, Shape::Phoenix, origin, position, 69., 0.85);
                }
            }
            (Solviana | Zyraen, 0) => {
                let range = if hero == Solviana { 144. } else { 118. };
                let damage = physical(self, 13., 0.79);
                let targets = self.line_targets(source, direction, range, 19.);
                self.fx(
                    source,
                    Shape::Arc,
                    origin,
                    origin + direction * range,
                    range * 0.67,
                    0.41,
                );
                if targets
                    .iter()
                    .any(|&target| self.stacks(source, target) > 0)
                {
                    self.units[source].q = if hero == Solviana {
                        self.units[source].q - 2.
                    } else {
                        self.units[source].q * 0.5
                    };
                }
                for target in targets {
                    self.skill_hit(source, target, damage, true);
                }
            }
            (Solviana, 2) => {
                self.shield(source, source, 37., 4.);
                self.units[source].status.guard.set(0.25, 4.);
                self.units[source].status.sustain.set(0.15, 4.);
            }
            (Solviana | Zyraen, 1) => {
                let Some(target) = self.enemy_target(source, point, 204., false) else {
                    return false;
                };
                let marked = self.stacks(source, target) >= 3;
                self.dash_enemy(source, target);
                let damage = physical(self, 13., 0.75);
                self.skill_hit(source, target, damage, true);
                if hero == Solviana {
                    self.weaken(target, (0.40, 2.), 0.);
                } else if marked {
                    self.weaken(target, (0., 0.), 1.);
                }
            }
            (Solviana, 3) => {
                let damage = physical(self, 33., 0.9);
                self.fx(source, Shape::Burst, origin, origin, 121., 0.78);
                for target in self.enemies(source, origin, 108.) {
                    self.skill_hit(source, target, damage, true);
                    self.knockback(origin, target, 62.);
                }
                self.units[source].status.power.set(0.30, 6.);
            }
            (Veyrion, 0) => {
                let damage = physical(self, 13., 0.65);
                for target in self.fan(source, direction, 105., 0.32) {
                    self.skill_hit(source, target, damage, false);
                    self.weaken(target, (0.4, 2.5), 0.);
                    self.mark_tag(source, target, 5.);
                    let position = self.units[target].position;
                    self.fx(source, Shape::Mark, position, position, 25., 0.65);
                }
            }
            (Veyrion, 2) => {
                self.shield(source, source, hero.stats().health * 0.22, 4.);
                self.units[source].status.guard.set(0.25, 4.);
                self.units[source].status.reflect = 4.;
            }
            (Veyrion, 1) => {
                if let Some(target) = self
                    .line_targets(source, direction, 229., 13.)
                    .first()
                    .copied()
                {
                    let stun = if self.tagged(source, target) { 1. } else { 0. };
                    self.pull(source, target, stun);
                    let damage = physical(self, 15., 0.64);
                    self.skill_hit(source, target, damage, false);
                } else {
                    self.fx(
                        source,
                        Shape::Beam,
                        origin,
                        origin + direction * 229.,
                        11.,
                        0.42,
                    );
                }
            }
            (Veyrion, 3) => {
                for target in self.enemies(source, origin, 135.) {
                    self.knockback(origin, target, 69.);
                }
                for ally in self.allies(source, origin, 145.) {
                    self.shield(source, ally, 29., 1.);
                }
                self.place_zone(
                    source,
                    origin,
                    145.,
                    0.12,
                    5.12,
                    0.55,
                    ZoneAction::Renewal {
                        healing: 0.014,
                        guard: 0.30,
                        haste: 0.,
                        barrier: 24.,
                        cleanse: false,
                    },
                );
                self.fx(source, Shape::Phoenix, origin, origin, 149., 0.85);
            }
            (Zyraen, 3) => {
                let Some(target) = self.enemy_target(source, point, 245., false) else {
                    return false;
                };
                self.dash_enemy(source, target);
                let position = self.units[target].position;
                self.fx(source, Shape::Phoenix, origin, position, 69., 0.88);
                self.place_zone(
                    source,
                    position,
                    51.,
                    0.08,
                    0.84,
                    0.19,
                    ZoneAction::Flurry { target },
                );
            }
            _ => return false,
        }
        true
    }

    pub(in crate::gameplay::simulation) fn featured_bot(
        &mut self,
        hero: Hero,
        target: usize,
        ratio: f32,
    ) {
        let bot = self.units[BOT];
        let position = self.units[target].position;
        let distance = bot.position.distance(position);
        let support = matches!(hero, Hero::Luness | Hero::Selune);
        if support && ratio < 0.80 {
            self.cast(BOT, 0, bot.position);
            self.cast(BOT, 2, bot.position);
            self.cast(BOT, 1, bot.position);
        } else if ratio < 0.72 || (distance < 155. && matches!(hero, Hero::Arxen | Hero::Aurelia)) {
            self.cast(BOT, 2, position);
        }

        if self.units[target].alive() && distance < 225. && !self.units[target].status.concealed() {
            if distance > 68. || matches!(hero, Hero::Solviana | Hero::Veyrion | Hero::Aurelia) {
                let aim = if support { bot.position } else { position };
                self.cast(BOT, 1, aim);
            }
            if target == PLAYER && self.elapsed > 7. && (!support || ratio < 0.50) {
                let aim = if support { bot.position } else { position };
                self.cast(BOT, 3, aim);
            }
        }
    }
}
