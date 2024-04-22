use log::{debug, warn};
use rand::{rngs::SmallRng, seq::IteratorRandom, Rng};

use crate::{
    agents::Agent,
    env::{v2, Direction},
    game::{Game, Outcome},
    grid::CellT,
};

pub use snork_engine::simulate::init_game;

pub async fn play_game(
    agents: &[Agent],
    game: &mut Game,
    timeout: u64,
    food_rate: f64,
    shrink_turns: usize,
    rng: &mut SmallRng,
) -> Outcome {
    let mut food_count = 4;

    debug!("init: {game:?}");

    let mut hazard_insets = [0; 4];

    for turn in game.turn.. {
        let mut moves = [Direction::Up; 4];
        for i in 0..game.snakes.len() {
            if game.snakes[i].alive() {
                // Agents assume player 0 is you.
                game.snakes.swap(0, i);

                let response = agents[i].step_internal(timeout, game).await;
                moves[i] = response.r#move;

                game.snakes.swap(0, i);
            }
        }
        debug!("Moves: {moves:?}");

        game.step(&moves);

        debug!("{}: {:?}", turn, game);

        let outcome = game.outcome();
        if outcome != Outcome::None {
            warn!("game: {outcome:?} after {turn} turns");
            return outcome;
        }

        // Check if snakes have consumed food
        for snake in &game.snakes {
            if snake.alive() && snake.health == 100 {
                food_count -= 1;
            }
        }

        // Spawn food
        if food_count == 0 || rng.gen::<f64>() < food_rate {
            if let Some(cell) = game
                .grid
                .cells
                .iter_mut()
                .filter(|c| c.t == CellT::Free)
                .choose(rng)
            {
                cell.t = CellT::Food;
                food_count += 1;
            }
        }

        // Hazards
        if turn > 0
            && turn % shrink_turns == 0
            && hazard_insets[0] + hazard_insets[2] < game.grid.height
            && hazard_insets[1] + hazard_insets[3] < game.grid.width
        {
            let dir = rng.gen_range(0..4);
            hazard_insets[dir] += 1;
            if dir % 2 == 0 {
                let y = if dir == 0 {
                    hazard_insets[dir] - 1
                } else {
                    game.grid.height - hazard_insets[dir]
                };
                for x in 0..game.grid.width {
                    game.grid[v2(x as _, y as _)].hazard = true;
                }
            } else {
                let x = if dir == 1 {
                    hazard_insets[dir] - 1
                } else {
                    game.grid.width - hazard_insets[dir]
                };
                for y in 0..game.grid.height {
                    game.grid[v2(x as _, y as _)].hazard = true;
                }
            }
        }
    }
    Outcome::Match
}
