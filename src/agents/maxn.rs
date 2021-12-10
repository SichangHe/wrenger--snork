use std::time::Duration;
use std::time::Instant;

use crate::env::*;
use crate::game::search::{self, Heuristic};
use crate::game::Game;

use crate::util::argmax;

use tokio::sync::mpsc;
use tokio::time;

const FAST_TIMEOUT: u64 = 150;
const MAX_DEPTH: usize = 8;

pub async fn step<H: Heuristic>(
    heuristic: &H,
    request: &GameRequest,
    latency: u64,
) -> MoveResponse {
    let ms = request.game.timeout.saturating_sub(latency);
    if ms <= FAST_TIMEOUT {
        return step_fast(heuristic, request);
    }

    let game = Game::from_request(request);

    let (sender, mut receiver) = mpsc::channel(MAX_DEPTH);

    let _ = time::timeout(
        Duration::from_millis(ms),
        iterative_tree_search(heuristic, &game, sender),
    )
    .await;

    let mut result = None;
    while let Some(dir) = receiver.recv().await {
        result = Some(dir);
    }

    if let Some(dir) = result {
        return MoveResponse::new(Direction::from(dir as u8));
    }

    println!(">>> none");
    MoveResponse::new(game.valid_moves(0).next().unwrap_or(Direction::Up))
}

pub fn step_fast<H: Heuristic>(heuristic: &H, request: &GameRequest) -> MoveResponse {
    let game = Game::from_request(request);

    let start = Instant::now();
    let result = search::max_n(&game, 1, heuristic);

    println!(
        ">>> max_n 1 {:?}ms {:?}",
        start.elapsed().as_millis(),
        result
    );

    if let Some(dir) = argmax(result.iter()) {
        if result[dir] > search::LOSS {
            return MoveResponse::new(Direction::from(dir as u8));
        }
    }

    println!(">>> none");
    MoveResponse::new(game.valid_moves(0).next().unwrap_or(Direction::Up))
}

async fn iterative_tree_search<H: Heuristic>(
    heuristic: &H,
    game: &Game,
    sender: mpsc::Sender<Direction>,
) {
    // Iterative deepening
    for depth in 1..MAX_DEPTH {
        let (dir, value) = tree_search(heuristic, game, depth).await;

        // Stop and fallback to random possible move
        if value <= search::LOSS {
            break;
        }

        if sender.send(dir).await.is_err()
            // Terminate if we probably win/lose
            || value >= search::WIN
        {
            break;
        }
    }
}

/// Performes a tree search and returns the maximized heuristic and move.
pub async fn tree_search<H: Heuristic>(
    heuristic: &H,
    game: &Game,
    depth: usize,
) -> (Direction, f64) {
    let start = Instant::now();

    let result = search::async_max_n(&game, depth, heuristic).await;

    println!(
        ">>> max_n {} {:?}ms {:?}",
        depth,
        start.elapsed().as_millis(),
        result
    );

    argmax(result.iter())
        .map(|d| (Direction::from(d as u8), result[d]))
        .unwrap()
}