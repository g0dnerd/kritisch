use crate::{
    bitboard::Bitboard,
    magics::{BISHOP_MAGICS, BISHOP_MOVES, ROOK_MAGICS, ROOK_MOVES},
    movegen::{get_blockers_from_position, magic_index, pseudolegal_knight_moves},
    try_square_offset, CastlingRights, Color, File, Move, Piece, Square, PIECE_REPR_B,
    PIECE_REPR_W,
};
use anyhow::Context;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Game {
    pub color_bitboards: [Bitboard; 2],
    pub piece_bitboards: [Bitboard; 6],

    pub to_move: Color,
    pub castling_rights: u8,

    pub en_passant_square: Option<Square>,
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
            en_passant_square: None,
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
        for s in 0..64 {
            let file = s % 8;
            let rank = s / 8;
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
            if File::from_u8(file) == File::H {
                board.push('\n');
            }
        }
        write!(f, "{}", board)
    }
}

impl Game {
    fn empty() -> Self {
        let color_bitboards = [Bitboard::empty(); 2];
        let piece_bitboards = [Bitboard::empty(); 6];

        Self {
            color_bitboards,
            piece_bitboards,
            to_move: Color::WHITE,
            castling_rights: CastlingRights::ALL_LEGAL,
            en_passant_square: None,
            in_check: None,
            halfmove_clock: 0,
            fullmove_clock: 1,
        }
    }
    /// Tries to parse the given FEN string into a position
    /// TODO: Parse attacks
    pub fn from_fen(fen: &'static str) -> anyhow::Result<Self> {
        let mut pos = Self::empty();
        let mut square = Square::A8;

        let mut index = 0;

        for (i, c) in fen.chars().enumerate() {
            if c == ' ' {
                index = i + 1;
                break;
            }
            if c.is_ascii_digit() {
                let add = (c.to_digit(10).unwrap() as u8).clamp(1, 7);
                square = square + add;
                if square.get_file() == File::A {
                    square = square - 1u8;
                }
            } else if c == '/' {
                square = square - 15u8;
            } else if PIECE_REPR_B.contains(&c) || PIECE_REPR_W.contains(&c) {
                let piece = Piece::from_char(&c);
                let color = if c.is_ascii_lowercase() {
                    Color::BLACK
                } else {
                    Color::WHITE
                };
                pos.color_bitboards[color as usize] |= square;
                pos.piece_bitboards[piece as usize] |= square;

                if square.get_file() != File::H {
                    square = square + 1u8;
                }
            } else {
                anyhow::bail!("Unexpected character in FEN string");
            }
            index = i + 1;
        }

        if let Some(c) = fen.chars().nth(index) {
            match c {
                'w' => pos.to_move = Color::WHITE,
                'b' => pos.to_move = Color::BLACK,
                _ => anyhow::bail!("Expected color specification for player to move"),
            }
            index += 1
        }

        index += 1;

        pos.castling_rights = CastlingRights::NO_LEGAL;

        for c in fen[index..].chars() {
            if c == ' ' {
                index += 1;
                break;
            } else if c == '-' {
                index += 2;
                break;
            }
            match c {
                'K' => pos.castling_rights |= CastlingRights::WHITE_KINGSIDE,
                'Q' => pos.castling_rights |= CastlingRights::WHITE_QUEENSIDE,
                'k' => pos.castling_rights |= CastlingRights::BLACK_KINGSIDE,
                'q' => pos.castling_rights |= CastlingRights::BLACK_QUEENSIDE,
                _ => anyhow::bail!("Unexpected character in castling rights section of FEN string"),
            }
            index += 1;
        }

        match fen.chars().nth(index) {
            Some(c) => {
                if c.is_ascii_lowercase() {
                    match fen.chars().nth(index + 1) {
                        Some(d) => {
                            if d.is_ascii_digit() {
                                match Square::from_parts(&c, &d) {
                                    Ok(s) => pos.en_passant_square = Some(s),
                                    Err(_) => anyhow::bail!(
                                        "Couldn't parse en passant square in FEN string"
                                    ),
                                }
                            }
                        }
                        None => anyhow::bail!(
                            "Expected file while parsing en-passant square from FEN string"
                        ),
                    }
                } else if c == '-' {
                    index += 1;
                }
            }
            None => anyhow::bail!("Incomplete FEN string - move counts missing"),
        }

        if fen.chars().nth(index) != Some(' ') {
            anyhow::bail!(
                "Error while parsing FEN string - expected whitespace after en passant square"
            )
        }
        index += 1;

        match fen.chars().nth(index) {
            Some(c) => {
                if c.is_ascii_digit() {
                    let mut hmc = String::new();
                    hmc.push(c);
                    let mut peek = 1;
                    loop {
                        if let Some(n) = fen.chars().nth(index + peek) {
                            if n == ' ' {
                                break;
                            }
                            hmc.push(n);
                            peek += 1;
                        } else {
                            anyhow::bail!("Incomplete FEN string - fullmove clock missing")
                        }
                    }
                    pos.halfmove_clock = hmc
                        .parse()
                        .context("tried to cast FEN halfmove clock to usize")?;
                    index += peek;
                } else {
                    anyhow::bail!("Expected digit in halfmove clock position in FEN string")
                }
            }
            None => anyhow::bail!("Incomplete FEN string - halfmove clock missing"),
        }

        if fen.chars().nth(index) != Some(' ') {
            anyhow::bail!(
                "Error while parsing FEN string - expected whitespace after halfmove clock"
            )
        }
        index += 1;

        match fen.chars().nth(index) {
            Some(c) => {
                if c.is_ascii_digit() {
                    let mut fmc = String::new();
                    fmc.push(c);
                    let mut peek = 1;
                    while let Some(n) = fen.chars().nth(index + peek) {
                        if n == ' ' {
                            break;
                        }
                        fmc.push(n);
                        peek += 1;
                    }
                    pos.fullmove_clock = fmc
                        .parse()
                        .context("tried to cast FEN fullmove clock to usize")?;
                } else {
                    anyhow::bail!("Expected digit in fullmove clock position in FEN string")
                }
            }
            None => anyhow::bail!("Incomplete FEN string - fullmove clock missing"),
        }
        Ok(pos)
    }
    /// Returns `Some(Piece)` if one of `self`'s piece bitboards
    /// contains `s` and `None` otherwise.
    pub fn type_at(&self, s: Square) -> Piece {
        let mask = Bitboard::from_square(s);

        // Checks if there is a piece bitboard that contains the given square
        // by bitAnd-ing it with a bitboard of just that square.
        // Maps the found piece value to the `Piece` enum
        if let Some(piece) = (0..=5)
            .find(|i| !(self.piece_bitboards[*i as usize] & mask).is_empty())
            .map(|piece_idx| Piece::from_u8(piece_idx as u8))
        {
            return piece;
        } else {
            panic!("Tried to get piece type from empty square")
        }
    }

