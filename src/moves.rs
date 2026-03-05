use crate::piece::{Piece, PieceKind};
use arr_macro::arr;

/// A raw move from one square on the board to another
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RawMove(pub usize, pub usize);

impl RawMove {
    pub fn rotate(self) -> Self {
        // NOTE TO SELF, IT'S 63
        RawMove(63 - self.0, 63 - self.1)
    }
}

/// One of four possible chess moves:
/// - A standard move from one square to another, with a possible capture
/// - Castling, either kingside or queenside
/// - Promotion to a specific piece
/// - En passant
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Move {
    /// Option is a possible capture
    Standard(Piece, RawMove, Option<Piece>),
    /// true is kingside, false is queenside
    Castling(bool),
    Promotion(RawMove, PieceKind),
    /// The RawMove is the move of the capturing pawn
    EnPassant(RawMove),
}

impl Move {
    /// Takes a standard move and returns it with the given piece in the capture Option
    pub fn insert_capture(self, captured_piece: Piece) -> Self {
        if let Self::Standard(p, r, None) = self {
            return Self::Standard(p, r, Some(captured_piece));
        }

        panic!("Cannot insert capture into non-standard move");
    }

    pub fn rotate(self) -> Self {
        match self {
            Self::Standard(p, raw, c) => Self::Standard(p, raw.rotate(), c),
            Self::Castling(side) => Self::Castling(side),
            Self::Promotion(raw, k) => Self::Promotion(raw.rotate(), k),
            Self::EnPassant(raw) => Self::EnPassant(raw.rotate()),
        }
    }
}

/// The information that is otherwise unrecoverable when a move is made, specifically:
/// - A square that a pawn passed over, allowing for en passant
/// - The castling rights
pub struct MoveData {
    pub en_passant: Option<usize>,
    pub castling: [bool; 4],
}

pub struct PrecomputedMoves {
    pub king_moves: [Vec<RawMove>; 64],
    pub rook_moves: [Vec<Vec<RawMove>>; 64],
    pub bishop_moves: [Vec<Vec<RawMove>>; 64],
    pub knight_moves: [Vec<RawMove>; 64],
}

impl PrecomputedMoves {
    fn i12x12_to_8x8(i: usize) -> usize {
        let (y, x) = (i / 12, i % 12);
        (y - 2) * 8 + (x - 2)
    }

    fn add_valid_jumps(i: usize, directions: &[i32], moves: &mut [Vec<RawMove>; 64]) {
        let i8x8 = Self::i12x12_to_8x8(i);

        for dir in directions {
            let target = i as i32 + dir;

            if target >= (12 * 2 + 2)
                && target < (12 * 10 - 2)
                && target % 12 >= 2
                && target % 12 < 10
            {
                moves[i8x8].push(RawMove(i8x8, Self::i12x12_to_8x8(target as usize)));
            }
        }
    }

    fn add_valid_slides(i: usize, direction: &[i32], moves: &mut [Vec<Vec<RawMove>>; 64]) {
        let i8x8 = Self::i12x12_to_8x8(i);

        for dir in direction {
            let mut current_moves = Vec::new();
            let mut target = i as i32 + dir;

            while target >= (12 * 2 + 2)
                && target < (12 * 10 - 2)
                && target % 12 >= 2
                && target % 12 < 10
            {
                current_moves.push(RawMove(i8x8, Self::i12x12_to_8x8(target as usize)));
                target += dir;
            }

            moves[i8x8].push(current_moves);
        }
    }

    pub fn new() -> Self {
        // 12x12 index offsets, N,E,S,W first and then NE,SE,SW,NW
        let directions = [12, 1, -12, -1, 13, -11, -13, 11];
        // Starting NNE, rotating clockwise
        let knight_directions = [25, 14, -10, -23, -25, -14, 10, 23];

        let mut king_moves = arr![Vec::new(); 64];
        let mut rook_moves = arr![Vec::new(); 64];
        let mut bishop_moves = arr![Vec::new(); 64];
        let mut knight_moves = arr![Vec::new(); 64];

        for r in 0..8 {
            for c in 0..8 {
                let i = (12 * (9 - r) + 2) + c;

                Self::add_valid_jumps(i, &directions, &mut king_moves);
                Self::add_valid_slides(i, &directions[0..4], &mut rook_moves);
                Self::add_valid_slides(i, &directions[4..8], &mut bishop_moves);
                Self::add_valid_jumps(i, &knight_directions, &mut knight_moves);
            }
        }

        Self {
            king_moves,
            rook_moves,
            bishop_moves,
            knight_moves,
        }
    }
}
