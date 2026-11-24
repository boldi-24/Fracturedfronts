mod kits;

use super::*;

#[derive(Clone, Copy)]
pub(super) struct SkillHit {
    pub slow: (f32, f32),
    pub reveal: f32,
    pub pierce: bool,
    pub weaken: f32,
    pub stack: bool,
    pub group: u32,
    pub repeated: f32,
}

impl Default for SkillHit {
    fn default() -> Self {
        Self {
            slow: (0., 0.),
            reveal: 0.,
            pierce: false,
            weaken: 0.,
            stack: true,
            group: 0,
            repeated: 1.,
        }
    }
}

#[derive(Clone, Copy)]
pub(super) enum ZoneAction {
    Rain {
        damage: Damage,
        slow: f32,
        marked_bonus: f32,
    },
    Renewal {
        healing: f32,
        guard: f32,
        haste: f32,
        barrier: f32,
        cleanse: bool,
    },
    DawnNova,
    Trap,
    Dive,
    Flurry {
        target: usize,
    },
}

#[derive(Clone, Copy)]
pub(super) struct Zone {
    source: usize,
    point: Point,
    radius: f32,
    remaining: f32,
    next: f32,
    every: f32,
    action: ZoneAction,
}

impl Game {
    fn source_slot(source: usize) -> Option<usize> {
        (source <= BOT).then_some(source)
    }

    fn add_mark(&mut self, source: usize, target: usize, duration: f32, max: u8) -> u8 {
        Self::source_slot(source).map_or(0, |slot| {
            self.units[target].status.marks[slot].add(target, duration, max)
        })
    }

    fn stacks(&self, source: usize, target: usize) -> u8 {
        Self::source_slot(source).map_or(0, |slot| {
            self.units[target].status.marks[slot].count(target)
        })
    }

    fn consume_marks(&mut self, source: usize, target: usize) {
        if let Some(slot) = Self::source_slot(source) {
            self.units[target].status.marks[slot] = Trace::default();
        }
    }

    fn mark_tag(&mut self, source: usize, target: usize, duration: f32) {
        if let Some(slot) = Self::source_slot(source) {
            self.units[target].status.tags[slot] = duration;
        }
    }

    fn tagged(&self, source: usize, target: usize) -> bool {
        Self::source_slot(source).is_some_and(|slot| self.units[target].status.tags[slot] > 0.)
    }

    fn tone(&self, source: usize) -> Tone {
        if self.units[source].kind.hero().is_some_and(Hero::solar) {
            Tone::Solar
        } else {
            Tone::Eclipse
        }
    }

    fn fx(
        &mut self,
        source: usize,
        shape: Shape,
        from: Point,
        to: Point,
        radius: f32,
        duration: f32,
    ) {
        let tone = self.tone(source);
        self.draw_effect(shape, tone, from, to, radius, duration);
    }

    fn draw_effect(
        &mut self,
        shape: Shape,
        tone: Tone,
        from: Point,
        to: Point,
        radius: f32,
        duration: f32,
    ) {
        if self.effects.len() < 144 {
            self.effects.push(Effect {
                from,
                to,
                radius,
                shape,
                tone,
                t: duration,
                duration,
            });
        }
    }

    fn physical(&self, source: usize, base: f32, ratio: f32) -> Damage {
        Damage::Physical(
            base + self.units[source].kind.stats().map_or(0., |stats| stats.ad) * ratio,
        )
    }

    fn magic(&self, source: usize, base: f32, ratio: f32) -> Damage {
        Damage::Magic(base + self.units[source].kind.stats().map_or(0., |stats| stats.ap) * ratio)
    }

    fn weaken(&mut self, target: usize, slow: (f32, f32), stun: f32) {
        let status = &mut self.units[target].status;
        let control = 1. - status.control_resist.value();
        status.slow.set(slow.0, slow.1 * control);
        status.stun = status.stun.max(stun * control);
    }

    fn heal(&mut self, target: usize, amount: f32) {
        let unit = &mut self.units[target];
        if unit.alive() && !unit.kind.structure() {
            unit.health = (unit.health + amount).min(unit.kind.health());
        }
    }

