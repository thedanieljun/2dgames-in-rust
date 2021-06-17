use std::collections::BTreeMap;
use std::rc::Rc;

use crate::objects::{Rect, Vec2};
use crate::texture::Texture;

pub struct TextInfo {
    pub info: BTreeMap<char, Rect>,
    image: Rc<Texture>,
}

impl TextInfo {
    pub fn new(image: &Rc<Texture>, char_info: &[(char, Rect)]) -> Self {
        let mut text_info = TextInfo {
            info: BTreeMap::new(),
            image: Rc::clone(image),
        };
        for (character, rect) in char_info.iter() {
            text_info.info.insert(*character, *rect);
        }
        text_info
    }

    fn get_char_width(&self, ch: char) -> f32 {
        self.info.get(&ch).map_or(0.0, |rect| rect.w)
    }

    fn get_string_width(&self, string: &str) -> f32 {
        let mut width = 0.0;
        for ch in string.chars() {
            width += self.get_char_width(ch);
        }
        width
    }
}

pub trait DrawTextExt {
    fn draw_text_at_pos(&mut self, string: &str, pos: Vec2, font: &TextInfo);

    fn draw_text_in_rect(
        &mut self,
        string: &str,
        rect: Rect,
        font: &TextInfo,
        show_overflow: bool,
    ) -> Option<usize>;
}

use crate::screen::Screen;
impl<'fb> DrawTextExt for Screen<'fb> {
    // makes a bunch of assumptions, such as that all the characters are the same height. works because we're using a monospace/height font, won't necessarily work for others
    fn draw_text_at_pos(&mut self, string: &str, pos: Vec2, font: &TextInfo) {
        // starting positions
        let mut x = pos.x as f32;
        let y = pos.y as f32;
        for ch in string.chars() {
            if let Some(rect) = font.info.get(&ch) {
                self.bitblt(&font.image, *rect, Vec2::new(x, y));
                x += rect.w;
            }
        }
    }

    // assumes using a font with same height characters
    // option is idx in string of cutoff (like message_index in nemo)
    fn draw_text_in_rect(
        &mut self,
        string: &str,
        rect: Rect,
        font: &TextInfo,
        show_overflow: bool,
    ) -> Option<usize> {
        if string.is_empty() {
            return None;
        }
        let char_height = font.info.get(&' ').unwrap().h;
        let x = rect.x;
        let y = rect.y;
        let width = rect.w;
        let height = if show_overflow {
            self.size().1 as f32 - y
        } else {
            rect.h
        };

        let space_width = font.get_char_width(' ');
        let mut line = String::from("");
        let mut cur_x = 0.0;
        let mut cur_y = rect.y;
        let mut num_chars = 0;
        for word in string.split_whitespace() {
            let word_width = font.get_string_width(word);
            if word_width > width - cur_x {
                self.draw_text_at_pos(&line, Vec2::new(x, cur_y), font);
                line.clear();
                cur_x = 0.0;
                cur_y += char_height;
                if cur_y >= y + height {
                    return Some(num_chars);
                }
            }
            num_chars += word.len() + 1;
            line += word;
            line += " ";
            cur_x += word_width + space_width;
        }
        self.draw_text_at_pos(&line, Vec2::new(x, cur_y), font);
        None
    }
}
