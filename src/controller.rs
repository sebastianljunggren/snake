use model;
use std::thread;
use std::time;
use std::sync::mpsc;

const STEP_MILLIS: u64 = 200;
pub const SLEEP_MILLIS: u64 = 1;

pub enum GameControl {
    Start,
    Move(model::Direction),
}

pub fn init(
    width: usize,
    height: usize,
) -> (mpsc::Sender<GameControl>, mpsc::Receiver<model::GameStep>) {
    let (step_tx, step_rx) = mpsc::channel();
    let (control_tx, control_rx) = mpsc::channel();

    thread::spawn(move || {
        await_start(&control_rx);
        let game = model::Game::new(width, height);
        match run(game, step_tx, control_rx) {
            Err(err) => println!("{}", err),
            _ => (),
        }
    });
    (control_tx, step_rx)
}

fn await_start(control_rx: &mpsc::Receiver<GameControl>) {
    loop {
        match control_rx.recv() {
            Ok(GameControl::Start) => return,
            _ => (),
        }
    }
}

fn run(
    mut game: model::Game,
    step_tx: mpsc::Sender<model::GameStep>,
    control_rx: mpsc::Receiver<GameControl>,
) -> Result<(), mpsc::SendError<model::GameStep>> {
    let step_length = time::Duration::from_millis(STEP_MILLIS);
    let sleep_length = time::Duration::from_millis(SLEEP_MILLIS);
    let mut last_step = time::SystemTime::now();
    let mut active_direction = model::Direction::Up;
    step_tx.send(model::GameStep::Continue(game.board()))?;
    loop {
        for control_message in control_rx.try_iter() {
            match control_message {
                GameControl::Move(direction) => active_direction = direction,
                _ => (),
            }
        }
        let now = time::SystemTime::now();
        if now - step_length >= last_step {
            let step = game.step(active_direction);
            let end_game = step == model::GameStep::Lose;
            step_tx.send(step)?;
            if end_game {
                return Ok(());
            }
            last_step = last_step + step_length;
        }
        thread::sleep(sleep_length);
    }
}