    fn grace(&mut self, source: usize, target: usize) {
        let healing = match self.units[source].kind.hero() {
            Some(Hero::Selune) => {
                self.units[target].status.guard.set(0.1, 4.);
                self.units[target].status.renewal.set(4., 4.);
                true
            }
            Some(Hero::Luness) => {
                self.units[target].status.light = 4.;
                true
            }
            _ => false,
        };
        if healing {
            let position = self.units[target].position;
            self.fx(source, Shape::Mark, position, position, 19., 0.4);
        }
    }

    fn shield(&mut self, source: usize, target: usize, amount: f32, time: f32) {
        let position = self.units[target].position;
        let status = &mut self.units[target].status;
        status.barrier = status.barrier.max(amount);
        status.barrier_time = status.barrier_time.max(time);
        self.fx(source, Shape::Shield, position, position, 26., 0.48);
        self.grace(source, target);
    }

    fn allies(&self, source: usize, point: Point, range: f32) -> Vec<usize> {
        let side = self.units[source].side;
        self.units
            .iter()
            .enumerate()
            .filter(|(_, unit)| {
                unit.alive()
                    && !unit.kind.structure()
                    && unit.side == side
                    && unit.position.distance(point) <= range + unit.kind.radius()
            })
            .map(|(index, _)| index)
            .collect()
    }

    fn enemies(&self, source: usize, point: Point, range: f32) -> Vec<usize> {
        let side = self.units[source].side;
        self.units
            .iter()
            .enumerate()
            .filter(|(_, unit)| {
                unit.alive()
                    && !unit.kind.structure()
                    && unit.side != side
                    && unit.position.distance(point) <= range + unit.kind.radius()
            })
            .map(|(index, _)| index)
            .collect()
    }

    fn enemy_target(
        &self,
        source: usize,
        point: Point,
        range: f32,
        hero_only: bool,
    ) -> Option<usize> {
        let origin = self.units[source].position;
        self.enemies(source, origin, range)
            .into_iter()
            .filter(|&target| {
                !self.units[target].status.concealed()
                    && self.units[target].position.distance(point) <= 75.
                    && (!hero_only || self.units[target].kind.hero().is_some())
            })
            .min_by(|&a, &b| {
                self.units[a]
                    .position
                    .distance(point)
                    .total_cmp(&self.units[b].position.distance(point))
            })
    }

    fn ally_target(&self, source: usize, point: Point) -> usize {
        let origin = self.units[source].position;
        self.allies(source, origin, 295.)
            .into_iter()
            .filter(|&target| self.units[target].position.distance(point) <= 46.)
            .min_by(|&a, &b| {
                self.units[a]
                    .position
                    .distance(point)
                    .total_cmp(&self.units[b].position.distance(point))
            })
            .unwrap_or(source)
    }

    fn chain_heal(&mut self, source: usize, point: Point, amount: f32) {
        let first = self.ally_target(source, point);
        let mut healed = vec![first];
        let mut anchor = first;
        self.heal(first, amount);
        self.grace(source, first);
        self.draw_effect(
            Shape::Beam,
            Tone::Heal,
            self.units[source].position,
            self.units[first].position,
            15.,
            0.55,
        );
        for multiplier in [0.68, 0.45] {
            let origin = self.units[anchor].position;
            let nearest = self
                .allies(source, origin, 125.)
                .into_iter()
                .filter(|target| {
                    !healed.contains(target)
                        && self.units[*target].health < self.units[*target].kind.health()
                })
                .min_by(|a, b| {
                    self.units[*a]
                        .position
                        .distance(origin)
                        .total_cmp(&self.units[*b].position.distance(origin))
                });
            let Some(target) = nearest else {
                break;
            };
            self.heal(target, amount * multiplier);
            self.grace(source, target);
            self.draw_effect(
                Shape::Beam,
                Tone::Heal,
                origin,
                self.units[target].position,
                16.,
                0.55,
            );
            healed.push(target);
            anchor = target;
        }
    }

    fn aim_within(&self, source: usize, point: Point, range: f32) -> Point {
        let origin = self.units[source].position;
        let delta = point - origin;
        (origin + delta.direction() * delta.length().min(range)).lane()
    }

