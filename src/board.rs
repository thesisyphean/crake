use arr_macro::arr;
use std::fmt::{self, Display, Formatter};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Colour {
    White,
    Black,
}

impl Colour {
    fn invert(self) -> Self {
        if let Self::White = self {
            Self::Black
        } else {
            Self::White
        }
    }
}

impl From<bool> for Colour {
    fn from(white: bool) -> Self {
        if white { Colour::White } else { Colour::Black }
    }
}

impl From<Colour> for bool {
    fn from(colour: Colour) -> Self {
        colour == Colour::White
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PieceKind {
    King,
    Queen,
    Rook,
    Bishop,
    Knight,
    Pawn,
}

impl PieceKind {
    fn from_algebraic(p: char) -> Self {
        match p.to_ascii_uppercase() {
            'K' => PieceKind::King,
            'Q' => PieceKind::Queen,
            'R' => PieceKind::Rook,
            'B' => PieceKind::Bishop,
            'N' => PieceKind::Knight,
            'P' => PieceKind::Pawn,
            _ => {
                panic!("Invalid algebraic notation for piece '{p}'")
            }
        }
    }

    pub fn to_algebraic(self) -> char {
        match self {
            PieceKind::King => 'K',
            PieceKind::Queen => 'Q',
            PieceKind::Rook => 'R',
            PieceKind::Bishop => 'B',
            PieceKind::Knight => 'N',
            PieceKind::Pawn => 'P',
        }
    }

    // TODO: Make these white pieces, and change math for them in other to_visual_char
    fn to_visual_char(self) -> char {
        match self {
            PieceKind::King => '♔',
            PieceKind::Queen => '♕',
            PieceKind::Rook => '♖',
            PieceKind::Bishop => '♗',
            PieceKind::Knight => '♘',
            PieceKind::Pawn => '♙',
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Piece {
    pub kind: PieceKind,
    pub colour: Colour,
}

impl Piece {
    fn new(kind: PieceKind, colour: Colour) -> Self {
        Piece { kind, colour }
    }

    // TODO: Should this mention unchecked, write an option version?
    fn from_algebraic(p: char) -> Self {
        Piece {
            kind: PieceKind::from_algebraic(p),
            colour: Colour::from(p.is_ascii_uppercase()),
        }
    }

    fn to_visual_char(self) -> char {
        if let Colour::White = self.colour {
            // The white pieces are directly after the black pieces in unicode
            char::from_u32(self.kind.to_visual_char() as u32 + 6).unwrap()
        } else {
            self.kind.to_visual_char()
        }
    }

    // TODO: Colour difference?
    fn to_algebraic(self) -> char {
        if let Colour::White = self.colour {
            self.kind.to_algebraic()
        } else {
            self.kind.to_algebraic().to_ascii_lowercase()
        }
    }
}

// TODO: Change access modifiers everywhere!
pub struct Board {
    // TODO: Explain how this is indexed
    pub squares: [Option<Piece>; 64],
    pub turn: Colour,
    pub castling: [bool; 4],
    pub en_passant: Option<usize>,
    pub halfmoves: u32,
    pub fullmoves: u32,
    pub precomputed_moves: PrecomputedMoves,
}

impl Board {
    pub fn from_fen_unchecked(fen: &str) -> Self {
        let parts: Vec<_> = fen.split_whitespace().collect();

        let mut squares = [None; 64];
        let mut i = 56;
        for c in parts[0].chars() {
            if c == '/' {
                i -= 16;
            } else if c.is_ascii_digit() {
                // This is safe as we know it is a digit
                i += c.to_digit(10).unwrap() as usize;
            } else {
                squares[i] = Some(Piece::from_algebraic(c));
                i += 1;
            }
        }

        let castling = [
            parts[2].contains('K'),
            parts[2].contains('Q'),
            parts[2].contains('k'),
            parts[2].contains('q'),
        ];

        Board {
            squares,
            turn: Colour::from(parts[1] == "w"),
            castling,
            en_passant: if parts[3] == "-" {
                None
            } else {
                Some(parts[3].parse().unwrap())
            },
            halfmoves: parts[4].parse().unwrap(),
            fullmoves: parts[5].parse().unwrap(),
            precomputed_moves: PrecomputedMoves::new(),
        }
    }

    pub fn new() -> Self {
        Self::from_fen_unchecked("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
    }

    fn add_pseudomoves(&self, moves: &mut Vec<Move>, piece: Piece, possible_moves: &Vec<RawMove>) {
        for rmove @ &RawMove(_, to) in possible_moves {
            if let Some(target_piece) = self.squares[to] {
                if target_piece.colour != piece.colour {
                    moves.push(Move::Standard(piece, *rmove, Some(target_piece)));
                }
            } else {
                moves.push(Move::Standard(piece, *rmove, None));
            }
        }
    }

    fn add_sliding_pseudomoves(
        &self,
        moves: &mut Vec<Move>,
        piece: Piece,
        possible_lines: &Vec<Vec<RawMove>>,
    ) {
        for line in possible_lines {
            for rmove @ &RawMove(_, to) in line {
                if let Some(target_piece) = self.squares[to] {
                    if target_piece.colour != piece.colour {
                        moves.push(Move::Standard(piece, *rmove, Some(target_piece)));
                    }

                    break;
                }

                moves.push(Move::Standard(piece, *rmove, None));
            }
        }
    }

    fn add_pawn_pseudomoves(&self, moves: &mut Vec<Move>, i: usize, piece: Piece) {
        // Move one rank forward
        if let None = self.squares[i + 8] {
            moves.push(Move::Standard(piece, RawMove(i, i + 8), None));

            // If at starting rank, move two ranks forward
            if i / 8 == 1 {
                if let None = self.squares[i + 16] {
                    moves.push(Move::Standard(piece, RawMove(i, i + 16), None));
                }
            }
        }

        // TODO: Combine these elegantly
        // Attack to the left
        if i % 8 != 0 {
            if let Some(target_piece) = self.squares[i + 7] {
                if target_piece.colour != piece.colour {
                    moves.push(Move::Standard(piece, RawMove(i, i + 7), Some(target_piece)));
                }
            }

            if let Some(target_square) = self.en_passant {
                if target_square == i + 7 {
                    moves.push(Move::EnPassant(RawMove(i, i + 7)));
                }
            }
        }

        // Attack to the right
        if i % 8 != 7 {
            if let Some(target_piece) = self.squares[i + 9] {
                if target_piece.colour != piece.colour {
                    moves.push(Move::Standard(piece, RawMove(i, i + 9), Some(target_piece)));
                }
            }

            if let Some(target_square) = self.en_passant {
                if target_square == i + 9 {
                    moves.push(Move::EnPassant(RawMove(i, i + 9)));
                }
            }
        }

        // Promotion
        if i / 8 == 6 {
            if let None = self.squares[i + 8] {
                moves.push(Move::Promotion(RawMove(i, i + 8), PieceKind::Queen));
                moves.push(Move::Promotion(RawMove(i, i + 8), PieceKind::Rook));
                moves.push(Move::Promotion(RawMove(i, i + 8), PieceKind::Bishop));
                moves.push(Move::Promotion(RawMove(i, i + 8), PieceKind::Knight));
            }
        }
    }

    // TODO: Filter for legal moves
    // TODO: Unsafe to not require &mut self?
    pub fn generate_pseudomoves(&mut self) -> Vec<Move> {
        let mut moves = Vec::new();

        if self.turn == Colour::Black {
            self.rotate();
        }

        for i in 0..64 {
            if let Some(piece) = self.squares[i] {
                if self.turn == piece.colour {
                    match piece.kind {
                        PieceKind::King => self.add_pseudomoves(
                            &mut moves,
                            piece,
                            &self.precomputed_moves.king_moves[i],
                        ),
                        PieceKind::Queen => {
                            self.add_sliding_pseudomoves(
                                &mut moves,
                                piece,
                                &self.precomputed_moves.rook_moves[i],
                            );
                            self.add_sliding_pseudomoves(
                                &mut moves,
                                piece,
                                &self.precomputed_moves.bishop_moves[i],
                            );
                        }
                        PieceKind::Rook => self.add_sliding_pseudomoves(
                            &mut moves,
                            piece,
                            &self.precomputed_moves.rook_moves[i],
                        ),
                        PieceKind::Bishop => self.add_sliding_pseudomoves(
                            &mut moves,
                            piece,
                            &self.precomputed_moves.bishop_moves[i],
                        ),
                        PieceKind::Knight => self.add_pseudomoves(
                            &mut moves,
                            piece,
                            &self.precomputed_moves.knight_moves[i],
                        ),
                        PieceKind::Pawn => self.add_pawn_pseudomoves(&mut moves, i, piece),
                    }
                }
            }
        }

        if self.turn == Colour::Black {
            self.rotate();

            for cmove in &mut moves {
                *cmove = cmove.rotate();
            }
        }

        moves
    }

    fn rotate(&mut self) {
        for i in 0..32 {
            let temp = self.squares[i];
            self.squares[i] = self.squares[63 - i];
            self.squares[63 - i] = temp;
        }
    }

    pub fn make_move(&mut self, cmove: Move) -> MoveData {
        let move_data = MoveData {
            en_passant: self.en_passant,
            castling: self.castling,
        };

        match cmove {
            Move::Standard(piece, RawMove(from, to), _) => {
                self.squares[to] = self.squares[from];
                self.squares[from] = None;

                // En passant
                if piece.kind == PieceKind::Pawn && to == from + 8 {
                    self.en_passant = Some(to);
                }
            }

            Move::Castling(side) => {
                let king_square = if self.turn.into() { 4 } else { 3 };
                // This becomes 0 for black and 7 for white, which is the kingside rook square
                let mut rook_square = king_square % 3 * 7;

                if !side {
                    // If queenside, we swap the rook square from 0 to 7 and vice versa
                    rook_square = (rook_square + 7) % 14;
                }

                // Swap the king and the rook
                let king = self.squares[king_square];
                self.squares[king_square] = self.squares[rook_square];
                self.squares[rook_square] = king;

                if self.turn.into() {
                    self.castling = [false, false, self.castling[2], self.castling[3]];
                } else {
                    self.castling = [self.castling[0], self.castling[1], false, false];
                }
            }

            Move::Promotion(RawMove(from, to), kind) => {
                self.squares[to] = Some(Piece::new(kind, self.turn));
                self.squares[from] = None;
            }

            Move::EnPassant(RawMove(from, to)) => {
                self.squares[to] = self.squares[from];
                self.squares[from] = None;
                self.squares[self.en_passant.unwrap()] = None;
            }
        }

        self.turn = self.turn.invert();
        self.halfmoves += 1;
        if self.turn == Colour::White {
            self.fullmoves += 1;
        }

        move_data
    }

    pub fn unmake_move(&mut self, cmove: Move, move_data: MoveData) {
        match cmove {
            Move::Standard(piece, RawMove(from, to), capture) => {
                self.squares[from] = Some(piece);
                self.squares[to] = capture;
            }

            Move::Castling(side) => {
                // TODO: Create a castling function, because it's exactly the same going back
            }

            Move::Promotion(RawMove(from, to), _) => {
                // TODO: self.turn correct here?
                self.squares[from] = Some(Piece::new(PieceKind::Pawn, self.turn));
            }

            Move::EnPassant(RawMove(from, to)) => {}
        }

        self.en_passant = move_data.en_passant;
        self.castling = move_data.castling;

        self.turn = self.turn.invert();
        self.halfmoves -= 1;
        if self.turn == Colour::Black {
            self.fullmoves -= 1;
        }
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
                    piece.to_algebraic()
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RawMove(pub usize, pub usize);

impl RawMove {
    fn rotate(self) -> Self {
        // NOTE TO SELF, IT'S 63
        RawMove(63 - self.0, 63 - self.1)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Move {
    // Option is a possible capture
    Standard(Piece, RawMove, Option<Piece>),
    // true is kingside, false is queenside
    Castling(bool),
    Promotion(RawMove, PieceKind),
    EnPassant(RawMove),
}

impl Move {
    pub fn insert_capture(self, captured_piece: Piece) -> Self {
        if let Self::Standard(p, r, None) = self {
            return Move::Standard(p, r, Some(captured_piece));
        }

        // TODO: Return an error, assert or panic?
        panic!("Cannot insert capture into non-standard move");
    }

    fn rotate(self) -> Self {
        match self {
            Self::Standard(p, raw, c) => Self::Standard(p, raw.rotate(), c),
            Self::Castling(side) => Self::Castling(side),
            Self::Promotion(raw, k) => Self::Promotion(raw.rotate(), k),
            Self::EnPassant(raw) => Self::EnPassant(raw.rotate()),
        }
    }
}

struct MoveData {
    en_passant: Option<usize>,
}

struct PrecomputedMoves {
    king_moves: [Vec<RawMove>; 64],
    rook_moves: [Vec<Vec<RawMove>>; 64],
    bishop_moves: [Vec<Vec<RawMove>>; 64],
    knight_moves: [Vec<RawMove>; 64],
}

impl PrecomputedMoves {
    fn i8x8_to_12x12(i: usize) -> usize {
        let (y, x) = (i / 8, i % 8);
        (2 + y) * 12 + (2 + x)
    }

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

    fn new() -> Self {
        // 12x12 index offsets, NESW first and then NESESWNW
        let mut directions = [12, 1, -12, -1, 13, -11, -13, 11];
        // Starting NNE, rotating clockwise
        let mut knight_directions = [25, 14, -10, -23, -25, -14, 10, 23];

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
