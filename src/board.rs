//! The board: where every piece stands, whose move it is, and the rest of
//! what a chess position needs (castling rights, the en passant square, the
//! move counters).
//!
//! Stargazer keeps the pieces two ways at once (the "hybrid" design in
//! `docs/DESIGN.md`, section 1):
//! - **Bitboards:** one set of squares per piece type and colour, for
//!   questions about many pieces at once ("where are all of White's pawns?").
//! - **A 64-square array:** what stands on each square, for questions about
//!   one square ("what's on e4?").
//!
//! The two must always agree. Only `put_piece` and `remove_piece` change
//! them, and they always change both.
//!
//! A position is small and cheap to copy, which is how moves will be tried
//! during search ("copy-make", DESIGN.md section 2).

use std::fmt;

use crate::attacks::{bishop_attacks, king_attacks, knight_attacks, pawn_attacks, rook_attacks};
use crate::bitboard::Bitboard;
use crate::movegen::Move;

/// White or Black.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Color {
    White,
    Black,
}

impl Color {
    pub const BOTH: [Color; 2] = [Color::White, Color::Black];

    /// The other side.
    pub const fn opponent(self) -> Color {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }

    const fn index(self) -> usize {
        self as usize
    }
}

/// The six kinds of chess piece.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PieceKind {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

impl PieceKind {
    pub const ALL: [PieceKind; 6] = [
        PieceKind::Pawn,
        PieceKind::Knight,
        PieceKind::Bishop,
        PieceKind::Rook,
        PieceKind::Queen,
        PieceKind::King,
    ];

    const fn index(self) -> usize {
        self as usize
    }
}

/// A piece of one colour, such as a white knight.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Piece {
    pub color: Color,
    pub kind: PieceKind,
}

impl Piece {
    pub const fn new(color: Color, kind: PieceKind) -> Piece {
        Piece { color, kind }
    }

    /// The letter FEN uses for this piece: upper case for White
    /// (`N` is a white knight), lower case for Black (`n`).
    pub fn to_char(self) -> char {
        let letter = match self.kind {
            PieceKind::Pawn => 'p',
            PieceKind::Knight => 'n',
            PieceKind::Bishop => 'b',
            PieceKind::Rook => 'r',
            PieceKind::Queen => 'q',
            PieceKind::King => 'k',
        };
        match self.color {
            Color::White => letter.to_ascii_uppercase(),
            Color::Black => letter,
        }
    }

    /// The piece a FEN letter stands for, if it is one.
    pub fn from_char(letter: char) -> Option<Piece> {
        let kind = match letter.to_ascii_lowercase() {
            'p' => PieceKind::Pawn,
            'n' => PieceKind::Knight,
            'b' => PieceKind::Bishop,
            'r' => PieceKind::Rook,
            'q' => PieceKind::Queen,
            'k' => PieceKind::King,
            _ => return None,
        };
        let color = if letter.is_ascii_uppercase() {
            Color::White
        } else {
            Color::Black
        };
        Some(Piece::new(color, kind))
    }
}

/// One of the 64 squares, numbered a1 = 0, b1 = 1, … h1 = 7, a2 = 8, … h8 = 63.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Square(u8);

impl Square {
    /// The square on this file (0 = a … 7 = h) and rank (0 = rank 1 … 7 = rank 8).
    pub const fn new(file: u8, rank: u8) -> Square {
        Square(rank * 8 + file)
    }

    pub const fn from_index(index: u8) -> Square {
        Square(index)
    }

    pub const fn index(self) -> usize {
        self.0 as usize
    }

    /// 0 for the a-file up to 7 for the h-file.
    pub const fn file(self) -> u8 {
        self.0 % 8
    }

    /// 0 for rank 1 up to 7 for rank 8.
    pub const fn rank(self) -> u8 {
        self.0 / 8
    }

    /// Read a square name such as `e4`.
    pub fn parse(name: &str) -> Option<Square> {
        let mut letters = name.chars();
        let file = letters.next()?;
        let rank = letters.next()?;
        if letters.next().is_some() || !('a'..='h').contains(&file) || !('1'..='8').contains(&rank)
        {
            return None;
        }
        Some(Square::new(file as u8 - b'a', rank as u8 - b'1'))
    }
}

