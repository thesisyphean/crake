use crate::board::{Board, Move};
use std::cmp;

pub struct Engine<B: Board> {
    board: B,
    search_depth: u8,
}

impl<B: Board> Engine<B> {
    pub fn new(board_fen: Option<&str>, search_depth: u8) -> Self {
        Engine {
            // TODO: Check fen
            board: B::from_fen(
                board_fen.unwrap_or("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"),
            ),
            search_depth,
        }
    }

    pub fn player_move(&mut self, cmove: Move) {
        self.board.make_move(cmove);
    }

    pub fn engine_move(&mut self) -> Move {
        let moves = self.board.generate_moves();

        let mut best_move = moves[0];
        let mut best_score = i32::MIN;
        for cmove in moves {
            let move_data = self.board.make_move(cmove);
            let score = -self.negamax(self.search_depth);
            self.board.unmake_move(cmove, move_data);

            if score > best_score {
                best_move = cmove;
                best_score = score;
            }
        }

        self.board.make_move(best_move);
        best_move
    }

    // TODO: Alpha-beta pruning
    fn negamax(&mut self, depth: u8) -> i32 {
        if depth == 0 {
            return self.board.value();
        }

        let moves = self.board.generate_moves();

        // If there are no moves available, i.e. checkmate,
        // then the minimum score is returned, but i32::MIN cannot be negated
        // due to overflow, so 1 is added to avoid this issue
        let mut best_score = i32::MIN + 1;
        for cmove in moves {
            let move_data = self.board.make_move(cmove);
            let score = -self.negamax(depth - 1);
            self.board.unmake_move(cmove, move_data);
            best_score = cmp::max(best_score, score);
        }

        best_score
    }
}
