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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HuffmanEncodedQuats {
    pub symbol_count: u32,
    pub frequencies: [u64; 4],
    pub bit_len: usize,
    pub bitstream: Vec<u8>,
}

#[derive(Debug, Clone)]
enum HuffNode {
    Leaf { quat: Quat, freq: u64 },
    Internal { freq: u64, left: usize, right: usize },
}

pub struct QuatHuffmanCodec;

impl QuatHuffmanCodec {
    fn build_tree(frequencies: &[u64; 4]) -> (Vec<HuffNode>, usize) {
        let mut nodes: Vec<HuffNode> = Vec::new();
        let mut active: Vec<usize> = Vec::new();

        for (idx, &freq) in frequencies.iter().enumerate() {
            if freq > 0 {
                let node_idx = nodes.len();
                nodes.push(HuffNode::Leaf {
                    quat: Quat::from_u8(idx as u8),
                    freq,
                });
                active.push(node_idx);
            }
        }

        if active.is_empty() {
            nodes.push(HuffNode::Leaf {
                quat: Quat::Q0,
                freq: 0,
            });
            return (nodes, 0);
        }

        if active.len() == 1 {
            let leaf_idx = active[0];
            let freq = frequencies[leaf_idx];
            let root_idx = nodes.len();
            nodes.push(HuffNode::Internal {
                freq,
                left: leaf_idx,
                right: leaf_idx,
            });
            return (nodes, root_idx);
        }

        while active.len() > 1 {
            active.sort_by(|&a, &b| {
                let fa = match nodes[a] {
                    HuffNode::Leaf { freq, .. } | HuffNode::Internal { freq, .. } => freq,
                };
                let fb = match nodes[b] {
                    HuffNode::Leaf { freq, .. } | HuffNode::Internal { freq, .. } => freq,
                };
                fa.cmp(&fb).then_with(|| a.cmp(&b))
            });

            let left = active.remove(0);
            let right = active.remove(0);
            let f_left = match nodes[left] {
                HuffNode::Leaf { freq, .. } | HuffNode::Internal { freq, .. } => freq,
            };
            let f_right = match nodes[right] {
                HuffNode::Leaf { freq, .. } | HuffNode::Internal { freq, .. } => freq,
            };

            let parent_idx = nodes.len();
            nodes.push(HuffNode::Internal {
                freq: f_left.saturating_add(f_right),
                left,
                right,
            });
            active.push(parent_idx);
        }

        let root = active[0];
        (nodes, root)
    }

    fn collect_codes(
        nodes: &[HuffNode],
        node_idx: usize,
        bits: u32,
        depth: u8,
        table: &mut [Option<(u32, u8)>; 4],
    ) {
        match nodes[node_idx] {
            HuffNode::Leaf { quat, .. } => {
                let effective_depth = if depth == 0 { 1 } else { depth };
                table[quat.as_u8() as usize] = Some((bits, effective_depth));
            }
            HuffNode::Internal { left, right, .. } => {
                Self::collect_codes(nodes, left, bits << 1, depth + 1, table);
                Self::collect_codes(nodes, right, (bits << 1) | 1, depth + 1, table);
            }
        }
    }

    pub fn encode(quats: &[Quat]) -> HuffmanEncodedQuats {
        let mut frequencies = [0u64; 4];
        for &q in quats {
            frequencies[q.as_u8() as usize] += 1;
        }

        let (nodes, root) = Self::build_tree(&frequencies);
        let mut table: [Option<(u32, u8)>; 4] = [None; 4];
        Self::collect_codes(&nodes, root, 0, 0, &mut table);

        let mut bitstream: Vec<u8> = Vec::new();
        let mut current_byte: u8 = 0;
        let mut bit_pos: u8 = 0;
        let mut total_bits: usize = 0;

        for &q in quats {
            if let Some((code, len)) = table[q.as_u8() as usize] {
                for bit_offset in (0..len).rev() {
                    let bit = ((code >> bit_offset) & 1) as u8;
                    current_byte = (current_byte << 1) | bit;
                    bit_pos += 1;
                    total_bits += 1;

                    if bit_pos == 8 {
                        bitstream.push(current_byte);
                        current_byte = 0;
                        bit_pos = 0;
                    }
                }
            }
        }

        if bit_pos > 0 {
            current_byte <<= 8 - bit_pos;
            bitstream.push(current_byte);
        }

        HuffmanEncodedQuats {
            symbol_count: quats.len() as u32,
            frequencies,
            bit_len: total_bits,
            bitstream,
        }
    }

    pub fn decode(encoded: &HuffmanEncodedQuats) -> Result<Vec<Quat>, &'static str> {
        let total_symbols = encoded.symbol_count as usize;
        if total_symbols == 0 {
            return Ok(Vec::new());
        }

        let (nodes, root) = Self::build_tree(&encoded.frequencies);
        let mut out = Vec::with_capacity(total_symbols);
        let mut node_idx = root;
        let mut bits_read: usize = 0;

        for &byte in &encoded.bitstream {
            for bit_offset in (0..8).rev() {
                if bits_read >= encoded.bit_len || out.len() == total_symbols {
                    break;
                }
                let bit = (byte >> bit_offset) & 1;
                bits_read += 1;

                match nodes[node_idx] {
                    HuffNode::Leaf { quat, .. } => {
                        out.push(quat);
                        node_idx = root;
                    }
                    HuffNode::Internal { left, right, .. } => {
                        let next_idx = if bit == 0 { left } else { right };
                        match nodes[next_idx] {
                            HuffNode::Leaf { quat, .. } => {
                                out.push(quat);
                                node_idx = root;
                            }
                            HuffNode::Internal { .. } => {
                                node_idx = next_idx;
                            }
                        }
                    }
                }
            }
        }

        if out.len() != total_symbols {
            return Err("Incomplete Huffman bitstream for declared symbol count");
        }

        Ok(out)
    }
}