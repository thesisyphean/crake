#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Colour {
    White,
    Black,
}

impl Colour {
    pub fn invert(self) -> Self {
        if self == Colour::White {
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
            _ => panic!("Invalid algebraic notation for piece kind '{p}'"),
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
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Piece {
    pub kind: PieceKind,
    pub colour: Colour,
}

impl Piece {
    pub fn new(kind: PieceKind, colour: Colour) -> Self {
        Piece { kind, colour }
    }

    pub fn from_algebraic(p: char) -> Self {
        Piece {
            kind: PieceKind::from_algebraic(p),
            colour: Colour::from(p.is_ascii_uppercase()),
        }
    }

    pub fn to_algebraic(self) -> char {
        let mut chr = self.kind.to_algebraic();

        if self.colour == Colour::Black {
            chr.make_ascii_lowercase();
        }

        chr
    }
}
