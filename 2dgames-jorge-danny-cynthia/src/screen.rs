#![allow(dead_code)]
// We can pull in definitions from elsewhere in the crate!
use crate::objects::{Color, Rect, Vec2};
use crate::texture::Texture;
pub struct Screen<'fb> {
    framebuffer: &'fb mut [u8],
    width: usize,
    height: usize,
    depth: usize,
}

impl<'fb> Screen<'fb> {
    pub fn wrap(framebuffer: &'fb mut [u8], width: usize, height: usize, depth: usize) -> Self {
        Self {
            framebuffer,
            width,
            height,
            depth,
        }
    }

    pub fn size(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    // This is not going to be the most efficient API.
    // Lots of bounds checks!
    #[inline(always)]
    pub fn draw_at(&mut self, col: Color, x: usize, y: usize) {
        // No need to check x or y < 0, they're usizes!
        if self.width <= x || self.height <= y {
            return;
        }
        assert_eq!(self.depth, 4);
        let idx = y * self.width * self.depth + x * self.depth;
        // TODO should handle alpha blending!
        self.framebuffer[idx..(idx + self.depth)].copy_from_slice(&col);
    }

    // If we know the primitives in advance we're in much better shape:
    pub fn clear(&mut self, col: Color) {
        for px in self.framebuffer.chunks_exact_mut(4) {
            px.copy_from_slice(&col);
        }
    }

    pub fn rect_lines(&mut self, r: Rect, col: Color) {
        let x0 = r.x.max(0.0).min(self.width as f32) as usize;
        let x1 = (r.x + r.w).max(0.0).min(self.width as f32) as usize;
        let y0 = r.y.max(0.0).min(self.height as f32) as usize;
        let y1 = (r.y + r.h).max(0.0).min(self.height as f32) as usize;
        let depth = self.depth;
        let pitch = self.width * depth;

        // vertical lines
        if r.x > 0.0 {
            let x0 = x0.min(self.width - 1);
            for row in y0..y1 {
                let pixel_idx = row * pitch + x0 * depth;
                self.framebuffer[pixel_idx..pixel_idx + depth].copy_from_slice(&col);
            }
        }
        if r.x + r.w < self.width as f32 {
            let x1 = x1.min(self.width - 1);
            for row in y0..y1 {
                let pixel_idx = row * pitch + x1 * depth;
                self.framebuffer[pixel_idx..pixel_idx + depth].copy_from_slice(&col);
            }
        }

        if r.y > 0.0 {
            let y0 = y0.min(self.height - 1);
            for p in self.framebuffer[y0 * pitch + x0 * depth..y0 * pitch + x1 * depth]
                .chunks_exact_mut(depth)
            {
                p.copy_from_slice(&col);
            }
        }
        if r.y + r.h < self.height as f32 {
            let y1 = y1.min(self.height - 1);
            for p in self.framebuffer[y1 * pitch + x0 * depth..y1 * pitch + x1 * depth]
                .chunks_exact_mut(depth)
            {
                p.copy_from_slice(&col);
            }
        }
    }

    pub fn rect(&mut self, r: Rect, col: Color) {
        let x0 = r.x.max(0.0).min(self.width as f32) as usize;
        let x1 = (r.x + r.w).max(0.0).min(self.width as f32) as usize;
        let y0 = r.y.max(0.0).min(self.height as f32) as usize;
        let y1 = (r.y + r.h).max(0.0).min(self.height as f32) as usize;
        let depth = self.depth;
        let pitch = self.width * depth;
        for row in self.framebuffer[(y0 * pitch)..(y1 * pitch)].chunks_exact_mut(pitch) {
            for p in row[(x0 * depth)..(x1 * depth)].chunks_exact_mut(depth) {
                // TODO should handle alpha blending
                p.copy_from_slice(&col);
            }
        }
    }

    pub fn line(&mut self, Vec2 { x: x0, y: y0 }: Vec2, Vec2 { x: x1, y: y1 }: Vec2, col: Color) {
        let mut x = x0;
        let mut y = y0;
        let dx = (x1 - x0).abs();
        let sx = if x0 < x1 { 1.0 } else { -1.0 };
        let dy = -(y1 - y0).abs();
        let sy = if y0 < y1 { 1.0 } else { -1.0 };
        let mut err = dx + dy;
        let width = self.width as f32;
        let height = self.height as f32;
        let depth = self.depth;
        let pitch = self.width * depth;
        #[allow(clippy::all)]
        while x != x1 || y != y1 {
            // We couldn't just clamp x0/y0 and x1/y1 into bounds, because then
            // we might change the slope of the line.
            // We could find the intercept of the line with the left/right or top/bottom edges of the rect though, but that's work!
            if 0.0 <= x && x < width && 0.0 <= y && y < height {
                // TODO this bounds check could in theory be avoided with
                // the unsafe get_unchecked, but maybe better not...
                // TODO better handle alpha blending too, but not just yet...
                self.framebuffer[(y as usize * pitch + x as usize * depth)
                    ..(y as usize * pitch + (x as usize + 1) * depth)]
                    .copy_from_slice(&col);
            }
            let e2 = 2.0 * err;
            if dy <= e2 {
                err += dy;
                x += sx;
            }
            if e2 <= dx {
                err += dx;
                y += sy;
            }
        }
    }

    pub fn bitblt(&mut self, src: &Texture, from: Rect, Vec2 { x: to_x, y: to_y }: Vec2) {
        let (tw, th) = src.size();
        assert!(0.0 <= from.x);
        assert!(from.x < tw as f32);
        assert!(0.0 <= from.y);
        assert!(from.y < th as f32);
        let to_x = to_x as i32;
        let to_y = to_y as i32;
        if (to_x + from.w as i32) < 0
            || (self.width as i32) <= to_x
            || (to_y + from.h as i32) < 0
            || (self.height as i32) <= to_y
        {
            return;
        }
        let depth = self.depth;
        assert_eq!(depth, src.depth());
        let src_pitch = src.pitch();
        let dst_pitch = self.width * depth;
        // All this rigmarole is just to avoid bounds checks on each pixel of the blit.
        // We want to calculate which row/col of the src image to start at and which to end at.
        // This way there's no need to even check for out of bounds draws.
        let y_skip = to_y.max(0) - to_y;
        let x_skip = to_x.max(0) - to_x;
        let y_count = (to_y + from.h as i32).min(self.height as i32) - to_y;
        let x_count = (to_x + from.w as i32).min(self.width as i32) - to_x;
        let src_buf = src.buffer();
        for (row_a, row_b) in src_buf[(src_pitch * ((from.y as i32 + y_skip) as usize))
            ..(src_pitch * ((from.y as i32 + y_count) as usize))]
            .chunks_exact(src_pitch)
            .zip(
                self.framebuffer[(dst_pitch * ((to_y + y_skip) as usize))
                    ..(dst_pitch * ((to_y + y_count) as usize))]
                    .chunks_exact_mut(dst_pitch),
            )
        {
            let to_cols = row_b
                [(depth * (to_x + x_skip) as usize)..(depth * (to_x + x_count) as usize)]
                .chunks_exact_mut(depth);
            let from_cols = row_a[(depth * (from.x as i32 + x_skip) as usize)
                ..(depth * (from.x as i32 + x_count) as usize)]
                .chunks_exact(depth);
            // Composite over, assume premultiplied rgba8888
            for (to, from) in to_cols.zip(from_cols) {
                let ta = to[3] as f32 / 255.0;
                let fa = from[3] as f32 / 255.0;
                for i in 0..3 {
                    to[i] = from[i].saturating_add((to[i] as f32 * (1.0 - fa)).round() as u8);
                }
                to[3] = ((fa + ta * (1.0 - fa)) * 255.0).round() as u8;
            }
        }
    }

    pub fn filled_circle(&mut self, (x, y): (i32, i32), r: u64, col: Color) {
        for i in x - r as i32..x + r as i32 {
            for j in y - r as i32..y + r as i32 {
                if crate::objects::dist((i, j), (x, y)) < r as f32
                    && (x >= 0 && x < self.width as i32)
                    && (y >= 0 && y < self.height as i32)
                {
                    self.framebuffer[self.width * self.depth * j as usize + i as usize * self.depth
                        ..self.width * self.depth * j as usize + (i + 1) as usize * self.depth]
                        .copy_from_slice(&col);
                }
            }
        }
    }
}