    fn dash_to(&mut self, source: usize, endpoint: Point) {
        let from = self.units[source].position;
        self.units[source].position = endpoint;
        self.fx(source, Shape::Dash, from, endpoint, 18., 0.48);
        if source == PLAYER {
            self.destination = endpoint;
            self.target = None;
        }
    }

    fn dash_enemy(&mut self, source: usize, target: usize) {
        let from = self.units[source].position;
        let to = self.units[target].position;
        self.dash_to(source, (to - (to - from).direction() * 33.).lane());
    }

    fn pull(&mut self, source: usize, target: usize, stun: f32) {
        let from = self.units[source].position;
        let to = self.units[target].position;
        self.units[target].position = (from + (to - from).direction() * 49.).lane();
        self.fx(source, Shape::Beam, from, to, 12., 0.53);
        self.weaken(target, (0., 0.), stun);
    }

    fn knockback(&mut self, origin: Point, target: usize, distance: f32) {
        let direction = (self.units[target].position - origin).direction();
        self.units[target].position = (self.units[target].position + direction * distance).lane();
    }

    fn fan(&mut self, source: usize, direction: Point, range: f32, threshold: f32) -> Vec<usize> {
        let origin = self.units[source].position;
        self.fx(
            source,
            Shape::Arc,
            origin,
            origin + direction * range,
            range,
            0.43,
        );
        self.enemies(source, origin, range)
            .into_iter()
            .filter(|&index| {
                let delta = self.units[index].position - origin;
                (delta.x * direction.x + delta.y * direction.y) >= delta.length() * threshold
            })
            .collect()
    }

    fn line_targets(&self, source: usize, direction: Point, range: f32, width: f32) -> Vec<usize> {
        let origin = self.units[source].position;
        let projection = |point: Point| {
            let delta = point - origin;
            delta.x * direction.x + delta.y * direction.y
        };
        let mut targets: Vec<_> = self
            .enemies(source, origin, range)
            .into_iter()
            .filter(|&index| {
                let unit = self.units[index];
                let delta = unit.position - origin;
                let along = projection(unit.position);
                along >= 0.
                    && along <= range
                    && (delta.x * direction.y - delta.y * direction.x).abs()
                        < width + unit.kind.radius()
            })
            .collect();
        targets.sort_by(|&a, &b| {
            projection(self.units[a].position).total_cmp(&projection(self.units[b].position))
        });
        targets
    }

    fn launch(
        &mut self,
        source: usize,
        direction: Point,
        damage: Damage,
        hit: SkillHit,
        speed: f32,
        life: f32,
    ) {
        let unit = self.units[source];
        self.shots.push(Shot {
            side: unit.side,
            position: unit.position + direction * 18.,
            velocity: direction * speed,
            source,
            damage,
            remaining: life,
            skill: Some(hit),
            pierced: 0,
        });
    }

    fn place_zone(
        &mut self,
        source: usize,
        point: Point,
        radius: f32,
        wait: f32,
        duration: f32,
        every: f32,
        action: ZoneAction,
    ) {
        self.zones.push(Zone {
            source,
            point,
            radius,
            remaining: duration,
            next: wait,
            every,
            action,
        });
    }

