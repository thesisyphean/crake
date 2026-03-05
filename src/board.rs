use crate::{
    Board,
    moves::{Move, MoveData, PrecomputedMoves, RawMove},
    piece::{Colour, Piece, PieceKind},
};
use std::fmt::{self, Display, Formatter};

/// A mailbox implementation of a chess board, with associated information such as move clocks
pub struct MailboxBoard {
    /// Index starts at square a1, with the rest of the rank following, then the rank above and so on
    pub squares: [Option<Piece>; 64],
    pub turn: Colour,
    /// Castling rights are white kingside, white queenside, black kingside and black queenside
    castling: [bool; 4],
    /// If a pawn moves two squares, this is the square now inhabited by it
    en_passant: Option<usize>,
    halfmoves: u32,
    fullmoves: u32,
    precomputed_moves: PrecomputedMoves,
}

impl MailboxBoard {
    pub fn new() -> Self {
        Self::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
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
        if i / 8 < 6 {
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

    /// Generates a list of possible pseudomoves for the side to play.
    /// Many of these moves may be illegal due to check.
    fn generate_pseudomoves(&mut self) -> Vec<Move> {
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

    /// Rotates the MailboxBoard from white's perspective to black's perspective, and vice versa.
    fn rotate(&mut self) {
        for i in 0..32 {
            let temp = self.squares[i];
            self.squares[i] = self.squares[63 - i];
            self.squares[63 - i] = temp;
        }

        self.en_passant.as_mut().map(|s| *s = 63 - *s);
    }

    /// Updates the squares of the MailboxBoard such that the side to play has castled.
    /// true is kingside, and false is queenside.
    fn castle(&mut self, side: bool) {
        if self.turn == Colour::Black {
            self.rotate();
        }

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

        if self.turn == Colour::Black {
            self.rotate();
        }
    }

    /// Gives the standard piece values in centipawns
    fn piece_value(piece: Piece) -> i32 {
        match piece.kind {
            PieceKind::King => 100_000,
            PieceKind::Queen => 900,
            PieceKind::Rook => 500,
            PieceKind::Bishop => 300,
            PieceKind::Knight => 300,
            PieceKind::Pawn => 100,
        }
    }

    /// If the RawMove corresponds to a legal move, this returns it
    pub fn valid_move(&mut self, rmove @ RawMove(from, to): RawMove) -> Option<Move> {
        let moves = self.generate_moves();
        let square = self.squares[from];

        // Castling
        if let Some(Piece {
            kind: PieceKind::King,
            colour: _,
        }) = square
        {
            let diff = from.abs_diff(to);
            if diff == 2 || diff == 3 {
                return Some(Move::Castling(diff == 2)).filter(|c| moves.contains(c));
            }
        }

        if let Some(Piece {
            kind: PieceKind::Pawn,
            colour: _,
        }) = square
        {
            // Promotion
            if to / 8 == 7 || to / 8 == 0 {
                // If you can promote to the queen, you can promote to any other piece,
                // so the queen is simply used as a dummy option
                return Some(Move::Promotion(rmove, PieceKind::Queen))
                    .filter(|c| moves.contains(c));
            }

            if moves.contains(&Move::EnPassant(rmove)) {
                return Some(Move::EnPassant(rmove));
            }
        }

        // Standard move
        if let Some(piece) = self.squares[from] {
            return Some(Move::Standard(piece, rmove, self.squares[to]))
                .filter(|c| moves.contains(c));
        }

        None
    }
}

impl Board for MailboxBoard {
    /// Creates a new MailboxBoard using the given FEN string, assuming it is valid
    fn from_fen(fen: &str) -> Self {
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

        MailboxBoard {
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

    /// Filters the illegal pseudomoves by making them and checking for king captures
    fn generate_moves(&mut self) -> Vec<Move> {
        let mut pseudomoves = self.generate_pseudomoves();
        let mut legal_moves = Vec::new();

        'outer: for _ in 0..pseudomoves.len() {
            let cmove = pseudomoves.pop().unwrap();
            let move_data = self.make_move(cmove);

            let replies = self.generate_pseudomoves();
            for reply in replies {
                if let Move::Standard(_, _, Some(piece)) = reply {
                    if piece.kind == PieceKind::King {
                        self.unmake_move(cmove, move_data);
                        continue 'outer;
                    }
                }
            }

            self.unmake_move(cmove, move_data);
            legal_moves.push(cmove);
        }

        legal_moves
    }

    /// Updates the MailboxBoard to the state following the move, assuming it is legal.
    ///
    /// Certain information is not recoverable from the move alone:
    /// - Castling rights
    /// - A pawn vulnerable to en passant
    ///
    /// This information is returned in a MoveData struct,
    /// in order for the move to be unmade if necessary.
    fn make_move(&mut self, cmove: Move) -> MoveData {
        let move_data = MoveData {
            en_passant: self.en_passant,
            castling: self.castling,
        };

        match cmove {
            Move::Standard(piece, RawMove(from, to), _) => {
                self.squares[to] = self.squares[from];
                self.squares[from] = None;

                // Save en passant square
                if piece.kind == PieceKind::Pawn && from.abs_diff(to) == 8 {
                    self.en_passant = Some(to);
                }
            }

            Move::Castling(side) => {
                self.castle(side);

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

    /// Returns the MailboxBoard to the state preceding the move, assuming it is legal.
    ///
    /// The additional information that cannot be known, given only the move, is:
    /// - Castling rights
    /// - A pawn vulnerable to en passant
    ///
    /// This is provided by the MoveData struct.
    fn unmake_move(&mut self, cmove: Move, move_data: MoveData) {
        // Invert the turn so the castling method castles the correct side
        self.turn = self.turn.invert();

        match cmove {
            Move::Standard(piece, RawMove(from, to), capture) => {
                self.squares[from] = Some(piece);
                self.squares[to] = capture;
            }

            Move::Castling(side) => {
                self.castle(side);
            }

            Move::Promotion(RawMove(from, to), _) => {
                self.squares[from] = Some(Piece::new(PieceKind::Pawn, self.turn));
                self.squares[to] = None;
            }

            Move::EnPassant(RawMove(from, to)) => {
                self.squares[from] = self.squares[to];
                self.squares[to] = None;
                self.squares[move_data.en_passant.unwrap()] =
                    Some(Piece::new(PieceKind::Pawn, self.turn.invert()));
            }
        }

        self.en_passant = move_data.en_passant;
        self.castling = move_data.castling;

        self.halfmoves -= 1;
        if self.turn == Colour::Black {
            self.fullmoves -= 1;
        }
    }

    /// Calculates an evaluation for the current board relative to the side to play
    // TODO: Add value maps for piece positions
    fn value(&self) -> i32 {
        let mut value = 0;

        for i in 0..64 {
            if let Some(piece) = self.squares[i] {
                value += Self::piece_value(piece)
                    * if let Colour::White = piece.colour {
                        1
                    } else {
                        -1
                    };
            }
        }

        if self.turn == Colour::Black {
            value *= -1;
        }

        value
    }
}

impl Display for MailboxBoard {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        writeln!(f, "┌───┬───┬───┬───┬───┬───┬───┬───┐")?;

        for r in 0..8 {
            for c in 0..8 {
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

        write!(f, "└───┴───┴───┴───┴───┴───┴───┴───┘")
    }
}
