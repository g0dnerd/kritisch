use crate::{
    bitboard::Bitboard, CastlingRights, Color, Move, Piece, Square, PIECE_REPR_B, PIECE_REPR_W,
};
use anyhow::Context;

#[derive(Debug)]
enum IllegalMoveError {
    CaptureOwnPiece(Move),
}

impl std::fmt::Display for IllegalMoveError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Self::CaptureOwnPiece(m) => write!(
                f,
                "Illegal move from {} to {} attempted - would capture own piece",
                m.start, m.end
            ),
        }
    }
}

impl std::error::Error for IllegalMoveError {}

pub struct Game {
    pub color_bitboards: [Bitboard; 2],
    pub piece_bitboards: [Bitboard; 6],

    pub to_move: Color,
    pub castling_rights: u8,

    pub en_passant: Option<Square>,
    pub in_check: Option<Color>,

    pub halfmove_clock: usize,
    pub fullmove_clock: usize,
}

impl std::default::Default for Game {
    fn default() -> Self {
        let white_bb = Bitboard::from_squares(vec![
            Square::A1,
            Square::B1,
            Square::C1,
            Square::D1,
            Square::E1,
            Square::F1,
            Square::G1,
            Square::H1,
            Square::A2,
            Square::B2,
            Square::C2,
            Square::D2,
            Square::E2,
            Square::F2,
            Square::G2,
            Square::H2,
        ]);

        let black_bb = Bitboard::from_squares(vec![
            Square::A8,
            Square::B8,
            Square::C8,
            Square::D8,
            Square::E8,
            Square::F8,
            Square::G8,
            Square::H8,
            Square::A7,
            Square::B7,
            Square::C7,
            Square::D7,
            Square::E7,
            Square::F7,
            Square::G7,
            Square::H7,
        ]);

        let color_bitboards = [white_bb, black_bb];

        let rook_bb = Bitboard::from_squares(vec![Square::A1, Square::H1, Square::A8, Square::H8]);
        let knight_bb =
            Bitboard::from_squares(vec![Square::B1, Square::G1, Square::B8, Square::G8]);
        let bishop_bb =
            Bitboard::from_squares(vec![Square::C1, Square::F1, Square::C8, Square::F8]);
        let queen_bb = Bitboard::from_squares(vec![Square::D1, Square::D8]);
        let king_bb = Bitboard::from_squares(vec![Square::E1, Square::E8]);
        let pawn_bb = Bitboard::from_squares(vec![
            Square::A2,
            Square::B2,
            Square::C2,
            Square::D2,
            Square::E2,
            Square::F2,
            Square::G2,
            Square::H2,
            Square::A7,
            Square::B7,
            Square::C7,
            Square::D7,
            Square::E7,
            Square::F7,
            Square::G7,
            Square::H7,
        ]);

        let piece_bitboards = [pawn_bb, knight_bb, bishop_bb, rook_bb, queen_bb, king_bb];

        Self {
            color_bitboards,
            piece_bitboards,
            to_move: Color::WHITE,
            castling_rights: CastlingRights::ALL_LEGAL,
            en_passant: None,
            in_check: None,
            halfmove_clock: 0,
            fullmove_clock: 1,
        }
    }
}

impl std::fmt::Display for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut board = String::new();
        board.push('\n');
        for i in 0..64 {
            let file = i % 8;
            let rank = i / 8;
            let square = Square::from_u8(64 - (rank * 8 + 8 - file));
            if self.color_bitboards[0].contains(square) {
                for (piece_idx, piece_bb) in self.piece_bitboards.iter().enumerate() {
                    if piece_bb.contains(square) {
                        board.push(PIECE_REPR_W[piece_idx]);
                        board.push(' ');
                    }
                }
            } else if self.color_bitboards[1].contains(square) {
                for (piece_idx, piece_bb) in self.piece_bitboards.iter().enumerate() {
                    if piece_bb.contains(square) {
                        board.push(PIECE_REPR_B[piece_idx]);
                        board.push(' ');
                    }
                }
            } else {
                board.push_str(". ");
            }
            if file == 7 {
                board.push('\n');
            }
        }
        write!(f, "{}", board)
    }
}

impl Game {
    /// Returns `Some(Piece)` if one of `self`'s piece bitboards
    /// contains `s` and `None` otherwise.
    pub fn type_at(&self, s: Square) -> Option<Piece> {
        let mask = Bitboard::from_square(s);

        // Checks if there is a piece bitboard that contains the given square
        // by bitAnd-ing it with a bitboard of just that square.
        // Maps the found piece value to the `Piece` enum
        (0..=5)
            .find(|i| !(self.piece_bitboards[*i as usize] & mask).is_empty())
            .map(|piece_idx| Piece::from_u8(piece_idx as u8))
    }

    /// Returns `Some(Color)` if one of `self`'s color bitboards
    /// contains `s` and `None` otherwise.
    pub fn color_at(&self, s: Square) -> Option<Color> {
        let mask = Bitboard::from_square(s);

        // Checks if there is a color bitboard that contains the given square
        // by bitAnd-ing it with a bitboard of just that square.
        // Maps the found piece value to the `Color` enum
        (0..=1)
            .find(|i| !(self.color_bitboards[*i as usize] & mask).is_empty())
            .map(|color_idx| Color::from_u8(color_idx as u8))
    }

    /// Returns a combined `Bitboard` of all pieces on the board
    pub fn all_pieces(&self) -> Bitboard {
        self.color_bitboards[0] | self.color_bitboards[1]
    }

    /// Returns `true` if there is any piece on `s`, `false` otherwise.
    pub fn is_square_empty(&self, s: Square) -> bool {
        !self.all_pieces().contains(s)
    }