impl fmt::Display for Square {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let file = (b'a' + self.file()) as char;
        let rank = (b'1' + self.rank()) as char;
        write!(f, "{file}{rank}")
    }
}

/// Which castling moves are still possible, as far as king and rook moves
/// go (castling through check is a separate question, asked when moving).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct CastlingRights {
    pub white_kingside: bool,
    pub white_queenside: bool,
    pub black_kingside: bool,
    pub black_queenside: bool,
}

/// The standard starting position, in FEN.
pub const START_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

/// A complete chess position.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Position {
    /// Bitboards: for each colour and piece kind, the squares it stands on.
    pieces: [[Bitboard; 6]; 2],
    /// For each colour, every square one of its pieces stands on.
    occupied: [Bitboard; 2],
    /// The 64-square array: what stands on each square, if anything.
    squares: [Option<Piece>; 64],
    pub side_to_move: Color,
    pub castling: CastlingRights,
    /// The square a pawn just skipped over with a two-square move, which an
    /// enemy pawn could capture onto ("en passant") this move only.
    pub en_passant: Option<Square>,
    /// Half-moves since the last capture or pawn move (for the 50-move rule).
    pub halfmove_clock: u16,
    /// The move number, starting at 1 and going up after each Black move.
    pub fullmove_number: u16,
}

/// Why a FEN string couldn't be read.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum FenError {
    MissingField(&'static str),
    WrongNumberOfRanks(usize),
    BadRank(String),
    UnknownPiece(char),
    PawnOnBackRank,
    KingCount { color: Color, count: u32 },
    BadSideToMove(String),
    BadCastling(String),
    BadEnPassant(String),
    BadCounter(String),
    TooManyFields,
}

impl fmt::Display for FenError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FenError::MissingField(field) => write!(f, "missing the {field}"),
            FenError::WrongNumberOfRanks(n) => write!(f, "expected 8 ranks, found {n}"),
            FenError::BadRank(rank) => write!(f, "rank \"{rank}\" doesn't add up to 8 squares"),
            FenError::UnknownPiece(c) => write!(f, "'{c}' is not a piece letter"),
            FenError::PawnOnBackRank => write!(f, "a pawn can't stand on rank 1 or rank 8"),
            FenError::KingCount { color, count } => {
                write!(f, "{color:?} has {count} kings; it needs exactly one")
            }
            FenError::BadSideToMove(s) => write!(f, "side to move must be w or b, not \"{s}\""),
            FenError::BadCastling(s) => write!(f, "castling rights \"{s}\" aren't valid"),
            FenError::BadEnPassant(s) => write!(f, "en passant square \"{s}\" isn't valid"),
            FenError::BadCounter(s) => write!(f, "move counter \"{s}\" isn't a number"),
            FenError::TooManyFields => write!(f, "too many fields"),
        }
    }
}

impl Position {
    /// An empty board with White to move. Pieces are added with `put_piece`.
    fn empty() -> Position {
        Position {
            pieces: [[Bitboard::EMPTY; 6]; 2],
            occupied: [Bitboard::EMPTY; 2],
            squares: [None; 64],
            side_to_move: Color::White,
            castling: CastlingRights::default(),
            en_passant: None,
            halfmove_clock: 0,
            fullmove_number: 1,
        }
    }

    /// The standard starting position.
    pub fn startpos() -> Position {
        Position::from_fen(START_FEN).expect("the starting FEN is valid")
    }

    /// What stands on this square, if anything.
    pub fn piece_on(&self, square: Square) -> Option<Piece> {
        self.squares[square.index()]
    }

    /// Every square where this piece (say, a white knight) stands.
    pub fn pieces(&self, piece: Piece) -> Bitboard {
        self.pieces[piece.color.index()][piece.kind.index()]
    }

    /// Every square holding one of this side's pieces.
    pub fn occupied_by(&self, color: Color) -> Bitboard {
        self.occupied[color.index()]
    }

    /// Every square holding any piece.
    pub fn occupied(&self) -> Bitboard {
        self.occupied[0] | self.occupied[1]
    }

    /// Place a piece on an empty square, updating both the bitboards and
    /// the 64-square array.
    fn put_piece(&mut self, piece: Piece, square: Square) {
        debug_assert!(self.squares[square.index()].is_none());
        self.pieces[piece.color.index()][piece.kind.index()].insert(square);
        self.occupied[piece.color.index()].insert(square);
        self.squares[square.index()] = Some(piece);
    }

