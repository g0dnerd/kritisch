use anyhow::Context;

use crate::{
    bitboard::Bitboard,
    game::Game,
    magics::{BISHOP_MAGICS, BISHOP_MOVES, ROOK_MAGICS, ROOK_MOVES},
    try_square_offset, CastlingRights, Color, MagicTableEntry, Piece, Rank, Square,
};

/// Pawn attack patterns are known at compile time and
/// can be masked to get them from the correct rank
const PAWN_ATTACKS: [[u64; 8]; 2] = [
    // White
    [
        131072,   // a2 -> b3
        327680,   // b2 -> [a3, c3]
        655360,   // c2 -> [b3, d3]
        1310720,  // d2 -> [c3, e3]
        2621440,  // e2 -> [d3, f3]
        5242880,  // f2 -> [e3, g3]
        10485760, // g2 -> [f3, h3]
        4194304,  // h2 -> g3
    ],
    // Black
    [
        2,   // a2 -> b1
        5,   // b2 -> [a1, c1]
        10,  // c2 -> [b1, d1]
        20,  // d2 -> [c1, e1]
        40,  // e2 -> [d1, f1]
        80,  // f2 -> [e1, g1]
        160, // g2 -> [f1, h1]
        64,  // h2 -> g1
    ],
];

/// All knight moves are known at compile time
const KNIGHT_MOVES: [u64; 64] = [
    132096,
    329728,
    659712,
    1319424,
    2638848,
    5277696,
    10489856,
    4202496,
    33816580,
    84410376,
    168886289,
    337772578,
    675545156,
    1351090312,
    2685403152,
    1075839008,
    8657044482,
    21609056261,
    43234889994,
    86469779988,
    172939559976,
    345879119952,
    687463207072,
    275414786112,
    2216203387392,
    5531918402816,
    11068131838464,
    22136263676928,
    44272527353856,
    88545054707712,
    175990581010432,
    70506185244672,
    567348067172352,
    1416171111120896,
    2833441750646784,
    5666883501293568,
    11333767002587136,
    22667534005174272,
    45053588738670592,
    18049583422636032,
    145241105196122112,
    362539804446949376,
    725361088165576704,
    1450722176331153408,
    2901444352662306816,
    5802888705324613632,
    11533718717099671552,
    4620693356194824192,
    288234782788157440,
    576469569871282176,
    1224997833292120064,
    2449995666584240128,
    4899991333168480256,
    9799982666336960512,
    1152939783987658752,
    2305878468463689728,
    1128098930098176,
    2257297371824128,
    4796069720358912,
    9592139440717824,
    19184278881435648,
    38368557762871296,
    4679521487814656,
    9077567998918656,
];

/// Retrieves the pseudo-legal knight moves for `square` from the lookup table.
/// Does NOT check for positional legality.
///
/// # Example
///
/// ```
/// use kritisch::{movegen::pseudolegal_knight_moves, Square};
/// let moves = pseudolegal_knight_moves(Square::C3);
/// assert_eq!(moves.0, 43234889994);
/// ```
pub fn pseudolegal_knight_moves(square: Square) -> Bitboard {
    Bitboard::from_u64(KNIGHT_MOVES[square as usize])
}

/// Returns the squares a pawn on `square` could pseudolegally attack.
/// Does NOT check for positional legality.
///
/// # Example
///
/// ```
/// use kritisch::{Color, movegen::pawn_attacks, Square};
/// let attacks = pawn_attacks(Square::E2, Color::WHITE);
/// assert_eq!(attacks.0, 2621440);
/// ```
pub fn pawn_attacks(square: Square, color: Color) -> Bitboard {
    let file_idx = square.get_file() as usize;
    let rank_idx = (square.get_rank() as usize) - 1.clamp(0, 5);
    let attacks = PAWN_ATTACKS[color as usize][file_idx] << (8 * rank_idx);
    Bitboard::from_u64(attacks)
}

