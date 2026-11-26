mod abilities;
mod effects;

use super::geometry::Point;
#[cfg(test)]
use super::geometry::{LANE_CENTER, WORLD_WIDTH};
use crate::characters::roster::Hero;
use crate::characters::Stats;
use abilities::{SkillHit, Zone};
pub(crate) use effects::{Effect, Shape, Tone};
use effects::{Status, Trace};

const MATCH_LENGTH: f32 = 20. * 60.;

pub(crate) const PLAYER: usize = 0;
const BOT: usize = 1;
const BLUE_NEXUS: usize = 2;
const RED_NEXUS: usize = 3;
const UNIT_COUNT: usize = 4;
pub(crate) const RESULT_DRAW: u32 = 3;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Side {
    Blue,
    Red,
}

impl Side {
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

    fn objective(self) -> Point {
        match self {
            Self::Blue => Point::at(157.*4., 221.*4.),
            Self::Red => Point::at(509.*4., 301.*4.),
        }
    }
    fn home(self) -> Point {
        self.objective() + Point::at(-self.advance()*60., 0.)
    }
    fn advance(self) -> f32 {
        if self == Self::Blue { 1. } else { -1. }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Kind {
    Hero(Hero),
    Nexus,
}

#[derive(Clone, Copy)]
enum Damage {
    Physical(f32),

}



impl Kind {
    pub(crate) fn structure(self) -> bool {
        matches!(self, Self::Nexus)
    }

    pub(crate) fn hero(self) -> Option<Hero> {
        match self {
            Self::Hero(hero) => Some(hero),
            Self::Nexus => None,
        }
    }


    fn stats(self) -> Option<&'static Stats> {
        match self {
            Self::Hero(hero) => Some(hero.stats()),
            Self::Nexus => None,
        }
    }
    fn radius(self) -> f32 {
        match self {
            Self::Nexus => 44.,
            _ => 20.,
        }
    }
    pub(crate) fn health(self) -> f32 {
        match self {
            Self::Hero(hero) => hero.stats().health,
            Self::Nexus => 4000.,
        }
    }
    fn speed(self) -> f32 {
        match self {
            Self::Hero(hero) => hero.stats().speed,
            Self::Nexus => 0.,
        }
    }
    fn reach(self) -> f32 {
        match self {
            Self::Hero(hero) => hero.stats().reach,
            Self::Nexus => 0.,
        }
    }
    fn attack(self) -> (f32, f32) {
        match self {
            Self::Hero(hero) => (hero.stats().ad, 1.0),
            Self::Nexus => (0., 1.),
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
            respawn: 0.,
        }
    }
    pub(crate) fn attack_range(&self) -> f32 {
        self.reach() + self.status.longshot.value()
    }

    fn reach(&self) -> f32 {
        self.kind.reach()
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
        ] {
            *timer = (*timer - dt).max(0.);
        }
    }
    fn walk(&mut self, destination: Point, dt: f32) {
        let delta = super::map::waypoint(self.position, destination) - self.position;
        let speed =
            self.kind.speed() * (1. - self.status.slow.value());
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
    pub(crate) elapsed: f32,
    pub(crate) result: u32,
    retreat: bool,
    pub(crate) shots: Vec<Shot>,
    pub(crate) marks: Vec<Mark>,
    pub(crate) effects: Vec<Effect>,
    zones: Vec<Zone>,
    focus: [Trace; 2],
    projectile_group: u32,
    projectile_hits: [u32; UNIT_COUNT],
}

impl Game {


    pub(crate) fn with_hero(hero: Hero) -> Self {
        let opponent = match hero { Hero::One => Hero::Two, Hero::Two => Hero::One };
        let units = [
            Unit::new(Kind::Hero(hero), Side::Blue, Side::Blue.home()),
            Unit::new(Kind::Hero(opponent), Side::Red, Side::Red.home()),
            Unit::new(Kind::Nexus, Side::Blue, Side::Blue.objective()),
            Unit::new(Kind::Nexus, Side::Red, Side::Red.objective()),
        ];
        Self {
            units,
            destination: Side::Blue.home(),
            target: None,
            elapsed: 0.,
            result: 0,
            retreat: false,
            shots: Vec::new(),
            marks: Vec::new(),
            effects: Vec::new(),
            zones: Vec::new(),
            focus: [Trace::default(); 2],
            projectile_group: 0,
            projectile_hits: [0; UNIT_COUNT],
        }
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
            .filter(|(_, unit)| valid_hit(unit))
            .min_by(|(_, a), (_, b)| {
                a.position
                    .distance(point)
                    .total_cmp(&b.position.distance(point))
            })
            .map(|(index, _)| index);
        self.destination = super::map::on_floor(point.lane());
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
            || self.units[victim].status.concealed()
        {
            return;
        }
        let (base_damage, cooldown) = self.units[source].kind.attack();
        let damage = self.basic_damage(source, victim, base_damage);
        self.units[source].attack = cooldown / (1. + self.units[source].status.frenzy.value());
        let from = self.units[source].position;
        let to = self.units[victim].position;
        self.units[source].facing = (to - from).direction();
        self.units[source].attack_animation = 1.;
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

    fn hurt(&mut self, source: usize, victim: usize, damage: Damage) {
        if self.result != 0
            || !self.units[victim].alive()
        {
            return;
        }
        let damage = self.defended_damage(source, victim, damage);
        let unit = &mut self.units[victim];

        unit.health = (unit.health - damage).max(0.);
        if unit.health == 0. {
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
        if let Kind::Hero(hero) = self.units[source].kind {
            self.cast_ability(source, hero, slot, point);
        }
    }

    pub(crate) fn step(&mut self, dt: f32) {
        if self.result != 0 || !dt.is_finite() || dt <= 0. {
            return;
        }
        let dt = dt.min(0.05);
        self.elapsed += dt;
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
            self.units[index].update(dt);
        }
        self.update_zones(dt);

        self.update_projectiles(dt);


        for i in 0..UNIT_COUNT {
            if !self.units[i].alive() || self.result != 0 {
                continue;
            }
            match i {
                PLAYER => self.player_update(dt),
                BOT => self.bot_update(dt),
                _ => {}
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
                self.bot_abilities(PLAYER, ratio);
            }
            self.units[BOT].walk(home, dt);
            return;
        }

        let objective = bot.side.enemy().nexus();
        let player = self.units[PLAYER];
        let distance = bot.position.distance(player.position);
        let target = if player.alive()
            && !player.status.concealed()
            && distance <= 205.
        {
            Some(PLAYER)
        } else {
            None
        };

        if let Some(target) = target {
            let enemy = self.units[target];
            self.bot_abilities(target, ratio);
            let reach = 255.;
            if bot.position.distance(enemy.position) <= reach {
                self.cast(BOT, 0, enemy.position);
            }
            if self.units[target].alive() {
                self.move_or_strike(BOT, target, dt);
            }
            return;
        }

        self.move_or_strike(BOT, objective, dt);
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