    /// Lift the piece off a square, updating both the bitboards and the
    /// 64-square array.
    fn remove_piece(&mut self, square: Square) -> Option<Piece> {
        let piece = self.squares[square.index()].take()?;
        self.pieces[piece.color.index()][piece.kind.index()].remove(square);
        self.occupied[piece.color.index()].remove(square);
        Some(piece)
    }

    /// Read a position from FEN, chess's standard one-line notation:
    /// the pieces rank by rank from rank 8 down, then whose move it is,
    /// castling rights, the en passant square and the two move counters.
    /// The two counters may be left off; they then default to 0 and 1.
    pub fn from_fen(fen: &str) -> Result<Position, FenError> {
        let mut fields = fen.split_whitespace();
        let mut position = Position::empty();

        // 1. Where the pieces stand, rank 8 first, separated by '/'.
        //    A digit means that many empty squares.
        let placement = fields.next().ok_or(FenError::MissingField("pieces"))?;
        let ranks: Vec<&str> = placement.split('/').collect();
        if ranks.len() != 8 {
            return Err(FenError::WrongNumberOfRanks(ranks.len()));
        }
        for (row, rank_text) in ranks.iter().enumerate() {
            let rank = 7 - row as u8;
            let mut file = 0u8;
            for c in rank_text.chars() {
                if let Some(empty) = c.to_digit(10) {
                    if !(1..=8).contains(&empty) {
                        return Err(FenError::BadRank(rank_text.to_string()));
                    }
                    file += empty as u8;
                } else {
                    let piece = Piece::from_char(c).ok_or(FenError::UnknownPiece(c))?;
                    if file >= 8 {
                        return Err(FenError::BadRank(rank_text.to_string()));
                    }
                    if piece.kind == PieceKind::Pawn && (rank == 0 || rank == 7) {
                        return Err(FenError::PawnOnBackRank);
                    }
                    position.put_piece(piece, Square::new(file, rank));
                    file += 1;
                }
                if file > 8 {
                    return Err(FenError::BadRank(rank_text.to_string()));
                }
            }
            if file != 8 {
                return Err(FenError::BadRank(rank_text.to_string()));
            }
        }
        for color in Color::BOTH {
            let count = position.pieces(Piece::new(color, PieceKind::King)).count();
            if count != 1 {
                return Err(FenError::KingCount { color, count });
            }
        }

        // 2. Whose move it is.
        position.side_to_move = match fields.next() {
            Some("w") => Color::White,
            Some("b") => Color::Black,
            Some(other) => return Err(FenError::BadSideToMove(other.to_string())),
            None => return Err(FenError::MissingField("side to move")),
        };

        // 3. Castling rights: some of KQkq, or '-' for none.
        let castling = fields
            .next()
            .ok_or(FenError::MissingField("castling rights"))?;
        if castling != "-" {
            for c in castling.chars() {
                let right = match c {
                    'K' => &mut position.castling.white_kingside,
                    'Q' => &mut position.castling.white_queenside,
                    'k' => &mut position.castling.black_kingside,
                    'q' => &mut position.castling.black_queenside,
                    _ => return Err(FenError::BadCastling(castling.to_string())),
                };
                if *right {
                    return Err(FenError::BadCastling(castling.to_string()));
                }
                *right = true;
            }
        }

        // 4. The en passant square, or '-'. It's always on rank 3 (behind a
        //    white pawn that just moved two squares) or rank 6 (behind a black one).
        let en_passant = fields
            .next()
            .ok_or(FenError::MissingField("en passant square"))?;
        if en_passant != "-" {
            let square = Square::parse(en_passant)
                .filter(|s| s.rank() == 2 || s.rank() == 5)
                .ok_or_else(|| FenError::BadEnPassant(en_passant.to_string()))?;
            position.en_passant = Some(square);
        }

        // 5 and 6. The move counters, if present.
        if let Some(text) = fields.next() {
            position.halfmove_clock = text
                .parse()
                .map_err(|_| FenError::BadCounter(text.to_string()))?;
        }
        if let Some(text) = fields.next() {
            position.fullmove_number = text
                .parse()
                .map_err(|_| FenError::BadCounter(text.to_string()))?;
        }
        if fields.next().is_some() {
            return Err(FenError::TooManyFields);
        }

        Ok(position)
    }

