//! # qzip
//! Ultra-lagana kompresija zasnovana na 2-bitnim kvatovima (Quad-bits).
//! Zero-dependency biblioteka za spajanje sa `qnetwork`, `qvfs` i ostatkom ekosistema.

#[derive(Debug, PartialEq, Eq)]
pub enum QzipError {
    EmptyData,
    CorruptStream,
    InvalidMagic,
}

impl std::fmt::Display for QzipError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QzipError::EmptyData => write!(f, "Input data is empty"),
            QzipError::CorruptStream => write!(f, "Compressed stream is corrupted or invalid"),
            QzipError::InvalidMagic => write!(f, "Magic header mismatch"),
        }
    }
}

impl std::error::Error for QzipError {}

/// Predstavlja jedan 2-bitni kvat (vrednosti 0..=3)
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
            _ => unreachable!(),
        }
    }

    #[inline]
    pub fn as_u8(self) -> u8 {
        self as u8
    }
}

/// "QZIP" magic zaglavlje za validaciju toka (4 bajta)
const MAGIC_HEADER: &[u8; 4] = b"QZIP";

/// Struktura koja pruža kompresiju i dekompresiju
pub struct Qzip;

impl Qzip {
    /// Kompresuje sirove bajtove u `qzip` 2-bitni RLE format.
    ///
    /// Struktura pakovanja po reču (1 bajt):
    /// [2 bita: Vrednost Kvata (0..3)] [6 bita: Dužina ponavljanja (1..64)]
    pub fn compress(data: &[u8]) -> Vec<u8> {
        if data.is_empty() {
            return Vec::new();
        }

        let mut output = Vec::with_capacity(data.len() + 8);
        
        // Zapisivanje Magic Header-a i originalne dužine u bajtovima (u8 za dužinu ili u64 za veće fajlove)
        output.extend_from_slice(MAGIC_HEADER);
        output.extend_from_slice(&(data.len() as u64).to_le_bytes());

        // Dekonstrukcija ulaznih bajtova u mlaz kvatova
        let mut current_quat: Option<Quat> = None;
        let mut run_length: u8 = 0;

        for &byte in data {
            // Razbijanje bajta na 4 kvata od najviših ka najnižim bitovima
            let quats = [
                Quat::from_u8((byte >> 6) & 0b11),
                Quat::from_u8((byte >> 4) & 0b11),
                Quat::from_u8((byte >> 2) & 0b11),
                Quat::from_u8(byte & 0b11),
            ];

            for q in quats {
                match current_quat {
                    Some(active) if active == q && run_length < 64 => {
                        run_length += 1;
                    }
                    Some(active) => {
                        // Upisi kvat i brojač (2 bita + 6 bita)
                        output.push(Self::encode_token(active, run_length));
                        current_quat = Some(q);
                        run_length = 1;
                    }
                    None => {
                        current_quat = Some(q);
                        run_length = 1;
                    }
                }
            }
        }

        // Upis preostalog poslednjeg kvata
        if let Some(active) = current_quat {
            output.push(Self::encode_token(active, run_length));
        }

        output
    }

    /// Dekompresuje `qzip` kompresovani tok nazad u originalne bajtove.
    pub fn decompress(compressed: &[u8]) -> Result<Vec<u8>, QzipError> {
        if compressed.len() < 12 {
            return Err(QzipError::CorruptStream);
        }

        // Provera Magic Header-a
        if &compressed[0..4] != MAGIC_HEADER {
            return Err(QzipError::InvalidMagic);
        }

        // Čitanje originalne dužine
        let orig_len = u64::from_le_bytes(
            compressed[4..12]
                .try_into()
                .map_err(|_| QzipError::CorruptStream)?,
        ) as usize;

        let mut reconstructed_quats = Vec::with_capacity(orig_len * 4);
        let payload = &compressed[12..];

        // Dekodovanje RLE paketa nazad u mlaz kvatova
        for &token in payload {
            let (quat, count) = Self::decode_token(token);
            for _ in 0..count {
                reconstructed_quats.push(quat);
            }
        }

        if reconstructed_quats.len() < orig_len * 4 {
            return Err(QzipError::CorruptStream);
        }

        // Pakovanje 4 kvata nazad u 1 bajt
        let mut decompressed = Vec::with_capacity(orig_len);
        for chunk in reconstructed_quats.chunks_exact(4).take(orig_len) {
            let byte = (chunk[0].as_u8() << 6)
                | (chunk[1].as_u8() << 4)
                | (chunk[2].as_u8() << 2)
                | chunk[3].as_u8();
            decompressed.push(byte);
        }

        Ok(decompressed)
    }

    /// Pakuje Quat (2 bita) i Dužinu ponavljanja (6 bita) u jedan bajt
    #[inline]
    fn encode_token(quat: Quat, run_length: u8) -> u8 {
        debug_assert!(run_length >= 1 && run_length <= 64);
        let len_bits = (run_length - 1) & 0b11_1111; // Storing 1..64 as 0..63
        (quat.as_u8() << 6) | len_bits
    }

    /// Razpakovuje jedan bajt u Quat i Dužinu ponavljanja
    #[inline]
    fn decode_token(token: u8) -> (Quat, u8) {
        let quat = Quat::from_u8((token >> 6) & 0b11);
        let run_length = (token & 0b11_1111) + 1;
        (quat, run_length)
    }
}

#[cfg(test)]
mod tests {
   use super::*;

    #[test]
    fn test_quat_conversion() {
        assert_eq!(Quat::from_u8(0b00), Quat::Q0);
        assert_eq!(Quat::from_u8(0b01), Quat::Q1);
        assert_eq!(Quat::from_u8(0b10), Quat::Q2);
        assert_eq!(Quat::from_u8(0b11), Quat::Q3);
    }

    #[test]
    fn test_roundtrip_basic() {
        let original = b"Hello, World! This is a test for qzip compression library.";
        let compressed = Qzip::compress(original);
        let decompressed = Qzip::decompress(&compressed).expect("Dekompresija ne uspeva");

        assert_eq!(original.to_vec(), decompressed);
    }

    #[test]
    fn test_high_repetitive_data() {
        // Podaci sa visokim nivoom ponavljanja gde kvat-RLE briljira
        let original = vec![0xAA; 1000]; // 0xAA = 10101010 (sekvence kvatova Q2, Q2, Q2, Q2)
        let compressed = Qzip::compress(&original);
        let decompressed = Qzip::decompress(&compressed).unwrap();

        assert_eq!(original, decompressed);
        assert!(compressed.len() < original.len()); // Provera da li je zaista kompresovao
    }

    #[test]
    fn test_corrupt_header() {
        let mut compressed = Qzip::compress(b"Test data");
        compressed[0] = b'X'; // Oštećenje zaglavlja

        assert_eq!(Qzip::decompress(&compressed), Err(QzipError::InvalidMagic));
    }
}