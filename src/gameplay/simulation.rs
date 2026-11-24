#[path = "../characters/taran/abilities.rs"]
mod taran_kit;
#[path = "../characters/nera/abilities.rs"]
mod nera_kit;
mod abilities;
mod effects;

use super::geometry::{LANE_CENTER, Point, WORLD_WIDTH};
use crate::characters::roster::Hero;
use crate::characters::{Stats, nera, taran};
use abilities::{SkillHit, Zone};
pub(crate) use effects::{Effect, Shape, Tone};
use effects::{Status, Trace};

const TOWER_WINDUP: f32 = 0.70;
const WAVE_INTERVAL: f32 = 16.;
const MATCH_LENGTH: f32 = 20. * 60.;

pub(crate) const PLAYER: usize = 0;
const BOT: usize = 1;
pub(crate) const BLUE_TOWER: usize = 2;
pub(crate) const RED_TOWER: usize = 3;
const BLUE_NEXUS: usize = 8;
const RED_NEXUS: usize = 9;
const FIRST_MINION: usize = 10;
const MINIONS_PER_SIDE: usize = 24;
const UNIT_COUNT: usize = FIRST_MINION + 2 * MINIONS_PER_SIDE;
pub(crate) const RESULT_DRAW: u32 = 3;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Side {
    Blue,
    Red,
}

impl Side {
    pub(crate) fn towers(self) -> [usize; 3] {
        match self {
            Self::Blue => [BLUE_TOWER, 4, 6],
            Self::Red => [RED_TOWER, 5, 7],
        }
    }

    pub(crate) fn nexus(self) -> usize {
        if self == Self::Blue {
            BLUE_NEXUS
        } else {
            RED_NEXUS
        }
    }

    fn enemy(self) -> Self {
        if self == Self::Blue {
            Self::Red
        } else {
            Self::Blue
        }
    }

