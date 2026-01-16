// DONE:
// (1): Basic board representation
// (2): Printing boards

// TODO:
// (3): Generating possible moves
// (4): Playing a random move
// (5): Evaluating boards
// (6): Playing via minimax
// (7): Spinners for loading, other niceities
// (8): Acting as an engine (hooking up to an interface)
// (9): Optimisations
// (A): New board representation

mod board;

use std::io;
use rand::Rng;
use crate::board::Board;

struct Args {
    // whether to play interactively, or act as an engine
    // backend (basic, bitboard, etc.)
    // optimisation level
    // verbosity
}

fn main() {
    println!("Starting Crake...");

    let mut board = Board::new();
    println!("{board}");

    let mut input = String::new();
    let stdin = io::stdin();
    let mut rng = rand::rng();
    loop {
        println!("(p)rint board, (m)ake a random move, (q)uit");
        stdin.read_line(&mut input).expect("Failed to read the line");
        let command = input.trim_ascii_end();

        if command == "p" {
            println!("{board}");
        } else if command == "m" {
            let moves = board.generate_pseudomoves();
            let move_index = rng.random_range(0..moves.len());
            board.make_move(moves[move_index]);
            println!("Move: {:?}", moves[move_index]);
        } else if command == "q" {
            break;
        } else {
            println!("Unknown command '{input}'");
        }

        input.clear();
    }
}
