use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::rc::Rc;
use std::time::{Duration, Instant};

use pixels::{Pixels, SurfaceTexture};
use rand::prelude::*;
use rodio::{OutputStreamHandle, Source};
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

mod background;
mod generation;

use background::Background;

use engine2d::{
    animation::{Animation, AnimationData},
    collision, input,
    objects::*,
    screen::Screen,
    sprite::{DrawSpriteExt, Sprite},
    text::{self, DrawTextExt},
    texture::Texture,
};

const DT: f64 = 1.0 / 60.0;
const WIDTH: usize = 240;
const HEIGHT: usize = 360;
const DEPTH: usize = 4;
const CHAR_SIZE: f32 = 16.0;

#[derive(Debug)]
enum Mode {
    Title,
    Play,
    EndGame,
}

struct GameState {
    player: MovingRect,
    player_sprite: Sprite,
    holding: Holding,
    obstacles: Vec<Rect>,
    obstacle_data: Vec<ObstacleData>,
    move_vel: f32,
    background: Background,
    last_flap_noise: Instant,
    score: u32,
    time_between: u32,
    mode: Mode,
}

pub struct Resources {
    pub animation_data: Vec<Rc<AnimationData>>,
    pub text_info: text::TextInfo,
    pub textures: Vec<Rc<Texture>>,
}

struct ObstacleData {
    filled: bool,
    passed: bool,
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
enum ActionID {
    Flap,
}

enum Holding {
    Worm(Sprite),
    Flower(Sprite),
    Letter(Sprite),
}

impl Holding {
    fn random(rsrc: &Resources) -> Self {
        let mut rng = thread_rng();
        match rng.gen_range(0..3) {
            0 => Self::Flower(Sprite::new(
                &Rc::clone(&rsrc.textures[0]),
                Animation::new(&rsrc.animation_data[7]),
                Vec2::new(45.0, HEIGHT as f32 / 2.0 - 10.0),
            )),
            1 => Self::Worm(Sprite::new(
                &Rc::clone(&rsrc.textures[0]),
                Animation::new(&rsrc.animation_data[8]),
                Vec2::new(47.0, HEIGHT as f32 / 2.0 - 9.0),
            )),
            2 => Self::Letter(Sprite::new(
                &Rc::clone(&rsrc.textures[0]),
                Animation::new(&rsrc.animation_data[9]),
                Vec2::new(46.0, HEIGHT as f32 / 2.0 - 9.0),
            )),
            _ => panic!("unreachable"),
        }
    }

    fn get_sprite(&self) -> &Sprite {
        match self {
            Self::Flower(sprite) => sprite,
            Self::Letter(sprite) => sprite,
            Self::Worm(sprite) => sprite,
        }
    }

    fn get_sprite_mut(&mut self) -> &mut Sprite {
        match self {
            Self::Flower(sprite) => sprite,
            Self::Letter(sprite) => sprite,
            Self::Worm(sprite) => sprite,
        }
    }