    pub(super) fn update_zones(&mut self, dt: f32) {
        let mut pending = std::mem::take(&mut self.zones);
        for mut zone in pending.drain(..) {
            zone.remaining -= dt;
            zone.next -= dt;
            if zone.next > 0. {
                if zone.remaining > 0. {
                    self.zones.push(zone);
                }
                continue;
            }

            let Zone {
                source,
                point,
                radius,
                action,
                ..
            } = zone;
            let pulse = match action {
                ZoneAction::Rain { .. } => Some(Shape::Rain),
                ZoneAction::Renewal { .. } => Some(Shape::Shield),
                ZoneAction::Flurry { .. } => Some(Shape::Arc),
                _ => None,
            };
            if let Some(shape) = pulse {
                self.fx(source, shape, point, point, radius, zone.every.min(0.65));
            }
            let mut repeat = true;
            match action {
                ZoneAction::Rain {
                    damage,
                    slow,
                    marked_bonus,
                } => {
                    for target in self.enemies(source, point, radius) {
                        let factor = if self.tagged(source, target) {
                            1. + marked_bonus
                        } else {
                            1.
                        };
                        self.skill_hit(source, target, Self::scale_damage(damage, factor), true);
                        self.weaken(target, (slow, 0.9), 0.);
                    }
                    if self.units[source].kind.hero() == Some(Hero::Nyxalune) && radius > 110. {
                        for target in self.allies(source, point, radius) {
                            self.shield(source, target, 24., 1.);
                        }
                        if zone.remaining <= zone.every {
                            self.fx(source, Shape::Phoenix, point, point, radius + 12., 0.55);
                            for target in self.enemies(source, point, radius) {
                                let burst = self.magic(source, 32., 0.42);
                                self.skill_hit(source, target, burst, true);
                            }
                            repeat = false;
                        }
                    }
                }
                ZoneAction::Renewal {
                    healing,
                    guard,
                    haste,
                    barrier,
                    cleanse,
                } => {
                    for target in self.allies(source, point, radius) {
                        self.heal(target, healing * self.units[target].kind.health());
                        self.grace(source, target);
                        if barrier > 0. {
                            self.shield(source, target, barrier, 0.9);
                        }
                        let status = &mut self.units[target].status;
                        status.guard.set(guard, 0.9);
                        status.haste.set(haste, 0.9);
                        if cleanse {
                            status.clear_one();
                        }
                    }
                }
                ZoneAction::DawnNova => {
                    repeat = false;
                    self.fx(source, Shape::Burst, point, point, radius, 0.60);
                    for target in self.enemies(source, point, radius) {
                        self.skill_hit(source, target, self.magic(source, 44., 0.98), true);
                        self.weaken(target, (0., 0.), 1.25);
                    }
                }
                ZoneAction::Trap => {
                    let targets = self.enemies(source, point, radius);
                    if !targets.is_empty() {
                        self.fx(source, Shape::Trap, point, point, radius + 20., 0.62);
                        for target in targets {
                            let damage = self.physical(source, 13., 0.55);
                            self.skill_hit(source, target, damage, false);
                            self.units[target].status.revealed = 3.;
                            self.weaken(target, (0.4, 2.5), 0.);
                        }
                        repeat = false;
                    }
                }
                ZoneAction::Dive => {
                    repeat = false;
                    if self.units[source].alive() {
                        self.units[source].status.hidden = 0.;
                        self.dash_to(source, point);
                        self.fx(source, Shape::Phoenix, point, point, radius, 0.7);
                        for target in self.enemies(source, point, radius) {
                            if self.stacks(source, target) >= 3
                                && self.units[target].health
                                    < self.units[target].kind.health() * 0.25
                            {
                                self.units[target].status.barrier = 0.;
                                self.hurt(source, target, Damage::Physical(2000.));
                            } else {
                                let damage = self.physical(source, 44., 0.85);
                                self.skill_hit(source, target, damage, true);
                            }
                        }
                    }
                }
                ZoneAction::Flurry { target } => {
                    repeat = false;
                    if self.units[source].alive() && self.units[target].alive() {
                        let position = self.units[target].position;
                        self.fx(
                            source,
                            Shape::Beam,
                            self.units[source].position,
                            position,
                            12.,
                            0.43,
                        );
                        let damage = self.physical(source, 13., 0.38);
                        self.skill_hit(source, target, damage, true);
                        if !self.units[target].alive() {
                            self.units[source].r = self.units[source].r.min(32.);
                        } else if zone.remaining > zone.every {
                            repeat = true;
                        }
                    }
                }
            }
            if repeat && zone.remaining > 0. {
                zone.next += zone.every;
                self.zones.push(zone);
            }
        }
    }

    fn scale_damage(damage: Damage, factor: f32) -> Damage {
        match damage {
            Damage::Physical(raw) => Damage::Physical(raw * factor),
            Damage::Magic(raw) => Damage::Magic(raw * factor),
        }
    }

