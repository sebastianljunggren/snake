use model;
use controller;
use sdl2;
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;
use std::cmp;
use std::sync::mpsc;
use std::thread;
use std::time;

const UP: [Keycode; 2] = [Keycode::Up, Keycode::W];
const DOWN: [Keycode; 2] = [Keycode::Down, Keycode::S];
const LEFT: [Keycode; 2] = [Keycode::Left, Keycode::A];
const RIGHT: [Keycode; 2] = [Keycode::Right, Keycode::D];

pub fn run(
    control_tx: mpsc::Sender<controller::GameControl>,
    step_rx: mpsc::Receiver<model::GameStep>,
    width: u32,
    height: u32,
) {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let desktop_display_mode = video_subsystem.desktop_display_mode(0).unwrap();
    let display_width = desktop_display_mode.w;
    let display_height = desktop_display_mode.h;
    let initial_window_margin = 200;
    let window_x_size = (display_height - initial_window_margin) as u32 / width;
    let window_y_size = (display_width - initial_window_margin) as u32 / height;
    let size = cmp::min(window_x_size, window_y_size);
    let grid_size = cmp::min(size * 11 / 12, size - 1);
    let grid_margin = size - grid_size;

    let window_width = width * grid_size + (width + 1) * grid_margin;
    let window_height = height * grid_size + (height + 1) * grid_margin;

    let window = video_subsystem
        .window("Snake", window_width, window_height)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump().unwrap();
    control_tx.send(controller::GameControl::Start).unwrap();
    let sleep_length = time::Duration::from_millis(controller::SLEEP_MILLIS);
    'running: loop {
        let mut active_direction = model::Direction::Up;

        for event in event_pump.poll_iter() {
            let mut change_direction = |direction| {
                active_direction = direction;
                control_tx
                    .send(controller::GameControl::Move(direction))
                    .unwrap();
            };
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::KeyDown {
                    keycode: Some(code),
                    ..
                } => {
                    let direction_maybe = match code {
                        code if UP.contains(&code) => Some(model::Direction::Up),
                        code if DOWN.contains(&code) => Some(model::Direction::Down),
                        code if LEFT.contains(&code) => Some(model::Direction::Left),
                        code if RIGHT.contains(&code) => Some(model::Direction::Right),
                        _ => None,
                    };
                    direction_maybe.map(|direction| change_direction(direction));
                }
                _ => (),
            }
        }

        match step_rx.try_iter().last() {
            Some(model::GameStep::Continue(board)) => draw_board(
                model::GameView(board, active_direction),
                &mut canvas,
                grid_size,
                grid_margin,
            ),
            Some(model::GameStep::Lose) => break 'running,
            None => (),
        };
        thread::sleep(sleep_length);
    }
}

fn draw_board(
    view: model::GameView,
    canvas: &mut Canvas<Window>,
    grid_size: u32,
    grid_margin: u32,
) {
    canvas.set_draw_color(Color::RGB(15, 15, 15));
    canvas.clear();
    let model::GameView(board, _) = view;
    for (y, row) in board.iter().enumerate() {
        for (x, tile) in row.iter().enumerate() {
            let color = match tile {
                &model::Tile::Snake => Color::RGB(179, 179, 179),
                &model::Tile::Head => Color::RGB(115, 115, 115),
                &model::Tile::Food => Color::RGB(0, 204, 68),
                &model::Tile::Empty => Color::RGB(31, 31, 31),
            };
            draw_rect(canvas, color, x as i32, y as i32, grid_size, grid_margin);
        }
    }
    canvas.present();
}

fn draw_rect(
    canvas: &mut Canvas<Window>,
    color: Color,
    x: i32,
    y: i32,
    grid_size: u32,
    grid_margin: u32,
) {
    canvas.set_draw_color(color);
    let grid_size_margin = (grid_size + grid_margin) as i32;
    let scaled_x = x * grid_size_margin + grid_margin as i32;
    let scaled_y = y * grid_size_margin + grid_margin as i32;
    canvas
        .fill_rect(Rect::new(scaled_x, scaled_y, grid_size, grid_size))
        .unwrap();
}
