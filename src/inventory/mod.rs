use crate::characters::roster::Hero;

pub(crate) struct Inventory {
    unlocked_characters: &'static [Hero],
}

impl Inventory {
    pub(crate) const fn new() -> Self {
        Self {
            unlocked_characters: &Hero::ALL,
        }
    }

    pub(crate) fn champions(&self) -> &'static [Hero] {
        self.unlocked_characters
    }
}
