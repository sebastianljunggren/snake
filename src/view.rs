use model;
use controller;
use sdl2;
use sdl2::event;
use sdl2::keyboard;
use sdl2::pixels;
use sdl2::render;
use sdl2::video;
use std::cmp;
use std::error;
use std::fmt;
use std::sync::mpsc;
use std::thread;
use std::time;

const UP: [keyboard::Keycode; 2] = [keyboard::Keycode::Up, keyboard::Keycode::W];
const DOWN: [keyboard::Keycode; 2] = [keyboard::Keycode::Down, keyboard::Keycode::S];
const LEFT: [keyboard::Keycode; 2] = [keyboard::Keycode::Left, keyboard::Keycode::A];
const RIGHT: [keyboard::Keycode; 2] = [keyboard::Keycode::Right, keyboard::Keycode::D];

pub fn run(
    control_tx: mpsc::Sender<controller::GameControl>,
    step_rx: mpsc::Receiver<model::GameStep>,
    width: u32,
    height: u32,
) -> Result<(), ViewError> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let desktop_display_mode = video_subsystem.desktop_display_mode(0)?;
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
        .build()?;

    let mut canvas = window.into_canvas().build()?;

    canvas.set_draw_color(pixels::Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump()?;
    control_tx.send(controller::GameControl::Start)?;
    let sleep_length = time::Duration::from_millis(controller::SLEEP_MILLIS);
    'running: loop {
        let mut active_direction = model::Direction::Up;
        let mut board_maybe = None;

        for event in event_pump.poll_iter() {
            match event {
                event::Event::Quit { .. }
                | event::Event::KeyDown {
                    keycode: Some(keyboard::Keycode::Escape),
                    ..
                } => break 'running,
                event::Event::KeyDown {
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
                    direction_maybe.map(|direction| {
                        active_direction = direction;
                        control_tx.send(controller::GameControl::Move(direction))
                    });
                }
                _ => (),
            }
        }

        for game_step in step_rx.try_iter() {
            match game_step {
                model::GameStep::Continue(board) => board_maybe = Some(board),
                model::GameStep::Lose => break 'running,
            };
        }

        board_maybe.map(|board| {
            let result = draw_board(
                model::GameView(board, active_direction),
                &mut canvas,
                grid_size,
                grid_margin,
            );
            thread::sleep(sleep_length);
            result
        });
    }
    thread::sleep(time::Duration::from_millis(5000));
    Ok(())
}

fn draw_board(
    view: model::GameView,
    canvas: &mut render::Canvas<video::Window>,
    grid_size: u32,
    grid_margin: u32,
) -> Result<(), String> {
    canvas.set_draw_color(pixels::Color::RGB(15, 15, 15));
    canvas.clear();
    let model::GameView(board, _) = view;
    for (y, row) in board.iter().enumerate() {
        for (x, tile) in row.iter().enumerate() {
            let color = match tile {
                &model::Tile::Snake => pixels::Color::RGB(179, 179, 179),
                &model::Tile::Head => pixels::Color::RGB(115, 115, 115),
                &model::Tile::Food => pixels::Color::RGB(0, 204, 68),
                &model::Tile::Empty => pixels::Color::RGB(31, 31, 31),
            };
            draw_rect(canvas, color, x as i32, y as i32, grid_size, grid_margin)?;
        }
    }
    canvas.present();
    Ok(())
}

fn draw_rect(
    canvas: &mut render::Canvas<video::Window>,
    color: pixels::Color,
    x: i32,
    y: i32,
    grid_size: u32,
    grid_margin: u32,
) -> Result<(), String> {
    canvas.set_draw_color(color);
    let grid_size_margin = (grid_size + grid_margin) as i32;
    let scaled_x = x * grid_size_margin + grid_margin as i32;
    let scaled_y = y * grid_size_margin + grid_margin as i32;
    canvas.fill_rect(sdl2::rect::Rect::new(
        scaled_x,
        scaled_y,
        grid_size,
        grid_size,
    ))
}

#[derive(Debug)]
pub enum ViewError {
    WindowBuildError(video::WindowBuildError),
    IntegerOrSdlError(sdl2::IntegerOrSdlError),
    SendControlError(mpsc::SendError<controller::GameControl>),
    ErrorMessage(String),
}

impl fmt::Display for ViewError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ViewError::WindowBuildError(ref err) => write!(fmt, "Window build error: {}", err),
            ViewError::IntegerOrSdlError(ref err) => write!(fmt, "Integer or SDL error: {}", err),
            ViewError::SendControlError(ref err) => write!(fmt, "Send control error: {}", err),
            ViewError::ErrorMessage(ref err) => write!(fmt, "Error message: {}", err),
        }
    }
}

impl error::Error for ViewError {
    fn description(&self) -> &str {
        match *self {
            ViewError::WindowBuildError(ref err) => err.description(),
            ViewError::IntegerOrSdlError(ref err) => err.description(),
            ViewError::SendControlError(ref err) => err.description(),
            ViewError::ErrorMessage(ref err) => err,
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            ViewError::WindowBuildError(ref err) => Some(err),
            ViewError::IntegerOrSdlError(ref err) => Some(err),
            ViewError::SendControlError(ref err) => Some(err),
            ViewError::ErrorMessage(_) => None,
        }
    }
}

impl From<video::WindowBuildError> for ViewError {
    fn from(err: video::WindowBuildError) -> ViewError {
        ViewError::WindowBuildError(err)
    }
}

impl From<mpsc::SendError<controller::GameControl>> for ViewError {
    fn from(err: mpsc::SendError<controller::GameControl>) -> ViewError {
        ViewError::SendControlError(err)
    }
}

impl From<sdl2::IntegerOrSdlError> for ViewError {
    fn from(err: sdl2::IntegerOrSdlError) -> ViewError {
        ViewError::IntegerOrSdlError(err)
    }
}

impl From<String> for ViewError {
    fn from(err: String) -> ViewError {
        ViewError::ErrorMessage(err)
    }
}