    pub(crate) fn faction(self) -> &'static str {
        if self == Self::Blue {
            "Księżycowa"
        } else {
            "Słoneczna"
        }
    }

    fn base_point(self, distance: f32) -> Point {
        Point::at(
            if self == Self::Blue {
                distance
            } else {
                WORLD_WIDTH - distance
            },
            LANE_CENTER,
        )
    }

    fn home(self) -> Point {
        self.base_point(105.)
    }
    fn advance(self) -> f32 {
        if self == Self::Blue { 1. } else { -1. }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Kind {
    Taran,
    Nera,
    Featured(Hero),
    Tower,
    Nexus,
    Minion,
}

#[derive(Clone, Copy)]
enum Damage {
    Physical(f32),
    Magic(f32),
}

impl Damage {
    #[cfg(test)]
    fn after_resistance(self, target: Option<&Stats>) -> f32 {
        let (raw, resistance) = match self {
            Self::Physical(raw) => (raw, target.map_or(0., |stats| stats.armor)),
            Self::Magic(raw) => (raw, target.map_or(0., |stats| stats.magic_resist)),
        };
        raw * (100. / (100. + resistance))
    }
}

impl Kind {
    pub(crate) fn structure(self) -> bool {
        matches!(self, Self::Tower | Self::Nexus)
    }

    pub(crate) fn hero(self) -> Option<Hero> {
        match self {
            Self::Taran => Some(Hero::Taran),
            Self::Nera => Some(Hero::Nera),
            Self::Featured(hero) => Some(hero),
            Self::Tower | Self::Nexus | Self::Minion => None,
        }
    }

    fn from_hero(hero: Hero) -> Self {
        match hero {
            Hero::Taran => Self::Taran,
            Hero::Nera => Self::Nera,
            featured => Self::Featured(featured),
        }
    }

    fn stats(self) -> Option<&'static Stats> {
        match self {
            Self::Taran => Some(&taran::STATS),
            Self::Nera => Some(&nera::STATS),
            Self::Featured(hero) => Some(hero.stats()),
            Self::Tower | Self::Nexus | Self::Minion => None,
        }
    }
    fn radius(self) -> f32 {
        match self {
            Self::Tower => 29.,
            Self::Nexus => 44.,
            Self::Minion => 10.,
            _ => 20.,
        }
    }
    pub(crate) fn health(self) -> f32 {
        match self {
            Self::Taran => taran::STATS.health,
            Self::Nera => nera::STATS.health,
            Self::Featured(hero) => hero.stats().health,
            Self::Tower => 3200.,
            Self::Nexus => 4000.,
            Self::Minion => 44.,
        }
    }
    fn speed(self) -> f32 {
        match self {
            Self::Taran => taran::STATS.speed,
            Self::Nera => nera::STATS.speed,
            Self::Featured(hero) => hero.stats().speed,
            Self::Tower | Self::Nexus => 0.,
            Self::Minion => 52.,
        }
    }
    fn reach(self) -> f32 {
        match self {
            Self::Taran => taran::STATS.reach,
            Self::Nera => nera::STATS.reach,
            Self::Featured(hero) => hero.stats().reach,
            Self::Tower => 135.,
            Self::Nexus => 0.,
            Self::Minion => 23.,
        }
    }
    fn attack(self) -> (f32, f32) {
        match self {
            Self::Taran => (taran::STATS.ad, taran::STATS.attack_speed.recip()),
            Self::Nera => (nera::STATS.ad, nera::STATS.attack_speed.recip()),
            Self::Featured(hero) => (hero.stats().ad, hero.stats().attack_speed.recip()),
            Self::Tower => (36., 0.45),
            Self::Nexus => (0., 1.),
            Self::Minion => (9., 1.05),
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct Unit {
    pub(crate) kind: Kind,
    pub(crate) side: Side,
    pub(crate) position: Point,
    pub(crate) health: f32,
    attack: f32,
    pub(crate) minion_ranged: bool,
    tower_target: Option<usize>,
    tower_charge: f32,
    pub(crate) facing: Point,
    pub(crate) moving: bool,
    pub(crate) attack_animation: f32,
    pub(crate) cast_animation: f32,
    pub(crate) cast_slot: u32,
    pub(crate) cast_point: Point,
    pub(crate) q: f32,
    pub(crate) e: f32,
    pub(crate) w: f32,
    pub(crate) r: f32,
    pub(crate) status: Status,
    pub(crate) shield: f32,
    pub(crate) respawn: f32,
}

impl Unit {
    fn new(kind: Kind, side: Side, position: Point) -> Self {
        Self {
            kind,
            side,
            position,
            health: kind.health(),
            attack: 0.,
            minion_ranged: false,
            tower_target: None,
            tower_charge: 0.,
            facing: Point::at(side.advance(), 0.),
            moving: false,
            attack_animation: 0.,
            cast_animation: 0.,
            cast_slot: 0,
            cast_point: position,
            q: 0.,
            e: 0.,
            w: 0.,
            r: 0.,
            status: Status::default(),
            shield: 0.,
            respawn: 0.,
        }
    }
    pub(crate) fn attack_range(&self) -> f32 {
        self.reach() + self.status.longshot.value()
    }

    fn reach(&self) -> f32 {
        if self.minion_ranged { 102. } else { self.kind.reach() }
    }
    pub(crate) fn alive(self) -> bool {
        self.health > 0.
    }
    fn update(&mut self, dt: f32) {
        self.status.tick(dt);
        self.moving = false;
        self.attack_animation = (self.attack_animation - dt / 0.35).max(0.);
        self.cast_animation = (self.cast_animation - dt / 0.72).max(0.);
        for timer in [
            &mut self.attack,
            &mut self.q,
            &mut self.w,
            &mut self.e,
            &mut self.r,
            &mut self.shield,
        ] {
            *timer = (*timer - dt).max(0.);
        }
    }
    fn walk(&mut self, destination: Point, dt: f32) {
        if self.status.stun > 0. {
            return;
        }
        let delta = destination - self.position;
        let speed =
            self.kind.speed() * (1. + self.status.haste.value()) * (1. - self.status.slow.value());
        let next = (self.position + delta.direction() * delta.length().min(speed * dt)).lane();
        let motion = next - self.position;
        self.moving = motion.length() > 0.01;
        if self.moving && self.cast_animation < 0.7 {
            self.facing = motion.direction();
        }
        self.position = next;
    }
}

#[derive(Clone)]
pub(crate) struct Shot {
    pub(crate) side: Side,
    pub(crate) position: Point,
    pub(crate) velocity: Point,
    source: usize,
    damage: Damage,
    remaining: f32,
    skill: Option<SkillHit>,
    pierced: u64,
}

impl Shot {
    pub(crate) fn spell_tone(&self, game: &Game) -> Option<Tone> {
        self.skill.map(|_| {
            if game.units[self.source].kind.hero().is_some_and(Hero::solar) {
                Tone::Solar
            } else {
                Tone::Eclipse
            }
        })
    }
}

#[derive(Clone)]
pub(crate) struct Bolt {
    pub(crate) side: Side,
    pub(crate) from: Point,
    pub(crate) position: Point,
    pub(crate) progress: f32,
    pub(crate) from_tower: bool,
    source: usize,
    target: usize,
    damage: Damage,
    duration: f32,
}

#[derive(Clone)]
pub(crate) struct Mark {
    pub(crate) from: Point,
    pub(crate) to: Point,
    pub(crate) kind: u8,
    remaining: f32,
}

#[derive(Clone)]
pub(crate) struct Game {
    pub(crate) units: [Unit; UNIT_COUNT],
    pub(crate) destination: Point,
    pub(crate) target: Option<usize>,
    next_wave: f32,
    pub(crate) elapsed: f32,
    pub(crate) result: u32,
    retreat: bool,
    pub(crate) shots: Vec<Shot>,
    pub(crate) bolts: Vec<Bolt>,
    pub(crate) marks: Vec<Mark>,
    pub(crate) effects: Vec<Effect>,
    zones: Vec<Zone>,
    focus: [Trace; 2],
    projectile_group: u32,
    projectile_hits: [u32; UNIT_COUNT],
    reflecting: bool,
}

impl Game {
    #[cfg(test)]
    pub(crate) fn new(ranged: bool) -> Self {
        Self::with_hero(if ranged { Hero::Nera } else { Hero::Taran })
    }

    pub(crate) fn with_hero(hero: Hero) -> Self {
        let opponent = match hero {
            Hero::Taran => Hero::Nera,
            Hero::Nera => Hero::Taran,
            other => Hero::ALL[((other as usize - 2 + 5) % (Hero::ALL.len() - 2)) + 2],
        };
        let mut units = [Unit::new(Kind::Minion, Side::Blue, Point::default()); UNIT_COUNT];
        for unit in &mut units {
            unit.health = 0.;
        }
        units[PLAYER] = Unit::new(Kind::from_hero(hero), Side::Blue, Side::Blue.home());
        units[BOT] = Unit::new(Kind::from_hero(opponent), Side::Red, Side::Red.home());
        for side in [Side::Blue, Side::Red] {
            for (index, distance) in side.towers().into_iter().zip([1330., 865., 420.]) {
                units[index] = Unit::new(Kind::Tower, side, side.base_point(distance));
            }
            units[side.nexus()] = Unit::new(Kind::Nexus, side, side.base_point(195.));
        }
        let mut game = Self {
            units,
            destination: Side::Blue.home(),
            target: None,
            next_wave: WAVE_INTERVAL,
            elapsed: 0.,
            result: 0,
            retreat: false,
            shots: Vec::new(),
            bolts: Vec::new(),
            marks: Vec::new(),
            effects: Vec::new(),
            zones: Vec::new(),
            focus: [Trace::default(); 2],
            projectile_group: 0,
            projectile_hits: [0; UNIT_COUNT],
            reflecting: false,
        };
        game.spawn_wave();
        game
    }

    fn spawn_wave(&mut self) {
        for side in [Side::Blue, Side::Red] {
            let start = FIRST_MINION
                + if side == Side::Red {
                    MINIONS_PER_SIDE
                } else {
                    0
                };
            let spawn_x = side.base_point(248.).x;
            for (offset, ranged) in [(-28., false), (28., false), (0., true)] {
                let Some(slot) = self.units[start..start + MINIONS_PER_SIDE]
                    .iter_mut()
                    .find(|unit| !unit.alive() && unit.status.fallen <= 0.)
                else {
                    break;
                };
                let x = spawn_x - if ranged { 32. * side.advance() } else { 0. };
                *slot = Unit::new(Kind::Minion, side, Point::at(x, LANE_CENTER + offset));
                slot.minion_ranged = ranged;
            }
        }
    }

    /// First surviving structure along the approach from the opponent's base.
    pub(crate) fn objective(&self, side: Side) -> usize {
        side.towers()
            .into_iter()
            .find(|&index| self.units[index].alive())
            .unwrap_or(side.nexus())
    }

    fn exposed(&self, index: usize) -> bool {
        let unit = self.units[index];
        !unit.kind.structure() || index == self.objective(unit.side)
    }

    pub(crate) fn stop_player(&mut self) {
        self.target = None;
        self.destination = self.units[PLAYER].position;
        self.units[PLAYER].moving = false;
    }

    pub(crate) fn attack_unit_near(&mut self, cursor: Point) {
        let player = self.units[PLAYER];
        if self.result != 0 || !player.alive() { return; }
        let closest = self.units.iter().enumerate()
            .filter(|(_, u)| u.side != player.side && !u.kind.structure()
                && u.alive() && !u.status.concealed())
            .min_by(|(_, a), (_, b)| a.position.distance(cursor).total_cmp(&b.position.distance(cursor)));
        if let Some((index, _)) = closest {
            self.target = Some(index);
        }
    }

    pub(crate) fn command(&mut self, point: Point) {
        if self.result != 0 || !self.units[PLAYER].alive() {
            return;
        }
        let valid_hit = |unit: &&Unit| {
            unit.alive()
                && unit.side == Side::Red
                && !unit.status.concealed()
                && unit.position.distance(point) <= unit.kind.radius() + 9.
        };
        self.target = self
            .units
            .iter()
            .enumerate()
            .filter(|(i, unit)| self.exposed(*i) && valid_hit(unit))
            .min_by(|(_, a), (_, b)| {
                a.position
                    .distance(point)
                    .total_cmp(&b.position.distance(point))
            })
            .map(|(index, _)| index);
        self.destination = point.lane();
    }

    fn enemy_in(&self, source: usize, range: f32, minions_only: bool) -> Option<usize> {
        let actor = self.units[source];
        self.units
            .iter()
            .enumerate()
            .filter(|(i, unit)| {
                unit.alive()
                    && self.exposed(*i)
                    && unit.side != actor.side
                    && !unit.status.concealed()
                    && (!minions_only || unit.kind == Kind::Minion)
                    && actor.position.distance(unit.position) <= range + unit.kind.radius()
            })
            .min_by(|(_, a), (_, b)| {
                actor
                    .position
                    .distance(a.position)
                    .total_cmp(&actor.position.distance(b.position))
            })
            .map(|(index, _)| index)
    }

    fn reach(&self, a: usize, b: usize) -> bool {
        self.units[a].position.distance(self.units[b].position)
            <= self.units[a].attack_range()
                + self.units[b].kind.radius()
    }

    fn mark(&mut self, from: Point, to: Point, kind: u8, remaining: f32) {
        if self.marks.len() < 48 {
            self.marks.push(Mark {
                from,
                to,
                kind,
                remaining,
            });
        }
    }

    fn strike(&mut self, source: usize, victim: usize) {
        if self.units[source].attack > 0.
            || self.units[source].status.stun > 0.
            || !self.exposed(victim)
            || self.units[victim].status.concealed()
        {
            return;
        }
        let (base_damage, cooldown) = if self.units[source].minion_ranged {
            (8., 1.30)
        } else {
            self.units[source].kind.attack()
        };
        let damage = self.basic_damage(source, victim, base_damage);
        self.units[source].attack = cooldown / (1. + self.units[source].status.frenzy.value());
        let from = self.units[source].position;
        let to = self.units[victim].position;
        self.units[source].facing = (to - from).direction();
        self.units[source].attack_animation = 1.;
        if self.units[source].kind == Kind::Tower || self.units[source].minion_ranged {
            let from_tower = self.units[source].kind == Kind::Tower;
            let speed = if from_tower { 350. } else { 255. };
            self.bolts.push(Bolt {
                source,
                target: victim,
                side: self.units[source].side,
                from,
                position: from,
                progress: 0.,
                from_tower,
                damage: Damage::Physical(damage),
                duration: (from.distance(to) / speed).clamp(0.18, 0.65),
            });
            return;
        }
        let kind = if self.units[source].side == Side::Blue {
            0
        } else {
            1
        };
        self.mark(
            self.units[source].position,
            self.units[victim].position,
            kind,
            0.17,
        );
        self.hurt(source, victim, Damage::Physical(damage));
        self.after_basic(source, victim);
    }

    fn update_bolts(&mut self, dt: f32) {
        let mut i = 0;
        while i < self.bolts.len() {
            let bolt = &mut self.bolts[i];
            let victim = self.units[bolt.target];
            if !victim.alive() {
                self.bolts.swap_remove(i);
                continue;
            }
            bolt.progress = (bolt.progress + dt / bolt.duration).min(1.);
            bolt.position = bolt.from + (victim.position - bolt.from) * bolt.progress;
            if bolt.progress < 1. {
                i += 1;
            } else {
                let bolt = self.bolts.swap_remove(i);
                self.hurt(bolt.source, bolt.target, bolt.damage);
                self.after_basic(bolt.source, bolt.target);
            }
        }
    }

    fn hurt(&mut self, source: usize, victim: usize, damage: Damage) {
        if self.result != 0
            || !self.exposed(victim)
            || !self.units[victim].alive()
            || self.units[victim].status.invulnerable > 0.
        {
            return;
        }
        let damage = self.defended_damage(source, victim, damage);
        let unit = &mut self.units[victim];
        let damage = if unit.shield > 0. {
            damage * 0.45
        } else {
            damage
        };
        let damage = (damage - unit.status.absorb(damage)).max(0.);
        let previous_health = unit.health;
        unit.health = (unit.health - damage).max(0.);
        let health_lost = previous_health - unit.health;
        let can_heal = !unit.kind.structure();
        if unit.health == 0. {
            unit.status.fallen = 3.;
            match victim {
                PLAYER | BOT => unit.respawn = 5.,
                BLUE_NEXUS => self.result = 2,
                RED_NEXUS => self.result = 1,
                _ => {}
            }
            if victim == PLAYER || self.target == Some(victim) {
                self.target = None;
                self.destination = self.units[PLAYER].position;
            }
        }
        if can_heal && health_lost > 0. {
            let attacker = &mut self.units[source];
            if attacker.alive() {
                if let Some(stats) = attacker.kind.stats() {
                    attacker.health = (attacker.health
                        + health_lost * (stats.omnivamp + attacker.status.sustain.value()))
                    .min(stats.health);
                }
            }
        }
        self.hurt_passives(source, victim, health_lost);
    }

    pub(crate) fn cast(&mut self, source: usize, slot: u32, point: Point) {
        let before = self.units[source].cooldown(slot);
        let origin = self.units[source].position;
        self.execute_cast(source, slot, point);
        if self.units[source].cooldown(slot) > before {
            let caster = &mut self.units[source];
            caster.cast_animation = 1.;
            caster.cast_slot = slot;
            caster.cast_point = point;
            if point.distance(origin) > 1. {
                caster.facing = (point - origin).direction();
            }
        }
    }

    fn execute_cast(&mut self, source: usize, slot: u32, point: Point) {
        if let Kind::Featured(hero) = self.units[source].kind {
            self.featured_cast(source, hero, slot, point);
            return;
        }
        match self.units[source].kind {
            Kind::Taran => taran_kit::cast(self, source, slot, point),
            Kind::Nera => nera_kit::cast(self, source, slot, point),
            _ => {}
        }
    }

    pub(crate) fn step(&mut self, dt: f32) {
        if self.result != 0 || !dt.is_finite() || dt <= 0. {
            return;
        }
        let dt = dt.min(0.05);
        self.elapsed += dt;
        self.next_wave -= dt;
        if self.next_wave <= 0. {
            self.spawn_wave();
            self.next_wave += WAVE_INTERVAL;
        }
        self.effects.retain_mut(|effect| {
            effect.t -= dt;
            effect.t > 0.
        });
        for focus in &mut self.focus {
            focus.tick(dt);
        }
        self.marks.retain_mut(|mark| {
            mark.remaining -= dt;
            mark.remaining > 0.
        });

        for i in [PLAYER, BOT] {
            let unit = &mut self.units[i];
            if unit.alive() {
                let home = unit.side.home();
                if unit.position.distance(home) <= 65. {
                    unit.health = (unit.health + 28. * dt).min(unit.kind.health());
                }
            } else {
                unit.respawn -= dt;
                if unit.respawn <= 0. {
                    *unit = Unit::new(unit.kind, unit.side, unit.side.home());
                    if i == PLAYER {
                        self.destination = unit.position;
                    }
                    if i == BOT {
                        self.retreat = false;
                    }
                }
            }
        }

        for index in 0..UNIT_COUNT {
            self.update_effects(index, dt);
        }
        self.update_zones(dt);

        self.update_projectiles(dt);
        self.update_bolts(dt);

        for i in 0..UNIT_COUNT {
            if !self.units[i].alive() || self.result != 0 {
                continue;
            }
            match i {
                PLAYER => self.player_update(dt),
                BOT => self.bot_update(dt),
                _ if self.units[i].kind == Kind::Tower => self.tower_update(i, dt),
                _ if self.units[i].kind == Kind::Nexus => {}
                _ => self.minion_update(i, dt),
            }
        }

        if self.result == 0 && self.elapsed >= MATCH_LENGTH {
            self.result = RESULT_DRAW;
        }
    }

    fn player_update(&mut self, dt: f32) {
        if let Some(target) = self
            .target
            .filter(|&target| self.units[target].alive() && !self.units[target].status.concealed())
        {
            self.move_or_strike(PLAYER, target, dt);
        } else if self.target.is_some() {
            self.stop_player();
        } else {
            self.units[PLAYER].walk(self.destination, dt);
        }
    }

    fn bot_update(&mut self, dt: f32) {
        let bot = self.units[BOT];
        let ratio = bot.health / bot.kind.health();
        if ratio < 0.3 {
            self.retreat = true;
        }
        if ratio > 0.82 {
            self.retreat = false;
        }
        if self.retreat {
            let home = bot.side.home();
            if bot.position.distance(home) > 170. {
                if let Kind::Featured(hero) = bot.kind {
                    self.featured_bot(hero, PLAYER, ratio);
                } else {
                    self.cast(BOT, 1, home);
                }
            }
            self.units[BOT].walk(home, dt);
            return;
        }

        let objective = self.objective(bot.side.enemy());
        let player = self.units[PLAYER];
        let distance = bot.position.distance(player.position);
        let target = if player.alive()
            && !player.status.concealed()
            && distance <= 205.
            && self.covered_tower(BOT, objective, 170.)
        {
            Some(PLAYER)
        } else {
            self.enemy_in(BOT, 126., true)
        };

        if let Some(target) = target {
            let enemy = self.units[target];
            if let Kind::Featured(hero) = bot.kind {
                self.featured_bot(hero, target, ratio);
            }
            let reach = if bot.kind == Kind::Taran { 86. } else { 255. };
            if bot.position.distance(enemy.position) <= reach {
                self.cast(BOT, 0, enemy.position);
            }
            if self.units[target].alive() {
                self.move_or_strike(BOT, target, dt);
            }
            return;
        }

        if self.covered_tower(BOT, objective, 158.) {
            self.move_or_strike(BOT, objective, dt);
        } else {
            let guard_x = self.units[objective].position.x - bot.side.advance() * 172.;
            self.units[BOT].walk(Point::at(guard_x, LANE_CENTER + 27.), dt);
        }
    }

    fn covered_tower(&self, actor: usize, enemy_tower: usize, distance: f32) -> bool {
        let turret = self.units[enemy_tower];
        let unit = self.units[actor];
        turret.kind == Kind::Nexus
            || unit.position.distance(turret.position) >= distance
            || self.units[FIRST_MINION..].iter().any(|minion| {
                minion.alive()
                    && minion.side == unit.side
                    && minion.position.distance(turret.position) < 142.
            })
    }

    /// Target and normalized progress of a shot being prepared.
    pub(crate) fn tower_aim(&self, tower: usize) -> Option<(usize, f32)> {
        let unit = self.units[tower];
        unit.alive().then_some(())?;
        Some((unit.tower_target?, (unit.tower_charge / TOWER_WINDUP).clamp(0., 1.)))
    }

    fn tower_update(&mut self, tower: usize, dt: f32) {
        if self.units[tower].attack > 0. {
            return;
        }
        if let Some(target) = self.units[tower].tower_target {
            let victim = self.units[target];
            if victim.alive() && !victim.status.concealed() && self.reach(tower, target) {
                self.units[tower].tower_charge += dt;
                if self.units[tower].tower_charge < TOWER_WINDUP {
                    return;
                }
                self.strike(tower, target);
                self.units[tower].tower_target = None;
                self.units[tower].tower_charge = 0.;
                return;
            }
        }
        let range = self.units[tower].reach();
        self.units[tower].tower_target = self.enemy_in(tower, range, true)
            .or_else(|| self.enemy_in(tower, range, false));
        self.units[tower].tower_charge = 0.;
    }

    fn minion_update(&mut self, minion: usize, dt: f32) {
        let unit = self.units[minion];
        if let Some(target) = self
            .enemy_in(minion, unit.reach() + 24., true)
            .or_else(|| self.enemy_in(minion, unit.reach() + 16., false))
        {
            self.move_or_strike(minion, target, dt);
        } else {
            let tower = self.units[self.objective(unit.side.enemy())].position;
            self.units[minion].walk(
                Point::at(
                    tower.x,
                    LANE_CENTER + (unit.position.y - LANE_CENTER) * 0.985,
                ),
                dt,
            );
        }
    }

    fn move_or_strike(&mut self, actor: usize, target: usize, dt: f32) {
        if self.reach(actor, target) {
            self.strike(actor, target);
        } else {
            let position = self.units[target].position;
            self.units[actor].walk(position, dt);
        }
    }
}

#[cfg(test)]
mod tests;
