use crate::inventory::Inventory;
use std::cell::Cell;

pub const HOME: u16 = 100;
pub const PLAY: u16 = 101;
pub const INVENTORY: u16 = 102;
pub const SHOP: u16 = 103;
pub const PROFILE: u16 = 104;
pub const BOT_DUEL: u16 = 105;

pub struct Menu {
    screen: Cell<u16>,
    selected_duel: Cell<bool>,
    pub inventory: Inventory,
}

impl Menu {
    pub const fn new() -> Self {
        Self {
            screen: Cell::new(HOME),
            selected_duel: Cell::new(false),
            inventory: Inventory::new(),
        }
    }

    pub fn screen(&self) -> u16 {
        self.screen.get()
    }

    pub fn selected_duel(&self) -> bool {
        self.selected_duel.get()
    }

    pub fn choose(&self, action: u16) -> bool {
        match action {
            HOME | PLAY | INVENTORY | SHOP | PROFILE => self.screen.set(action),
            BOT_DUEL if self.screen() == PLAY => self.selected_duel.set(true),
            _ => return false,
        }
        true
    }
}
