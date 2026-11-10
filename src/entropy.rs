use crate::quat::Quat;
use std::collections::HashMap;

#[derive(Debug)]
pub struct QuatEntropyCoder {
    frequencies: [u64; 4],
}

impl QuatEntropyCoder {
    pub fn new() -> Self {
        Self { frequencies: [1, 1, 1, 1] } // Uniform initial weights
    }

    /// Analyzes quaternary frequency distribution in stream.
    pub fn observe_stream(&mut self, quats: &[Quat]) {
        for &q in quats {
            let idx = q.as_u8() as usize;
            self.frequencies[idx] += 1;
        }
    }

    /// Generates variable-length bit codes based on Quat probabilities.
    pub fn build_bit_codes(&self) -> HashMap<Quat, (u8, u8)> {
        let mut pairs: Vec<(Quat, u64)> = vec![
            (Quat::Q0, self.frequencies[0]),
            (Quat::Q1, self.frequencies[1]),
            (Quat::Q2, self.frequencies[2]),
            (Quat::Q3, self.frequencies[3]),
        ];

        // Sort ascending to give most frequent symbols shortest codes
        pairs.sort_by_key(|&(_, freq)| freq);

        let mut map = HashMap::new();
        // Generate prefix codes (Code value, Bit length)
        map.insert(pairs[3].0, (0b0, 1));       // Most frequent: 1 bit
        map.insert(pairs[2].0, (0b10, 2));      // 2 bits
        map.insert(pairs[1].0, (0b110, 3));     // 3 bits
        map.insert(pairs[0].0, (0b111, 3));     // Least frequent: 3 bits

        map
    }
}