#[path = "../../../characters/elorim/abilities.rs"]
mod elorim_kit;
#[path = "../../../characters/aelor/abilities.rs"]
mod aelor_kit;
#[path = "../../../characters/arxen/abilities.rs"]
mod arxen_kit;
#[path = "../../../characters/aurelia/abilities.rs"]
mod aurelia_kit;
#[path = "../../../characters/helvara/abilities.rs"]
mod helvara_kit;
#[path = "../../../characters/kaelith/abilities.rs"]
mod kaelith_kit;
#[path = "../../../characters/zyrean/abilities.rs"]
mod zyrean_kit;
#[path = "../../../characters/luness/abilities.rs"]
mod luness_kit;
#[path = "../../../characters/selune/abilities.rs"]
mod selune_kit;
#[path = "../../../characters/nyxalune/abilities.rs"]
mod nyxalune_kit;
#[path = "../../../characters/solviana/abilities.rs"]
mod solviana_kit;
#[path = "../../../characters/veyrion/abilities.rs"]
mod veyrion_kit;
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
        match hero {
            Hero::Elorim => elorim_kit::cast(self, source, slot, point),
            Hero::Aelor => aelor_kit::cast(self, source, slot, point),
            Hero::Arxen => arxen_kit::cast(self, source, slot, point),
            Hero::Aurelia => aurelia_kit::cast(self, source, slot, point),
            Hero::Helvara => helvara_kit::cast(self, source, slot, point),
            Hero::Kaelith => kaelith_kit::cast(self, source, slot, point),
            Hero::Zyraen => zyrean_kit::cast(self, source, slot, point),
            Hero::Luness => luness_kit::cast(self, source, slot, point),
            Hero::Selune => selune_kit::cast(self, source, slot, point),
            Hero::Nyxalune => nyxalune_kit::cast(self, source, slot, point),
            Hero::Solviana => solviana_kit::cast(self, source, slot, point),
            Hero::Veyrion => veyrion_kit::cast(self, source, slot, point),
            Hero::Taran | Hero::Nera => false,
        }
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