    /// Write the position as FEN (the reverse of `from_fen`).
    pub fn to_fen(&self) -> String {
        let mut fen = String::new();
        for rank in (0..8).rev() {
            let mut empty = 0;
            for file in 0..8 {
                match self.piece_on(Square::new(file, rank)) {
                    Some(piece) => {
                        if empty > 0 {
                            fen.push_str(&empty.to_string());
                            empty = 0;
                        }
                        fen.push(piece.to_char());
                    }
                    None => empty += 1,
                }
            }
            if empty > 0 {
                fen.push_str(&empty.to_string());
            }
            if rank > 0 {
                fen.push('/');
            }
        }

        fen.push_str(match self.side_to_move {
            Color::White => " w ",
            Color::Black => " b ",
        });

        let c = self.castling;
        let rights: String = [
            (c.white_kingside, 'K'),
            (c.white_queenside, 'Q'),
            (c.black_kingside, 'k'),
            (c.black_queenside, 'q'),
        ]
        .iter()
        .filter(|(allowed, _)| *allowed)
        .map(|(_, letter)| *letter)
        .collect();
        fen.push_str(if rights.is_empty() { "-" } else { &rights });

        match self.en_passant {
            Some(square) => fen.push_str(&format!(" {square}")),
            None => fen.push_str(" -"),
        }
        fen.push_str(&format!(
            " {} {}",
            self.halfmove_clock, self.fullmove_number
        ));
        fen
    }

    /// Check that the bitboards and the 64-square array agree on every
    /// square. Tests call this after every change; a disagreement would be
    /// a bug in `put_piece` or `remove_piece`.
    pub fn is_consistent(&self) -> bool {
        let mut seen = Bitboard::EMPTY;
        for color in Color::BOTH {
            let mut all_of_color = Bitboard::EMPTY;
            for kind in PieceKind::ALL {
                let piece = Piece::new(color, kind);
                let set = self.pieces(piece);
                // No square may hold two pieces.
                if !(set & seen).is_empty() {
                    return false;
                }
                seen |= set;
                all_of_color |= set;
                for square in set.squares() {
                    if self.piece_on(square) != Some(piece) {
                        return false;
                    }
                }
            }
            if all_of_color != self.occupied_by(color) {
                return false;
            }
        }
        // Every piece in the array must also be in the bitboards.
        (0..64)
            .map(Square::from_index)
            .all(|square| self.piece_on(square).is_some() == seen.contains(square))
    }
}

/// Attacks, check, and playing a move.
impl Position {
    /// The square this side's king stands on.
    pub fn king_square(&self, color: Color) -> Square {
        self.pieces(Piece::new(color, PieceKind::King))
            .squares()
            .next()
            .expect("every position has exactly one king per side")
    }

    /// Does any piece of colour `by` attack `square`?
    ///
    /// The trick is to look outward from the square: "if a knight stood
    /// here, which squares could it jump to?" If one of those holds an enemy
    /// knight, that knight attacks this square. The same works for every
    /// piece, because attacks are symmetrical (pawns use the other colour's
    /// pattern, since they attack forward).
    pub fn is_attacked(&self, square: Square, by: Color) -> bool {
        let theirs = |kind| self.pieces(Piece::new(by, kind));
        let blockers = self.occupied();
        let diagonal_sliders = theirs(PieceKind::Bishop) | theirs(PieceKind::Queen);
        let straight_sliders = theirs(PieceKind::Rook) | theirs(PieceKind::Queen);

        !(pawn_attacks(by.opponent(), square) & theirs(PieceKind::Pawn)).is_empty()
            || !(knight_attacks(square) & theirs(PieceKind::Knight)).is_empty()
            || !(king_attacks(square) & theirs(PieceKind::King)).is_empty()
            || !(bishop_attacks(square, blockers) & diagonal_sliders).is_empty()
            || !(rook_attacks(square, blockers) & straight_sliders).is_empty()
    }

    /// Is this side's king in check?
    pub fn in_check(&self, color: Color) -> bool {
        self.is_attacked(self.king_square(color), color.opponent())
    }

