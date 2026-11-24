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
* **Versioned Binary Header & Flags:** Structured header with format version (`v1`), capability flags, and chunk size metadata.
* **Chunked & Framed Streaming:** Frame-based quaternary streaming (`FramedQuatEncoder` / `FramedQuatDecoder`) with per-frame `CRC32` validation and end-of-stream markers.
* **75% Quaternary Savings & Quantization:** Explicit 2-bit-per-symbol packing (4 quats per byte) and lossy 8-to-4-level quantization helpers for quaternary workflows.
* **Huffman Entropy Codec:** Full binary Huffman encoding and decoding (`QuatHuffmanCodec`) with frequency metadata for skewed `Quat` streams.
* **Accelerated LZ77 Engine:** Trigram-indexed sliding-window search (`QuatLz77Fast`) with longer match lengths and token decompression.
* **Tokio `spawn_blocking` & Async Streams:** Offloads heavy packing/unpacking via `spawn_blocking` alongside `AsyncQuatPackWriter` and `AsyncQuatUnpackReader`.
* **Stable Architecture-Aware Unpacking:** Stable Rust `#[cfg(target_arch)]` quaternary unpacking without requiring `nightly`.
* **Expanded C-ABI FFI:** Native quaternary packing and unpacking entry points (`qzip_pack_quats`, `qzip_unpack_quats`) and extended `QzipStatus` error codes.

# Third-Party Licenses

This software includes open-source components developed by third parties. Below are the corresponding license notices and full legal texts for each component used in this project.

---

## 1. Serde

* **Author:** David Tolnay
* **Repository:** https://github.com/serde-rs/serde
* **License:** MIT / Apache 2.0 (Dual License)

### MIT License

Copyright (c) 2017 David Tolnay

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.

---

### Apache License, Version 2.0

Copyright (c) 2017 David Tolnay

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.

---

## 2. Tokio

* **Author:** Tokio Contributors
* **Repository:** https://github.com/tokio-rs/tokio
* **License:** MIT

### MIT License

Copyright (c) Tokio Contributors

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.

## 📊 Benchmark result

| Test Name | Status | Measured Time | Additional Metrics / Notes |
| --- | --- | --- | --- |
| `test_quat_conversion` | `ok` | `300ns`[cite: 39] | `Quat::from_u8` (`0b00..0b11`) |
| `test_empty_input` | `ok` | `2µs`[cite: 39] | Empty slice compression/decompression |
| `test_invalid_magic_header` | `ok` | `3.6µs`[cite: 39] | Expects `QzipError::InvalidMagic` |
| `test_ffi_null_pointer_handling` | `ok` | `6.2µs`[cite: 39] | Expects `QzipStatus::NullPointer` |
| `test_ffi_buffer_too_small` | `ok` | `6.5µs`[cite: 39] | Expects `QzipStatus::BufferTooSmall` |
| `test_ffi_corrupt_decompress_input` | `ok` | `9.4µs`[cite: 39] | Expects `CorruptStream` or `InvalidMagic` |
| `test_decompress_to_slice_small_buffer` | `ok` | `13.3µs`[cite: 39] | Truncated stream check (`CorruptStream`) |
| `test_ffi_pack_and_unpack_quats_benchmark` | `ok` | `17.5µs`[cite: 39] | FFI `qzip_pack_quats` & `qzip_unpack_quats` |
| `test_truncated_stream` | `ok` | `19.2µs`[cite: 39, 40] | Expects `QzipError::CorruptStream` |
| `test_streaming_encoder_decoder` | `ok` | `21.4µs`[cite: 39] | Stream roundtrip verification |
| `test_ffi_compress_and_decompress_success` | `ok` | `23.3µs`[cite: 39] | Two-pass size query + compress/decompress |
| `test_roundtrip_basic` | `ok` | `25.1µs`[cite: 39] | Input size: `58 bytes`[cite: 39] |
| `test_run_length_overflow` | `ok` | `27µs`[cite: 39] | Run-length boundary handling |
| `test_compress_into_buffer_reuse` | `ok` | `65.6µs`[cite: 39] | Buffer reuse without extra allocation |
| `test_high_repetitive_data` | `ok` | `221.7µs`[cite: 39] | Compressed len: `75` bytes[cite: 39] |
| `test_async_quat_blocking_and_streams_benchmark` | `ok` | `3.3324ms`[cite: 39] | Processed `16384` quats (`spawn_blocking` & async I/O)[cite: 39] |
| `test_quat_packing_quantization_and_arch_unpack_benchmark` | `ok` | `9.2695ms`[cite: 40] | Processed `100000` quats (packing, quantization & arch unpack)[cite: 40] |
| `test_header_flags_crc32_and_framed_stream_benchmark` | `ok` | `9.9431ms`[cite: 39, 40] | Framed output: `10502` bytes[cite: 39, 40] |
| `test_huffman_quat_codec_benchmark` | `ok` | `15.293ms`[cite: 40] | Stream bytes: `17500`[cite: 40] |
| `test_simd_packing_and_benchmark` | `ok` | `15.1925ms`[cite: 40] | `1600000` bytes processed (`100.44 MB/s`)[cite: 40] |
| `test_lz77_fast_roundtrip_benchmark` | `ok` | `18.2766ms`[cite: 40] | Generated `7` tokens[cite: 40] |
| `test_lz77_quats_and_benchmark` | `ok` | `21.3711ms`[cite: 40] | `40000` quats, `161` tokens (`1.87 MQuats/s`)[cite: 40] |
| `test_stress_random_data` | `ok` | `28.9182ms`[cite: 40] | Input size: `100000` bytes[cite: 40] |
| `test_entropy_coder_and_benchmark` | `ok` | `1.0278ms` (Huffman build)[cite: 39] | Stream: `100000` quats, `Q0` code: `(0, 1)`[cite: 39] |
| `test_checksum_verification_and_benchmark` | `ok` | `158.3134ms` (comp) / `93.1714ms` (decomp)[cite: 40] | `1000000` bytes -> `3000016` bytes (`10.24 MB/s` decomp)[cite: 40] |

