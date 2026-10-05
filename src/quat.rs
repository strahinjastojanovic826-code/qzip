/// Predstavlja jedan 2-bitni kvat (vrijednosti 0..=3).
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Quat {
    Q0 = 0b00,
    Q1 = 0b01,
    Q2 = 0b10,
    Q3 = 0b11,
}

impl Quat {
    #[inline]
    pub fn from_u8(val: u8) -> Self {
        match val & 0b11 {
            0b00 => Quat::Q0,
            0b01 => Quat::Q1,
            0b10 => Quat::Q2,
            0b11 => Quat::Q3,
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }

    #[inline]
    pub fn as_u8(self) -> u8 {
        self as u8
    }
}

impl From<u8> for Quat {
    #[inline]
    fn from(val: u8) -> Self {
        Quat::from_u8(val)
    }
}

impl From<Quat> for u8 {
    #[inline]
    fn from(q: Quat) -> Self {
        q.as_u8()
    }
}