/// Returns a bitboard of squares a pawn on `square` can move to.
/// This checks for positional legality, but not whether or not it leaves the king in check.
///
/// # Example
///
/// ```
/// use kritisch::{game::Game, movegen::pawn_moves, Square};
/// let game = Game::default();
/// let moves = pawn_moves(&game, Square::E2).unwrap();
/// assert_eq!(moves.0, 269484032);
/// ```
pub fn pawn_moves(game: &Game, square: Square) -> anyhow::Result<Bitboard> {
    let mut moves = Bitboard::empty();

    let color = game
        .color_at(square)
        .context("Tried to get color of piece on empty square while generating pawn moves")?;

    // White pawns move up, black pawns move down the board
    let direction = match color {
        Color::WHITE => 1,
        _ => -1,
    };

    // Check if the square one ahead is within bounds
    if let Some(offset) = try_square_offset(square, 0, direction) {
        // Check if the square in front on the pawn is empty
        if game.is_square_empty(offset) {
            moves |= offset;

            // If the pawn is on its initial rank, check if the square two ahead is empty
            if color == Color::WHITE && square.get_rank() == Rank::SECOND
                || (color == Color::BLACK && square.get_rank() == Rank::SEVENTH)
            {
                let rank = offset.get_rank();
                let file = offset.get_file();
                let two_ahead = Square::from_u8(((rank as i8 + direction) * 8 + file as i8) as u8);

                if game.is_square_empty(two_ahead) {
                    moves |= two_ahead;
                }
            }
        }
    }

    // Check for captures
    for dx in [-1, 1] {
        if let Some(offset) = try_square_offset(square, dx, direction) {
            // If the the piece on the destination (or en passant) square has the target color,
            // add the move
            if let Some(target_color) = game.color_at(offset) {
                if (color ^ 1) == target_color || game.en_passant_square == Some(offset) {
                    moves |= offset;
                }
            }
        }
    }

    Ok(moves)
}

/// Returns a bitboard of squares a king on `square` can move to.
/// This checks for positional legality, but not whether or not it leaves the king in check.
///
/// # Example
///
/// ```
/// use kritisch::{game::Game, movegen::king_moves, Color, Move, Square};
/// let mut game = Game::from_fen("rnbq1bnr/pppp1ppp/6k1/4p3/4P3/1K6/PPPP1PPP/RNBQ1BNR b - - 7 5").unwrap();
/// let moves = king_moves(&game, Color::WHITE).unwrap();
/// assert_eq!(moves.0, 117768192);
///
///
/// ```
pub fn king_moves(game: &Game, color: Color) -> anyhow::Result<Bitboard> {
    let mut moves = Bitboard::empty();

    let king_mask =
        game.color_bitboards[color as usize] & game.piece_bitboards[Piece::KING as usize];
    if king_mask.is_empty() {
        anyhow::bail!("No king found");
    }
    let square = Square::from_u8(king_mask.trailing_zeros() as u8);

    for (dx, dy) in [
        (1, 1),
        (1, 0),
        (1, -1),
        (0, 1),
        (0, -1),
        (-1, 1),
        (-1, 0),
        (-1, -1),
    ] {
        // Add all moves by one square in all nine directions, filter out moves that would capture
        // own color later
        if let Some(offset) = try_square_offset(square, dx, dy) {
            moves |= offset;
        }
    }

    if game.in_check.is_none() {
        match color {
            Color::WHITE => {
                if game.castling_rights & CastlingRights::WHITE_KINGSIDE != 0
                    && game.is_square_empty(Square::F1)
                    && game.is_square_empty(Square::G1)
                {
                    moves |= Square::G1;
                } else if game.castling_rights & CastlingRights::WHITE_QUEENSIDE != 0
                    && game.is_square_empty(Square::B1)
                    && game.is_square_empty(Square::C1)
                    && game.is_square_empty(Square::D1)
                {
                    moves |= Square::C1;
                }
            }
            Color::BLACK => {
                if game.castling_rights & CastlingRights::BLACK_KINGSIDE != 0
                    && game.is_square_empty(Square::F8)
                    && game.is_square_empty(Square::G8)
                {
                    moves |= Square::G8;
                } else if game.castling_rights & CastlingRights::BLACK_QUEENSIDE != 0
                    && game.is_square_empty(Square::B8)
                    && game.is_square_empty(Square::C8)
                    && game.is_square_empty(Square::D8)
                {
                    moves |= Square::C8;
                }
            }
        }
    }

    // Remove moves that would capture a piece of the same color before returning
    Ok(moves & !game.color_bitboards[color as usize])
}