    fn draw(&self, screen: &mut Screen) {
        screen.draw_sprite(self.get_sprite());
    }
}

fn main() {
    let rsrc = Resources::new();
    let mut state = GameState {
        player: MovingRect::new(
            30.0,
            HEIGHT as f32 / 2.0 - 13.0,
            13.0,
            20.0,
            Vec2::new(0.0, 0.0),
        ),
        player_sprite: Sprite::new(
            &Rc::clone(&rsrc.textures[0]),
            Animation::new(&rsrc.animation_data[0]),
            Vec2::new(30.0, HEIGHT as f32 / 2.0 - 10.0),
        ),
        holding: Holding::random(&rsrc),
        background: Background::new(&rsrc),
        obstacles: Vec::new(),
        obstacle_data: Vec::new(),
        score: 0,
        move_vel: 1.0,
        time_between: 3000,
        mode: Mode::Title,
        last_flap_noise: Instant::now(),
    };
    let mut rng = thread_rng();

    let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
    play_coo(&stream_handle);

    let file = File::open("content/city-quiet.mp3").unwrap();
    let background = rodio::Decoder::new(BufReader::new(file))
        .unwrap()
        .take_duration(Duration::from_secs(31))
        .amplify(1.5)
        .repeat_infinite();

    let _ = stream_handle.play_raw(background.convert_samples());

    let event_loop = EventLoop::new();
    let mut input_events = WinitInputHelper::new();
    let generate = generation::Obstacles {
        obstacles: vec![(80, 120), (160, 130), (70, 230)],
        frequency_values: vec![1, 1, 1],
    };

    let mut input = input::Input::new();
    input.add_key_to_map(ActionID::Flap, VirtualKeyCode::Space);

    let window = {
        let size = LogicalSize::new(WIDTH as f64 * 2.0, HEIGHT as f64 * 2.0);
        WindowBuilder::new()
            .with_title("flappy bird")
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

    let mut last_added_rect = Instant::now();
    let mut available_time = 0.0;
    let mut since = Instant::now();

    event_loop.run(move |event, _, control_flow| {
        match state.mode {
            Mode::Title => {
                // Draw the current frame
                if let Event::RedrawRequested(_) = event {
                    let mut screen = Screen::wrap(pixels.get_frame(), WIDTH, HEIGHT, DEPTH);
                    screen.clear([135, 206, 250, 150]);

                    screen.draw_text_at_pos(
                        format!("score: {}", state.score).as_str(),
                        Vec2::new(0.0, 0.0),
                        &rsrc.text_info,
                    );
                    screen.draw_text_at_pos(
                        "flappy pigeon",
                        Vec2::new(20.0, 60.0),
                        &rsrc.text_info,
                    );
                    screen.draw_text_at_pos("press space", Vec2::new(40.0, 190.0), &rsrc.text_info);
                    screen.draw_text_at_pos("to flap", Vec2::new(73.0, 210.0), &rsrc.text_info);
                    screen.draw_text_at_pos("press enter", Vec2::new(40.0, 240.0), &rsrc.text_info);
                    screen.draw_text_at_pos("to start", Vec2::new(65.0, 260.0), &rsrc.text_info);

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
                        state.mode = Mode::Play;
                        state.last_flap_noise = Instant::now();
                        state.player.x = 30.0;
                        state.player.y = HEIGHT as f32 / 2.0 - 10.0;
                        state.player.vel = Vec2::new(0.0, 0.0);
                        state.time_between = 3000;
                        state.move_vel = 1.0;
                        last_added_rect = Instant::now();
                        since = Instant::now();
                    }
                    // Resize the window
                    if let Some(size) = input_events.window_resized() {
                        pixels.resize(size.width, size.height);
                    }
                }
            }
            Mode::Play => {
                // Draw the current frame
                if let Event::RedrawRequested(_) = event {
                    state.player_sprite.animation.animate();
                    let mut screen = Screen::wrap(pixels.get_frame(), WIDTH, HEIGHT, DEPTH);
                    state.background.draw(&mut screen);

                    screen.draw_sprite(&state.player_sprite);
                    state.holding.draw(&mut screen);

                    // draw state.obstacles
                    for (obstacle, data) in state.obstacles.iter().zip(state.obstacle_data.iter()) {
                        if data.filled {
                            screen.rect(*obstacle, [255, 0, 0, 255]);
                        } else {
                            screen.rect_lines(*obstacle, [255, 0, 0, 255]);
                        }
                    }

                    screen.draw_text_at_pos(
                        format!("score: {}", state.score).as_str(),
                        Vec2::new(0.0, 0.0),
                        &rsrc.text_info,
                    );

                    if pixels.render().is_err() {
                        *control_flow = ControlFlow::Exit;
                        return;
                    }

                    available_time += since.elapsed().as_secs_f64();
                }

                // Handle input_events events
                if input_events.update(&event) {
                    // Close events
                    if input_events.key_pressed(VirtualKeyCode::Escape) || input_events.quit() {
                        *control_flow = ControlFlow::Exit;
                        return;
                    }

                    // Resize the window
                    if let Some(size) = input_events.window_resized() {
                        pixels.resize(size.width, size.height);
                    }
                }

                while available_time >= DT {
                    since = Instant::now();
                    available_time -= DT;

                    if state.last_flap_noise.elapsed() > Duration::from_secs(9) {
                        play_flap(&stream_handle);
                        state.last_flap_noise = Instant::now();
                    }

                    state.background.update(&rsrc);
                    input.update(&input_events);
                    if input.is_pressed(ActionID::Flap) {
                        state.player.vel.y = 2.0;
                        state
                            .player_sprite
                            .set_animation(&rsrc.animation_data[1], true);
                    }

                    // update velocity for bird
                    state.player.vel.y -= 0.04;
                    if state.player.vel.y < 0.0 {
                        state
                            .player_sprite
                            .set_animation(&rsrc.animation_data[0], true);
                    }

                    // update position
                    state.player.x -= state.player.vel.x;
                    state.player.y -= state.player.vel.y;
                    state.player_sprite.position.x -= state.player.vel.x;
                    state.player_sprite.position.y -= state.player.vel.y;
                    state.holding.get_sprite_mut().position.x -= state.player.vel.x;
                    state.holding.get_sprite_mut().position.y -= state.player.vel.y;

                    for obstacle in state.obstacles.iter_mut() {
                        obstacle.x -= state.move_vel;
                    }

                    let contacts = collision::gather_contacts(&state.player, &state.obstacles);
                    for contact in contacts.iter() {
                        use collision::ContactID;
                        if let (ContactID::Player, ContactID::Obstacle) = contact.get_ids() {
                            // TODO: have a function that resets the game state??
                            play_coo(&stream_handle);
                            state.mode = Mode::EndGame;
                        }
                    }

                    if state.obstacles.len() >= 2
                        && state.obstacles[0].x + state.obstacles[0].w <= 0.0
                    {
                        // remove the first two state.obstacles
                        state.obstacles.remove(0);
                        state.obstacles.remove(0);
                        state.obstacle_data.remove(0);
                        state.obstacle_data.remove(0);
                    }

                    if since.duration_since(last_added_rect)
                        >= Duration::from_millis(state.time_between as u64)
                    {
                        let (top, bottom) = generate.generate_obstacles();
                        state
                            .obstacles
                            .push(Rect::new(WIDTH as f32, 0.0, 20.0, top as f32));
                        state.obstacles.push(Rect::new(
                            WIDTH as f32,
                            HEIGHT as f32 - bottom as f32,
                            20.0,
                            bottom as f32,
                        ));
                        state.obstacle_data.push(ObstacleData {
                            passed: false,
                            filled: rng.gen_bool(0.8),
                        });
                        state.obstacle_data.push(ObstacleData {
                            passed: false,
                            filled: rng.gen_bool(0.8),
                        });
                        last_added_rect = Instant::now();
                        since = Instant::now();
                    }

                    for (i, (obst, data)) in state
                        .obstacles
                        .iter_mut()
                        .zip(state.obstacle_data.iter_mut())
                        .enumerate()
                    {
                        if state.player.x > obst.x && !data.passed {
                            data.passed = true;
                            if i % 2 == 0 {
                                // let _ = stream_handle.play_raw(coo.convert_samples());
                                state.score += 1;
                                if state.move_vel < 3.0 {
                                    state.move_vel *= 1.1;
                                }
                                if state.obstacles.len() >= 4
                                    && state.obstacles[state.obstacles.len() - 1].x
                                        - state.obstacles[state.obstacles.len() - 3].x
                                        > state.obstacles[state.obstacles.len() - 3].w * 2.0
                                {
                                    state.time_between = (state.time_between - 200).max(800);
                                }
                                break;
                            }
                        }
                    }
                }
            }
            Mode::EndGame => {
                if let Event::RedrawRequested(_) = event {
                    let mut screen = Screen::wrap(pixels.get_frame(), WIDTH, HEIGHT, DEPTH);
                    screen.clear([255, 150, 0, 255]);

                    screen.draw_text_at_pos(
                        format!("score: {}", state.score).as_str(),
                        Vec2::new(0.0, 0.0),
                        &rsrc.text_info,
                    );

                    screen.draw_text_at_pos("game over!!!", Vec2::new(20.0, 60.0), &rsrc.text_info);

                    screen.draw_text_at_pos("press enter", Vec2::new(40.0, 240.0), &rsrc.text_info);
                    screen.draw_text_at_pos(
                        "to try again",
                        Vec2::new(30.0, 260.0),
                        &rsrc.text_info,
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
                        state.mode = Mode::Play;
                        state.player.x = 30.0;
                        state.player.y = HEIGHT as f32 / 2.0 - 13.0;
                        state.player_sprite.position.x = 30.0;
                        state.player_sprite.position.y = HEIGHT as f32 / 2.0 - 10.0;
                        state.player.vel = Vec2::new(0.0, 0.0);
                        state.background.clear(&rsrc);
                        state.obstacles.clear();
                        state.last_flap_noise = Instant::now();
                        state.obstacle_data.clear();
                        state.time_between = 3000;
                        state.move_vel = 1.0;
                        state.score = 0;
                        state.holding = Holding::random(&rsrc);
                        last_added_rect = Instant::now();
                        since = Instant::now();
                        //return;
                    }
                    // Resize the window
                    if let Some(size) = input_events.window_resized() {
                        pixels.resize(size.width, size.height);
                    }
                }
            }
        }
        window.request_redraw();
    });
}

fn play_coo(stream_handle: &OutputStreamHandle) {
    let file = File::open("content/birdcoo.mp3").unwrap();
    let coo = rodio::Decoder::new(BufReader::new(file))
        .unwrap()
        .take_duration(Duration::from_secs(5));
    let _ = stream_handle.play_raw(coo.convert_samples());
}

fn play_flap(stream_handle: &OutputStreamHandle) {
    let file = File::open("content/birdflap.mp3").unwrap();
    let flap = rodio::Decoder::new(BufReader::new(file))
        .unwrap()
        .take_duration(Duration::from_secs(9));
    let _ = stream_handle.play_raw(flap.convert_samples());
}

impl Resources {
    fn new() -> Self {
        Self {
            animation_data: vec![
                // bird glide (pigeon.png)
                Rc::new(AnimationData {
                    frames: vec![(Rect::new(0.0, 0.0, 20.0, 17.0), 1)],
                    looping: false,
                }),
                // bird flap
                Rc::new(AnimationData {
                    frames: vec![
                        (Rect::new(20.0, 0.0, 20.0, 17.0), 13),
                        (Rect::new(40.0, 0.0, 20.0, 17.0), 1),
                    ],
                    looping: false,
                }),
                // building (buildings.png)
                Rc::new(AnimationData {
                    frames: vec![(Rect::new(0.0, 56.0, 23.0, 82.0), 1)],
                    looping: false,
                }),
                // building
                Rc::new(AnimationData {
                    frames: vec![(Rect::new(23.0, 21.0, 22.0, 117.0), 1)],
                    looping: false,
                }),
                // building
                Rc::new(AnimationData {
                    frames: vec![(Rect::new(45.0, 79.0, 63.0, 58.0), 1)],
                    looping: false,
                }),
                // building
                Rc::new(AnimationData {
                    frames: vec![(Rect::new(108.0, 0.0, 29.0, 138.0), 1)],
                    looping: false,
                }),
                // cloud
                Rc::new(AnimationData {
                    frames: vec![(Rect::new(45.0, 0.0, 42.0, 29.0), 1)],
                    looping: false,
                }),
                // flower (pigeon.png)
                Rc::new(AnimationData {
                    frames: vec![(Rect::new(0.0, 17.0, 8.0, 7.0), 1)],
                    looping: false,
                }),
                // worm
                Rc::new(AnimationData {
                    frames: vec![(Rect::new(8.0, 17.0, 6.0, 5.0), 1)],
                    looping: false,
                }),
                // letter
                Rc::new(AnimationData {
                    frames: vec![(Rect::new(14.0, 17.0, 9.0, 8.0), 1)],
                    looping: false,
                }),
            ],
            textures: vec![
                Rc::new(Texture::with_file(Path::new("content/pigeon.png")).unwrap()),
                Rc::new(Texture::with_file(Path::new("content/buildings.png")).unwrap()),
            ],
            text_info: {
                let image =
                    Rc::new(Texture::with_file(Path::new("content/ascii-dark.png")).unwrap());
                let info = [
                    (' ', Rect::new(0.0, 0.0, CHAR_SIZE, CHAR_SIZE)),
                    ('!', Rect::new(16.0, 0.0, CHAR_SIZE, CHAR_SIZE)),
                    ('a', Rect::new(16.0, 64.0, CHAR_SIZE, CHAR_SIZE)),
                    ('b', Rect::new(32.0, 64.0, CHAR_SIZE, CHAR_SIZE)),
                    ('c', Rect::new(48.0, 64.0, CHAR_SIZE, CHAR_SIZE)),
                    ('d', Rect::new(64.0, 64.0, CHAR_SIZE, CHAR_SIZE)),
                    ('e', Rect::new(80.0, 64.0, CHAR_SIZE, CHAR_SIZE)),
                    ('f', Rect::new(96.0, 64.0, CHAR_SIZE, CHAR_SIZE)),
                    ('g', Rect::new(112.0, 64.0, CHAR_SIZE, CHAR_SIZE)),
                    ('h', Rect::new(128.0, 64.0, CHAR_SIZE, CHAR_SIZE)),
                    ('i', Rect::new(144.0, 64.0, CHAR_SIZE, CHAR_SIZE)),
                    ('j', Rect::new(160.0, 64.0, CHAR_SIZE, CHAR_SIZE)),
                    ('k', Rect::new(176.0, 64.0, CHAR_SIZE, CHAR_SIZE)),
                    ('l', Rect::new(192.0, 64.0, CHAR_SIZE, CHAR_SIZE)),
                    ('m', Rect::new(208.0, 64.0, CHAR_SIZE, CHAR_SIZE)),
                    ('n', Rect::new(224.0, 64.0, CHAR_SIZE, CHAR_SIZE)),
                    ('o', Rect::new(240.0, 64.0, CHAR_SIZE, CHAR_SIZE)),
                    ('p', Rect::new(0.0, 80.0, CHAR_SIZE, CHAR_SIZE)),
                    ('q', Rect::new(16.0, 80.0, CHAR_SIZE, CHAR_SIZE)),
                    ('r', Rect::new(32.0, 80.0, CHAR_SIZE, CHAR_SIZE)),
                    ('s', Rect::new(48.0, 80.0, CHAR_SIZE, CHAR_SIZE)),
                    ('t', Rect::new(64.0, 80.0, CHAR_SIZE, CHAR_SIZE)),
                    ('u', Rect::new(80.0, 80.0, CHAR_SIZE, CHAR_SIZE)),
                    ('v', Rect::new(96.0, 80.0, CHAR_SIZE, CHAR_SIZE)),
                    ('w', Rect::new(112.0, 80.0, CHAR_SIZE, CHAR_SIZE)),
                    ('x', Rect::new(128.0, 80.0, CHAR_SIZE, CHAR_SIZE)),
                    ('y', Rect::new(144.0, 80.0, CHAR_SIZE, CHAR_SIZE)),
                    ('z', Rect::new(160.0, 80.0, CHAR_SIZE, CHAR_SIZE)),
                    (':', Rect::new(160.0, 16.0, CHAR_SIZE, CHAR_SIZE)),
                    ('0', Rect::new(0.0, 16.0, CHAR_SIZE, CHAR_SIZE)),
                    ('1', Rect::new(16.0, 16.0, CHAR_SIZE, CHAR_SIZE)),
                    ('2', Rect::new(32.0, 16.0, CHAR_SIZE, CHAR_SIZE)),
                    ('3', Rect::new(48.0, 16.0, CHAR_SIZE, CHAR_SIZE)),
                    ('4', Rect::new(64.0, 16.0, CHAR_SIZE, CHAR_SIZE)),
                    ('5', Rect::new(80.0, 16.0, CHAR_SIZE, CHAR_SIZE)),
                    ('6', Rect::new(96.0, 16.0, CHAR_SIZE, CHAR_SIZE)),
                    ('7', Rect::new(112.0, 16.0, CHAR_SIZE, CHAR_SIZE)),
                    ('8', Rect::new(128.0, 16.0, CHAR_SIZE, CHAR_SIZE)),
                    ('9', Rect::new(144.0, 16.0, CHAR_SIZE, CHAR_SIZE)),
                ];
                text::TextInfo::new(&image, &info)
            },
        }
    }
}
