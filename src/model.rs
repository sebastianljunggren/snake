use std::collections;
use std::fmt;
use std::iter;
use std::slice;
use rand;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    fn opposite(self) -> Direction {
        match self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }
}

impl rand::Rand for Direction {
    fn rand<R: rand::Rng>(rng: &mut R) -> Direction {
        let index: usize = rng.gen();
        [
            Direction::Up,
            Direction::Down,
            Direction::Right,
            Direction::Left,
        ][index % 4]
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Position {
    x: usize,
    y: usize,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Tile {
    Snake,
    Head,
    Food,
    Empty,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Board {
    width: usize,
    height: usize,
    tiles: Vec<Tile>,
}

impl Board {
    pub fn new(
        width: usize,
        height: usize,
        food: Option<Position>,
        snake: &collections::VecDeque<Position>,
    ) -> Board {
        let mut board = Board {
            width,
            height,
            tiles: vec![Tile::Empty; width * height],
        };
        food.map(|f| board.tiles[f.y * width + f.x] = Tile::Food);
        ;
        for position in snake {
            board.tiles[position.y * width + position.x] = Tile::Snake
        }
        snake
            .front()
            .map(|head| board.tiles[head.y * width + head.x] = Tile::Head);
        board
    }

    pub fn iter(&self) -> slice::Chunks<Tile> {
        self.tiles.chunks(self.width).into_iter()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum GameStep {
    Lose,
    Continue(Board),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Game {
    width: usize,
    height: usize,
    snake: collections::VecDeque<Position>,
    food: Option<Position>,
    last_direction: Direction,
}

impl Game {
    pub fn new(width: usize, height: usize) -> Game {
        let mut snake = collections::VecDeque::new();
        snake.push_back(Position { x: 7, y: 5 });
        snake.push_back(Position { x: 7, y: 6 });
        snake.push_back(Position { x: 7, y: 7 });
        snake.push_back(Position { x: 6, y: 7 });
        snake.push_back(Position { x: 5, y: 7 });

        let food = Game::next_food(&snake, width, height);

        Game {
            width,
            height,
            snake,
            food,
            last_direction: Direction::Up,
        }
    }

    pub fn step(&mut self, direction: Direction) -> GameStep {
        let direction = if direction == self.last_direction.opposite() {
            self.last_direction
        } else {
            direction
        };
        let head = self.next_head(direction);
        let die = self.snake.contains(&head);
        if die {
            GameStep::Lose
        } else {
            self.snake.push_front(head);
            let eat = self.food.map(|f| f == head).unwrap_or(false);
            if eat {
                self.food = Game::next_food(&self.snake, self.width, self.height)
            } else {
                self.snake.pop_back();
            };
            self.last_direction = direction;
            GameStep::Continue(self.board())
        }
    }

    fn next_head(&self, direction: Direction) -> Position {
        let head = self.head();
        match direction {
            Direction::Up => Position {
                x: head.x,
                y: if head.y > 0 {
                    head.y - 1
                } else {
                    self.height - 1
                },
            },
            Direction::Down => Position {
                x: head.x,
                y: if head.y + 1 < self.height {
                    head.y + 1
                } else {
                    0
                },
            },
            Direction::Left => Position {
                x: if head.x > 0 {
                    head.x - 1
                } else {
                    self.width - 1
                },
                y: head.y,
            },
            Direction::Right => Position {
                x: if head.x + 1 < self.width {
                    head.x + 1
                } else {
                    0
                },
                y: head.y,
            },
        }
    }

    fn next_food(
        snake: &collections::VecDeque<Position>,
        width: usize,
        height: usize,
    ) -> Option<Position> {
        let candidates = (0..height)
            .flat_map(|y| (0..width).zip(iter::repeat(y)))
            .map(|(x, y)| Position { x, y })
            .filter(|p| !snake.contains(p));
        let mut rng = rand::thread_rng();
        match rand::seq::sample_iter(&mut rng, candidates, 1) {
            Ok(sample) => Some(sample[0]),
            _ => None,
        }
    }

    fn head(&self) -> Position {
        *self.snake.front().unwrap()
    }

    pub fn board(&self) -> Board {
        Board::new(self.width, self.height, self.food, &self.snake)
    }
}

impl fmt::Display for Game {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        GameView(self.board(), self.last_direction).fmt(f)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct GameView(pub Board, pub Direction);

impl fmt::Display for GameView {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let &GameView(ref board, ref direction) = self;
        writeln!(f, "")?;
        let wall = '□';
        for _ in 0..board.width + 1 {
            write!(f, "{} ", wall)?;
        }
        write!(f, "{}\n", wall)?;
        for row in board.iter() {
            write!(f, "{} ", wall)?;
            for tile in row {
                let tile = match tile {
                    &Tile::Snake => '■',
                    &Tile::Head => match direction {
                        &Direction::Up => '▲',
                        &Direction::Down => '▼',
                        &Direction::Left => '◀',
                        &Direction::Right => '▶',
                    },
                    &Tile::Food => '❤',
                    &Tile::Empty => ' ',
                };
                write!(f, "{} ", tile)?;
            }
            write!(f, "{}\n", wall)?;
        }
        for _ in 0..board.width + 1 {
            write!(f, "{} ", wall)?;
        }
        write!(f, "{}\n", wall)?;
        Ok(())
    }
}
