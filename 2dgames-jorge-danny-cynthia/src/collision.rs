use crate::objects::{MovingRect, Rect};

pub struct Contact(ContactID, ContactID);

impl Contact {
    pub fn get_ids(&self) -> (ContactID, ContactID) {
        (self.0, self.1)
    }
}

#[derive(Copy, Clone)]
pub enum ContactID {
    Obstacle,
    Player,
}

pub fn gather_contacts(player: &MovingRect, obstacles: &[Rect]) -> Vec<Contact> {
    let mut contacts = Vec::new();
    for obstacle in obstacles.iter() {
        if player.x <= obstacle.x + obstacle.w
            && obstacle.x <= player.x + player.w
            && player.y <= obstacle.y + obstacle.h
            && obstacle.y <= player.y + player.h
        {
            contacts.push(Contact(ContactID::Player, ContactID::Obstacle));
        }
    }
    contacts
}
