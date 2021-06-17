use crate::animation::{Animation, AnimationData};
use crate::objects::Vec2;
use crate::texture::Texture;
use std::rc::Rc;

pub struct Sprite {
    image: Rc<Texture>,
    pub animation: Animation,
    pub position: Vec2,
}

impl Sprite {
    pub fn new(image: &Rc<Texture>, animation: Animation, position: Vec2) -> Self {
        Self {
            image: Rc::clone(image),
            animation,
            position,
        }
    }

    pub fn set_animation(&mut self, data: &Rc<AnimationData>, force: bool) {
        self.animation.set_animation(data, force);
    }
}

pub trait DrawSpriteExt {
    fn draw_sprite(&mut self, s: &Sprite);
}

use crate::screen::Screen;
impl<'fb> DrawSpriteExt for Screen<'fb> {
    fn draw_sprite(&mut self, s: &Sprite) {
        // This works because we're only using a public method of Screen here,
        // and the private fields of sprite are visible inside this module
        self.bitblt(&s.image, s.animation.get_current_frame(), s.position);
    }
}