    pub(super) fn update_projectiles(&mut self, dt: f32) {
        let mut index = 0;
        while index < self.shots.len() {
            let shot = &mut self.shots[index];
            let from = shot.position;
            shot.remaining -= dt;
            shot.position = shot.position + shot.velocity * dt;
            let segment = shot.position - from;
            let len = segment.length();
            let direction = segment.direction();
            let hit = self
                .units
                .iter()
                .enumerate()
                .filter(|(i, unit)| {
                    unit.alive()
                        && unit.side != shot.side
                        && !unit.kind.structure()
                        && shot.pierced & (1 << i) == 0
                })
                .filter_map(|(i, unit)| {
                    let delta = unit.position - from;
                    let projection = (delta.x * direction.x + delta.y * direction.y).clamp(0., len);
                    (unit.position.distance(from + direction * projection)
                        <= unit.kind.radius() + 7.)
                        .then_some((i, projection))
                })
                .min_by(|(_, a), (_, b)| a.total_cmp(b))
                .map(|(i, _)| i);

            if let Some(target) = hit {
                let source = shot.source;
                let damage = shot.damage;
                let skill = shot.skill;
                let keep = skill.is_some_and(|hit| hit.pierce);
                let already_hit = skill
                    .is_some_and(|hit| hit.group != 0 && self.projectile_hits[target] == hit.group);
                if let Some(skill) = skill {
                    if skill.group != 0 {
                        self.projectile_hits[target] = skill.group;
                    }
                }
                if keep {
                    shot.pierced |= 1 << target;
                    index += 1;
                } else {
                    self.shots.swap_remove(index);
                }
                if let Some(skill) = skill {
                    let damage =
                        Self::scale_damage(damage, if already_hit { skill.repeated } else { 1. });
                    self.projectile_hit(source, target, damage, skill);
                } else {
                    self.hurt(source, target, damage);
                }
            } else if shot.remaining <= 0. {
                self.shots.swap_remove(index);
            } else {
                index += 1;
            }
        }
    }

    fn projectile_hit(&mut self, source: usize, target: usize, damage: Damage, hit: SkillHit) {
        let position = self.units[target].position;
        self.fx(source, Shape::Burst, position, position, 22., 0.3);
        self.skill_hit(source, target, damage, true);
        self.weaken(target, hit.slow, 0.);
        self.units[target].status.revealed = self.units[target].status.revealed.max(hit.reveal);
        if hit.weaken > 0. {
            self.units[target].status.weaken_magic.set(hit.weaken, 3.);
        }
        if hit.stack && self.units[source].kind.hero() == Some(Hero::Aurelia) {
            if let Some(slot) = Self::source_slot(source) {
                self.focus[slot].add(target, 4., 5);
            }
        }
    }

    fn skill_hit(&mut self, source: usize, target: usize, damage: Damage, mark: bool) {
        self.hurt(source, target, damage);
        if mark && self.units[target].alive() {
            self.offensive_mark(source, target, true);
        }
    }

    fn offensive_mark(&mut self, source: usize, target: usize, from_skill: bool) {
        let Some(hero) = self.units[source].kind.hero() else {
            return;
        };
        let position = self.units[target].position;
        match hero {
            Hero::Elorim if from_skill && self.units[target].status.resonance[source] <= 0. => {
                self.units[target].status.resonance[source] = 0.72;
                self.fx(source, Shape::Mark, position, position, 19., 0.72);
            }
            Hero::Kaelith | Hero::Solviana | Hero::Nyxalune | Hero::Zyraen
                if from_skill || matches!(hero, Hero::Kaelith | Hero::Solviana) =>
            {
                let count = self.add_mark(source, target, 5., 3);
                if count == 3 {
                    match hero {
                        Hero::Kaelith if !self.tagged(source, target) => {
                            self.mark_tag(source, target, 3.);
                            self.units[target].status.revealed = 3.;
                            self.hurt(source, target, self.physical(source, 10., 0.35));
                        }
                        Hero::Solviana => {
                            self.consume_marks(source, target);
                            self.weaken(target, (0.3, 1.25), 0.);
                            self.hurt(source, target, self.physical(source, 10., 0.3));
                        }
                        Hero::Nyxalune => {
                            self.consume_marks(source, target);
                            self.units[target].status.revealed = 2.;
                            self.hurt(source, target, self.magic(source, 10., 0.36));
                        }
                        _ => {}
                    }
                }
                self.fx(
                    source,
                    Shape::Mark,
                    position,
                    position,
                    19. + 3. * count as f32,
                    0.38,
                );
            }
            _ => {}
        }
    }