    /// Attempts to make a move on the board. This is the lowest level of doing so and inherently
    /// only checks for very few error conditions.
    pub fn make_move(&mut self, m: Move) -> anyhow::Result<()> {
        let (start, end) = (m.start, m.end);

        // Check if the move would capture a piece of the same color
        if self.color_at(start) == self.color_at(end) {
            anyhow::bail!(IllegalMoveError::CaptureOwnPiece(m))
        }

        let piece = self.type_at(start).context("No piece at starting square")?;
        let color = self
            .color_at(start)
            .context("No piece at starting square while getting piece color")?;

        let is_capture = self.is_capture(m);

        let is_castle = if piece == Piece::ROOK || piece == Piece::KING {
            self.is_castle(m, piece, color)
        } else {
            false
        };

        // If the move castles, dispatch the move handling to `self.castle` instead
        if is_castle {
            return self.castle(m);
        }

        if is_capture {
            self.handle_capture(m, piece, color)?;
        }

        // TODO: Handle promotions
        // TODO: Where do I want to handle attack maps?

        self.move_piece(m, piece, color);

        // Increment the halfmove clock if the move was not a pawn move or a capture.
        if piece == Piece::PAWN || is_capture {
            self.halfmove_clock = 0;
        } else {
            self.halfmove_clock += 1;
        }
        // Increment the fullmove clock if white is to move next
        if self.to_move == Color::BLACK {
            self.fullmove_clock += 1;
        }

        self.to_move = Color::from_u8(self.to_move as u8 ^ 1);

        Ok(())
    }

    /// Actually 'moves' a piece by creating a bitboard mask and XOR/OR-ing it with
    /// the respective color and piece bitboards
    fn move_piece(&mut self, m: Move, p: Piece, c: Color) {
        let from_mask = Bitboard::from_square(m.start);
        let to_mask = Bitboard::from_square(m.end);
        self.color_bitboards[c as usize] ^= from_mask;
        self.color_bitboards[c as usize] |= to_mask;
        self.piece_bitboards[p as usize] ^= from_mask;
        self.piece_bitboards[p as usize] |= to_mask;
    }

    /// Handles a capture move by removing the captured piece from the board
    fn handle_capture(&mut self, m: Move, p: Piece, c: Color) -> anyhow::Result<()> {
        let captured_piece = self
            .type_at(m.end)
            .context("Tried to capture on empty square")?;

        let is_en_passant = if p == Piece::PAWN {
            self.is_en_passant(m, captured_piece)
        } else {
            false
        };

        // Remove the captured piece from the board.
        // If the move is en_passant, remove the piece from the EP square
        // instead of the move end square.
        if !is_en_passant {
            self.remove_piece(m.end, captured_piece)?;
        } else {
            match c {
                Color::WHITE => {
                    let target_square = Square::from_u8(m.end as u8 - 8);
                    eprintln!("target square for removal: {}", target_square);
                    self.remove_piece(target_square, captured_piece)?;
                }
                Color::BLACK => {
                    let target_square = Square::from_u8(m.end as u8 + 8);
                    eprintln!("target square for removal: {}", target_square);
                    self.remove_piece(target_square, captured_piece)?;
                }
            }
        }
        Ok(())
    }

    /// Returns `true` if there is a piece on `m.end` and if
    /// it does not have the same color as the piece on `m.start`.
    pub fn is_capture(&self, m: Move) -> bool {
        if self.is_square_empty(m.end) {
            return false;
        }
        if self.color_at(m.end) == self.color_at(m.start) {
            return false;
        }
        true
    }

    /// Returns `true` if `m` is one of eight possible castling moves in check.
    pub fn is_castle(&self, m: Move, piece: Piece, color: Color) -> bool {
        matches!(
            (piece, color, m.start, m.end),
            (Piece::ROOK, Color::WHITE, Square::A1, Square::D1)
                | (Piece::ROOK, Color::WHITE, Square::H1, Square::F1)
                | (Piece::KING, Color::WHITE, Square::E1, Square::C1)
                | (Piece::KING, Color::WHITE, Square::E1, Square::G1)
                | (Piece::ROOK, Color::BLACK, Square::A8, Square::D8)
                | (Piece::ROOK, Color::BLACK, Square::H8, Square::F8)
                | (Piece::KING, Color::BLACK, Square::E8, Square::C8)
                | (Piece::KING, Color::BLACK, Square::E8, Square::G8)
        )
    }

    pub fn is_en_passant(&self, m: Move, captured_piece: Piece) -> bool {
        self.en_passant == Some(m.end) && captured_piece == Piece::PAWN
    }

    fn castle(&mut self, _m: Move) -> anyhow::Result<()> {
        Ok(())
    }

    fn remove_piece(&mut self, s: Square, piece: Piece) -> anyhow::Result<()> {
        let mask = Bitboard::from_square(s);

        let color = self
            .color_at(s)
            .context("Tried to remove piece from empty square")?;

        // If a rook was captured on its initial square, update castling rights accordingly
        if piece == Piece::ROOK {
            match (s, color) {
                (Square::A1, Color::WHITE) => {
                    self.castling_rights &= !CastlingRights::WHITE_QUEENSIDE
                }
                (Square::H1, Color::WHITE) => {
                    self.castling_rights &= !CastlingRights::WHITE_KINGSIDE
                }
                (Square::A8, Color::BLACK) => {
                    self.castling_rights &= !CastlingRights::BLACK_QUEENSIDE
                }
                (Square::H8, Color::BLACK) => {
                    self.castling_rights &= !CastlingRights::BLACK_KINGSIDE
                }
                _ => (),
            }
        }

        self.color_bitboards[color as usize] ^= mask;
        self.piece_bitboards[piece as usize] ^= mask;
        Ok(())
    }
}
