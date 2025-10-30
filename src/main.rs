// TODO:
// (1): Basic board representation
// (2): Printing boards
// (3): Generating possible moves
// (4): Playing a random move
// (5): Evaluating boards
// (6): Playing via minimax
// (7): Spinners for loading, other niceities
// (8): Acting as an engine (hooking up to an interface)
// (9): Optimisations
// (A): New board representation

mod board;

use crate::board::Board;

struct Args {
    // whether to play interactively, or act as an engine
    // backend (basic, bitboard, etc.)
    // optimisation level
    // verbosity
}

fn main() {
    println!("This is crake!");

    let board = Board::new();
    println!("{board}");
}