    /// Returns the `Color` of the piece on `s`.
    pub fn color_at(&self, s: Square) -> Color {
        let mask = Bitboard::from_square(s);

        // Checks if there is a color bitboard that contains the given square
        // by bitAnd-ing it with a bitboard of just that square.
        // Maps the found piece value to the `Color` enum
        (0..=1)
            .find(|i| !(self.color_bitboards[*i as usize] & mask).is_empty())
            .map(|color_idx| Color::from_u8(color_idx as u8))
            .unwrap()
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
    pub fn make_move(&mut self, m: Move) {
        let piece = self.type_at(m.start);
        let color = self.color_at(m.start);

        let is_capture = self.is_capture(m);

        let is_castle = if piece == Piece::KING {
            self.is_castle(m, piece, color)
        } else {
            false
        };

        // If the move castles, dispatch the move handling to `self.castle` instead
        if is_castle {
            match m.end {
                Square::C1 => self.move_piece(
                    Move {
                        start: Square::A1,
                        end: Square::D1,
                    },
                    Piece::ROOK,
                    color,
                ),
                Square::G1 => self.move_piece(
                    Move {
                        start: Square::H1,
                        end: Square::F1,
                    },
                    Piece::ROOK,
                    color,
                ),
                Square::C8 => self.move_piece(
                    Move {
                        start: Square::A8,
                        end: Square::D8,
                    },
                    Piece::ROOK,
                    color,
                ),
                Square::G8 => self.move_piece(
                    Move {
                        start: Square::H8,
                        end: Square::F8,
                    },
                    Piece::ROOK,
                    color,
                ),
                _ => panic!(
                    "Castling to illegal square (move: {:?} {:?} -> {:?})",
                    piece, m.start, m.end
                ),
            }
        }

        if is_capture {
            self.handle_capture(m, piece, color);
        }

        // TODO: Handle promotions

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

        // Change which player's turn it is
        self.to_move = self.to_move ^ 1;
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
    fn handle_capture(&mut self, m: Move, p: Piece, c: Color) {
        let captured_piece = self.type_at(m.end);

        let is_en_passant = if p == Piece::PAWN {
            self.is_en_passant(m, captured_piece)
        } else {
            false
        };

        // Remove the captured piece from the board.
        // If the move is en_passant, remove the piece from the EP square
        // instead of the move end square.
        if !is_en_passant {
            self.remove_piece(m.end, captured_piece);
        } else {
            match c {
                Color::WHITE => {
                    let target_square = m.end - 8u8;
                    self.remove_piece(target_square, captured_piece);
                }
                Color::BLACK => {
                    let target_square = m.end + 8u8;
                    self.remove_piece(target_square, captured_piece);
                }
            }
        }
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
        matches!((piece, color, m.start, m.end), |(
            Piece::KING,
            Color::WHITE,
            Square::E1,
            Square::C1,
        )| (
            Piece::KING,
            Color::WHITE,
            Square::E1,
            Square::G1
        ) | (
            Piece::KING,
            Color::BLACK,
            Square::E8,
            Square::C8
        ) | (
            Piece::KING,
            Color::BLACK,
            Square::E8,
            Square::G8
        ))
    }

    pub fn is_en_passant(&self, m: Move, captured_piece: Piece) -> bool {
        self.en_passant_square == Some(m.end) && captured_piece == Piece::PAWN
    }

    fn remove_piece(&mut self, s: Square, piece: Piece) {
        let mask = Bitboard::from_square(s);

        let color = self.color_at(s);

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
    }

    pub fn is_attacked_by(&self, color: Color, square: Square) -> bool {
        match color {
            Color::WHITE => {
                if let Some(offset) = try_square_offset(square, -1, -1) {
                    if (self.piece_bitboards[Piece::PAWN as usize]
                        & self.color_bitboards[Color::WHITE as usize])
                        .contains(offset)
                    {
                        return true;
                    }
                }
                if let Some(offset) = try_square_offset(square, 1, -1) {
                    if (self.piece_bitboards[Piece::PAWN as usize]
                        & self.color_bitboards[Color::WHITE as usize])
                        .contains(offset)
                    {
                        return true;
                    }
                }
            }
            Color::BLACK => {
                if let Some(offset) = try_square_offset(square, -1, 1) {
                    if (self.piece_bitboards[Piece::PAWN as usize]
                        & self.color_bitboards[Color::BLACK as usize])
                        .contains(offset)
                    {
                        return true;
                    }
                }
                if let Some(offset) = try_square_offset(square, 1, 1) {
                    if (self.piece_bitboards[Piece::PAWN as usize]
                        & self.color_bitboards[Color::BLACK as usize])
                        .contains(offset)
                    {
                        return true;
                    }
                }
            }
        }

        if self.is_attacked_by_knight(color, square) {
            return true;
        }
        if self.is_attacked_by_king(color, square) {
            return true;
        }
        self.is_attacked_by_slider(color, square)
    }

    // Returns `true` if `square` can be reached by a knight of `color`.
    fn is_attacked_by_knight(&self, color: Color, square: Square) -> bool {
        // Since knight moves are fully symmetrical, get knight moves from `square`
        let mut origins = pseudolegal_knight_moves(square);
        while !origins.is_empty() {
            let s = Square::from_u8(origins.trailing_zeros() as u8);
            if (self.color_bitboards[color as usize] & self.piece_bitboards[Piece::KNIGHT as usize])
                .contains(s)
            {
                return true;
            }
            origins.clear_lsb();
        }
        false
    }

    // Returns `true` if `square` can be reached by the king of `color`.
    fn is_attacked_by_king(&self, color: Color, square: Square) -> bool {
        // Since king moves are fully symmetrical, get knight moves from `square`
        for (dx, dy) in [
            (-1, -1),
            (-1, 1),
            (1, -1),
            (1, -1),
            (0, -1),
            (0, 1),
            (-1, 0),
            (1, 0),
        ] {
            if let Some(s) = try_square_offset(square, dx, dy) {
                if (self.piece_bitboards[Piece::KING as usize]
                    & self.color_bitboards[color as usize])
                    .contains(s)
                {
                    return true;
                }
            }
        }
        false
    }

    fn is_attacked_by_slider(&self, color: Color, square: Square) -> bool {
        let blockers = get_blockers_from_position(&self, Piece::QUEEN, square);
        let mut moves = Bitboard::from_u64(
            ROOK_MOVES[magic_index(&ROOK_MAGICS[square as usize], blockers)]
                | BISHOP_MOVES[magic_index(&BISHOP_MAGICS[square as usize], blockers)],
        );
        while !moves.is_empty() {
            let s = Square::from_u8(moves.trailing_zeros() as u8);
            if self.color_bitboards[color as usize].contains(s) {
                if self.piece_bitboards[Piece::ROOK as usize].contains(s)
                    || self.piece_bitboards[Piece::BISHOP as usize].contains(s)
                    || self.piece_bitboards[Piece::QUEEN as usize].contains(s)
                {
                    return true;
                }
            }
            moves.clear_lsb();
        }
        false
    }
}