/// Calculates the pseudo-legal slider moves for `square` by using the pre-calculated slider
/// magics. Checks for blockers in the slider's way, but does NOT check for positional legality.
///
/// # Example
///
/// ```
/// use kritisch::{movegen::pseudolegal_slider_moves, game::Game, Piece, Square};
/// let game = Game::default();
/// let moves = pseudolegal_slider_moves(&game, Square::F1).unwrap();
/// assert_eq!(moves.0, 20480);
/// ```
pub fn pseudolegal_slider_moves(game: &Game, square: Square) -> anyhow::Result<Bitboard> {
    let piece = game
        .type_at(square)
        .context("Tried to get slider piece for pseudolegal_slider_moves")?;

    // Get the blockers for the slider type and square
    let blockers = get_blockers_from_position(game, piece, square);

    // Retrieve the moves from the magic table
    match piece {
        Piece::ROOK => Ok(Bitboard::from_u64(
            ROOK_MOVES[magic_index(&ROOK_MAGICS[square as usize], blockers)],
        )),
        Piece::BISHOP => Ok(Bitboard::from_u64(
            BISHOP_MOVES[magic_index(&BISHOP_MAGICS[square as usize], blockers)],
        )),
        Piece::QUEEN => Ok(Bitboard::from_u64(
            ROOK_MOVES[magic_index(&ROOK_MAGICS[square as usize], blockers)]
                | BISHOP_MOVES[magic_index(&BISHOP_MAGICS[square as usize], blockers)],
        )),
        _ => panic!("Non-slider piece passed to `pseudolegal_slider_moves`"),
    }
}

/// Returns a bitboard of squares a slider piece on `square` can move to.
/// This checks for positional legality, but not whether or not it leaves the king in check.
///
/// # Example
///
/// ```
/// use kritisch::{game::Game, movegen::slider_moves, Square};
/// // Position after 1. e2 e4
/// let game = Game::from_fen("rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 2").unwrap();
/// let moves = slider_moves(&game, Square::F1).unwrap();
/// assert_eq!(moves.0, 1108169199616);
/// ```
pub fn slider_moves(game: &Game, square: Square) -> anyhow::Result<Bitboard> {
    let moves = pseudolegal_slider_moves(game, square)
        .context("Tried to get pseudolegal slider moves to calculate legal slider moves")?;

    let color = game
        .color_at(square)
        .context("Tried to get slider color for slider_moves")?;

    Ok(moves & !game.color_bitboards[color as usize])
}

// Gets the index in the magic table for the given blocker mask
fn magic_index(entry: &MagicTableEntry, mut blockers: Bitboard) -> usize {
    blockers &= entry.mask;
    let hash = blockers.0.wrapping_mul(entry.magic);
    let index = (hash >> entry.shift) as usize;
    entry.offset as usize + index
}

// Retrieves the blockers for a slider piece type and square from the pre-calculated magics table
fn get_blockers_from_position(game: &Game, piece: Piece, square: Square) -> Bitboard {
    let blockers = match piece {
        Piece::ROOK => Bitboard::from_u64(ROOK_MAGICS[square as usize].mask),
        Piece::BISHOP => Bitboard::from_u64(BISHOP_MAGICS[square as usize].mask),
        Piece::QUEEN => Bitboard::from_u64(
            ROOK_MAGICS[square as usize].mask | BISHOP_MAGICS[square as usize].mask,
        ),
        _ => panic!("Non slider-piece passed to `get_blockers_from_position`"),
    };

    // Only return the pieces that are actually on the board
    blockers & game.all_pieces()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slider_blockers() {
        let game = Game::default();
        let blockers = get_blockers_from_position(&game, Piece::BISHOP, Square::F1);
        assert_eq!(blockers.0, 20480);
    }
}
