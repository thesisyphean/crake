use crate::board::{Board, Colour, Move, Piece, PieceKind, RawMove};
use rand::Rng;
use std::cmp;

const BOARD_SIZE: usize = 8;

// Remove pub on board?
pub struct Engine {
    pub board: RefCell<Board>,
    pub engine_colour: Colour,
    search_depth: u8,
    current_moves: Option<Vec<Move>>,
}

impl Engine {
    pub fn new(board_fen: Option<&str>, engine_colour: Colour, search_depth: u8) -> Self {
        Engine {
            // TODO: Check fen
            board: if let Some(fen) = board_fen {
                Board::from_fen_unchecked(fen)
            } else {
                Board::new()
            },
            engine_colour,
            search_depth,
            current_moves: None,
        }
    }

    pub fn board(&self) -> &[Option<Piece>; 64] {
        &self.board.squares
    }

    pub fn get_square(&self, square: usize) -> Option<Piece> {
        self.board.squares[square]
    }

    pub fn valid_move(&mut self, rmove @ RawMove(from, to): RawMove) -> bool {
        self.load_moves();
        let current_moves = self.current_moves.as_ref().unwrap();
        let square = self.board.squares[from];

        // Castling
        if let Some(Piece {
            kind: PieceKind::King,
            colour,
        }) = square
        {
            let diff = if to >= from { to - from } else { from - to };
            if diff == 2 || diff == 3 {
                return current_moves.contains(&Move::Castling(diff == 2));
            }
        }

        if let Some(Piece {
            kind: PieceKind::Pawn,
            colour,
        }) = square
        {
            // Promotion
            if to / BOARD_SIZE == BOARD_SIZE - 1 {
                // If you can promote to the queen, you can promote to any other piece,
                //   the queen is simply used as a dummy option
                return current_moves.contains(&Move::Promotion(rmove, PieceKind::Queen));
            }

            // En passant
            if let None = self.board.squares[to] {
                return current_moves.contains(&Move::EnPassant(rmove));
            }
        }

        // Standard move
        if let Some(piece) = self.board.squares[from] {
            return current_moves.contains(&Move::Standard(piece, rmove, self.board.squares[to]));
        }

        false
    }

    // TODO: Assert or return an error if invalid move
    pub fn player_move(&mut self, cmove: Move) {
        self.board.make_move(cmove);
        self.current_moves = None;
    }

    fn load_moves(&mut self) {
        if let None = self.current_moves {
            self.current_moves = Some(self.board.generate_pseudomoves());
        }
    }

    fn piece_value(piece: Piece) -> i32 {
        match piece.kind {
            PieceKind::King => 10_000,
            PieceKind::Queen => 9,
            PieceKind::Rook => 5,
            PieceKind::Bishop => 3,
            PieceKind::Knight => 3,
            PieceKind::Pawn => 1,
        }
    }

    // Calculates a value for the current board relative to the engine colour
    // TODO: Add value maps for board positions
    fn evaluate_board(&self) -> i32 {
        let mut value = 0;

        for i in 0..64 {
            if let Some(piece) = self.board.squares[i] {
                value += Self::piece_value(piece)
                    * if let Colour::White = piece.colour {
                        1
                    } else {
                        -1
                    };
            }
        }

        // Negamax needs the valuation to be relative to the side to play
        if self.board.turn == Colour::Black {
            value *= -1;
        }

        value
    }

    pub fn engine_move(&mut self) {
        self.load_moves();
        let moves = self.current_moves.as_ref().unwrap();

        let mut best_move = moves[0];
        let mut best_score = i32::MIN;
        for &cmove in moves {
            self.board.make_move(cmove);
            let score = -self.negamax(self.search_depth);
            self.board.unmake_move(cmove);

            if score > best_score {
                best_move = cmove;
                best_score = score;
            }
        }

        self.board.make_move(best_move);
        self.current_moves = None;
    }

    fn negamax(&mut self, depth: u8) -> i32 {
        if depth == 0 {
            return self.evaluate_board();
        }

        let moves = self.board.generate_pseudomoves();
        let mut best_score = i32::MIN;
        for cmove in moves {
            self.board.make_move(cmove);
            let score = -self.negamax(self.search_depth - 1);
            self.board.unmake_move(cmove);
            best_score = cmp::max(best_score, score);
        }

        best_score
    }
}
