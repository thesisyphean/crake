pub mod board;
pub mod engine;
pub mod moves;
pub mod piece;

use moves::{Move, MoveData};

pub trait Board {
    fn from_fen(fen: &str) -> Self;
    fn generate_moves(&mut self) -> Vec<Move>;
    fn make_move(&mut self, cmove: Move) -> MoveData;
    fn unmake_move(&mut self, cmove: Move, move_data: MoveData);
    fn value(&self) -> i32;
}
