//! # Safety & FFI Contracts
//!
//! All `extern "C"` functions in this module (`qzip_compress`, `qzip_decompress`,
//! `qzip_pack_quats`, and `qzip_unpack_quats`) are exposed across an FFI boundary and
//! rely on the following safety invariants:
//!
//! - **Pointer validity & lifetimes:** When `in_len > 0`, `in_ptr` must point to a
//!   valid, initialized, and properly aligned region of at least `in_len` bytes that
//!   remains valid and immutable for the duration of the call[cite: 34, 35, 36, 37].
//!   `out_len` must point to a valid, aligned, writable `usize` whenever non-null[cite: 34, 35, 36, 37].
//! - **Output buffer & non-overlapping memory:** When `out_ptr` is non-null and
//!   `*out_len >= required_len`, `out_ptr` must point to a valid, aligned, writable
//!   buffer of at least `*out_len` bytes that does not overlap (`copy_nonoverlapping` /
//!   pointer writes) with the input slice[cite: 34, 35, 36, 37].
//! - **Raw slice & pointer operations:** Internal `unsafe` blocks assume null checks
//!   have already guarded against invalid pointers, `slice::from_raw_parts` receives a
//!   valid pointer/length pair, and `catch_unwind` prevents panics from unwinding
//!   across `extern "C"` frames into foreign callers[cite: 34, 35, 36, 37].

use crate::qzip::Qzip;
use crate::error::QzipError;
use std::panic::catch_unwind;
use std::slice;
use crate::quat::Quat;

/// FFI Status codes returned to C / C++ / C#
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum QzipStatus {
    Success = 0,
    NullPointer = 1,
    BufferTooSmall = 2,
    EmptyData = 3,
    CorruptStream = 4,
    InvalidMagic = 5,
    ChecksumMismatch = 6,
    IoError = 7,
    PanicOccurred = 8,
    UnsupportedVersion = 9,
    InvalidFlags = 10,
    InvalidFrame = 11,
}

impl From<QzipError> for QzipStatus {
    fn from(err: QzipError) -> Self {
        match err {
            QzipError::EmptyData => QzipStatus::EmptyData,
            QzipError::CorruptStream => QzipStatus::CorruptStream,
            QzipError::InvalidMagic => QzipStatus::InvalidMagic,
            QzipError::ChecksumMismatch => QzipStatus::ChecksumMismatch,
            QzipError::IoError(_) => QzipStatus::IoError,
            QzipError::UnsupportedVersion(_) => QzipStatus::UnsupportedVersion,
            QzipError::InvalidFlags(_) => QzipStatus::InvalidFlags,
            QzipError::InvalidFrame(_) => QzipStatus::InvalidFrame,
        }
    }
}

/// Compresses input bytes into caller-allocated output buffer.
///
/// - If `out_ptr` is NULL, `out_len` will be updated with the required buffer size.
/// - Returns `QzipStatus::BufferTooSmall` if `*out_len` is insufficient.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn qzip_compress(
    in_ptr: *const u8,
    in_len: usize,
    out_ptr: *mut u8,
    out_len: *mut usize,
) -> QzipStatus { unsafe {
    if in_ptr.is_null() || out_len.is_null() {
        return QzipStatus::NullPointer;
    }

    let result = catch_unwind(|| {
        let input = slice::from_raw_parts(in_ptr, in_len);
        let compressed = Qzip::compress(input);
        let required_len = compressed.len();

        if out_ptr.is_null() {
            *out_len = required_len;
            return QzipStatus::Success;
        }

        if *out_len < required_len {
            *out_len = required_len;
            return QzipStatus::BufferTooSmall;
        }

        std::ptr::copy_nonoverlapping(compressed.as_ptr(), out_ptr, required_len);
        *out_len = required_len;

        QzipStatus::Success
    });

    result.unwrap_or(QzipStatus::PanicOccurred)
}}

