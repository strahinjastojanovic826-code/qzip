# qzip

`qzip` is a lightweight, zero-dependency data compression library built around **Quad-bit (2-bit) processing** and optimized **Run-Length Encoding (RLE)**. 

Designed to seamlessly integrate with custom low-level ecosystems, `qzip` breaks standard bytes down into 2-bit quats (`00`, `01`, `10`, `11`) and compresses repetitive bit sequences with minimal memory overhead.

## Features

* **Quad-bit Native:** Operates natively at the 2-bit level (4 quats packed per byte).
* **6-bit RLE Packing:** Compresses quat values and repeat lengths (1..64) into compact single-byte tokens.
* **Data Integrity & Safety:** Includes a 4-byte magic header (`QZIP`), stream validation, and complete length checks.
* **Zero-Allocation Options:** Provides `decompress_to_slice` and `compress_into` for high-performance buffer reuse.
* **Background Threading:** Supports non-blocking background compression and decompression via standard threads.
* **Standard I/O Streaming:** Full support for `std::io::Read` and `std::io::Write` interfaces via `QzipEncoder` and `QzipDecoder`.
* **Async Tokio I/O:** Modern asynchronous compression and decompression support using Tokio (`AsyncRead` / `AsyncWrite`).
* **Standard Serde Support:** Seamless integration with the Rust `serde` ecosystem for data serialization and deserialization.
* **SIMD Accelerated:** High-performance vector-accelerated chunk processing for faster bit packaging.
* **Zero-Copy Iterator API:** Stream quats on-the-fly without heap allocations using `QuatIterator`.
* **Checksum Verification:** Enhanced data integrity checks to detect stream corruption.
* **Hybrid LZ77 Engine:** Sliding-window dictionary matching combined with RLE for superior compression ratios on complex sequences.
* **Adaptive Entropy Coding:** Dynamic frequency analysis and variable-length bit-code assignment (Huffman-style) for skewed quaternary distributions.
* **Strict Checksum Enforcement:** Advanced data integrity validation (`decompress_verified`) to guarantee absolute protection against stream corruption.

# Third-Party Licenses

This project relies on the following third-party crates (`serde` and `tokio`). Their respective licenses and copyright notices are listed below:

## 1. serde
* **License:** MIT OR Apache-2.0 (at your option)
* **Copyright:** Copyright (c) 2017 David Tolnay
* **Terms:** Licensed under either of the Apache License, Version 2.0 or the MIT license at your option.

---

## 2. tokio
* **License:** MIT
* **Copyright:** Copyright (c) Tokio Contributors
* **Terms:** Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction...

## 📊 Benchmark result

| Test Name | Execution Time / Duration | Performance / Metrics & Details |
| :--- | :--- | :--- |
| `test_quat_conversion` | 700ns[cite: 24] | Basic quaternary conversion verification |
| `test_ffi_null_pointer_handling` | 700ns[cite: 24] | FFI safety check for null pointers |
| `test_ffi_corrupt_decompress_input` | 1.9µs[cite: 24] | FFI error handling for corrupted streams |
| `test_empty_input` | 2.7µs[cite: 24] | Handling empty byte inputs |
| `test_invalid_magic_header` | 3.4µs[cite: 24] | Error handling for invalid magic headers |
| `test_decompress_to_slice_small_buffer` | 6.9µs[cite: 24] | Slice decompression with limited buffer |
| `test_ffi_buffer_too_small` | 9.6µs[cite: 24] | FFI size query and buffer limit check |
| `test_truncated_stream` | 13.4µs[cite: 24] | Detection of truncated byte streams |
| `test_ffi_compress_and_decompress_success` | 17.2µs[cite: 24] | Successful C-ABI interop roundtrip |
| `test_run_length_overflow` | 17.4µs[cite: 24] | RLE chunk overflow handling |
| `test_streaming_encoder_decoder` | 20.8µs[cite: 24] | Stream pipeline writer/reader validation |
| `test_roundtrip_basic` | 21.7µs[cite: 24] | Basic roundtrip (Input size: 58 bytes)[cite: 24] |
| `test_compress_into_buffer_reuse` | 33µs[cite: 24] | Zero-allocation buffer reuse verification |
| `test_high_repetitive_data` | 156.2µs[cite: 24] | Compressed length: 75 bytes[cite: 24] |
| `test_stress_random_data` | 26.22ms[cite: 25] | Stress test with 100,000 random bytes[cite: 25] |
| `test_entropy_coder_and_benchmark` | 1.02ms[cite: 24] | Huffman Build Time (100,000 quats stream)[cite: 24] |
| `test_simd_packing_and_benchmark` | 9.63ms[cite: 24] | SIMD Throughput: **158.45 MB/s** (1.6 MB processed)[cite: 24] |
| `test_lz77_quats_and_benchmark` | 13.85ms[cite: 25] | Quat Speed: **2.89 MQuats/s** (40k quats, 161 tokens)[cite: 25] |
| `test_checksum_verification_and_benchmark` | Comp: 152.48ms<br>Decomp: 93.70ms[cite: 25] | Decomp Throughput: **10.18 MB/s** (1 MB dataset)[cite: 25] |

