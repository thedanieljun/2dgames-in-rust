#![allow(dead_code)]
const DEPTH: usize = 4;
const WIDTH: usize = 240;
const HEIGHT: usize = 360;

pub type Color = [u8; DEPTH];

#[derive(Copy, Clone, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

#[derive(Copy, Clone, PartialEq)]
pub struct MovingRect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub vel: Vec2,
}

impl MovingRect {
    pub fn new(x: f32, y: f32, w: f32, h: f32, vel: Vec2) -> Self {
        Self { x, y, w, h, vel }
    }

    pub fn as_rect(&self) -> Rect {
        Rect::new(self.x, self.y, self.w, self.h)
    }
}

#[derive(Copy, Clone, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rect {
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { x, y, w, h }
    }

    pub fn pos(&self) -> Vec2 {
        Vec2::new(self.x, self.y)
    }

    pub fn size(&self) -> Vec2 {
        Vec2::new(self.w, self.h)
    }
}

pub fn dist((x0, y0): (i32, i32), (x1, y1): (i32, i32)) -> f32 {
    let dx = (x0 - x1) as f32;
    let dy = (y0 - y1) as f32;
    (dx * dx + dy * dy).sqrt()
}
