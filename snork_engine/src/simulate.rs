use log::warn;
use rand::prelude::*;
use rand::{seq::IteratorRandom, Rng};

use crate::{
    env::v2,
    game::{Game, Snake},
    grid::CellT,
};

pub fn init_game<R: RngCore>(width: usize, height: usize, num_agents: usize, rng: &mut R) -> Game {
    if width % 2 == 0 || height % 2 == 0 {
        warn!("If the dimension are even, the initial board configuration is unfair!");
    }
    if width != height {
        warn!("If width != height, the initial board configuration is unfair!");
    }

    // Either start in the corners or in the middle of the edges
    let mut start_positions = if rng.gen() {
        // Corners
        [
            v2(1, 1),
            v2((width - 2) as _, 1),
            v2((width - 2) as _, (height - 2) as _),
            v2(1, (height - 2) as _),
        ]
    } else {
        // Edges
        [
            v2((width / 2) as _, 1),
            v2((width - 2) as _, (height / 2) as _),
            v2((width / 2) as _, (height - 2) as _),
            v2(1, (height / 2) as _),
        ]
    }
    .into_iter()
    .choose_multiple(rng, num_agents);

    start_positions.shuffle(rng);

    let snakes = start_positions
        .into_iter()
        .map(|p| Snake::new(vec![p; 3].into(), 100))
        .collect();

    let mut game = Game::new(0, width, height, snakes, &[], &[]);

    // Food at center
    game.grid[(width / 2, height / 2).into()].t = CellT::Food;

    // Spawn 1 food 2 steps away from each snake
    for snake in game.snakes.clone() {
        let p = [v2(-1, -1), v2(-1, 1), v2(1, 1), v2(1, -1)]
            .into_iter()
            .map(|p| snake.head() + p)
            // Only free cells on the board
            .filter(|&p| game.grid.has(p) && game.grid[p].t != CellT::Owned)
            // Limit to a border cells (excluding the corners)
            .filter(|&p| {
                (p.x == 0 || p.x == game.grid.width as i16 - 1)
                    ^ (p.y == 0 || p.y == game.grid.height as i16 - 1)
            })
            .choose(rng);
        if let Some(p) = p {
            game.grid[p].t = CellT::Food;
        }
    }

    game
}
