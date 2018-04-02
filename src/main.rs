extern crate rand;
extern crate sdl2;

mod model;
mod view;
mod controller;

fn main() {
    let width = 12;
    let height = 12;
    let (control_tx, step_rx) = controller::init(width, height);
    view::run(control_tx, step_rx, width as u32, height as u32);
}