    /// Play a move and return the new position, leaving this one untouched
    /// ("copy-make", DESIGN.md section 2). To take the move back, just keep
    /// using the old position.
    ///
    /// The move must come from move generation for this position. It may
    /// leave the mover's own king in check; the caller checks that.
    pub fn make_move(&self, mv: Move) -> Position {
        let mut next = *self;
        let us = self.side_to_move;
        let moving = self
            .piece_on(mv.from)
            .expect("a move starts from a square with a piece on it");
        let captured = next.remove_piece(mv.to);

        next.remove_piece(mv.from);
        let arriving = match mv.promotion {
            Some(kind) => Piece::new(us, kind),
            None => moving,
        };
        next.put_piece(arriving, mv.to);

        // En passant: a pawn moving diagonally onto the en passant square
        // captures the enemy pawn that is beside it, not on the square itself.
        if moving.kind == PieceKind::Pawn && Some(mv.to) == self.en_passant {
            let victim = Square::new(mv.to.file(), mv.from.rank());
            next.remove_piece(victim);
        }

        // Castling: a king moving two squares brings its rook across.
        if moving.kind == PieceKind::King && mv.from.file().abs_diff(mv.to.file()) == 2 {
            let rank = mv.from.rank();
            let (rook_from, rook_to) = if mv.to.file() == 6 {
                (Square::new(7, rank), Square::new(5, rank)) // kingside: h to f
            } else {
                (Square::new(0, rank), Square::new(3, rank)) // queenside: a to d
            };
            let rook = next
                .remove_piece(rook_from)
                .expect("castling rights mean the rook is in place");
            next.put_piece(rook, rook_to);
        }

        // A pawn that moves two squares can be captured en passant on the
        // square it skipped, for one move only.
        next.en_passant = None;
        if moving.kind == PieceKind::Pawn && mv.from.rank().abs_diff(mv.to.rank()) == 2 {
            let skipped_rank = (mv.from.rank() + mv.to.rank()) / 2;
            next.en_passant = Some(Square::new(mv.from.file(), skipped_rank));
        }

        // Castling rights are lost for good when the king moves, or when a
        // rook moves from (or is captured on) its starting corner.
        for corner in [mv.from, mv.to] {
            next.lose_castling_rights_touching(corner);
        }

        // The 50-move rule counter resets on any pawn move or capture.
        if moving.kind == PieceKind::Pawn || captured.is_some() {
            next.halfmove_clock = 0;
        } else {
            next.halfmove_clock += 1;
        }
        if us == Color::Black {
            next.fullmove_number += 1;
        }
        next.side_to_move = us.opponent();
        next
    }

    /// Something moved from or to this square: clear any castling right
    /// that depends on a king or rook starting there.
    fn lose_castling_rights_touching(&mut self, square: Square) {
        const A1: Square = Square::new(0, 0);
        const E1: Square = Square::new(4, 0);
        const H1: Square = Square::new(7, 0);
        const A8: Square = Square::new(0, 7);
        const E8: Square = Square::new(4, 7);
        const H8: Square = Square::new(7, 7);
        let rights = &mut self.castling;
        match square {
            E1 => {
                rights.white_kingside = false;
                rights.white_queenside = false;
            }
            E8 => {
                rights.black_kingside = false;
                rights.black_queenside = false;
            }
            H1 => rights.white_kingside = false,
            A1 => rights.white_queenside = false,
            H8 => rights.black_kingside = false,
            A8 => rights.black_queenside = false,
            _ => {}
        }
    }
}

