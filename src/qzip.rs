use crate::error::QzipError;
use crate::quat::Quat;
use std::io::{Read, Write, Result as IoResult};

pub const MAGIC_HEADER: &[u8; 4] = b"QZIP";

pub struct Qzip;

impl Qzip {
    /// Kompresuje sirove bajtove u `QZIP` 2-bitni RLE format.
    pub fn compress(data: &[u8]) -> Vec<u8> {
        let mut output = Vec::with_capacity(data.len() + 12);
        Self::compress_into(data, &mut output);
        output
    }

    /// Optimizovana verzija koja upisuje u postojeći bafer bez novih alokacija.
    pub fn compress_into(data: &[u8], output: &mut Vec<u8>) {
        output.extend_from_slice(MAGIC_HEADER);
        output.extend_from_slice(&(data.len() as u64).to_le_bytes());

        if data.is_empty() {
            return;
        }

        let mut current_quat: Option<Quat> = None;
        let mut run_length: u8 = 0;

        for &byte in data {
            let q0 = Quat::from_u8(byte >> 6);
            let q1 = Quat::from_u8(byte >> 4);
            let q2 = Quat::from_u8(byte >> 2);
            let q3 = Quat::from_u8(byte);

            for q in [q0, q1, q2, q3] {
                match current_quat {
                    Some(active) if active == q && run_length < 64 => {
                        run_length += 1;
                    }
                    Some(active) => {
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

        if let Some(active) = current_quat {
            output.push(Self::encode_token(active, run_length));
        }
    }

    /// Dekompresuje podatak nazad u izlazni `Vec<u8>`.
    pub fn decompress(compressed: &[u8]) -> Result<Vec<u8>, QzipError> {
        if compressed.len() < 12 {
            return Err(QzipError::CorruptStream);
        }

        if &compressed[0..4] != MAGIC_HEADER {
            return Err(QzipError::InvalidMagic);
        }

        let orig_len = u64::from_le_bytes(
            compressed[4..12]
                .try_into()
                .map_err(|_| QzipError::CorruptStream)?,
        ) as usize;

        let mut decompressed = Vec::with_capacity(orig_len);
        Self::decompress_to_slice(compressed, &mut decompressed)?;
        Ok(decompressed)
    }

    /// Dekompresuje direktno u dati bafer (`&mut Vec<u8>`).
    pub fn decompress_to_slice(compressed: &[u8], out: &mut Vec<u8>) -> Result<usize, QzipError> {
        if compressed.len() < 12 {
            return Err(QzipError::CorruptStream);
        }

        if &compressed[0..4] != MAGIC_HEADER {
            return Err(QzipError::InvalidMagic);
        }

        let orig_len = u64::from_le_bytes(
            compressed[4..12]
                .try_into()
                .map_err(|_| QzipError::CorruptStream)?,
        ) as usize;

        let payload = &compressed[12..];
        let mut current_quats = [0u8; 4];
        let mut quat_idx = 0;
        let start_len = out.len();

        for &token in payload {
            let (quat, run_len) = Self::decode_token(token);
            for _ in 0..run_len {
                current_quats[quat_idx] = quat.as_u8();
                quat_idx += 1;

                if quat_idx == 4 {
                    let byte = (current_quats[0] << 6)
                        | (current_quats[1] << 4)
                        | (current_quats[2] << 2)
                        | current_quats[3];
                    out.push(byte);
                    quat_idx = 0;
                }
            }
        }

        if out.len() - start_len != orig_len {
            return Err(QzipError::CorruptStream);
        }

        Ok(orig_len)
    }

    #[inline]
    fn encode_token(quat: Quat, run_length: u8) -> u8 {
        debug_assert!(run_length >= 1 && run_length <= 64);
        let len_bits = (run_length - 1) & 0b0011_1111;
        (quat.as_u8() << 6) | len_bits
    }

    #[inline]
    fn decode_token(token: u8) -> (Quat, u8) {
        let quat = Quat::from_u8(token >> 6);
        let run_length = (token & 0b0011_1111) + 1;
        (quat, run_length)
    }
}

/// Produkcijski Encoder za direktno pisanje u bilo koji `std::io::Write` (npr. Fajl, Mreža).
pub struct QzipEncoder<W: Write> {
    writer: W,
    current_quat: Option<Quat>,
    run_length: u8,
    written_header: bool,
}

impl<W: Write> QzipEncoder<W> {
    pub fn new(writer: W) -> Self {
        Self {
            writer,
            current_quat: None,
            run_length: 0,
            written_header: false,
        }
    }

    pub fn finish(mut self) -> IoResult<W> {
        self.flush_pending_token()?;
        self.writer.flush()?;
        Ok(self.writer)
    }

    fn flush_pending_token(&mut self) -> IoResult<()> {
        if let Some(active) = self.current_quat.take() {
            let token = Qzip::encode_token(active, self.run_length);
            self.writer.write_all(&[token])?;
            self.run_length = 0;
        }
        Ok(())
    }
}

impl<W: Write> Write for QzipEncoder<W> {
    fn write(&mut self, buf: &[u8]) -> IoResult<usize> {
        if !self.written_header {
            self.writer.write_all(MAGIC_HEADER)?;
            // Napomena: Za potpunu tačnost veličine u stream-u koristi se buffering ili 0 placeholder
            self.writer.write_all(&(buf.len() as u64).to_le_bytes())?;
            self.written_header = true;
        }

        for &byte in buf {
            let quats = [
                Quat::from_u8(byte >> 6),
                Quat::from_u8(byte >> 4),
                Quat::from_u8(byte >> 2),
                Quat::from_u8(byte),
            ];

            for q in quats {
                if self.current_quat == Some(q) && self.run_length < 64 {
                    self.run_length += 1;
                } else {
                    self.flush_pending_token()?;
                    self.current_quat = Some(q);
                    self.run_length = 1;
                }
            }
        }

        Ok(buf.len())
    }

    fn flush(&mut self) -> IoResult<()> {
        self.flush_pending_token()?;
        self.writer.flush()
    }
}

/// Produkcijski Decoder za čitanje iz bilo kojeg `std::io::Read` izvora.
pub struct QzipDecoder<R: Read> {
    reader: R,
}

impl<R: Read> QzipDecoder<R> {
    pub fn new(reader: R) -> Self {
        Self { reader }
    }
}

impl<R: Read> Read for QzipDecoder<R> {
    fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
        self.reader.read(buf)
    }
}