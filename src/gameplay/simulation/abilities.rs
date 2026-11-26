mod kits;

use super::*;

#[derive(Clone, Copy)]
pub(super) struct SkillHit {
    pub group: u32,
    pub repeated: f32,
}

impl Default for SkillHit {
    fn default() -> Self {
        Self {
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

    fn cast_direction(&self, source: usize, point: Point) -> Point {
        let unit = self.units[source];
        let delta = point - unit.position;
        if delta.length() < 1. { Point::at(unit.side.advance(), 0.) } else { delta.direction() }
    }

    fn aim_within(&self, source: usize, point: Point, range: f32) -> Point {
        let origin = self.units[source].position;
        let delta = point - origin;
        (origin + delta.direction() * delta.length().min(range)).lane()
    }

    fn dash_to(&mut self, source: usize, endpoint: Point) {
        let from = self.units[source].position;
        let endpoint = crate::gameplay::map::clear_segment(from, endpoint);
        self.units[source].position = endpoint;
        self.fx(source, Shape::Dash, from, endpoint, 18., 0.48);
        if source == PLAYER {
            self.destination = endpoint;
            self.target = None;
        }
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
            self.fx(source, Shape::Rain, point, point, radius, zone.every.min(0.65));
            let ZoneAction::Rain { damage, slow, marked_bonus } = action;
            for target in self.enemies(source, point, radius) {
                let factor = if self.tagged(source, target) { 1. + marked_bonus } else { 1. };
                self.hurt(source, target, Self::scale_damage(damage, factor));
                self.units[target].status.slow.set(slow, 0.9);
            }
            if zone.remaining > 0. {
                zone.next += zone.every;
                self.zones.push(zone);
            }
        }
    }

    fn scale_damage(damage: Damage, factor: f32) -> Damage {
        match damage {
            Damage::Physical(raw) => Damage::Physical(raw * factor),

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
                .filter(|(_, unit)| {
                    unit.alive()
                        && unit.side != shot.side
                        && !unit.kind.structure()

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
                let already_hit = skill
                    .is_some_and(|hit| hit.group != 0 && self.projectile_hits[target] == hit.group);
                if let Some(skill) = skill {
                    if skill.group != 0 {
                        self.projectile_hits[target] = skill.group;
                    }
                }
                self.shots.swap_remove(index);
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

    fn projectile_hit(&mut self, source: usize, target: usize, damage: Damage, _hit: SkillHit) {
        let position = self.units[target].position;
        self.fx(source, Shape::Burst, position, position, 22., 0.3);
        self.hurt(source, target, damage);
    }

    pub(super) fn basic_damage(&mut self, source: usize, victim: usize, raw: f32) -> f32 {
        if self.units[victim].kind.structure() { return raw; }
        let stacks = self.focus[source].count(victim);
        let next = self.focus[source].add(victim, 4., 5);
        if next == 5 { self.units[source].status.frenzy.set(0.1, 4.); }
        let bonus = if self.tagged(source, victim) { 1.2 } else { 1. };
        raw * (1. + stacks as f32 * 0.04) * bonus
    }

    pub(super) fn after_basic(&mut self, source: usize, victim: usize) {
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

    pub(super) fn defended_damage(&self, _source: usize, target: usize, damage: Damage) -> f32 {
        let Damage::Physical(raw) = damage;
        let armor = self.units[target].kind.stats().map_or(0., |s| s.armor);
        raw * 100. / (100. + armor)
    }
}
