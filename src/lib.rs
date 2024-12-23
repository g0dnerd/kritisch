#![feature(test)]

pub mod bitboard;
pub mod game;
pub mod magics;
pub mod movegen;

const PIECE_REPR_W: [char; 6] = ['P', 'N', 'B', 'R', 'Q', 'K'];
const PIECE_REPR_B: [char; 6] = ['p', 'n', 'b', 'r', 'q', 'k'];

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Color {
    WHITE = 0,
    BLACK = 1,
}
impl Color {
    pub fn from_u8(v: u8) -> Self {
        match v {
            0 => Self::WHITE,
            1 => Self::BLACK,
            _ => panic!(),
        }
    }
}
impl std::ops::BitXor<u8> for Color {
    type Output = Self;

    fn bitxor(self, rhs: u8) -> Self::Output {
        Color::from_u8(self as u8 ^ rhs)
    }
}

pub struct MagicTableEntry {
    pub mask: u64,
    pub magic: u64,
    pub shift: u8,
    pub offset: u32,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Piece {
    PAWN = 0,
    KNIGHT = 1,
    BISHOP = 2,
    ROOK = 3,
    QUEEN = 4,
    KING = 5,
}
impl Piece {
    pub fn from_char(c: &char) -> Self {
        match c.to_ascii_lowercase() {
            'p' => Self::PAWN,
            'n' => Self::KNIGHT,
            'b' => Self::BISHOP,
            'r' => Self::ROOK,
            'q' => Self::QUEEN,
            'k' => Self::KING,
            _ => panic!(),
        }
    }
    pub fn from_u8(i: u8) -> Self {
        match i {
            0 => Self::PAWN,
            1 => Self::KNIGHT,
            2 => Self::BISHOP,
            3 => Self::ROOK,
            4 => Self::QUEEN,
            5 => Self::KING,
            _ => panic!(),
        }
    }
}

#[derive(Debug)]
pub struct CastlingRights;
impl CastlingRights {
    pub const NO_LEGAL: u8 = 0;
    pub const WHITE_KINGSIDE: u8 = 1;
    pub const WHITE_QUEENSIDE: u8 = 2;
    pub const BLACK_QUEENSIDE: u8 = 4;
    pub const BLACK_KINGSIDE: u8 = 8;

