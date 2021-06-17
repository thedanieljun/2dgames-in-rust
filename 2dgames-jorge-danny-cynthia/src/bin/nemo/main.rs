use std::collections::HashMap;
use std::rc::Rc;


use pixels::{Pixels, SurfaceTexture};
#[allow(unused)]
use rodio::Source;
use std::fs::File;
use std::io::BufReader;
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper; //, PlayError};

use engine2d::{
    animation::{Animation, AnimationData},
    objects::*,
    screen::Screen,
    sprite::{DrawSpriteExt, Sprite},
    text::*,
    texture::Texture,
};

mod storyparser;

const WIDTH: usize = 1280;
const HEIGHT: usize = 720;
const DEPTH: usize = 4;
pub const CHAR_SIZE: f32 = 16.0;
const BOX_COLOR: Color = [255, 255, 255, 255];
const BOX_X: f32 = WIDTH as f32 / 10.0;
const BOX_Y: f32 = 6.0 * HEIGHT as f32 / 11.0;
const BOX_WIDTH: f32 = 8.0 * WIDTH as f32 / 10.0;
const BOX_HEIGHT: f32 = 4.0 * HEIGHT as f32 / 10.0;

#[derive(Debug)]
enum Mode {
    Title,
    Read,
    Respond,
    EndGame,
}

use storyparser::*;
struct GameState {
    scene_map: HashMap<String, Scene>,
    current_scene: Scene,
    box_read: bool,
    message_index: usize,
    box_text_index: usize,
    response_index: usize,
    text_info: TextInfo,
    mode: Mode,
}

impl GameState {
    pub fn reset_read_info(&mut self) {
        self.message_index = 0;
        self.box_text_index = 0;
        self.response_index = 0;
        self.box_read = false;
    }

    pub fn reset_game(&mut self) {
        self.reset_read_info();
        self.current_scene = self.scene_map.get("intro").unwrap().clone();
        self.mode = Mode::Title;
    }
}

mod textinfo;