/// Decompresses input bytes into caller-allocated output buffer.
///
/// - If `out_ptr` is NULL, `out_len` will be updated with the required buffer size.
/// - Returns `QzipStatus::BufferTooSmall` if `*out_len` is insufficient.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn qzip_decompress(
    in_ptr: *const u8,
    in_len: usize,
    out_ptr: *mut u8,
    out_len: *mut usize,
) -> QzipStatus { unsafe {
    if in_ptr.is_null() || out_len.is_null() {
        return QzipStatus::NullPointer;
    }

    let result = catch_unwind(|| {
        let input = slice::from_raw_parts(in_ptr, in_len);

        match Qzip::decompress(input) {
            Ok(decompressed) => {
                let required_len = decompressed.len();

                if out_ptr.is_null() {
                    *out_len = required_len;
                    return QzipStatus::Success;
                }

                if *out_len < required_len {
                    *out_len = required_len;
                    return QzipStatus::BufferTooSmall;
                }

                std::ptr::copy_nonoverlapping(decompressed.as_ptr(), out_ptr, required_len);
                *out_len = required_len;

                QzipStatus::Success
            }
            Err(err) => QzipStatus::from(err),
        }
    });

    result.unwrap_or(QzipStatus::PanicOccurred)
}}

/// Packs raw 2-bit quaternary values (`0..=3`, one per byte in `in_ptr`)
/// into packed bytes (4 quats per output byte) using caller-allocated buffer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn qzip_pack_quats(
    in_ptr: *const u8,
    in_len: usize,
    out_ptr: *mut u8,
    out_len: *mut usize,
) -> QzipStatus { unsafe {
    if out_len.is_null() {
        return QzipStatus::NullPointer;
    }
    if in_len > 0 && in_ptr.is_null() {
        return QzipStatus::NullPointer;
    }

    let result = catch_unwind(|| {
        let input = if in_len == 0 {
            &[]
        } else {
            slice::from_raw_parts(in_ptr, in_len)
        };

        let mut packed = Vec::with_capacity(input.len().div_ceil(4));
        for chunk in input.chunks(4) {
            let q0 = Quat::from_u8(*chunk.first().unwrap_or(&0));
            let q1 = Quat::from_u8(*chunk.get(1).unwrap_or(&0));
            let q2 = Quat::from_u8(*chunk.get(2).unwrap_or(&0));
            let q3 = Quat::from_u8(*chunk.get(3).unwrap_or(&0));
            packed.push(Quat::pack_four(q0, q1, q2, q3));
        }

        let required_len = packed.len();
        if out_ptr.is_null() {
            *out_len = required_len;
            return QzipStatus::Success;
        }
        if *out_len < required_len {
            *out_len = required_len;
            return QzipStatus::BufferTooSmall;
        }

        std::ptr::copy_nonoverlapping(packed.as_ptr(), out_ptr, required_len);
        *out_len = required_len;
        QzipStatus::Success
    });

    result.unwrap_or(QzipStatus::PanicOccurred)
}}

/// Unpacks packed quaternary bytes (`in_ptr`) into `quat_count` 2-bit values (`0..=3`)
/// written to `out_ptr`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn qzip_unpack_quats(
    in_ptr: *const u8,
    in_len: usize,
    quat_count: usize,
    out_ptr: *mut u8,
    out_len: *mut usize,
) -> QzipStatus { unsafe {
    if out_len.is_null() {
        return QzipStatus::NullPointer;
    }
    if in_len > 0 && in_ptr.is_null() {
        return QzipStatus::NullPointer;
    }

    let result = catch_unwind(|| {
        let input = if in_len == 0 {
            &[]
        } else {
            slice::from_raw_parts(in_ptr, in_len)
        };

        let unpacked_quats = Quat::unpack_quats_arch(input, quat_count);
        let required_len = unpacked_quats.len();

        if out_ptr.is_null() {
            *out_len = required_len;
            return QzipStatus::Success;
        }
        if *out_len < required_len {
            *out_len = required_len;
            return QzipStatus::BufferTooSmall;
        }

        for (i, q) in unpacked_quats.iter().enumerate() {
            *out_ptr.add(i) = q.as_u8();
        }
        *out_len = required_len;
        QzipStatus::Success
    });

    result.unwrap_or(QzipStatus::PanicOccurred)
}}