/// Prints the board as a diagram, White at the bottom, then the FEN.
/// This is what the UCI `d` command shows.
impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for rank in (0..8).rev() {
            write!(f, "{} ", rank + 1)?;
            for file in 0..8 {
                let mark = self
                    .piece_on(Square::new(file, rank))
                    .map_or('.', Piece::to_char);
                write!(f, " {mark}")?;
            }
            writeln!(f)?;
        }
        writeln!(f, "   a b c d e f g h")?;
        writeln!(f)?;
        write!(f, "FEN: {}", self.to_fen())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// "Kiwipete", a famous test position full of castling, pins and
    /// en passant chances.
    const KIWIPETE: &str = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";

    fn sq(name: &str) -> Square {
        Square::parse(name).unwrap()
    }

    #[test]
    fn square_names() {
        assert_eq!(sq("a1").index(), 0);
        assert_eq!(sq("h1").index(), 7);
        assert_eq!(sq("a2").index(), 8);
        assert_eq!(sq("h8").index(), 63);
        assert_eq!(sq("e4").to_string(), "e4");
        assert_eq!(Square::parse("i1"), None);
        assert_eq!(Square::parse("a9"), None);
        assert_eq!(Square::parse("e44"), None);
    }

    #[test]
    fn start_position_has_the_pieces_where_they_belong() {
        let p = Position::startpos();
        let white = |kind| Some(Piece::new(Color::White, kind));
        let black = |kind| Some(Piece::new(Color::Black, kind));
        assert_eq!(p.piece_on(sq("e1")), white(PieceKind::King));
        assert_eq!(p.piece_on(sq("d1")), white(PieceKind::Queen));
        assert_eq!(p.piece_on(sq("g1")), white(PieceKind::Knight));
        assert_eq!(p.piece_on(sq("e8")), black(PieceKind::King));
        assert_eq!(p.piece_on(sq("a7")), black(PieceKind::Pawn));
        assert_eq!(p.piece_on(sq("e4")), None);

        assert_eq!(p.occupied().count(), 32);
        assert_eq!(
            p.pieces(Piece::new(Color::White, PieceKind::Pawn)).count(),
            8
        );
        assert_eq!(p.side_to_move, Color::White);
        assert!(p.castling.white_kingside && p.castling.black_queenside);
        assert_eq!(p.en_passant, None);
        assert!(p.is_consistent());
    }

    #[test]
    fn fen_round_trips() {
        for fen in [
            START_FEN,
            KIWIPETE,
            // Black to move, with an en passant square and no castling.
            "rnbqkbnr/ppp1pppp/8/8/3pP3/8/PPPP1PPP/RNBQKBNR b - e3 0 3",
            // A bare endgame, deep into a game.
            "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 12 57",
        ] {
            let position = Position::from_fen(fen).unwrap();
            assert_eq!(position.to_fen(), fen);
            assert!(position.is_consistent());
        }
    }

    #[test]
    fn move_counters_are_optional() {
        let p = Position::from_fen("4k3/8/8/8/8/8/8/4K3 w - -").unwrap();
        assert_eq!(p.halfmove_clock, 0);
        assert_eq!(p.fullmove_number, 1);
    }

    #[test]
    fn removing_a_piece_keeps_both_views_in_step() {
        let mut p = Position::from_fen(KIWIPETE).unwrap();
        let knight = p.remove_piece(sq("e5"));
        assert_eq!(knight, Some(Piece::new(Color::White, PieceKind::Knight)));
        assert_eq!(p.piece_on(sq("e5")), None);
        assert!(!p.occupied().contains(sq("e5")));
        assert!(p.is_consistent());

        p.put_piece(knight.unwrap(), sq("d4"));
        assert!(p.pieces(knight.unwrap()).contains(sq("d4")));
        assert!(p.is_consistent());
    }

    #[test]
    fn a_disagreement_between_the_views_is_caught() {
        let mut p = Position::startpos();
        // Break the array behind the bitboards' back.
        p.squares[sq("e4").index()] = Some(Piece::new(Color::White, PieceKind::Queen));
        assert!(!p.is_consistent());
    }

    #[test]
    fn bad_fens_are_rejected_with_a_reason() {
        let cases = [
            ("", FenError::MissingField("pieces")),
            (
                "rnbqkbnr/pppppppp/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
                FenError::WrongNumberOfRanks(7),
            ),
            (
                "rnbqkbnr/ppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
                FenError::BadRank("ppppppp".into()),
            ),
            (
                "rnbqkbnr/ppppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
                FenError::BadRank("ppppppppp".into()),
            ),
            (
                "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNX w KQkq - 0 1",
                FenError::UnknownPiece('X'),
            ),
            (
                "rnbqkbnP/pppppppp/8/8/8/8/PPPPPPP1/RNBQKBNR w KQkq - 0 1",
                FenError::PawnOnBackRank,
            ),
            (
                "rnbqqbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
                FenError::KingCount {
                    color: Color::Black,
                    count: 0,
                },
            ),
            (
                "4k3/8/8/8/8/8/8/4K3 x - - 0 1",
                FenError::BadSideToMove("x".into()),
            ),
            (
                "4k3/8/8/8/8/8/8/4K3 w KK - 0 1",
                FenError::BadCastling("KK".into()),
            ),
            (
                "4k3/8/8/8/8/8/8/4K3 w - e4 0 1",
                FenError::BadEnPassant("e4".into()),
            ),
            (
                "4k3/8/8/8/8/8/8/4K3 w - - x 1",
                FenError::BadCounter("x".into()),
            ),
            (
                "4k3/8/8/8/8/8/8/4K3 w - - 0 1 extra",
                FenError::TooManyFields,
            ),
            (
                "4k3/8/8/8/8/8/8/4K3 w",
                FenError::MissingField("castling rights"),
            ),
        ];
        for (fen, expected) in cases {
            assert_eq!(Position::from_fen(fen), Err(expected), "FEN: {fen}");
        }
    }

    /// Play a move given in UCI form (no promotion) and return the new FEN.
    fn after(fen: &str, from: &str, to: &str) -> String {
        let position = Position::from_fen(fen).unwrap();
        let next = position.make_move(Move::new(sq(from), sq(to)));
        assert!(next.is_consistent());
        next.to_fen()
    }

    #[test]
    fn making_a_move_leaves_the_old_position_untouched() {
        let start = Position::startpos();
        let next = start.make_move(Move::new(sq("e2"), sq("e4")));
        assert_eq!(start.to_fen(), START_FEN);
        assert_eq!(
            next.to_fen(),
            "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1"
        );
    }

    #[test]
    fn castling_brings_the_rook_across_and_ends_the_rights() {
        let fen = "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 3 10";
        assert_eq!(
            after(fen, "e1", "g1"),
            "r3k2r/8/8/8/8/8/8/R4RK1 b kq - 4 10"
        );
        assert_eq!(
            after(fen, "e1", "c1"),
            "r3k2r/8/8/8/8/8/8/2KR3R b kq - 4 10"
        );
    }

    #[test]
    fn moving_or_losing_a_rook_ends_that_side_only() {
        let fen = "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1";
        // The h1 rook moves: White keeps queenside only.
        assert_eq!(
            after(fen, "h1", "h2"),
            "r3k2r/8/8/8/8/8/7R/R3K3 b Qkq - 1 1"
        );
        // The a1 rook captures on a8: White loses queenside, Black too.
        assert_eq!(after(fen, "a1", "a8"), "R3k2r/8/8/8/8/8/8/4K2R b Kk - 0 1");
    }

    #[test]
    fn en_passant_removes_the_pawn_beside_not_ahead() {
        let fen = "4k3/8/8/3pP3/8/8/8/4K3 w - d6 0 1";
        assert_eq!(after(fen, "e5", "d6"), "4k3/8/3P4/8/8/8/8/4K3 b - - 0 1");
    }

    #[test]
    fn promotion_replaces_the_pawn() {
        let position = Position::from_fen("4k3/1P6/8/8/8/8/8/4K3 w - - 0 1").unwrap();
        let mv = Move {
            from: sq("b7"),
            to: sq("b8"),
            promotion: Some(PieceKind::Knight),
        };
        assert_eq!(
            position.make_move(mv).to_fen(),
            "1N2k3/8/8/8/8/8/8/4K3 b - - 0 1"
        );
    }

    #[test]
    fn attacks_and_check() {
        let p = Position::from_fen("4k3/8/8/8/1b6/8/8/4K3 w - - 0 1").unwrap();
        // The bishop on b4 gives check along b4-c3-d2-e1.
        assert!(p.in_check(Color::White));
        assert!(!p.in_check(Color::Black));
        assert!(p.is_attacked(sq("a5"), Color::Black));
        assert!(!p.is_attacked(sq("b5"), Color::Black));
        // Pawns attack diagonally forward, not straight ahead.
        let p = Position::from_fen("4k3/8/8/8/8/8/4P3/4K3 w - - 0 1").unwrap();
        assert!(p.is_attacked(sq("d3"), Color::White));
        assert!(!p.is_attacked(sq("e3"), Color::White));
    }

    #[test]
    fn diagram_shows_rank_8_on_top_and_the_fen_below() {
        let text = Position::startpos().to_string();
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines[0], "8  r n b q k b n r");
        assert_eq!(lines[7], "1  R N B Q K B N R");
        assert_eq!(lines[8], "   a b c d e f g h");
        assert_eq!(lines[10], format!("FEN: {START_FEN}"));
    }
}