### Detailed Subsystem Benchmarks

| Subsystem | Workload / Input | Time | Throughput / Output |
| --- | --- | --- | --- |
| Adaptive Quat Entropy Coder | `100000` quats[cite: 39] | Huffman build: `1.0278ms`[cite: 39] | `Q0` bit code assignment: `(0, 1)`[cite: 39] |
| SIMD Vectorized Processing | `1600000` bytes[cite: 40] | `15.1925ms`[cite: 40] | `100.44 MB/s`[cite: 40] |
| Hybrid LZ77 Quat Engine | `40000` quats[cite: 40] | Compression: `21.3711ms`[cite: 40] | `161` tokens, `1.87 MQuats/s`[cite: 40] |
| Strict Checksum Enforcement | `1000000` bytes[cite: 40] | Comp: `158.3134ms` / Decomp: `93.1714ms`[cite: 40] | Compressed: `3000016` bytes, Decomp: `10.24 MB/s`[cite: 40] |

## Licensing

`quat_zip` is dual-licensed:

1. **Open Source (AGPLv3):** Free to use for open-source software under the terms of the GNU Affero General Public License v3.0.
2. **Commercial License:** If you wish to use this library in proprietary software without releasing your source code under AGPLv3, a commercial license must be purchased. Contact [strahinjastojanovic826@gmail.com] for licensing terms.

## Installation

Add `qzip` to your `Cargo.toml`:

```toml
[dependencies]
qzip = "1.2.0"

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

## Quaternary Streaming, Framing, Huffman & LZ77

```rust
use quat_zip::async_zip::{pack_quats_blocking, unpack_quats_blocking};
use quat_zip::entropy::QuatHuffmanCodec;
use quat_zip::error::QzipError;
use quat_zip::lz77::QuatLz77Fast;
use quat_zip::quat::Quat;
use quat_zip::qzip::{FramedQuatDecoder, FramedQuatEncoder};

#[tokio::main]
async fn main() -> Result<(), QzipError> {
    let quats = vec![
        Quat::Q0, Quat::Q1, Quat::Q2, Quat::Q3,
        Quat::Q0, Quat::Q1, Quat::Q2, Quat::Q3,
        Quat::Q0, Quat::Q0, Quat::Q1, Quat::Q3,
    ];

    // 1. Framed & chunked quaternary stream with CRC32
    let mut encoder = FramedQuatEncoder::new(Vec::new(), 4, true)?;
    encoder.push_quats(&quats)?;
    let framed = encoder.finish()?;

    let mut decoder = FramedQuatDecoder::new(&framed[..])?;
    let mut decoded_quats = Vec::new();
    while let Some(chunk) = decoder.next_quats_chunk()? {
        decoded_quats.extend(chunk);
    }
    assert_eq!(decoded_quats, quats);

    // 2. Huffman entropy coding on Quat symbols
    let huff = QuatHuffmanCodec::encode(&quats);
    let huff_decoded = QuatHuffmanCodec::decode(&huff)
        .map_err(|e| QzipError::InvalidFrame(e.to_string()))?;
    assert_eq!(huff_decoded, quats);

    // 3. Fast LZ77 on Quat sequence
    let lz = QuatLz77Fast::new(64);
    let tokens = lz.compress_fast(&quats);
    let lz_decoded = QuatLz77Fast::decompress_fast(&tokens)
        .map_err(|e| QzipError::InvalidFrame(e.to_string()))?;
    assert_eq!(lz_decoded, quats);

    // 4. Blocking thread-pool packing & unpacking
    let packed = pack_quats_blocking(quats.clone()).await?;
    let unpacked = unpack_quats_blocking(packed, quats.len()).await?;
    assert_eq!(unpacked, quats);

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

```c
#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include "quat_zip.h"

int main(void) {
    // Quaternary values (0..=3), 1 symbol per byte on input
    uint8_t quats[] = {0x00, 0x01, 0x02, 0x03, 0x03, 0x02, 0x01, 0x00};
    size_t quat_count = sizeof(quats) / sizeof(quats[0]);

    // 1. Query packed size (4 quats per byte -> 75% size reduction vs 1 byte/quat)
    size_t packed_len = 0;
    QzipStatus status = qzip_pack_quats(quats, quat_count, NULL, &packed_len);
    if (status != Success) {
        printf("Failed to query packed size: %d\n", status);
        return 1;
    }

    // 2. Pack quats into caller-allocated buffer
    uint8_t *packed = (uint8_t *)malloc(packed_len ? packed_len : 1);
    if (!packed) {
        return 1;
    }
    status = qzip_pack_quats(quats, quat_count, packed, &packed_len);
    if (status != Success) {
        printf("Failed to pack quats: %d\n", status);
        free(packed);
        return 1;
    }

    // 3. Unpack back to quat_count bytes
    size_t unpacked_len = 0;
    status = qzip_unpack_quats(packed, packed_len, quat_count, NULL, &unpacked_len);
    if (status == Success) {
        uint8_t *unpacked = (uint8_t *)malloc(unpacked_len ? unpacked_len : 1);
        if (unpacked) {
            status = qzip_unpack_quats(packed, packed_len, quat_count, unpacked, &unpacked_len);
            if (status == Success) {
                printf("Packed %zu quats into %zu bytes and unpacked %zu values.\n",
                       quat_count, packed_len, unpacked_len);
            }
            free(unpacked);
        }
    }

    free(packed);
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
