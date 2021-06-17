use std::time::{Duration, Instant};

use engine2d::{
    animation::Animation,
    objects::{Rect, Vec2},
    screen::Screen,
    sprite::{DrawSpriteExt, Sprite},
};
use rand::prelude::*;

use crate::{Resources, HEIGHT, WIDTH};

pub struct Background {
    buildings: Vec<Sprite>,
    clouds: Vec<Sprite>,
    last_building: Option<Instant>,
    last_cloud: Instant,
}

impl Background {
    pub fn new(rsrc: &Resources) -> Self {
        let mut bg = Self {
            buildings: Vec::new(),
            clouds: Vec::new(),
            last_building: None,
            last_cloud: Instant::now(),
        };
        bg.populate(rsrc);
        bg
    }

    fn populate(&mut self, rsrc: &Resources) {
        let mut x_pos = thread_rng().gen_range(30..50);
        for _ in 0..5 {
            let which_building = &rsrc.animation_data[thread_rng().gen_range(2..6)];
            self.buildings.push(Sprite::new(
                &rsrc.textures[1],
                Animation::new(which_building),
                Vec2::new(x_pos as f32, 280.0 - which_building.frames[0].0.h),
            ));
            x_pos += which_building.frames[0].0.w as u32 + thread_rng().gen_range(30..50);
        }

        x_pos = thread_rng().gen_range(30..50);
        let anim = &rsrc.animation_data[6];
        let anim_height = anim.frames[0].0.h;
        for _ in 0..3 {
            let mut y = thread_rng().gen_range(0..140 - 2 * anim_height as u32) as f32;
            if !self.clouds.is_empty() {
                if y >= self.clouds[self.clouds.len() - 1].position.y {
                    y += anim_height;
                } else if y + anim_height >= self.clouds[self.clouds.len() - 1].position.y {
                    y += anim_height * 2.0;
                }
            }
            self.clouds.push(Sprite::new(
                &rsrc.textures[1],
                Animation::new(anim),
                Vec2::new(x_pos as f32, y),
            ));
            x_pos += thread_rng().gen_range(40..100);
        }
    }

    pub fn clear(&mut self, rsrc: &Resources) {
        self.buildings.clear();
        self.clouds.clear();
        self.populate(rsrc);
        self.last_cloud = Instant::now();
        self.last_building = None;
    }

    pub fn update(&mut self, rsrc: &Resources) {
        if self.buildings[0].position.x + self.buildings[0].animation.get_current_frame().w < 0.0 {
            self.buildings.remove(0);
        }
        if !self.clouds.is_empty()
            && self.clouds[0].position.x + self.clouds[0].animation.get_current_frame().w < 0.0
        {
            self.clouds.remove(0);
        }

        let last = &self.buildings[self.buildings.len() - 1];
        if self.last_building.is_none()
            && last.position.x + last.animation.get_current_frame().w <= WIDTH as f32
        {
            self.last_building = Some(Instant::now());
        }

        if let Some(time) = self.last_building {
            if time.elapsed() > Duration::from_millis(100) && thread_rng().gen_bool(0.04) {
                self.add_building(rsrc);
            }
        }
        if self.last_cloud.elapsed() > Duration::from_millis(400) && thread_rng().gen_bool(0.04) {
            self.add_cloud(rsrc);
        }

        for sprite in self.buildings.iter_mut().chain(self.clouds.iter_mut()) {
            sprite.position.x -= 1.0;
        }
    }

    pub fn draw(&self, screen: &mut Screen) {
        screen.rect(
            Rect::new(0.0, 0.0, WIDTH as f32, 280.0),
            [130, 177, 255, 255],
        );
        screen.rect(
            Rect::new(0.0, 280.0, WIDTH as f32, HEIGHT as f32 - 280.0),
            [76, 175, 80, 255],
        );
        for sprite in self.clouds.iter().chain(self.buildings.iter()) {
            screen.draw_sprite(sprite);
        }
    }

    fn add_building(&mut self, rsrc: &Resources) {
        let which_building = &rsrc.animation_data[thread_rng().gen_range(2..6)];
        self.buildings.push(Sprite::new(
            &rsrc.textures[1],
            Animation::new(which_building),
            Vec2::new(WIDTH as f32, 280.0 - which_building.frames[0].0.h),
        ));
        self.last_building = None;
    }

    fn add_cloud(&mut self, rsrc: &Resources) {
        let anim = &rsrc.animation_data[6];
        let anim_height = anim.frames[0].0.h;
        let mut y = thread_rng().gen_range(0..140 - 2 * anim_height as u32) as f32;
        if !self.clouds.is_empty() {
            if y >= self.clouds[self.clouds.len() - 1].position.y {
                y += anim_height;
            } else if y + anim_height >= self.clouds[self.clouds.len() - 1].position.y {
                y += anim_height * 2.0;
            }
        }
        self.clouds.push(Sprite::new(
            &rsrc.textures[1],
            Animation::new(anim),
            Vec2::new(WIDTH as f32, y),
        ));
        self.last_cloud = Instant::now();
    }
}
