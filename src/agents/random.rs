use super::Agent;
use crate::env::*;
use rand::seq::IteratorRandom;

#[derive(Debug)]
pub struct Random {}

impl Default for Random {
    fn default() -> Random {
        Random {}
    }
}

impl Agent for Random {
    fn start(&mut self, _: &GameRequest) {}
    fn step(&mut self, _: &GameRequest) -> MoveResponse {
        let mut rng = rand::thread_rng();
        MoveResponse::new(Direction::iter().choose(&mut rng).unwrap_or(Direction::Up))
    }
    fn end(&mut self, _: &GameRequest) {}
}