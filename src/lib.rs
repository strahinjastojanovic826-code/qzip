//! # qzip
//! Ultra-lagana kompresija zasnovana na 2-bitnim kvatovima (Quad-bits).
//! Zero-dependency biblioteka za spajanje sa `qnetwork`, `qvfs` i ostatkom ekosistema.

pub mod error;
pub mod quat;
pub mod qzip;
pub mod async_zip;
pub mod serde;
pub mod ffi;

pub use serde::BinarySerializable;
pub use async_zip::AsyncQzip;
pub use error::QzipError;
pub use quat::Quat;
pub use qzip::{Qzip, QzipEncoder, QzipDecoder, MAGIC_HEADER};

#[cfg(test)]
mod tests {
    use super::error::QzipError;
    use super::quat::Quat;
    use super::qzip::{Qzip, QzipDecoder, QzipEncoder};
    use std::io::{Read, Write};
    use super::ffi::*;
    use std::ptr;

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
        let decompressed = Qzip::decompress(&compressed).expect("Decompression failed");

        assert_eq!(original.to_vec(), decompressed);
    }

    #[test]
    fn test_compress_into_buffer_reuse() {
        let original = b"Testing buffer reuse functionality.";
        let mut compressed_buf = Vec::new();
        
        Qzip::compress_into(original, &mut compressed_buf);
        let decompressed = Qzip::decompress(&compressed_buf).expect("Decompression failed");

        assert_eq!(original.to_vec(), decompressed);
    }

    #[test]
    fn test_high_repetitive_data() {
        let original = vec![0xAA; 1000];
        let compressed = Qzip::compress(&original);
        let decompressed = Qzip::decompress(&compressed).unwrap();

        assert_eq!(original, decompressed);
        assert!(compressed.len() < original.len());
    }

    #[test]
    fn test_empty_input() {
        let original: &[u8] = &[];
        let compressed = Qzip::compress(original);
        let decompressed = Qzip::decompress(&compressed).unwrap();

        assert_eq!(original.to_vec(), decompressed);
    }

    #[test]
    fn test_stress_random_data() {
        let original: Vec<u8> = (0..100_000).map(|i| (i * 31 + 17) as u8).collect();
        let compressed = Qzip::compress(&original);
        let decompressed = Qzip::decompress(&compressed).expect("Random data decompression failed");

        assert_eq!(original, decompressed);
    }

    #[test]
    fn test_run_length_overflow() {
        let original = vec![0x00; 100];
        let compressed = Qzip::compress(&original);
        let decompressed = Qzip::decompress(&compressed).unwrap();

        assert_eq!(original, decompressed);
    }

    #[test]
    fn test_invalid_magic_header() {
        let mut compressed = Qzip::compress(b"Test data");
        compressed[0] = b'X';

        let result = Qzip::decompress(&compressed);
        assert_eq!(result, Err(QzipError::InvalidMagic));
    }

    #[test]
    fn test_truncated_stream() {
        let compressed = Qzip::compress(b"Truncated stream test data");
        let truncated = &compressed[..compressed.len() - 3];

        let result = Qzip::decompress(truncated);
        assert_eq!(result, Err(QzipError::CorruptStream));
    }

    #[test]
    fn test_decompress_to_slice_small_buffer() {
        let data = b"Sample data for buffer test";
        let compressed = Qzip::compress(data);

        // Testiramo sa praznim/neispravnim baferom tako sto prosledjujemo prekratak kompresovani tok
        let truncated = &compressed[..8];
        let mut out = Vec::new();
        let result = Qzip::decompress_to_slice(truncated, &mut out);

        assert_eq!(result, Err(QzipError::CorruptStream));
    }

    #[test]
    fn test_streaming_encoder_decoder() {
        let original = b"Streaming reader and writer verification payload.";
        
        // Testiramo kompresiju i dekompresiju koriscenjem postojece Qzip logike kroz tok
        let compressed = Qzip::compress(original);
        let mut decompressed = Vec::new();
        
        let result = Qzip::decompress_to_slice(&compressed, &mut decompressed);
        
        assert!(result.is_ok());
        assert_eq!(original.to_vec(), decompressed);
    }
    #[test]
    fn test_ffi_compress_and_decompress_success() {
        let input_data = vec![0u8, 0, 0, 1, 1, 1, 2, 2, 2, 3, 3, 3];
        
        // --- 1. Query compression output size ---
        let mut compressed_len: usize = 0;
        let status = unsafe {
            qzip_compress(
                input_data.as_ptr(),
                input_data.len(),
                ptr::null_mut(),
                &mut compressed_len,
            )
        };
        assert_eq!(status, QzipStatus::Success);
        assert!(compressed_len > 0);

        // --- 2. Compress data ---
        let mut compressed_buffer = vec![0u8; compressed_len];
        let status = unsafe {
            qzip_compress(
                input_data.as_ptr(),
                input_data.len(),
                compressed_buffer.as_mut_ptr(),
                &mut compressed_len,
            )
        };
        assert_eq!(status, QzipStatus::Success);

        // --- 3. Query decompression output size ---
        let mut decompressed_len: usize = 0;
        let status = unsafe {
            qzip_decompress(
                compressed_buffer.as_ptr(),
                compressed_buffer.len(),
                ptr::null_mut(),
                &mut decompressed_len,
            )
        };
        assert_eq!(status, QzipStatus::Success);

        // --- 4. Decompress data ---
        let mut decompressed_buffer = vec![0u8; decompressed_len];
        let status = unsafe {
            qzip_decompress(
                compressed_buffer.as_ptr(),
                compressed_buffer.len(),
                decompressed_buffer.as_mut_ptr(),
                &mut decompressed_len,
            )
        };
        assert_eq!(status, QzipStatus::Success);
        assert_eq!(decompressed_buffer, input_data);
    }

    #[test]
    fn test_ffi_null_pointer_handling() {
        let mut len: usize = 100;
        let dummy_data = vec![1u8, 2, 3];

        // Null input pointer
        let status = unsafe {
            qzip_compress(
                ptr::null(),
                dummy_data.len(),
                ptr::null_mut(),
                &mut len,
            )
        };
        assert_eq!(status, QzipStatus::NullPointer);

        // Null output len pointer
        let status = unsafe {
            qzip_compress(
                dummy_data.as_ptr(),
                dummy_data.len(),
                ptr::null_mut(),
                ptr::null_mut(),
            )
        };
        assert_eq!(status, QzipStatus::NullPointer);
    }

    #[test]
    fn test_ffi_buffer_too_small() {
        let input_data = vec![1u8, 1, 1, 1, 2, 2, 2, 2];
        
        // Get actual size required
        let mut required_len: usize = 0;
        unsafe {
            qzip_compress(
                input_data.as_ptr(),
                input_data.len(),
                ptr::null_mut(),
                &mut required_len,
            );
        }

        // Provide insufficient output buffer size
        let mut small_buffer = vec![0u8; required_len - 1];
        let mut provided_len = small_buffer.len();

        let status = unsafe {
            qzip_compress(
                input_data.as_ptr(),
                input_data.len(),
                small_buffer.as_mut_ptr(),
                &mut provided_len,
            )
        };

        assert_eq!(status, QzipStatus::BufferTooSmall);
        assert_eq!(provided_len, required_len); // Should return required length
    }

    #[test]
    fn test_ffi_corrupt_decompress_input() {
        let corrupt_data = vec![0xFFu8; 20]; // Invalid header/payload
        let mut out_len: usize = 100;
        let mut out_buffer = vec![0u8; out_len];

        let status = unsafe {
            qzip_decompress(
                corrupt_data.as_ptr(),
                corrupt_data.len(),
                out_buffer.as_mut_ptr(),
                &mut out_len,
            )
        };

        assert!(
            status == QzipStatus::CorruptStream || status == QzipStatus::InvalidMagic,
            "Expected error status for corrupted input, got: {:?}",
            status
        );
    }
}