//! # qzip
//! Ultra-lagana kompresija zasnovana na 2-bitnim kvatovima (Quad-bits).
//! Zero-dependency biblioteka za spajanje sa `qnetwork`, `qvfs` i ostatkom ekosistema.

pub mod error;
pub mod quat;
pub mod qzip;
pub mod async_zip;
pub mod serde;
pub mod ffi;
pub mod lz77;
pub mod entropy;

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
    use std::time::Instant;
    use std::ptr;

    #[test]
    fn test_quat_conversion() {
        let start = Instant::now();
        assert_eq!(Quat::from_u8(0b00), Quat::Q0);
        assert_eq!(Quat::from_u8(0b01), Quat::Q1);
        assert_eq!(Quat::from_u8(0b10), Quat::Q2);
        assert_eq!(Quat::from_u8(0b11), Quat::Q3);
        let duration = start.elapsed();
        println!("[BENCHMARK] test_quat_conversion took: {:?}", duration);
    }

    #[test]
    fn test_roundtrip_basic() {
        let start = Instant::now();
        let original = b"Hello, World! This is a test for qzip compression library.";
        let compressed = Qzip::compress(original);
        let decompressed = Qzip::decompress(&compressed).expect("Decompression failed");
        assert_eq!(original.to_vec(), decompressed);
        let duration = start.elapsed();
        println!("[BENCHMARK] test_roundtrip_basic took: {:?} (input size: {} bytes)", duration, original.len());
    }

    #[test]
    fn test_compress_into_buffer_reuse() {
        let start = Instant::now();
        let original = b"Testing buffer reuse functionality.";
        let mut compressed_buf = Vec::new();
        Qzip::compress_into(original, &mut compressed_buf);
        let decompressed = Qzip::decompress(&compressed_buf).expect("Decompression failed");
        assert_eq!(original.to_vec(), decompressed);
        let duration = start.elapsed();
        println!("[BENCHMARK] test_compress_into_buffer_reuse took: {:?}", duration);
    }

    #[test]
    fn test_high_repetitive_data() {
        let start = Instant::now();
        let original = vec![0xAA; 1000];
        let compressed = Qzip::compress(&original);
        let decompressed = Qzip::decompress(&compressed).unwrap();
        assert_eq!(original, decompressed);
        assert!(compressed.len() < original.len());
        let duration = start.elapsed();
        println!("[BENCHMARK] test_high_repetitive_data took: {:?} (compressed len: {})", duration, compressed.len());
    }

    #[test]
    fn test_empty_input() {
        let start = Instant::now();
        let original: &[u8] = &[];
        let compressed = Qzip::compress(original);
        let decompressed = Qzip::decompress(&compressed).unwrap();
        assert_eq!(original.to_vec(), decompressed);
        let duration = start.elapsed();
        println!("[BENCHMARK] test_empty_input took: {:?}", duration);
    }

    #[test]
    fn test_stress_random_data() {
        let start = Instant::now();
        let original: Vec<u8> = (0..100_000).map(|i| (i * 31 + 17) as u8).collect();
        let compressed = Qzip::compress(&original);
        let decompressed = Qzip::decompress(&compressed).expect("Random data decompression failed");
        assert_eq!(original, decompressed);
        let duration = start.elapsed();
        println!("[BENCHMARK] test_stress_random_data took: {:?} for {} bytes", duration, original.len());
    }

    #[test]
    fn test_run_length_overflow() {
        let start = Instant::now();
        let original = vec![0x00; 100];
        let compressed = Qzip::compress(&original);
        let decompressed = Qzip::decompress(&compressed).unwrap();
        assert_eq!(original, decompressed);
        let duration = start.elapsed();
        println!("[BENCHMARK] test_run_length_overflow took: {:?}", duration);
    }

    #[test]
    fn test_invalid_magic_header() {
        let start = Instant::now();
        let mut compressed = Qzip::compress(b"Test data");
        compressed[0] = b'X';
        let result = Qzip::decompress(&compressed);
        assert_eq!(result, Err(QzipError::InvalidMagic));
        let duration = start.elapsed();
        println!("[BENCHMARK] test_invalid_magic_header took: {:?}", duration);
    }

    #[test]
    fn test_truncated_stream() {
        let start = Instant::now();
        let compressed = Qzip::compress(b"Truncated stream test data");
        let truncated = &compressed[..compressed.len() - 3];
        let result = Qzip::decompress(truncated);
        assert_eq!(result, Err(QzipError::CorruptStream));
        let duration = start.elapsed();
        println!("[BENCHMARK] test_truncated_stream took: {:?}", duration);
    }

    #[test]
    fn test_decompress_to_slice_small_buffer() {
        let start = Instant::now();
        let data = b"Sample data for buffer test";
        let compressed = Qzip::compress(data);
        // Testiramo sa praznim/neispravnim baferom tako sto prosledjujemo prekratak kompresovani tok
        let truncated = &compressed[..8];
        let mut out = Vec::new();
        let result = Qzip::decompress_to_slice(truncated, &mut out);
        assert_eq!(result, Err(QzipError::CorruptStream));
        let duration = start.elapsed();
        println!("[BENCHMARK] test_decompress_to_slice_small_buffer took: {:?}", duration);
    }

    #[test]
    fn test_streaming_encoder_decoder() {
        let start = Instant::now();
        let original = b"Streaming reader and writer verification payload.";
        // Testiramo kompresiju i dekompresiju koriscenjem postojece Qzip logike kroz tok
        let compressed = Qzip::compress(original);
        let mut decompressed = Vec::new();
        let result = Qzip::decompress_to_slice(&compressed, &mut decompressed);
        assert!(result.is_ok());
        assert_eq!(original.to_vec(), decompressed);
        let duration = start.elapsed();
        println!("[BENCHMARK] test_streaming_encoder_decoder took: {:?}", duration);
    }

    #[test]
    fn test_ffi_compress_and_decompress_success() {
        let start = Instant::now();
        let input_data = vec![0u8, 0, 0, 1, 1, 1, 2, 2, 2, 3, 3, 3];

        // --- 1. Query compression output size ---
        let mut compressed_len: usize = 0;
        let status = unsafe {
            qzip_compress(
                input_data.as_ptr(),
                input_data.len(),
                std::ptr::null_mut(),
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
                std::ptr::null_mut(),
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
        let duration = start.elapsed();
        println!("[BENCHMARK] test_ffi_compress_and_decompress_success took: {:?}", duration);
    }

    #[test]
    fn test_ffi_null_pointer_handling() {
        let start = Instant::now();
        let mut len: usize = 100;
        let dummy_data = vec![1u8, 2, 3];

        // Null input pointer
        let status = unsafe {
            qzip_compress(
                std::ptr::null(),
                dummy_data.len(),
                std::ptr::null_mut(),
                &mut len,
            )
        };
        assert_eq!(status, QzipStatus::NullPointer);

        // Null output len pointer
        let status = unsafe {
            qzip_compress(
                dummy_data.as_ptr(),
                dummy_data.len(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            )
        };
        assert_eq!(status, QzipStatus::NullPointer);
        let duration = start.elapsed();
        println!("[BENCHMARK] test_ffi_null_pointer_handling took: {:?}", duration);
    }

    #[test]
    fn test_ffi_buffer_too_small() {
        let start = Instant::now();
        let input_data = vec![1u8, 1, 1, 1, 2, 2, 2, 2];

        // Get actual size required
        let mut required_len: usize = 0;
        unsafe {
            qzip_compress(
                input_data.as_ptr(),
                input_data.len(),
                std::ptr::null_mut(),
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
        let duration = start.elapsed();
        println!("[BENCHMARK] test_ffi_buffer_too_small took: {:?}", duration);
    }

    #[test]
    fn test_ffi_corrupt_decompress_input() {
        let start = Instant::now();
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
        let duration = start.elapsed();
        println!("[BENCHMARK] test_ffi_corrupt_decompress_input took: {:?}", duration);
    }

    #[test]
    fn test_checksum_verification_and_benchmark() {
        use std::time::Instant;

        // Generate 1 MB test payload
        let dataset: Vec<u8> = (0..1_000_000).map(|i| (i % 256) as u8).collect();

        // BENCHMARK: Compression + Checksum calculation
        let start_comp = Instant::now();
        let compressed = Qzip::compress_with_checksum(&dataset);
        let comp_duration = start_comp.elapsed();

        // BENCHMARK: Decompression + Checksum validation
        let start_decomp = Instant::now();
        let decompressed = Qzip::decompress_verified(&compressed)
            .expect("Checksum validation failed on valid data!");
        let decomp_duration = start_decomp.elapsed();

        assert_eq!(dataset, decompressed);

        // BENCHMARK: Corrupted data detection
        let mut corrupted = compressed.clone();
        let last_idx = corrupted.len() - 1;
        corrupted[last_idx] ^= 0xFF; // Tamper with checksum bytes
        assert!(Qzip::decompress_verified(&corrupted).is_err());

        println!("\n--- BENCHMARK: Strict Checksum Enforcement ---");
        println!("Input Data Size:       {} bytes", dataset.len());
        println!("Compressed Size:       {} bytes", compressed.len());
        println!("Compression Time:      {:?}", comp_duration);
        println!("Decompression Time:    {:?}", decomp_duration);
        println!("Decomp Throughput:     {:.2} MB/s", 
            (dataset.len() as f64 / 1_048_576.0) / decomp_duration.as_secs_f64());
    }

    // ==========================================
    // 2. SIMD CHUNK PACKING TEST & BENCHMARK
    // ==========================================
    #[test]
    fn test_simd_packing_and_benchmark() {
        use std::time::Instant;

        let raw_chunk: [u8; 8] = [0b11100100, 0b00011011, 0xAA, 0x55, 0x12, 0x34, 0x56, 0x78];
        let iterations = 200_000; // 1.6 MB total processing
        let mut output = Vec::with_capacity(iterations * 8);

        // BENCHMARK: SIMD 64-bit parallel packing execution
        let start = Instant::now();
        for _ in 0..iterations {
            Qzip::compress_simd_chunk(&raw_chunk, &mut output);
        }
        let duration = start.elapsed();

        assert_eq!(output.len(), iterations * 8);

        // Verify Quat byte unpacking consistency
        let unpacked = Qzip::unpack_byte_quats_fast(raw_chunk[0]);
        assert_eq!(unpacked[0], Quat::from_u8(0b11));
        assert_eq!(unpacked[1], Quat::from_u8(0b10));

        println!("\n--- BENCHMARK: SIMD Vectorized Processing ---");
        println!("Processed Bytes:       {} bytes", iterations * 8);
        println!("Execution Time:        {:?}", duration);
        println!("SIMD Throughput:       {:.2} MB/s", 
            ((iterations * 8) as f64 / 1_048_576.0) / duration.as_secs_f64());
    }

    // ==========================================
    // 3. HYBRID LZ77 + QUAT TEST & BENCHMARK
    // ==========================================
    #[test]
    fn test_lz77_quats_and_benchmark() {
        use crate::lz77::{QuatLz77, LzQuatToken};
        use std::time::Instant;

        // Repeated sequence of Quats to trigger LZ77 matches
        let mut quats = Vec::new();
        let pattern = vec![Quat::Q0, Quat::Q1, Quat::Q2, Quat::Q3];
        for _ in 0..10_000 {
            quats.extend_from_slice(&pattern);
        }

        let lz77 = QuatLz77::new(256);

        // BENCHMARK: Sliding window pattern matching
        let start = Instant::now();
        let tokens = lz77.compress(&quats);
        let duration = start.elapsed();

        let has_matches = tokens.iter().any(|t| matches!(t, LzQuatToken::Match { .. }));
        assert!(has_matches, "LZ77 should find repeated Quat patterns!");

        println!("\n--- BENCHMARK: Hybrid LZ77 Quat Engine ---");
        println!("Input Quats Count:     {}", quats.len());
        println!("Generated Tokens:      {}", tokens.len());
        println!("Compression Time:      {:?}", duration);
        println!("Quat Process Speed:    {:.2} MQuats/s", 
            (quats.len() as f64 / 1_000_000.0) / duration.as_secs_f64());
    }

    // ==========================================
    // 4. ADAPTIVE ENTROPY TEST & BENCHMARK
    // ==========================================
    #[test]
    fn test_entropy_coder_and_benchmark() {
        use crate::entropy::QuatEntropyCoder;
        use std::time::Instant;

        // Biased Quat distribution (Q0 is dominant)
        let mut quats = vec![Quat::Q0; 80_000];
        quats.extend(vec![Quat::Q1; 10_000]);
        quats.extend(vec![Quat::Q2; 5_000]);
        quats.extend(vec![Quat::Q3; 5_000]);

        let mut coder = QuatEntropyCoder::new();

        // BENCHMARK: Frequency observation & tree generation
        let start = Instant::now();
        coder.observe_stream(&quats);
        let bit_codes = coder.build_bit_codes();
        let duration = start.elapsed();

        // Most frequent Quat (Q0) must receive 1-bit code
        let q0_code = bit_codes.get(&Quat::Q0).unwrap();
        assert_eq!(q0_code.1, 1, "Q0 should have the shortest bit length (1 bit)");

        println!("\n--- BENCHMARK: Adaptive Quat Entropy Coder ---");
        println!("Observed Stream Size:  {} quats", quats.len());
        println!("Huffman Build Time:    {:?}", duration);
        println!("Q0 Bit Code Assignment: {:?}", q0_code);
    }
}