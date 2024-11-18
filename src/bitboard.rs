use crate::Square;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Bitboard(pub u64);

impl std::ops::BitAnd<Bitboard> for Bitboard {
    type Output = Self;
    fn bitand(self, rhs: Bitboard) -> Self::Output {
        Self::from_u64(self.0 & rhs.0)
    }
}
impl std::ops::BitAnd<u64> for Bitboard {
    type Output = Self;
    fn bitand(self, rhs: u64) -> Self::Output {
        Self::from_u64(self.0 & rhs)
    }
}
impl std::ops::BitAnd<Square> for Bitboard {
    type Output = Self;
    fn bitand(self, rhs: Square) -> Self::Output {
        Self::from_u64(self.0 & rhs.to_u64())
    }
}
impl std::ops::BitOr<Bitboard> for Bitboard {
    type Output = Self;

    fn bitor(self, rhs: Bitboard) -> Self::Output {
        Self::from_u64(self.0 | rhs.0)
    }
}
impl std::ops::BitOr<u64> for Bitboard {
    type Output = Self;

    fn bitor(self, rhs: u64) -> Self::Output {
        Self::from_u64(self.0 | rhs)
    }
}
impl std::ops::BitOr<Square> for Bitboard {
    type Output = Self;
    fn bitor(self, rhs: Square) -> Self::Output {
        Self::from_u64(self.0 | rhs.to_u64())
    }
}
impl std::ops::BitXor<Bitboard> for Bitboard {
    type Output = Self;

    fn bitxor(self, rhs: Bitboard) -> Self::Output {
        Self::from_u64(self.0 ^ rhs.0)
    }
}
impl std::ops::BitXor<u64> for Bitboard {
    type Output = Self;

    fn bitxor(self, rhs: u64) -> Self::Output {
        Self::from_u64(self.0 ^ rhs)
    }
}
impl std::ops::BitXor<Square> for Bitboard {
    type Output = Self;
    fn bitxor(self, rhs: Square) -> Self::Output {
        Self::from_u64(self.0 ^ rhs.to_u64())
    }
}
impl std::ops::BitAndAssign<Bitboard> for Bitboard {
    fn bitand_assign(&mut self, rhs: Bitboard) {
        self.0 &= rhs.0;
    }
}
impl std::ops::BitAndAssign<u64> for Bitboard {
    fn bitand_assign(&mut self, rhs: u64) {
        self.0 &= rhs;
    }
}
impl std::ops::BitAndAssign<Square> for Bitboard {
    fn bitand_assign(&mut self, rhs: Square) {
        self.0 &= rhs.to_u64();
    }
}
impl std::ops::BitOrAssign<Bitboard> for Bitboard {
    fn bitor_assign(&mut self, rhs: Bitboard) {
        self.0 |= rhs.0;
    }
}
impl std::ops::BitOrAssign<u64> for Bitboard {
    fn bitor_assign(&mut self, rhs: u64) {
        self.0 |= rhs;
    }
}
impl std::ops::BitOrAssign<Square> for Bitboard {
    fn bitor_assign(&mut self, rhs: Square) {
        self.0 |= rhs.to_u64();
    }
}
impl std::ops::BitXorAssign<Bitboard> for Bitboard {
    fn bitxor_assign(&mut self, rhs: Bitboard) {
        self.0 ^= rhs.0;
    }
}
impl std::ops::BitXorAssign<u64> for Bitboard {
    fn bitxor_assign(&mut self, rhs: u64) {
        self.0 ^= rhs;
    }
}
impl std::ops::BitXorAssign<Square> for Bitboard {
    fn bitxor_assign(&mut self, rhs: Square) {
        self.0 ^= rhs.to_u64();
    }
}
impl std::ops::Not for Bitboard {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}

impl Bitboard {
    pub fn empty() -> Self {
        Bitboard::from_u64(0)
    }

    pub fn from_square(s: Square) -> Self {
        Bitboard::from_u64(0) | s.to_u64()
    }

    pub fn from_squares(sq: Vec<Square>) -> Self {
        let mut out = Bitboard::empty();
        for s in sq {
            out |= s.to_u64();
        }
        out
    }

    pub fn from_u64(v: u64) -> Self {
        Self(v)
    }

    pub fn contains(&self, s: Square) -> bool {
        self.0 & 1 << s as u64 != 0
    }

    pub fn is_empty(&self) -> bool {
        self.0 == 0
    }

    pub fn count_ones(&self) -> u32 {
        self.0.count_ones()
    }

    pub fn trailing_zeros(&self) -> u32 {
        self.0.trailing_zeros()
    }

    pub fn clear_lsb(&mut self) {
        self.0 &= self.0 - 1;
    }
}
