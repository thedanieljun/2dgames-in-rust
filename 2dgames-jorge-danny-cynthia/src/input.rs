use std::collections::{BTreeMap, BTreeSet};
use winit::event::VirtualKeyCode;
use winit_input_helper::WinitInputHelper;

pub struct Input<ActionID: Ord + Eq> {
    key_map: BTreeMap<ActionID, VirtualKeyCode>,
    this_frame_keys: BTreeSet<VirtualKeyCode>,
    last_frame_keys: BTreeSet<VirtualKeyCode>,
}

impl<ActionID: Ord + Eq> Input<ActionID> {
    pub fn new() -> Self {
        Self {
            key_map: BTreeMap::new(),
            this_frame_keys: BTreeSet::new(),
            last_frame_keys: BTreeSet::new(),
        }
    }

    pub fn add_key_to_map(&mut self, id: ActionID, key: VirtualKeyCode) {
        self.key_map.insert(id, key);
    }

    pub fn update(&mut self, events: &WinitInputHelper) {
        self.last_frame_keys = std::mem::take(&mut self.this_frame_keys);
        for (_, key) in self.key_map.iter() {
            if events.key_held(*key) {
                self.this_frame_keys.insert(*key);
            }
        }
    }

    pub fn is_held(&self, id: ActionID) -> bool {
        if let Some(key) = self.key_map.get(&id) {
            self.this_frame_keys.contains(&key)
        } else {
            false
        }
    }

    pub fn is_pressed(&self, id: ActionID) -> bool {
        if let Some(key) = self.key_map.get(&id) {
            self.this_frame_keys.contains(&key) && !self.last_frame_keys.contains(&key)
        } else {
            false
        }
    }

    pub fn is_released(&self, id: ActionID) -> bool {
        if let Some(key) = self.key_map.get(&id) {
            !self.this_frame_keys.contains(&key) && self.last_frame_keys.contains(&key)
        } else {
            false
        }
    }
}
