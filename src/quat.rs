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
            // SAFETY: `val & 0b11` masks out all bits except the lowest two,
            // guaranteeing the discriminant is always in `0..=3`, which matches
            // `Quat::Q0` (`0b00`), `Quat::Q1` (`0b01`), `Quat::Q2` (`0b10`),
            // and `Quat::Q3` (`0b11`). Reaching `_` is therefore impossible.
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }

    #[inline]
    pub fn as_u8(self) -> u8 {
        self as u8
    }

    #[inline]
    pub fn pack_four(q0: Quat, q1: Quat, q2: Quat, q3: Quat) -> u8 {
        ((q0.as_u8() & 0b11) << 6)
            | ((q1.as_u8() & 0b11) << 4)
            | ((q2.as_u8() & 0b11) << 2)
            | (q3.as_u8() & 0b11)
    }

    #[inline]
    pub fn unpack_four(byte: u8) -> [Quat; 4] {
        [
            Quat::from_u8(byte >> 6),
            Quat::from_u8((byte >> 4) & 0b11),
            Quat::from_u8((byte >> 2) & 0b11),
            Quat::from_u8(byte & 0b11),
        ]
    }

    pub fn pack_quats_to_bytes(quats: &[Quat]) -> Vec<u8> {
        let mut out = Vec::with_capacity(quats.len().div_ceil(4));
        for chunk in quats.chunks(4) {
            let q0 = chunk.first().copied().unwrap_or(Quat::Q0);
            let q1 = chunk.get(1).copied().unwrap_or(Quat::Q0);
            let q2 = chunk.get(2).copied().unwrap_or(Quat::Q0);
            let q3 = chunk.get(3).copied().unwrap_or(Quat::Q0);
            out.push(Self::pack_four(q0, q1, q2, q3));
        }
        out
    }

    pub fn unpack_bytes_to_quats(bytes: &[u8], quat_len: usize) -> Vec<Quat> {
        let mut out = Vec::with_capacity(quat_len);
        for &b in bytes {
            for q in Self::unpack_four(b) {
                if out.len() == quat_len {
                    return out;
                }
                out.push(q);
            }
        }
        out
    }
    
    /// Describes the 75% memory savings of 4-state quaternary symbols (2 bits per Quat)
    /// compared to 1 byte per symbol (8 bits): 2 / 8 = 0.25 -> 75% size reduction.
    pub const fn quaternary_savings_summary() -> (&'static str, bool) {
        (
            "75% memory savings applies to 2-bit quaternary streams (4 Quats per byte = 25% of 8-bit/symbol size). Lossless for Quat streams; lossy if general u8 bytes are quantized into 4 levels.",
            true,
        )
    }

    /// Quantizes a general byte (0..=255) into one of 4 quaternary levels (0..=3).
    /// Note: This is a lossy reduction from 8 bits to 2 bits (75% size reduction).
    #[inline]
    pub fn quantize_u8_lossy(val: u8) -> Self {
        Self::from_u8(val >> 6)
    }

    /// Expands a quaternary symbol back to a representative 8-bit sample value
    /// (center of each 64-value bucket: 32, 96, 160, 224).
    #[inline]
    pub fn dequantize_to_u8_center(self) -> u8 {
        (self.as_u8() << 6) | 0b0010_0000
    }

    /// Unpacks a slice of packed bytes (4 Quats per byte) into a `Vec<Quat>`
    /// using architecture-aware paths on stable Rust.
    pub fn unpack_quats_arch(packed: &[u8], quat_count: usize) -> Vec<Quat> {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            Self::unpack_quats_x86_family(packed, quat_count)
        }

        #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
        {
            Self::unpack_quats_fallback(packed, quat_count)
        }
    }

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    fn unpack_quats_x86_family(packed: &[u8], quat_count: usize) -> Vec<Quat> {
        let mut out = Vec::with_capacity(quat_count);
        let mut chunks = packed.chunks_exact(8);

        for chunk in &mut chunks {
            let word = u64::from_le_bytes([
                chunk[0], chunk[1], chunk[2], chunk[3],
                chunk[4], chunk[5], chunk[6], chunk[7],
            ]);
            for byte_idx in 0..8 {
                let b = ((word >> (byte_idx * 8)) & 0xFF) as u8;
                for q in Self::unpack_four(b) {
                    if out.len() == quat_count {
                        return out;
                    }
                    out.push(q);
                }
            }
        }

        for &b in chunks.remainder() {
            for q in Self::unpack_four(b) {
                if out.len() == quat_count {
                    return out;
                }
                out.push(q);
            }
        }

        out
    }

    fn unpack_quats_fallback(packed: &[u8], quat_count: usize) -> Vec<Quat> {
        let mut out = Vec::with_capacity(quat_count);
        for &b in packed {
            for q in Self::unpack_four(b) {
                if out.len() == quat_count {
                    return out;
                }
                out.push(q);
            }
        }
        out
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