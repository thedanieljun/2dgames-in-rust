use crate::objects::Rect;
use std::rc::Rc;

#[derive(PartialEq)]
pub struct AnimationData {
    pub frames: Vec<(Rect, usize)>, // position of frame and how many frames it'll take
    pub looping: bool,
}

pub struct Animation {
    current_frame: (usize, usize),
    data: Rc<AnimationData>,
}

impl Animation {
    pub fn new(data: &Rc<AnimationData>) -> Self {
        Self {
            current_frame: (0, 0),
            data: Rc::clone(data),
        }
    }

    pub fn set_animation(&mut self, data: &Rc<AnimationData>, force: bool) {
        if force || *data != self.data {
            self.current_frame = (0, 0);
            self.data = Rc::clone(data);
        }
    }

    pub fn animate(&mut self) {
        let (frame_idx, frame_timer) = &mut self.current_frame;
        *frame_timer += 1;
        if *frame_timer == self.data.frames[*frame_idx].1 {
            *frame_timer = 0;
            if self.data.looping {
                *frame_idx = (*frame_idx + 1) % self.data.frames.len();
            } else {
                *frame_idx = (*frame_idx + 1).min(self.data.frames.len() - 1);
            }
        }
    }

    pub fn get_current_frame(&self) -> Rect {
        self.data.frames[self.current_frame.0].0
    }
}
