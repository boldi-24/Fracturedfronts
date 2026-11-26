
#[path = "../../../characters/shared/abilities.rs"]
mod shared_kit;
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
    pub(in crate::gameplay::simulation) fn cast_ability(
        &mut self,
        source: usize,
        hero: Hero,
        slot: u32,
        point: Point,
    ) {
        let unit = self.units[source];
        if self.result != 0
            || !unit.alive()
            || slot > 3
            || unit.cooldown(slot) > 0.
        {
            return;
        }
        let profile = hero.profile();
        let index = [0, 2, 1, 3][slot as usize];
        self.units[source].set_cooldown(slot, profile.cooldowns[index]);
        if !shared_kit::cast(self, source, slot, point) {
            self.units[source].set_cooldown(slot, 0.);
        }
    }


    pub(in crate::gameplay::simulation) fn bot_abilities(
        &mut self,
        target: usize,
        ratio: f32,
    ) {
        let bot = self.units[BOT];
        let position = self.units[target].position;
        let distance = bot.position.distance(position);
        if ratio < 0.72 || distance < 155. { self.cast(BOT, 2, position); }
        if self.units[target].alive() && distance < 225. {
            if distance > 68. { self.cast(BOT, 1, position); }
            if target == PLAYER && self.elapsed > 7. { self.cast(BOT, 3, position); }
        }
    }
}
