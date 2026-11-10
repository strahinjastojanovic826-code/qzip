use crate::quat::Quat;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LzQuatToken {
    Literal(Quat),
    Match { distance: u16, length: u8 },
}

pub struct QuatLz77 {
    window_size: usize,
}

impl QuatLz77 {
    pub fn new(window_size: usize) -> Self {
        Self { window_size }
    }

    /// Compresses a stream of Quats using hybrid LZ77 sliding window.
    pub fn compress(&self, quats: &[Quat]) -> Vec<LzQuatToken> {
        let mut tokens = Vec::new();
        let mut cursor = 0;

        while cursor < quats.len() {
            let window_start = cursor.saturating_sub(self.window_size);
            let window = &quats[window_start..cursor];

            let mut best_distance = 0;
            let mut best_length = 0;

            // Search sliding window for matching sequence of Quats
            for dist in 1..=window.len() {
                let match_start = cursor - dist;
                let mut len = 0;

                while cursor + len < quats.len() 
                    && quats[match_start + (len % dist)] == quats[cursor + len] 
                    && len < 255 
                {
                    len += 1;
                }

                if len > best_length && len >= 3 {
                    best_length = len;
                    best_distance = dist;
                }
            }

            if best_length >= 3 {
                tokens.push(LzQuatToken::Match {
                    distance: best_distance as u16,
                    length: best_length as u8,
                });
                cursor += best_length;
            } else {
                tokens.push(LzQuatToken::Literal(quats[cursor]));
                cursor += 1;
            }
        }

        tokens
    }
}