## Licensing

`quat_zip` is dual-licensed:

1. **Open Source (AGPLv3):** Free to use for open-source software under the terms of the GNU Affero General Public License v3.0.
2. **Commercial License:** If you wish to use this library in proprietary software without releasing your source code under AGPLv3, a commercial license must be purchased. Contact [strahinjastojanovic826@gmail.com] for licensing terms.

## Installation

Add `qzip` to your `Cargo.toml`:

```toml
[dependencies]
qzip = "1.1.0"

```
## Quick Start

```rust
use quat_zip::qzip::Qzip;
use quat_zip::error::QzipError;
use quat_zip::async_zip::AsyncQzip;
use quat_zip::quat::Quat;

#[tokio::main]
async fn main() -> Result<(), QzipError> {
    let data = b"Hello, World! Quad-bit compression in action.";

    // 1. Standard compression and decompression
    let compressed = Qzip::compress(data);
    let decompressed = Qzip::decompress(&compressed)?;
    assert_eq!(data.to_vec(), decompressed);

    // 2. Zero-allocation decompression into existing buffer
    let mut buffer = vec![0u8; data.len()];
    let decompressed_bytes = Qzip::decompress_to_slice(&compressed, &mut buffer)?;
    assert_eq!(&buffer[..decompressed_bytes], data);

    // 3. Strict Checksum Enforcement (Industrial Data Integrity)
    let secure_compressed = Qzip::compress_with_checksum(data);
    let verified_decompressed = Qzip::decompress_verified(&secure_compressed)?;
    assert_eq!(data.to_vec(), verified_decompressed);

    // 4. Async compression and decompression (Tokio I/O)
    let mut compressed_async_buf = Vec::new();
    AsyncQzip::compress_async(tokio::io::Cursor::new(data), &mut compressed_async_buf).await?;

    let mut decompressed_async_buf = Vec::new();
    AsyncQzip::decompress_async(tokio::io::Cursor::new(compressed_async_buf), &mut decompressed_async_buf).await?;
    assert_eq!(data.to_vec(), decompressed_async_buf);

    // 5. Standard Serde serialization for Quats
    let quats = vec![Quat::Q0, Quat::Q1, Quat::Q2, Quat::Q3];
    let bytes = bincode::serialize(&quats).unwrap();
    let restored_quats: Vec<Quat> = bincode::deserialize(&bytes).unwrap();
    assert_eq!(quats, restored_quats);

    println!("All compression, streaming, checksum, and serialization tests passed!");
    Ok(())
}
```

## How It Works

1. Deconstruction: Each byte is split into four 2-bit quats (Q0 = 00, Q1 = 01, Q2 = 10, Q3 = 11).
2. RLE Processing: Consecutive identical quats are grouped together (up to 64 repetitions per chunk).
3. Token Encoding: Each chunk is packed into a single byte:
   - Bits 7-6: 2-bit Quat Value (0..3)
   - Bits 5-0: 6-bit Run Length (1..64)

# Integration Guide (C, C++, C#)

This library exposes a native C-compatible API (`cdylib` / `staticlib`) designed for seamless integration with C, C++, and C# (.NET).

---

## 1. C Integration

### Build Artifacts Required
- `quat_zip.h` (Header file)
- `quat_zip.dll` / `libquat_zip.so` / `libquat_zip.dylib` (Dynamic library) OR `quat_zip.lib` / `libquat_zip.a` (Static library)

### Example Usage (`main.c`)

