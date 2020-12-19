use std::collections::VecDeque;

use super::{Cell, Grid};
use crate::env::{Direction, SnakeData, Vec2D};

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Outcome {
    None,
    Match,
    Winner(u8),
}

#[derive(Debug, Clone)]
struct Snake {
    /// tail to head
    id: u8,
    body: VecDeque<Vec2D>,
    health: u8,
}
impl Snake {
    fn new(id: u8, body: VecDeque<Vec2D>, health: u8) -> Snake {
        Snake { id, body, health }
    }

    fn head(&self) -> Vec2D {
        *self.body.back().unwrap()
    }
}

#[derive(Debug, Clone)]
pub struct Game {
    snakes: Vec<Snake>,
    grid: Grid,
}

impl Game {
    pub fn new(width: usize, height: usize, snakes: &[SnakeData], food: &[Vec2D]) -> Game {
        let mut grid = Grid::new(width, height);
        grid.add_food(food);

        let mut game_snakes = Vec::new();
        for (i, snake) in snakes.iter().enumerate() {
            grid.add_snake(snake.body.iter().cloned());
            game_snakes.push(Snake::new(
                i as u8,
                snake.body.iter().cloned().rev().collect(),
                snake.health,
            ))
        }

        Game {
            snakes: game_snakes,
            grid,
        }
    }

    pub fn outcome(&self) -> Outcome {
        match self.snakes.len() {
            0 => Outcome::Match,
            1 => Outcome::Winner(self.snakes[0].id),
            _ => Outcome::None,
        }
    }

    pub fn snake_is_alive(&self, snake: u8) -> bool {
        self.snakes.iter().any(|s| s.id == snake)
    }

    pub fn valid_moves(&self, snake: u8) -> [bool; 4] {
        let mut moves = [false; 4];
        if let Some(snake) = self.snakes.iter().find(|s| s.id == snake) {
            for (i, d) in Direction::iter().enumerate() {
                let p = snake.head().apply(d);
                moves[i] = self.grid.has(p) && self.grid[p] != Cell::Occupied;
            }
        }
        moves
    }

    /// Moves the given snake in the given direction
    pub fn step(&mut self, moves: [Direction; 4]) {
        // pop tail
        for snake in &mut self.snakes {
            let tail = snake.body.pop_front().unwrap();
            let new_tail = snake.body[0];
            if tail != new_tail {
                self.grid[tail] = Cell::Free;
            }
        }

        let mut survivors = [None; 4];

        // move head & eat
        for snake in &mut self.snakes {
            let dir = moves[snake.id as usize];
            let head = snake.head().apply(dir);
            if self.grid.has(head) && snake.health > 0 && self.grid[head] != Cell::Occupied {
                if self.grid[head] == Cell::Food {
                    snake.body.push_front(snake.body[0]);
                    snake.health = 100;
                } else {
                    snake.health -= 1;
                };
                snake.body.push_back(head);
                survivors[snake.id as usize] = Some((head, snake.body.len()));
            }
        }

        // check head to head
        for i in 0..3 {
            for j in i + 1..4 {
                if let Some(((head_i, len_i), (head_j, len_j))) = survivors[i].zip(survivors[j]) {
                    if head_i == head_j {
                        use std::cmp::Ordering;
                        match len_i.cmp(&len_j) {
                            Ordering::Less => survivors[i] = None,
                            Ordering::Greater => survivors[j] = None,
                            Ordering::Equal => {
                                survivors[i] = None;
                                survivors[j] = None;
                            }
                        }
                    }
                }
            }
        }

        // remove died snakes
        for (i, survivor) in survivors.iter().enumerate() {
            if let Some(survivor) = *survivor {
                self.grid[survivor.0] = Cell::Occupied;
            } else if let Some(pos) = self.snakes.iter().position(|s| s.id == i as u8) {
                for &p in &self.snakes[pos].body {
                    self.grid[p] = Cell::Free
                }
                self.snakes.remove(pos);
            }
        }
    }
}

#[cfg(test)]
mod test {