    pub const BOTH_KINGSIDES: u8 = Self::WHITE_KINGSIDE | Self::BLACK_KINGSIDE;
    pub const BOTH_QUEENSIDES: u8 = Self::WHITE_QUEENSIDE | Self::BLACK_QUEENSIDE;
    pub const WHITE_CASTLING: u8 = Self::WHITE_KINGSIDE | Self::WHITE_QUEENSIDE;
    pub const BLACK_CASTLING: u8 = Self::BLACK_KINGSIDE | Self::BLACK_QUEENSIDE;
    pub const ALL_LEGAL: u8 = Self::WHITE_CASTLING | Self::BLACK_CASTLING;
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Move {
    pub start: Square,
    pub end: Square,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Rank {
    FIRST = 0,
    SECOND = 1,
    THIRD = 2,
    FOURTH = 3,
    FIFTH = 4,
    SIXTH = 5,
    SEVENTH = 6,
    EIGHTH = 7,
}
impl Rank {
    pub fn from_u8(r: u8) -> Self {
        match r {
            0 => Self::FIRST,
            1 => Self::SECOND,
            2 => Self::THIRD,
            3 => Self::FOURTH,
            4 => Self::FIFTH,
            5 => Self::SIXTH,
            6 => Self::SEVENTH,
            7 => Self::EIGHTH,
            _ => panic!(),
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum File {
    A = 0,
    B = 1,
    C = 2,
    D = 3,
    E = 4,
    F = 5,
    G = 6,
    H = 7,
}
impl File {
    pub fn from_u8(f: u8) -> Self {
        match f {
            0 => Self::A,
            1 => Self::B,
            2 => Self::C,
            3 => Self::D,
            4 => Self::E,
            5 => Self::F,
            6 => Self::G,
            7 => Self::H,
            _ => panic!(),
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Square {
    A1 = 0,
    B1 = 1,
    C1 = 2,
    D1 = 3,
    E1 = 4,
    F1 = 5,
    G1 = 6,
    H1 = 7,
    A2 = 8,
    B2 = 9,
    C2 = 10,
    D2 = 11,
    E2 = 12,
    F2 = 13,
    G2 = 14,
    H2 = 15,
    A3 = 16,
    B3 = 17,
    C3 = 18,
    D3 = 19,
    E3 = 20,
    F3 = 21,
    G3 = 22,
    H3 = 23,
    A4 = 24,
    B4 = 25,
    C4 = 26,
    D4 = 27,
    E4 = 28,
    F4 = 29,
    G4 = 30,
    H4 = 31,
    A5 = 32,
    B5 = 33,
    C5 = 34,
    D5 = 35,
    E5 = 36,
    F5 = 37,
    G5 = 38,
    H5 = 39,
    A6 = 40,
    B6 = 41,
    C6 = 42,
    D6 = 43,
    E6 = 44,
    F6 = 45,
    G6 = 46,
    H6 = 47,
    A7 = 48,
    B7 = 49,
    C7 = 50,
    D7 = 51,
    E7 = 52,
    F7 = 53,
    G7 = 54,
    H7 = 55,
    A8 = 56,
    B8 = 57,
    C8 = 58,
    D8 = 59,
    E8 = 60,
    F8 = 61,
    G8 = 62,
    H8 = 63,
}
impl Square {
    pub fn from_parts(c: &char, d: &char) -> anyhow::Result<Self> {
        let file = match c {
            'a' => 0,
            'b' => 1,
            'c' => 2,
            'd' => 3,
            'e' => 4,
            'f' => 5,
            'g' => 6,
            'h' => 7,
            _ => anyhow::bail!("File for square out of bounds while parsing square from parts"),
        };

        let rank = match d {
            '1' => 0,
            '2' => 1,
            '3' => 2,
            '4' => 3,
            '5' => 4,
            '6' => 5,
            '7' => 6,
            '8' => 7,
            _ => anyhow::bail!("Rank for square out of bounds while parsing square from parts"),
        };

        Ok(Self::from_u8(file + rank * 8))
    }
    pub fn from_u8(v: u8) -> Self {
        match v {
            0 => Self::A1,
            1 => Self::B1,
            2 => Self::C1,
            3 => Self::D1,
            4 => Self::E1,
            5 => Self::F1,
            6 => Self::G1,
            7 => Self::H1,
            8 => Self::A2,
            9 => Self::B2,
            10 => Self::C2,
            11 => Self::D2,
            12 => Self::E2,
            13 => Self::F2,
            14 => Self::G2,
            15 => Self::H2,
            16 => Self::A3,
            17 => Self::B3,
            18 => Self::C3,
            19 => Self::D3,
            20 => Self::E3,
            21 => Self::F3,
            22 => Self::G3,
            23 => Self::H3,
            24 => Self::A4,
            25 => Self::B4,
            26 => Self::C4,
            27 => Self::D4,
            28 => Self::E4,
            29 => Self::F4,
            30 => Self::G4,
            31 => Self::H4,
            32 => Self::A5,
            33 => Self::B5,
            34 => Self::C5,
            35 => Self::D5,
            36 => Self::E5,
            37 => Self::F5,
            38 => Self::G5,
            39 => Self::H5,
            40 => Self::A6,
            41 => Self::B6,
            42 => Self::C6,
            43 => Self::D6,
            44 => Self::E6,
            45 => Self::F6,
            46 => Self::G6,
            47 => Self::H6,
            48 => Self::A7,
            49 => Self::B7,
            50 => Self::C7,
            51 => Self::D7,
            52 => Self::E7,
            53 => Self::F7,
            54 => Self::G7,
            55 => Self::H7,
            56 => Self::A8,
            57 => Self::B8,
            58 => Self::C8,
            59 => Self::D8,
            60 => Self::E8,
            61 => Self::F8,
            62 => Self::G8,
            63 => Self::H8,
            _ => panic!("Unable to parse {v} to square"),
        }
    }

    pub fn to_u64(self) -> u64 {
        1 << self as u8
    }

    pub fn get_rank(self) -> Rank {
        Rank::from_u8(self as u8 / 8)
    }

    pub fn get_file(self) -> File {
        File::from_u8(self as u8 % 8)
    }
}
impl std::ops::Add<u8> for Square {
    type Output = Self;

    fn add(self, rhs: u8) -> Self::Output {
        Self::from_u8(self as u8 + rhs)
    }
}
impl std::ops::Add<i8> for Square {
    type Output = Self;

    fn add(self, rhs: i8) -> Self::Output {
        Self::from_u8((self as i8 + rhs) as u8)
    }
}
impl std::ops::Sub<u8> for Square {
    type Output = Self;

    fn sub(self, rhs: u8) -> Self::Output {
        Self::from_u8(self as u8 - rhs)
    }
}
impl std::ops::Sub<i8> for Square {
    type Output = Self;

    fn sub(self, rhs: i8) -> Self::Output {
        Self::from_u8((self as i8 - rhs) as u8)
    }
}
impl std::fmt::Display for Square {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Square::A1 => write!(f, "a1"),
            Square::B1 => write!(f, "b1"),
            Square::C1 => write!(f, "c1"),
            Square::D1 => write!(f, "d1"),
            Square::E1 => write!(f, "e1"),
            Square::F1 => write!(f, "f1"),
            Square::G1 => write!(f, "g1"),
            Square::H1 => write!(f, "h1"),
            Square::A2 => write!(f, "a2"),
            Square::B2 => write!(f, "b2"),
            Square::C2 => write!(f, "c2"),
            Square::D2 => write!(f, "d2"),
            Square::E2 => write!(f, "e2"),
            Square::F2 => write!(f, "f2"),
            Square::G2 => write!(f, "g2"),
            Square::H2 => write!(f, "h2"),
            Square::A3 => write!(f, "a3"),
            Square::B3 => write!(f, "b3"),
            Square::C3 => write!(f, "c3"),
            Square::D3 => write!(f, "d3"),
            Square::E3 => write!(f, "e3"),
            Square::F3 => write!(f, "f3"),
            Square::G3 => write!(f, "g3"),
            Square::H3 => write!(f, "h3"),
            Square::A4 => write!(f, "a4"),
            Square::B4 => write!(f, "b4"),
            Square::C4 => write!(f, "c4"),
            Square::D4 => write!(f, "d4"),
            Square::E4 => write!(f, "e4"),
            Square::F4 => write!(f, "f4"),
            Square::G4 => write!(f, "g4"),
            Square::H4 => write!(f, "h4"),
            Square::A5 => write!(f, "a5"),
            Square::B5 => write!(f, "b5"),
            Square::C5 => write!(f, "c5"),
            Square::D5 => write!(f, "d5"),
            Square::E5 => write!(f, "e5"),
            Square::F5 => write!(f, "f5"),
            Square::G5 => write!(f, "g5"),
            Square::H5 => write!(f, "h5"),
            Square::A6 => write!(f, "a6"),
            Square::B6 => write!(f, "b6"),
            Square::C6 => write!(f, "c6"),
            Square::D6 => write!(f, "d6"),
            Square::E6 => write!(f, "e6"),
            Square::F6 => write!(f, "f6"),
            Square::G6 => write!(f, "g6"),
            Square::H6 => write!(f, "h6"),
            Square::A7 => write!(f, "a7"),
            Square::B7 => write!(f, "b7"),
            Square::C7 => write!(f, "c7"),
            Square::D7 => write!(f, "d7"),
            Square::E7 => write!(f, "e7"),
            Square::F7 => write!(f, "f7"),
            Square::G7 => write!(f, "g7"),
            Square::H7 => write!(f, "h7"),
            Square::A8 => write!(f, "a8"),
            Square::B8 => write!(f, "b8"),
            Square::C8 => write!(f, "c8"),
            Square::D8 => write!(f, "d8"),
            Square::E8 => write!(f, "e8"),
            Square::F8 => write!(f, "f8"),
            Square::G8 => write!(f, "g8"),
            Square::H8 => write!(f, "h8"),
        }
    }
}

/// Checks if `square` offset by `dx` and `dy` is within bounds.
/// Returns that new square if yes.
///
/// # Example
///
/// ```
/// use kritisch::{try_square_offset, Square};
///
/// let square = Square::H7;
/// assert!(try_square_offset(square, 1, 0).is_none());
/// assert_eq!(try_square_offset(square, 0, 1).unwrap(), Square::H8);
/// ```
pub fn try_square_offset(square: Square, dx: i8, dy: i8) -> Option<Square> {
    let square_idx = square as i8;
    let file = square_idx % 8;
    let rank = square_idx / 8;
    let new_file = file + dx;
    let new_rank = rank + dy;

    if new_file >= 0 && new_file < 8 && new_rank >= 0 && new_rank < 8 {
        Some(Square::from_u8((new_rank * 8 + new_file) as u8))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    mod bitboards {
        use crate::{bitboard::Bitboard, Square};

        #[test]
        fn bb_from_sq() {
            let sq = vec![Square::A4, Square::G3, Square::D7];
            let bb = Bitboard::from_squares(sq);
            assert_eq!(bb.0, 2251799834656768);
        }

        #[test]
        fn bb_and_bb() {
            // a2 and b2 set
            let lhs = Bitboard::from_u64(768);
            // c2 and d2 set
            let rhs = Bitboard::from_u64(3072);
            let res = Bitboard::empty();

            assert_eq!(lhs & rhs, res);
        }

        #[test]
        fn bb_and_u64() {
            // a2 and b2 set
            let lhs = Bitboard::from_u64(768);
            // c2 and d2 set
            let rhs = 3072;
            let res = Bitboard::empty();

            assert_eq!(lhs & rhs, res);
        }

        #[test]
        fn bb_or_bb() {
            // a2 and b2 set
            let lhs = Bitboard::from_u64(768);
            // c2 and d2 set
            let rhs = Bitboard::from_u64(3072);
            let res = Bitboard::from_u64(3840);

            assert_eq!(lhs | rhs, res);
        }

        #[test]
        fn bb_or_u64() {
            // a2 and b2 set
            let lhs = Bitboard::from_u64(768);
            // c2 and d2 set
            let rhs = 3072;
            let res = Bitboard::from_u64(3840);

            assert_eq!(lhs | rhs, res);
        }

        #[test]
        fn bb_xor_bb() {
            // a2, b2 and c2 set
            let lhs = Bitboard::from_u64(1792);
            // c2 and d2 set
            let rhs = Bitboard::from_u64(3072);
            let res = Bitboard::from_u64(2816);

            assert_eq!(lhs ^ rhs, res);
        }

        #[test]
        fn bb_xor_u64() {
            // a2, b2 and c2 set
            let lhs = Bitboard::from_u64(1792);
            // c2 and d2 set
            let rhs = 3072;
            let res = Bitboard::from_u64(2816);

            assert_eq!(lhs ^ rhs, res);
        }

        #[test]
        fn bb_and_assign_bb() {
            // a2 and b2 set
            let mut lhs = Bitboard::from_u64(768);
            // c2 and d2 set
            let rhs = Bitboard::from_u64(3072);
            lhs &= rhs;
            assert_eq!(lhs.0, 0);
        }

        #[test]
        fn bb_and_assign_u64() {
            // a2 and b2 set
            let mut lhs = Bitboard::from_u64(768);
            // c2 and d2 set
            let rhs = 3072;
            lhs &= rhs;
            assert_eq!(lhs.0, 0);
        }

        #[test]
        fn bb_or_assign_bb() {
            // a2 and b2 set
            let mut lhs = Bitboard::from_u64(768);
            // c2 and d2 set
            let rhs = Bitboard::from_u64(3072);
            lhs |= rhs;
            let res = 3840;

            assert_eq!(lhs.0, res);
        }

        #[test]
        fn bb_or_assign_u64() {
            // a2 and b2 set
            let mut lhs = Bitboard::from_u64(768);
            // c2 and d2 set
            let rhs = 3072;
            lhs |= rhs;
            let res = 3840;

            assert_eq!(lhs.0, res);
        }

        #[test]
        fn bb_xor_assign_bb() {
            // a2, b2 and c2 set
            let mut lhs = Bitboard::from_u64(1792);
            // c2 and d2 set
            let rhs = Bitboard::from_u64(3072);
            lhs ^= rhs;
            let res = 2816;

            assert_eq!(lhs.0, res);
        }

        #[test]
        fn bb_xor_assign_u64() {
            // a2, b2 and c2 set
            let mut lhs = Bitboard::from_u64(1792);
            // c2 and d2 set
            let rhs = 3072;
            lhs ^= rhs;
            let res = 2816;

            assert_eq!(lhs.0, res);
        }
    }

    mod game {
        use crate::{bitboard::Bitboard, game::Game, Color, Move, Piece, Square};

        #[test]
        fn game_from_fen() {
            let from_fen =
                Game::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();

            let default_game = Game::default();
            assert_eq!(from_fen, default_game);
        }

        #[test]
        fn game_display() {
            let game = Game::default();
            let str = game.to_string();
            let expected = String::from(
                "\nr n b q k b n r \np p p p p p p p \n. . . . . . . . \n. . . . . . . . \n. . . . . . . . \n. . . . . . . . \nP P P P P P P P \nR N B Q K B N R \n");
            assert_eq!(str, expected);
        }

        #[test]
        fn piece_type() {
            let game = Game::default();
            let black_king = game.type_at(Square::E8);
            assert_eq!(black_king, Piece::KING);
        }

        #[test]
        #[should_panic]
        fn piece_type_empty_square() {
            let game = Game::default();
            let _ = game.type_at(Square::E6);
        }

        #[test]
        fn all_pieces() {
            let game = Game::default();
            let all = game.all_pieces();
            let expected = Bitboard::from_u64(0xffff00000000ffff);
            assert_eq!(all, expected);
        }

        #[test]
        fn is_square_empty_false() {
            let game = Game::default();
            assert!(!game.is_square_empty(Square::E2));
        }

        #[test]
        fn is_square_empty_true() {
            let game = Game::default();
            assert!(game.is_square_empty(Square::E3));
        }

        #[test]
        fn piece_color() {
            let game = Game::default();
            let black_king = game.color_at(Square::E8);
            assert_eq!(black_king, Color::BLACK);
        }

        #[test]
        #[should_panic]
        fn piece_color_empty_square() {
            let game = Game::default();
            let _ = game.color_at(Square::E6);
        }

        #[test]
        fn make_move_legal() {
            let mut game = Game::default();
            let m = Move {
                start: Square::E2,
                end: Square::E3,
            };
            game.make_move(m);
            assert_eq!(game.all_pieces().0, 0xffff00000010efff);
            assert_eq!(game.to_move, Color::BLACK);
            assert_eq!(game.en_passant_square, None);
            assert_eq!(game.halfmove_clock, 0);
            assert_eq!(game.fullmove_clock, 1);
        }

        /* #[test]
        fn make_move_illegal() {
            let mut game = Game::default();
            let m = Move {
                start: Square::E2,
                end: Square::F2,
            };
            let res = game.make_move(m);
            assert!(res.is_err());
        } */

        #[test]
        fn make_move_capture() {
            let mut game = Game::default();
            let m = Move {
                start: Square::E2,
                end: Square::E7,
            };
            game.make_move(m);
            assert_eq!(game.all_pieces().0, 18446462598732902399);
            assert_eq!(game.to_move, Color::BLACK);
            assert_eq!(game.en_passant_square, None);
            assert_eq!(game.halfmove_clock, 0);
            assert_eq!(game.fullmove_clock, 1);
        }

        #[test]
        fn attackers_from_fen() {
            let game =
                Game::from_fen("rnbqkbnr/p1pppppp/8/1p6/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 2")
                    .unwrap();
            assert!(game.is_attacked_by(Color::WHITE, Square::B5));
        }
    }

    mod movegen {
        use crate::{
            game::Game,
            movegen::{self, all_legal_moves},
            Color, Move, Square,
        };

        #[test]
        fn pseudolegal_knight_moves() {
            let moves = movegen::pseudolegal_knight_moves(Square::C3);
            assert_eq!(moves.0, 43234889994);
        }

        #[test]
        fn slider_moves() {
            // Position after 1. e2 e4
            let game =
                Game::from_fen("rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 2")
                    .unwrap();
            let moves = movegen::slider_moves(&game, Square::F1);
            assert_eq!(moves.0, 1108169199616);
        }

        #[test]
        #[should_panic]
        fn slider_moves_wrong_piece() {
            // Position after 1. e2 e4
            let game =
                Game::from_fen("rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 2")
                    .unwrap();
            let moves = movegen::slider_moves(&game, Square::E1);
            assert_eq!(moves.0, 1108169199616);
        }

        #[test]
        fn pseudolegal_slider_moves() {
            let game = Game::default();
            let moves = movegen::pseudolegal_slider_moves(&game, Square::F1);
            assert_eq!(moves.0, 20480);
        }

        #[test]
        #[should_panic]
        fn pseudolegal_slider_moves_wrong_piece() {
            let game = Game::default();
            let moves = movegen::pseudolegal_slider_moves(&game, Square::E1);
            assert_eq!(moves.0, 20480);
        }

        #[test]
        fn king_moves() {
            let game =
                Game::from_fen("rnbq1bnr/pppp1ppp/6k1/4p3/4P3/1K6/PPPP1PPP/RNBQ1BNR b - - 7 5")
                    .unwrap();
            let moves = movegen::king_moves(&game, Color::WHITE);
            assert_eq!(moves.0, 117768192);
        }

        #[test]
        #[should_panic]
        fn king_moves_no_king() {
            let game =
                Game::from_fen("rnbq1bnr/pppp1ppp/6k1/4p3/4P3/26/PPPP1PPP/RNBQ1BNR b - - 7 5")
                    .unwrap();
            let _ = movegen::king_moves(&game, Color::WHITE);
        }

        #[test]
        fn all_legal_from_initial() {
            let game = Game::default();
            let moves = all_legal_moves(&game);
            assert!(!moves.is_empty());
            assert_eq!(
                moves,
                [
                    Move {
                        start: Square::B1,
                        end: Square::A3
                    },
                    Move {
                        start: Square::B1,
                        end: Square::C3
                    },
                    Move {
                        start: Square::G1,
                        end: Square::F3
                    },
                    Move {
                        start: Square::G1,
                        end: Square::H3
                    },
                    Move {
                        start: Square::A2,
                        end: Square::A3
                    },
                    Move {
                        start: Square::A2,
                        end: Square::A4
                    },
                    Move {
                        start: Square::B2,
                        end: Square::B3
                    },
                    Move {
                        start: Square::B2,
                        end: Square::B4
                    },
                    Move {
                        start: Square::C2,
                        end: Square::C3
                    },
                    Move {
                        start: Square::C2,
                        end: Square::C4
                    },
                    Move {
                        start: Square::D2,
                        end: Square::D3
                    },
                    Move {
                        start: Square::D2,
                        end: Square::D4
                    },
                    Move {
                        start: Square::E2,
                        end: Square::E3
                    },
                    Move {
                        start: Square::E2,
                        end: Square::E4
                    },
                    Move {
                        start: Square::F2,
                        end: Square::F3
                    },
                    Move {
                        start: Square::F2,
                        end: Square::F4
                    },
                    Move {
                        start: Square::G2,
                        end: Square::G3
                    },
                    Move {
                        start: Square::G2,
                        end: Square::G4
                    },
                    Move {
                        start: Square::H2,
                        end: Square::H3
                    },
                    Move {
                        start: Square::H2,
                        end: Square::H4
                    }
                ]
            );
        }

        #[test]
        fn all_legal_with_possible_check() {
            let game =
                Game::from_fen("rnbqkbnr/pppp1ppp/8/4p2Q/4P3/8/PPPP1PPP/RNB1KBNR b KQkq - 1 2")
                    .unwrap();
            let moves = all_legal_moves(&game);
            for m in &moves {
                println!("{:?}", m);
            }
            assert!(!moves.is_empty());
        }

        #[test]
        fn all_legal_from_check() {
            let game =
                Game::from_fen("rnb1kbnr/pppp1ppp/8/4p3/1P3P1q/8/P1PPP1PP/RNBQKBNR w KQkq - 1 3")
                    .unwrap();
            let moves = all_legal_moves(&game);
            assert_eq!(
                moves,
                vec![Move {
                    start: Square::G2,
                    end: Square::G3
                }]
            );
        }

        #[test]
        fn all_legal_with_castling() {
            let game =
                Game::from_fen("r2qk2r/1ppn1ppp/p2bbn2/3p2B1/3P4/2NBPN1P/PP3PP1/R2QK2R b KQkq - 2 9").unwrap();
            let moves = all_legal_moves(&game);
            for m in &moves {
                println!("{:?}", m);
            }
            assert!(!moves.is_empty());
        }
    }

    mod square {
        use crate::Square;

        #[test]
        fn square_display() {
            let square = Square::from_u8(15);
            let display = square.to_string();
            assert_eq!(display, String::from("h2"));
        }

        #[test]
        fn square_parse() {
            let square = Square::from_u8(15);
            assert_eq!(square as u8, 15);
        }

        #[test]
        fn square_from_parts() {
            let file = 'e';
            let rank = '2';
            let square = Square::from_parts(&file, &rank).unwrap();
            assert_eq!(square, Square::E2);

            let file = 'h';
            let rank = '7';
            let square = Square::from_parts(&file, &rank).unwrap();
            assert_eq!(square, Square::H7);
        }

        #[test]
        #[should_panic]
        fn square_from_parts_file_oob() {
            let file = 'i';
            let rank = '2';
            let _ = Square::from_parts(&file, &rank).unwrap();
        }

        #[test]
        #[should_panic]
        fn square_from_parts_rank_oob() {
            let file = 'e';
            let rank = '9';
            let _ = Square::from_parts(&file, &rank).unwrap();
        }
    }

    mod bench {
        extern crate test;

        use crate::{
            game::Game,
            movegen::{all_legal_moves, king_moves, pawn_moves, pseudolegal_slider_moves, slider_moves},
            try_square_offset, Color, Square,
        };
        use test::Bencher;

        #[bench]
        fn bench_square_offset(b: &mut Bencher) {
            let s = Square::E4;
            b.iter(|| try_square_offset(s, 1, -1));
        }

        #[bench]
        fn bench_color_at(b: &mut Bencher) {
            let g = Game::default();
            b.iter(|| g.color_at(Square::E2));
        }

        #[bench]
        fn bench_type_at(b: &mut Bencher) {
            let g = Game::default();
            b.iter(|| g.type_at(Square::E2));
        }

        #[bench]
        fn bench_pawn_moves(b: &mut Bencher) {
            let g = Game::default();
            b.iter(|| pawn_moves(&g, Square::E2));
        }

        #[bench]
        fn bench_square_get_file(b: &mut Bencher) {
            let s = Square::E2;
            b.iter(|| s.get_file());
        }

        #[bench]
        fn bench_square_get_rank(b: &mut Bencher) {
            let s = Square::E2;
            b.iter(|| s.get_rank());
        }

        #[bench]
        fn bench_slider_moves(b: &mut Bencher) {
            let g = Game::from_fen("rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 2")
                .unwrap();
            b.iter(|| slider_moves(&g, Square::F1));
        }

        #[bench]
        fn bench_pseudo_slider_moves(b: &mut Bencher) {
            let g = Game::from_fen("rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 2")
                .unwrap();
            b.iter(|| pseudolegal_slider_moves(&g, Square::F1));
        }

        #[bench]
        fn bench_king_moves(b: &mut Bencher) {
            let g = Game::from_fen("rnbq1bnr/pppp1ppp/6k1/4p3/4P3/1K6/PPPP1PPP/RNBQ1BNR b - - 7 5")
                .unwrap();
            b.iter(|| king_moves(&g, Color::WHITE));
        }

        #[bench]
        fn bench_attackers_simple(b: &mut Bencher) {
            let game =
                Game::from_fen("rnbqkbnr/p1pppppp/8/1p6/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 2")
                    .unwrap();
            b.iter(|| game.is_attacked_by(Color::WHITE, Square::B5));
        }

        #[bench]
        fn bench_attackers_complex(b: &mut Bencher) {
            let game = Game::from_fen(
                "r1bqk1nr/pp3pbp/2n1p1p1/2p5/3pP3/2NP1NP1/PPP2PBP/R1BQ1RK1 w kq - 0 8",
            )
            .unwrap();
            b.iter(|| game.is_attacked_by(Color::WHITE, Square::H5));
        }

        #[bench]
        fn bench_all_legal_from_default(b: &mut Bencher) {
            let game = Game::default();
            b.iter(|| all_legal_moves(&game));
        }

        #[bench]
        fn bench_all_legal_from_complex(b: &mut Bencher) {
            let game = Game::from_fen("r2qkb1r/1ppn1ppp/p3bn2/3p2B1/3P4/2N1PN1P/PP3PP1/R2QKB1R b KQkq - 0 8").unwrap();
            b.iter(|| all_legal_moves(&game));
        }
    }
}