    pub(super) fn basic_damage(&mut self, source: usize, victim: usize, raw: f32) -> f32 {
        let mut damage = raw;
        let empowered = self.units[source].status.empowered > 0.;
        if empowered {
            damage += 16.;
            self.units[source].status.empowered = 0.;
        }
        self.units[source].status.hidden = 0.;
        if self.units[victim].kind.structure() {
            return damage;
        }
        let position = self.units[victim].position;
        if let Some(hero) = self.units[source].kind.hero() {
            match hero {
                Hero::Aelor if self.units[source].status.stacks == 3 => {
                    self.units[source].status.stacks = 0;
                    damage += 17.;
                    self.heal(source, 18.);
                    self.fx(source, Shape::Phoenix, position, position, 32., 0.55);
                }
                Hero::Arxen | Hero::Aurelia => {
                    let slot = source;
                    let stacks = self.focus[slot].count(victim);
                    damage *= 1. + stacks as f32 * 0.04;
                    if stacks == 5 && hero == Hero::Aurelia {
                        damage += 15.;
                        self.heal(source, 14.);
                        self.focus[slot] = Trace::default();
                        self.fx(source, Shape::Burst, position, position, 28., 0.5);
                    } else {
                        let next = self.focus[slot].add(victim, 4., 5);
                        if next == 5 && hero == Hero::Arxen {
                            self.units[source].status.frenzy.set(0.1, 4.);
                        }
                        self.fx(
                            source,
                            Shape::Mark,
                            position,
                            position,
                            16. + next as f32 * 2.,
                            0.35,
                        );
                    }
                    if hero == Hero::Arxen && self.tagged(source, victim) {
                        damage *= 1.2;
                    }
                }
                Hero::Zyraen => {
                    if empowered {
                        self.add_mark(source, victim, 5., 3);
                    }
                    let stacks = self.stacks(source, victim);
                    if stacks > 0 {
                        self.consume_marks(source, victim);
                        damage += 5. * stacks as f32;
                        self.heal(source, 3.5 * stacks as f32);
                        self.fx(source, Shape::Burst, position, position, 25., 0.45);
                    }
                }
                _ => {}
            }
        }
        damage
    }

    pub(super) fn after_basic(&mut self, source: usize, victim: usize) {
        if !self.units[victim].kind.structure() {
            if self.units[source].status.light > 0. {
                self.heal(source, 5.);
                let point = self.units[source].position;
                self.draw_effect(Shape::Mark, Tone::Heal, point, point, 17., 0.37);
            }
            if self.units[victim].alive() {
                self.offensive_mark(source, victim, false);
            }
        }
        if self.units[source].status.longshot.value() > 0. {
            self.units[source].status.longshot = effects::Timed::default();
            let from = self.units[source].position;
            let target = self.units[victim].position;
            let direction = (target - from).direction();
            let primary_distance = from.distance(target);
            let behind = self
                .line_targets(source, direction, primary_distance + 130., 10.)
                .into_iter()
                .find(|&index| {
                    index != victim && self.units[index].position.distance(from) > primary_distance
                });
            if let Some(next) = behind {
                let damage = self.physical(source, 0., 0.75);
                self.hurt(source, next, damage);
            }
            self.fx(
                source,
                Shape::Beam,
                from,
                target + direction * 120.,
                8.,
                0.5,
            );
        }
    }