```c
#include <stdio.h>
#include <stdlib.h>
#include "quat_zip.h"

int main(void) {
    uint8_t input[] = {0x00, 0x01, 0x02, 0x03, 0x03, 0x03};
    size_t input_len = sizeof(input);

    // 1. Query required compression buffer size
    size_t compressed_len = 0;
    QzipStatus status = qzip_compress(input, input_len, NULL, &compressed_len);
    if (status != QZIP_SUCCESS) {
        printf("Failed to query buffer size. Error code: %d\n", status);
        return 1;
    }

    // 2. Allocate output buffer and compress
    uint8_t* compressed = (uint8_t*)malloc(compressed_len);
    status = qzip_compress(input, input_len, compressed, &compressed_len);
    if (status == QZIP_SUCCESS) {
        printf("Successfully compressed %size_t bytes into %size_t bytes.\n", input_len, compressed_len);
    }

    free(compressed);
    return 0;
}
```

---

## 2. C++ Integration

### Example Usage (`main.cpp`)

```cpp
#include <iostream>
#include <vector>
#include "quat_zip.h"

int main() {
    std::vector<uint8_t> input = {0x00, 0x01, 0x02, 0x03, 0x03, 0x03};

    // 1. Query compression output size
    size_t compressed_len = 0;
    QzipStatus status = qzip_compress(input.data(), input.size(), nullptr, &compressed_len);
    if (status != QZIP_SUCCESS) {
        std::cerr << "Compression size query failed with status: " << status << std::endl;
        return -1;
    }

    // 2. Allocate buffer and perform compression
    std::vector<uint8_t> compressed(compressed_len);
    status = qzip_compress(input.data(), input.size(), compressed.data(), &compressed_len);
    if (status != QZIP_SUCCESS) {
        std::cerr << "Compression failed with status: " << status << std::endl;
        return -1;
    }

    std::cout << "Compressed size: " << compressed_len << " bytes" << std::endl;
    return 0;
}
```

---

## 3. C# (.NET) Integration

### Project Setup
1. Copy the compiled library (`quat_zip.dll` on Windows, `libquat_zip.so` on Linux, `libquat_zip.dylib` on macOS) to your C# project directory.
2. In your `.csproj`, ensure the native library is copied to the output directory:

```xml
<ItemGroup>
  <None Update="quat_zip.dll">
    <CopyToOutputDirectory>PreserveNewest</CopyToOutputDirectory>
  </None>
</ItemGroup>
```

### Wrapper Class & Example Usage (`Program.cs`)

```csharp
using System;
using System.Runtime.InteropServices;

namespace QuatZipApp
{
    public enum QzipStatus : int
    {
        Success = 0,
        NullPointer = 1,
        BufferTooSmall = 2,
        EmptyData = 3,
        CorruptStream = 4,
        InvalidMagic = 5,
        ChecksumMismatch = 6,
        IoError = 7,
        PanicOccurred = 8
    }

    internal static class NativeMethods
    {
        private const string DllName = "quat_zip";

        [DllImport(DllName, CallingConvention = CallingConvention.Cdecl)]
        public static unsafe extern QzipStatus qzip_compress(
            byte* inPtr,
            UIntPtr inLen,
            byte* outPtr,
            ref UIntPtr outLen
        );

        [DllImport(DllName, CallingConvention = CallingConvention.Cdecl)]
        public static unsafe extern QzipStatus qzip_decompress(
            byte* inPtr,
            UIntPtr inLen,
            byte* outPtr,
            ref UIntPtr outLen
        );
    }

    public static class QuatZip
    {
        public static byte[] Compress(ReadOnlySpan<byte> input)
        {
            unsafe
            {
                fixed (byte* inPtr = input)
                {
                    UIntPtr requiredLen = UIntPtr.Zero;

                    // Query required buffer size
                    QzipStatus status = NativeMethods.qzip_compress(inPtr, (UIntPtr)input.Length, null, ref requiredLen);
                    if (status != QzipStatus.Success)
                        throw new InvalidOperationException($"Failed to query compress buffer size: {status}");

                    byte[] output = new byte[(int)requiredLen];
                    fixed (byte* outPtr = output)
                    {
                        status = NativeMethods.qzip_compress(inPtr, (UIntPtr)input.Length, outPtr, ref requiredLen);
                        if (status != QzipStatus.Success)
                            throw new InvalidOperationException($"Compression failed: {status}");
                    }

                    return output;
                }
            }
        }
    }

    class Program
    {
        static void Main()
        {
            byte[] input = new byte[] { 0x00, 0x01, 0x02, 0x03, 0x03, 0x03 };
            byte[] compressed = QuatZip.Compress(input);

            Console.WriteLine($"Compressed {input.Length} bytes into {compressed.Length} bytes.");
        }
    }
}
```
