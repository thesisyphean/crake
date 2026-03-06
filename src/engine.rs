use crate::{Board, moves::Move};
use std::cmp::{self, Ordering};

#[derive(Clone)]
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

    pub fn set_search_depth(&mut self, new_depth: u8) {
        self.search_depth = new_depth;
    }

    pub fn make_move(&mut self, cmove: Move) {
        self.board.make_move(cmove);
    }

    pub fn best_move(&mut self) -> Move {
        let moves = self.board.generate_moves();

        let mut best_move = moves[0];
        let mut best_score = i32::MIN;
        for cmove in moves {
            let move_data = self.board.make_move(cmove);
            let score = -self.negamax_alpha_beta_pruning(self.search_depth, i32::MIN + 1, i32::MAX);
            self.board.unmake_move(cmove, move_data);

            if score > best_score {
                best_move = cmove;
                best_score = score;
            }
        }

        best_move
    }

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

    fn negamax_alpha_beta_pruning(&mut self, depth: u8, mut alpha: i32, beta: i32) -> i32 {
        if depth == 0 {
            return self.board.value();
        }

        let mut moves = self.board.generate_moves();
        Self::order_moves(&mut moves);

        // If there are no moves available, i.e. checkmate,
        // then the minimum score is returned, but i32::MIN cannot be negated
        // due to overflow, so 1 is added to avoid this issue
        let mut best_score = i32::MIN + 1;
        for cmove in moves {
            let move_data = self.board.make_move(cmove);
            let score = -self.negamax_alpha_beta_pruning(depth - 1, -beta, -alpha);
            self.board.unmake_move(cmove, move_data);

            if score > best_score {
                best_score = score;
                alpha = cmp::max(alpha, score);
            }

            if score >= beta {
                return best_score;
            }
        }

        best_score
    }

    // TODO: Change this to a serious ordering
    fn order_moves(moves: &mut Vec<Move>) {
        moves.sort_unstable_by(|ma, mb| {
            if let Move::Standard(_, _, Some(_)) = ma {
                if let Move::Standard(_, _, Some(_)) = mb {
                    return Ordering::Equal;
                }
                return Ordering::Less;
            }

            if let Move::Standard(_, _, Some(_)) = mb {
                return Ordering::Greater;
            }

            Ordering::Equal
        });
    }
}