    pub(super) fn defended_damage(&self, source: usize, target: usize, damage: Damage) -> f32 {
        let victim = self.units[target];
        let attacker = self.units[source];
        let mut armor = victim.kind.stats().map_or(0., |stats| stats.armor);
        let mut magic = victim.kind.stats().map_or(0., |stats| stats.magic_resist);
        armor *= 1. + victim.status.armor.value();
        magic *= 1. + victim.status.magic.value();
        let stack_rate = match victim.kind.hero() {
            Some(Hero::Veyrion) => 0.04,
            Some(Hero::Helvara) => 0.03,
            _ => 0.,
        };
        let multiplier = 1. + victim.status.stacks as f32 * stack_rate;
        let grit = if victim.kind.hero() == Some(Hero::Helvara) && victim.status.stacks == 5 {
            0.8
        } else {
            1.
        };
        let (raw, resistance) = match damage {
            Damage::Physical(raw) => (
                raw,
                armor * multiplier * (1. - attacker.status.penetration.value()),
            ),
            Damage::Magic(raw) => (
                raw,
                magic * multiplier * (1. - victim.status.weaken_magic.value()),
            ),
        };
        raw * (100. / (100. + resistance))
            * (1. - victim.status.guard.value())
            * (1. + attacker.status.power.value())
            * grit
    }

    pub(super) fn hurt_passives(&mut self, source: usize, victim: usize, health_lost: f32) {
        if !self.units[victim].alive() || health_lost <= 0. {
            return;
        }
        let kind = self.units[victim].kind.hero();
        let max = self.units[victim].kind.health();
        let enemy_hero = self.units[source].kind.hero().is_some();
        let status = &mut self.units[victim].status;
        match kind {
            Some(Hero::Aelor) => status.stacks = (status.stacks + 1).min(3),
            Some(Hero::Helvara) => {
                if status.stacks == 5 {
                    status.stacks = 0;
                } else {
                    status.stacks += 1;
                }
            }
            Some(Hero::Veyrion) if enemy_hero => {
                if status.stacks == 5 && health_lost >= max * 0.11 {
                    status.stacks = 0;
                    self.shield(victim, victim, max * 0.18, 3.);
                } else {
                    status.stacks = (status.stacks + 1).min(5);
                }
            }
            _ => {}
        }
        if matches!(kind, Some(Hero::Aelor | Hero::Helvara | Hero::Veyrion)) {
            let at = self.units[victim].position;
            self.fx(victim, Shape::Mark, at, at, 28., 0.35);
        }
        if !self.reflecting
            && self.units[victim].status.reflect > 0.
            && self.tagged(victim, source)
            && self.units[source].alive()
        {
            self.reflecting = true;
            let from = self.units[victim].position;
            let to = self.units[source].position;
            self.fx(victim, Shape::Beam, from, to, 7., 0.35);
            self.hurt(victim, source, Damage::Magic(health_lost * 0.25));
            self.reflecting = false;
        }
    }

    pub(super) fn update_effects(&mut self, index: usize, dt: f32) {
        let unit = self.units[index];
        self.units[index].update(dt);
        if !unit.alive() {
            return;
        }
        for (source, remaining) in unit.status.resonance.into_iter().enumerate() {
            if remaining > 0. {
                self.units[index].status.resonance[source] = (remaining - dt).max(0.);
                if remaining <= dt {
                    self.fx(source, Shape::Burst, unit.position, unit.position, 32., 0.36);
                    self.hurt(source, index, self.magic(source, 6., 0.24));
                }
            }
        }
        if unit.status.renewal.value() > 0. {
            self.heal(index, unit.status.renewal.value() * dt);
        }
        if unit.status.burst > 0. && self.units[index].status.burst <= 0. {
            self.draw_effect(
                Shape::Burst,
                Tone::Heal,
                unit.position,
                unit.position,
                92.,
                0.54,
            );
            for ally in self.allies(index, unit.position, 92.) {
                self.heal(ally, 19.);
            }
        }
        if unit.status.aelor_fire > 0. && self.units[index].status.trail_emit <= 0. {
            self.units[index].status.trail_emit = 0.48;
            self.place_zone(
                index,
                unit.position,
                39.,
                0.25,
                1.05,
                0.56,
                ZoneAction::Rain {
                    damage: Damage::Physical(9.),
                    slow: 0.,
                    marked_bonus: 0.,
                },
            );
        }
    }
}
