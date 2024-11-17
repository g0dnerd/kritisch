use anyhow::Context;

use crate::{bitboard::Bitboard, game::Game, try_square_offset, Color, Square};

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
    let file_idx = square as usize % 8;
    let rank_idx = (square as usize / 8) - 1.clamp(0, 5);
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
    let mut moves = Bitboard::from_u64(0);

    let color = game
        .color_at(square)
        .context("Tried to get color of piece on empty square while generating pawn moves")?;

    // White pawns move up, black pawns move down the board
    let direction = if color == Color::WHITE { 1 } else { -1 };

    // Check if the square in front on the pawn is empty
    if let Some(offset_by_delta) = try_square_offset(square, 0, direction) {
        if game.is_square_empty(offset_by_delta) {
            moves |= offset_by_delta.to_u64();

            // If the pawn is on its initial rank, check if the square two ahead is empty
            if (color == Color::WHITE && square as u8 / 8 == 1)
                || (color == Color::BLACK && square as u8 / 8 == 6)
            {
                if let Some(offset_by_delta) = try_square_offset(offset_by_delta, 0, direction) {
                    if game.is_square_empty(offset_by_delta) {
                        moves |= offset_by_delta.to_u64();
                    }
                }
            }
        }
    }

    // Check for captures
    for dx in [-1, 1] {
        if let Some(offset_by_delta) = try_square_offset(square, dx, direction) {
            // If the the piece on the destination (or en passant) square has the target color,
            // add the move
            if let Some(target_color) = game.color_at(offset_by_delta) {
                if (color as u8 ^ 1) == target_color as u8
                    || game.en_passant == Some(offset_by_delta)
                {
                    moves |= offset_by_delta.to_u64();
                }
            }
        }
    }

    Ok(moves)
}
