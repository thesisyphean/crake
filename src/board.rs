use std::{fmt::{self, Display, Formatter}, collections::binary_heap};

#[derive(Clone, Copy)]
enum PieceType {
    King,
    Queen,
    Rook,
    Bishop,
    Knight,
    Pawn,
}

impl PieceType {
    fn from_algebraic(p: char) -> Self {
        match p.to_ascii_uppercase() {
            'K' => PieceType::King,
            'Q' => PieceType::Queen,
            'R' => PieceType::Rook,
            'B' => PieceType::Bishop,
            'N' => PieceType::Knight,
            'P' => PieceType::Pawn,
            _ => { panic!("Invalid algebraic notation for piece '{p}'") },
        }
    }

    fn to_algebraic(self) -> char {
        match self {
            PieceType::King   => 'K',
            PieceType::Queen  => 'Q',
            PieceType::Rook   => 'R',
            PieceType::Bishop => 'B',
            PieceType::Knight => 'N',
            PieceType::Pawn   => 'P',
        }
    }

    // TODO: Make these white pieces, and change math for them in other to_visual_char
    fn to_visual_char(self) -> char {
        match self {
            PieceType::King   => '♔',
            PieceType::Queen  => '♕',
            PieceType::Rook   => '♖',
            PieceType::Bishop => '♗',
            PieceType::Knight => '♘',
            PieceType::Pawn   => '♙',
        }
    }
}

#[derive(Clone, Copy)]
struct Piece(PieceType, bool);

impl Piece {
    // TODO: Should this mention unchecked, write an option version?
    fn from_algebraic(p: char) -> Self {
        Piece(PieceType::from_algebraic(p), p.is_ascii_uppercase())
    }

    fn to_visual_char(self) -> char {
        if self.1 {
            // The white pieces are directly after the black pieces in unicode
            char::from_u32(self.0.to_visual_char() as u32 + 6).unwrap()
        } else {
            self.0.to_visual_char()
        }
    }
}

pub struct Board {
    // TODO: Explain how this is indexed
    squares: [Option<Piece>; 64],
    white_to_play: bool,
    castling: [bool; 4],
    en_passant: Option<usize>,
    halfmoves: u32,
    fullmoves: u32,
    precomputed_moves: PrecomputedMoves,
}

impl Board {
    pub fn from_FEN_unchecked(FEN: &str) -> Self {
        let parts: Vec<_> = FEN.split_whitespace().collect();

        let mut squares = [None; 64];
        let mut i = 56;
        for c in parts[0].chars() {
            if c == '/' {
                i -= 16;
            }
            else if c.is_ascii_digit() {
                // This is safe as we know it is a digit
                i += c.to_digit(10).unwrap() as usize;
            }
            else {
                squares[i] = Some(Piece::from_algebraic(c));
                i += 1;
            }
        }

        let castling = [parts[2].contains('K'), parts[2].contains('Q'),
                        parts[2].contains('k'), parts[2].contains('q')];

        Board {
            squares,
            white_to_play: parts[1] == "w",
            castling,
            en_passant: if parts[3] == "-" { None } else { Some(parts[3].parse().unwrap()) },
            halfmoves: parts[4].parse().unwrap(),
            fullmoves: parts[5].parse().unwrap(),
            precomputed_moves: PrecomputedMoves::new(),
        }
    }

    pub fn new() -> Self {
        Self::from_FEN_unchecked("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
    }

    fn add_pseudomoves(&self, moves: &mut Vec<Move>, piece: Piece, possible_moves: &Vec<Move>) {
        for cmove in possible_moves {
            if let Some(target_piece) = self.squares[cmove.1] {
                if target_piece.1 == piece.1 {
                    continue;
                }
            }

            moves.push(cmove);
        }
    }

    // TODO: Filter for legal moves
    pub fn generate_pseudomoves(&self) -> Vec<Move> {
        let mut moves = Vec::new();

        for i in 0..64 {
            if let Some(piece) = self.squares[i] {
                if self.white_to_play == piece.1 {
                    match piece.0 {
                        PieceType::King => self.add_pseudomoves(&mut moves, piece, self.precomputed_moves.king_moves);
                        PieceType::Knight => self.add_pseudomoves(&mut moves, piece, self.precomputed_moves.knight_moves);
                        PieceType::Pawn => {
                            // TODO
                        },
                        // TODO
                    }
                }
            }
        }

        moves
    }
}

impl Display for Board {
    // TODO: Have settings on the board for type of printing - algebraic or visual? Better way to
    // pass this information?
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        writeln!(f, "┌───┬───┬───┬───┬───┬───┬───┬───┐")?;

        for r in 0..8 {
            for c in 0..8 {
                // TODO: Explain indexing?
                let square = if let Some(piece) = self.squares[(56 - 8 * r) + c] {
                    piece.to_visual_char()
                } else {
                    ' '
                };

                write!(f, "│ {square} ")?;
            }

            writeln!(f, "│")?;
            
            if r != 7 {
                writeln!(f, "├───┼───┼───┼───┼───┼───┼───┼───┤")?;
            }
        }

        write!(f, "└───┴───┴───┴───┴───┴───┴───┴───┘")?;

        Ok(())
    }
}

#[derive(Clone, Copy)];
struct Move(usize, usize);

struct PrecomputedMoves {
    king_moves: [Vec<Move>; 64],
    rook_moves: [Vec<Move>; 64],
    bishop_moves: [Vec<Move>; 64],
    knight_moves: [Vec<Move>; 64],
}

impl PrecomputedMoves {
    fn i8x8_to_12x12(i: usize) -> usize {
        let (y, x) = (i / 8, i % 8);
        (2 + y) * 12 + (2 + x)
    }

    fn i12x12_to_8x8(i: usize) -> usize {
        let (y, x) = (i / 12, i % 12);
        // TODO
    }

    // TODO: Fix these taking in i and start, typing
    fn add_valid_jumps(i: usize, start: i32, directions: &Vec<i32>, moves: &mut [Vec<Move>; 64]) {
        for dir in directions {
            let target = start + dir;

            if target >= (12 * 2 + 2) && target <= (12 * 10 - 2) && target % 12 >= 2 && target % 12 <= 9 {
                moves[i].push(Move(i, target ));
            }
        }
    }

    fn add_valid_slides(i: usize, start: i32, direction: &[i32], moves: &mut [Vec<Move>; 64]) {
        for dir in direction {
            let mut target = start + dir;

            while target >= (12 * 2 + 2) && target <= (12 * 10 - 2) && target % 12 >= 2 && target % 12 <= 9 {
                moves[start].push(Move(start, target));
                target += dir;
            }
        }
    }

    fn new() -> Self {
        let directions = [8, 1, -8, -1, 9, -7, -9, 7];
        let knight_directions = [17, 10, -6, -15, -17, -10, 6, 15];

        let mut king_moves: [Vec::new(); 64];
        let mut rook_moves: [Vec::new(); 64];
        let mut bishop_moves: [Vec::new(); 64];
        let mut knight_moves: [Vec::new(); 64];

        for r in 0..8 {
            for c in 0..8 {
                // TODO:
                let i = ((12 * 9 + 2) ) + c;

                // TODO: Fix these taking in i, start
                Self::add_valid_jumps(i, &directions, &mut king_moves);
                Self::add_valid_jumps(i, &knight_directions, &mut knight_moves);
                Self::add_valid_slides(i, &directions[0..4], &mut rook_moves);
                Self::add_valid_slides(i, &directions[4..8], &mut bishop_moves);
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