fn main() {
    let text_box = Rect::new(BOX_X, BOX_Y, BOX_WIDTH, BOX_HEIGHT);
    let text_box_text = Rect::new(
        BOX_X + 3.0 * BOX_WIDTH / 64.0,
        BOX_Y + CHAR_SIZE * 4.0,
        BOX_WIDTH - 6.0 * BOX_WIDTH / 64.0,
        BOX_HEIGHT - CHAR_SIZE * 6.0,
    );

    let story = parse_story().unwrap();
    let title = story.story_name.clone();
    let mut scene_map: HashMap<String, Scene> = HashMap::new();
    let mut sprites: HashMap<String, Sprite> = HashMap::new();
    use std::path::Path;
    story.scenes.iter().for_each(|s| {
        scene_map.insert(s.scene_name.clone(), s.scene.clone());
        if !s.scene.name.is_empty() {
            if let Ok(texture) = Texture::with_file(Path::new(&format!(
                "content/fishsprites/{}.png",
                s.scene.name.to_lowercase()
            ))) {
                let width = texture.width as f32;
                let height = texture.height as f32;
                let animation = Animation::new(&Rc::new(AnimationData {
                    frames: vec![(Rect::new(0.0, 0.0, width, height), 1)],
                    looping: false,
                }));
                sprites.insert(
                    s.scene.name.clone(),
                    Sprite::new(
                        &Rc::new(texture),
                        animation,
                        Vec2::new((WIDTH as f32 - width) / 2.0, 200.0 - height),
                    ),
                );
            }
        }
    });

    let current_scene = scene_map.get("intro").unwrap();
    let mut state = GameState {
        // add tree struct that will represent game text and options. empty until text parser implemented
        scene_map: scene_map.clone(),
        current_scene: current_scene.clone(),
        box_read: false,
        message_index: 0,
        box_text_index: 0,
        response_index: 0,
        // position in tree
        //ending_score: 0,
        // ending determiner
        text_info: {
            let image = Rc::new(Texture::with_file(Path::new("content/ascii-dark.png")).unwrap());
            TextInfo::new(&image, &textinfo::info())
        },
        mode: Mode::Title,
    };

    let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();

    let file = File::open("content/the-fish-who-dreamt-of-a-distant-planet.mp3").unwrap();
    let background = rodio::Decoder::new(BufReader::new(file))
        .unwrap()  
        .repeat_infinite();

    let _ = stream_handle.play_raw(background.convert_samples());

    let event_loop = EventLoop::new();
    let mut input_events = WinitInputHelper::new();

    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        WindowBuilder::new()
            .with_title(&title)
            .with_inner_size(size)
            .with_min_inner_size(size)
            .with_resizable(false)
            .build(&event_loop)
            .unwrap()
    };
    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(WIDTH as u32, HEIGHT as u32, surface_texture).unwrap()
    };

    event_loop.run(move |event, _, control_flow| {
        match state.mode {
            Mode::Title => {
                // Draw the current frame
                if let Event::RedrawRequested(_) = event {
                    let mut screen = Screen::wrap(pixels.get_frame(), WIDTH, HEIGHT, DEPTH);
                    screen.clear([0, 105, 148, 255]);

                    screen.draw_text_at_pos(&title, Vec2::new(450.0, 100.0), &state.text_info);
                    screen.draw_text_at_pos(
                        "press enter to start.",
                        Vec2::new(460.0, 440.0),
                        &state.text_info,
                    );
                    screen.draw_text_at_pos("", Vec2::new(65.0, 260.0), &state.text_info);

                    if pixels.render().is_err() {
                        *control_flow = ControlFlow::Exit;
                        return;
                    }

                    //available_time += since.elapsed().as_secs_f64();
                }
                // Handle input_events events
                if input_events.update(&event) {
                    // Close events
                    if input_events.key_pressed(VirtualKeyCode::Escape) || input_events.quit() {
                        *control_flow = ControlFlow::Exit;
                        return;
                    }
                    if input_events.key_pressed(VirtualKeyCode::Return) {
                        state.mode = Mode::Read;
                        window.request_redraw();
                    }

                    // Resize the window
                    if let Some(size) = input_events.window_resized() {
                        pixels.resize(size.width, size.height);
                    }
                }
            }
            Mode::Read => {
                // Draw the current frame
                if let Event::RedrawRequested(_) = event {
                    let mut screen = Screen::wrap(pixels.get_frame(), WIDTH, HEIGHT, DEPTH);
                    screen.clear([0, 105, 148, 255]);

                    //render text box
                    screen.rect(text_box, BOX_COLOR);
                    screen.rect_lines(text_box, [0, 0, 0, 0]);

                    // draw sprite
                    if let Some(sprite) = sprites.get(&state.current_scene.name) {
                        screen.draw_sprite(sprite);
                    }

                    // render text in box
                    if !state.current_scene.name.is_empty() {
                        screen.draw_text_at_pos(
                            &state.current_scene.name,
                            Vec2::new(text_box_text.x, BOX_Y + CHAR_SIZE * 2.0),
                            &state.text_info,
                        );
                    }
                    if let Some(idx) = screen.draw_text_in_rect(
                        &state.current_scene.message[state.box_text_index..],
                        text_box_text,
                        &state.text_info,
                        false,
                    ) {
                        state.message_index = idx;
                    } else {
                        state.message_index = state.current_scene.message.len();
                    }

                    if pixels.render().is_err() {
                        *control_flow = ControlFlow::Exit;
                        return;
                    }
                }

                // Handle input_events events
                if input_events.update(&event) {
                    // Close events

                    if input_events.key_pressed(VirtualKeyCode::Escape) || input_events.quit() {
                        *control_flow = ControlFlow::Exit;
                        return;
                    }

                    if input_events.key_pressed(VirtualKeyCode::Space) || input_events.quit() {
                        state.box_text_index = state.message_index;
                        if !state.current_scene.responses.is_empty()
                            && !state.current_scene.responses[0].response.is_empty()
                        {
                            // if player has read all text and has option to give response switch to response mode
                            if state.message_index >= state.current_scene.message.len() - 1 {
                                state.mode = Mode::Respond;
                                state.box_read = false;
                                state.box_text_index = 0;
                                state.message_index = 0;
                            }
                        } else {
                            // if player reached end of tree and no final response available switch to game over
                            if state.current_scene.responses.is_empty() {
                                state.mode = Mode::EndGame;
                            } else {
                                // if no response option available go forward in story
                                state.current_scene = state
                                    .scene_map
                                    .get(&*(state.current_scene.responses[0].goto))
                                    .unwrap()
                                    .clone();
                                state.reset_read_info();
                            }
                        }

                        window.request_redraw();
                    }

                    // Resize the window
                    if let Some(size) = input_events.window_resized() {
                        pixels.resize(size.width, size.height);
                    }
                }
            }

            Mode::Respond => {
                // Draw the current frame

                if let Event::RedrawRequested(_) = event {
                    let mut screen = Screen::wrap(pixels.get_frame(), WIDTH, HEIGHT, DEPTH);
                    // render background
                    screen.clear([0, 105, 148, 255]);

                    //render text box
                    screen.rect(text_box, BOX_COLOR);
                    screen.rect_lines(text_box, [0, 0, 0, 255]);

                    // vec of response y values for pointer to know location
                    let mut ypos_vec: Vec<f32> = vec![BOX_Y + CHAR_SIZE];

                    // render responses
                    for (i, resp_map) in state.current_scene.responses.iter().enumerate() {
                        let cur_rect = Rect::new(
                            BOX_X + 3.0 * BOX_WIDTH / 64.0,
                            ypos_vec[i],
                            BOX_WIDTH - 6.0 * BOX_WIDTH / 64.0,
                            BOX_HEIGHT,
                        );

                        screen.draw_text_in_rect(
                            &resp_map.response,
                            cur_rect,
                            &state.text_info,
                            false,
                        );
                        ypos_vec.push(cur_rect.y + CHAR_SIZE * 2.0);
                    }

                    // response pointer
                    let pointer = Rect {
                        x: BOX_X + 1.0 * BOX_WIDTH / 64.0,
                        y: ypos_vec[state.response_index],
                        h: 8.0,
                        w: 8.0,
                    };
                    screen.rect(pointer, [255, 0, 0, 255]);

                    if pixels.render().is_err() {
                        *control_flow = ControlFlow::Exit;
                        return;
                    }
                }

                //TODO update position in tree

                let mut input = false;

                // Handle input_events events
                if input_events.update(&event) {
                    // Close events
                    if input_events.key_pressed(VirtualKeyCode::Escape) || input_events.quit() {
                        *control_flow = ControlFlow::Exit;
                        return;
                    }

                    if input_events.key_pressed(VirtualKeyCode::Down) {
                        if state.response_index < state.current_scene.responses.len() - 1 {
                            state.response_index += 1;
                        } else {
                            state.response_index = 0;
                        }
                        input = true;
                    }

                    if input_events.key_pressed(VirtualKeyCode::Up) {
                        if state.response_index > 0 {
                            state.response_index -= 1;
                        } else {
                            state.response_index = state.current_scene.responses.len() - 1;
                        }
                        input = true;
                    }

                    if input_events.key_pressed(VirtualKeyCode::Space) {
                        //move to next value in tree based on response.
                        if state.current_scene.responses.is_empty() {
                            state.mode = Mode::EndGame;
                        } else {
                            state.current_scene = state
                                .scene_map
                                .get(&*(state.current_scene.responses[state.response_index].goto))
                                .unwrap()
                                .clone();
                            state.reset_read_info();
                            state.mode = Mode::Read;
                        }
                        input = true;
                    }

                    if input {
                        window.request_redraw();
                    }

                    // Resize the window
                    if let Some(size) = input_events.window_resized() {
                        pixels.resize(size.width, size.height);
                    }
                }
            }

            Mode::EndGame => {
                if let Event::RedrawRequested(_) = event {
                    let mut screen = Screen::wrap(pixels.get_frame(), WIDTH, HEIGHT, DEPTH);
                    screen.clear([100, 150, 200, 255]);

                    screen.draw_text_at_pos("the end", Vec2::new(400.0, 60.0), &state.text_info);

                    screen.draw_text_at_pos(
                        "press enter to return to title screen",
                        Vec2::new(400.0, 240.0),
                        &state.text_info,
                    );
                    screen.draw_text_at_pos(
                        "or escape to exit",
                        Vec2::new(300.0, 260.0),
                        &state.text_info,
                    );

                    if pixels.render().is_err() {
                        *control_flow = ControlFlow::Exit;
                        return;
                    }
                }
                if input_events.update(&event) {
                    // Close events
                    if input_events.key_pressed(VirtualKeyCode::Escape) || input_events.quit() {
                        *control_flow = ControlFlow::Exit;
                        return;
                    }
                    if input_events.key_pressed(VirtualKeyCode::Return) {
                        // reset game mode to title, state values to default
                        state.reset_game();
                        window.request_redraw();
                    }
                    // Resize the window
                    if let Some(size) = input_events.window_resized() {
                        pixels.resize(size.width, size.height);
                    }
                }
            }
        }
    });
}
