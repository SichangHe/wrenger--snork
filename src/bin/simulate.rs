use clap::Parser;
use log::{info, warn};
use owo_colors::OwoColorize;

use snork::agents::Agent;
use snork::env::*;
use snork::game::{Game, Outcome};
use snork::logging;

use rand::prelude::*;
use snork::simulate::{init_game, play_game};
use std::iter::repeat;
use std::time::Instant;

#[derive(clap::Parser)]
#[clap(version, author, about = "Simulate a game between agents.")]
struct Opts {
    /// Time each snake has for a turn.
    #[clap(long, default_value_t = 200)]
    timeout: u64,
    /// Board height.
    #[clap(long, default_value_t = 11)]
    width: usize,
    /// Board width.
    #[clap(long, default_value_t = 11)]
    height: usize,
    /// Chance new food spawns.
    #[clap(long, default_value_t = 0.15)]
    food_rate: f64,
    /// Number of turns after which the hazard expands.
    #[clap(short, long, default_value_t = 25)]
    shrink_turns: usize,
    /// Number of games that are played.
    #[clap(short, long, default_value_t = 1)]
    game_count: usize,
    /// Swap agent positions to get more accurate results.
    #[clap(long)]
    swap: bool,
    /// Seed for the random number generator.
    #[clap(long, default_value_t = 0)]
    seed: u64,
    /// Start config.
    #[clap(long, value_parser = parse_request)]
    init: Option<GameRequest>,
    /// Configurations.
    #[clap()]
    agents: Vec<Agent>,
}

fn parse_request(s: &str) -> Result<GameRequest, serde_json::Error> {
    serde_json::from_str(s)
}

#[tokio::main]
async fn main() {
    logging();

    let Opts {
        timeout,
        width,
        height,
        food_rate,
        shrink_turns,
        game_count,
        swap,
        seed,
        init,
        mut agents,
    } = Opts::parse();

    assert!(agents.len() <= 4, "Only up to 4 snakes are supported");
    info!("agents: {agents:?}");

    let start = Instant::now();

    let mut wins = repeat(0).take(agents.len()).collect::<Vec<usize>>();

    for _ in 0..agents.len() {
        let mut rng = if seed == 0 {
            SmallRng::from_entropy()
        } else {
            SmallRng::seed_from_u64(seed)
        };

        for i in 0..game_count {
            let mut game = if let Some(request) = &init {
                Game::from_request(request)
            } else {
                init_game(width, height, agents.len(), &mut rng)
            };

            let outcome = play_game(
                &agents,
                &mut game,
                timeout,
                food_rate,
                shrink_turns,
                &mut rng,
            )
            .await;
            if let Outcome::Winner(winner) = outcome {
                wins[winner as usize] += 1;
            }
            warn!(
                "{}: {i} {}ms",
                "Finish Game".bright_green(),
                start.elapsed().as_millis()
            );
        }

        if !swap {
            break;
        }
        // Swap agents
        wins.rotate_left(1);
        agents.rotate_left(1);
    }

    println!("Agents: {agents:?}");
    println!("Result: {wins:?}");
}
