use crate::quat::Quat;
use std::collections::HashMap;

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LongLzQuatMatch {
    pub distance: u32,
    pub length: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExtendedLzQuatToken {
    Literal(Quat),
    Match(LongLzQuatMatch),
}

pub struct QuatLz77Fast {
    pub window_size: usize,
    pub min_match: usize,
    pub max_match: usize,
    pub max_candidates_per_pos: usize,
}

impl QuatLz77Fast {
    pub fn new(window_size: usize) -> Self {
        Self {
            window_size: window_size.max(4),
            min_match: 3,
            max_match: u16::MAX as usize,
            max_candidates_per_pos: 32,
        }
    }

    #[inline]
    fn trigram_key(quats: &[Quat], pos: usize) -> Option<u8> {
        if pos + 2 >= quats.len() {
            return None;
        }
        let q0 = quats[pos].as_u8() & 0b11;
        let q1 = quats[pos + 1].as_u8() & 0b11;
        let q2 = quats[pos + 2].as_u8() & 0b11;
        Some((q0 << 4) | (q1 << 2) | q2)
    }

    pub fn compress_fast(&self, quats: &[Quat]) -> Vec<ExtendedLzQuatToken> {
        let mut tokens = Vec::new();
        let mut history: HashMap<u8, Vec<usize>> = HashMap::new();
        let mut cursor = 0;

        while cursor < quats.len() {
            let window_start = cursor.saturating_sub(self.window_size);
            let mut best_len = 0usize;
            let mut best_dist = 0usize;

            if let Some(key) = Self::trigram_key(quats, cursor) {
                if let Some(positions) = history.get(&key) {
                    let mut checked = 0usize;
                    for &candidate_pos in positions.iter().rev() {
                        if candidate_pos < window_start {
                            break;
                        }
                        if candidate_pos >= cursor {
                            continue;
                        }

                        let dist = cursor - candidate_pos;
                        if dist == 0 || dist > self.window_size {
                            continue;
                        }

                        let max_possible = (quats.len() - cursor).min(self.max_match);
                        let mut match_len = 0usize;
                        while match_len < max_possible {
                            let src_idx = candidate_pos + (match_len % dist);
                            if src_idx >= cursor || quats[src_idx] != quats[cursor + match_len] {
                                break;
                            }
                            match_len += 1;
                        }

                        if match_len >= self.min_match && match_len > best_len {
                            best_len = match_len;
                            best_dist = dist;
                            if best_len == max_possible {
                                break;
                            }
                        }

                        checked += 1;
                        if checked >= self.max_candidates_per_pos {
                            break;
                        }
                    }
                }
            }

            if best_len >= self.min_match {
                tokens.push(ExtendedLzQuatToken::Match(LongLzQuatMatch {
                    distance: best_dist as u32,
                    length: best_len as u16,
                }));

                for pos in cursor..(cursor + best_len) {
                    if let Some(k) = Self::trigram_key(quats, pos) {
                        history.entry(k).or_default().push(pos);
                    }
                }
                cursor += best_len;
            } else {
                tokens.push(ExtendedLzQuatToken::Literal(quats[cursor]));
                if let Some(k) = Self::trigram_key(quats, cursor) {
                    history.entry(k).or_default().push(cursor);
                }
                cursor += 1;
            }
        }

        tokens
    }

    pub fn decompress_fast(tokens: &[ExtendedLzQuatToken]) -> Result<Vec<Quat>, &'static str> {
        let mut out = Vec::new();
        for tok in tokens {
            match tok {
                ExtendedLzQuatToken::Literal(q) => out.push(*q),
                ExtendedLzQuatToken::Match(m) => {
                    let dist = m.distance as usize;
                    let len = m.length as usize;
                    if dist == 0 || dist > out.len() {
                        return Err("Invalid LZ77 match distance");
                    }
                    let start = out.len() - dist;
                    for i in 0..len {
                        let q = out[start + (i % dist)];
                        out.push(q);
                    }
                }
            }
        }
        Ok(out)
    }
}