    #[test]
    fn game_step_test() {
        use super::*;
        use Direction::*;
        let snakes = [
            SnakeData::new(
                100,
                vec![Vec2D::new(4, 8), Vec2D::new(4, 7), Vec2D::new(4, 6)],
            ),
            SnakeData::new(
                100,
                vec![Vec2D::new(6, 8), Vec2D::new(6, 7), Vec2D::new(6, 6)],
            ),
        ];
        let mut game = Game::new(11, 11, &snakes, &[]);
        println!("{:?}", game.grid);
        game.step([Right, Right, Up, Up]);

        println!("{:?}", game.grid);
        assert!(game.snake_is_alive(0));
        assert!(game.snake_is_alive(1));
        assert_eq!(game.grid[Vec2D::new(4, 6)], Cell::Free);
        assert_eq!(game.grid[Vec2D::new(5, 8)], Cell::Occupied);
        assert_eq!(game.grid[Vec2D::new(6, 6)], Cell::Free);
        assert_eq!(game.grid[Vec2D::new(7, 8)], Cell::Occupied);

        game.step([Right, Right, Up, Up]);
        println!("{:?}", game.grid);
        assert!(!game.snake_is_alive(0));
        assert_eq!(game.grid[Vec2D::new(5, 8)], Cell::Free);
        assert!(game.snake_is_alive(1));
        assert_eq!(game.grid[Vec2D::new(8, 8)], Cell::Occupied);

        let mut game = Game::new(11, 11, &snakes, &[]);
        game.step([Right, Left, Up, Up]);
        println!("{:?}", game.grid);
        assert!(!game.snake_is_alive(0));
        assert!(!game.snake_is_alive(1));
    }

    #[test]
    #[ignore]
    fn game_step_circle() {
        use super::*;
        use std::time::Instant;
        let snakes = [SnakeData::new(
            100,
            vec![
                Vec2D::new(4, 8),
                Vec2D::new(4, 7),
                Vec2D::new(4, 6),
                Vec2D::new(5, 6),
                Vec2D::new(6, 6),
                Vec2D::new(6, 6),
                Vec2D::new(6, 6),
            ],
        )];
        let mut game = Game::new(11, 11, &snakes, &[]);
        println!("{:?}", game.grid);

        let start = Instant::now();
        loop {
            use Direction::*;
            game.step([Up, Up, Up, Up]);
            game.step([Up, Up, Up, Up]);
            game.step([Right, Up, Up, Up]);
            game.step([Right, Up, Up, Up]);
            game.step([Down, Up, Up, Up]);
            game.step([Down, Up, Up, Up]);
            game.step([Left, Up, Up, Up]);
            game.step([Left, Up, Up, Up]);
            if !game.snake_is_alive(0) {
                break;
            }
        }
        println!("Dead after {}us", (Instant::now() - start).as_nanos());
    }

    #[test]
    #[ignore]
    fn game_step_random() {
        use super::*;
        use rand::{
            distributions::{Distribution, Uniform},
            seq::IteratorRandom,
        };
        use std::time::{Duration, Instant};
        const SIMULATION_TIME: usize = 200;

        let snakes = [
            SnakeData::new(
                100,
                vec![Vec2D::new(6, 7), Vec2D::new(6, 7), Vec2D::new(6, 7)],
            ),
            SnakeData::new(
                100,
                vec![Vec2D::new(3, 2), Vec2D::new(3, 2), Vec2D::new(3, 2)],
            ),
            SnakeData::new(
                100,
                vec![Vec2D::new(7, 3), Vec2D::new(7, 3), Vec2D::new(7, 3)],
            ),
            SnakeData::new(
                100,
                vec![Vec2D::new(3, 8), Vec2D::new(3, 8), Vec2D::new(3, 8)],
            ),
        ];
        let mut rng = rand::thread_rng();
        let mut game = Game::new(11, 11, &snakes, &[]);

        let dist = Uniform::from(0..11);
        for _ in 0..20 {
            let p = Vec2D::new(dist.sample(&mut rng), dist.sample(&mut rng));
            if game.grid[p] == Cell::Free {
                game.grid[p] = Cell::Food;
            }
        }

        println!("{:?}", game.grid);

        let start = Instant::now();
        let mut game_num = 0_usize;
        loop {
            let mut turn = 0;
            let mut game = game.clone();
            loop {
                let mut moves = [Direction::Up; 4];
                for i in 0..4 {
                    moves[i as usize] = game
                        .valid_moves(i)
                        .iter()
                        .enumerate()
                        .filter(|&(_, valid)| *valid)
                        .map(|v| Direction::from(v.0 as u8))
                        .choose(&mut rng)
                        .unwrap_or(Direction::Up);
                }
                game.step(moves);

                // println!("{} {:?}", turn, game.grid);

                if game.outcome() != Outcome::None {
                    println!(
                        "game {}: {:?} after {} turns",
                        game_num,
                        game.outcome(),
                        turn
                    );
                    break;
                }
                turn += 1;
            }
            game_num += 1;

            if Instant::now() > start + Duration::from_millis(SIMULATION_TIME as _) {
                break;
            }
        }
        println!("Played {} games in {}ms", game_num, SIMULATION_TIME);
    }
}
