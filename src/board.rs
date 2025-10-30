use std::fmt::{self, Display, Formatter};

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
}

#[derive(Clone, Copy)]
struct Piece(PieceType, bool);

impl Piece {
    // TODO: Should this mention unchecked, write an option version?
    fn from_algebraic(p: char) -> Self {
        Piece(PieceType::from_algebraic(p), p.is_ascii_uppercase())
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
        }
    }

    pub fn new() -> Self {
        Self::from_FEN_unchecked("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
    }
}

// TODO NEXT: Implement display, check that the board is being created, displayed properly

impl Display for Board {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Board will be printed here...");

        Ok(())
    }
}
