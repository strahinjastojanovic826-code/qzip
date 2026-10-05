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
* **Zero External Dependencies:** Built entirely with pure standard Rust for minimal crate footprint and ultra-fast compilation.

## Licensing

`quat_zip` is dual-licensed:

1. **Open Source (AGPLv3):** Free to use for open-source software under the terms of the GNU Affero General Public License v3.0.
2. **Commercial License:** If you wish to use this library in proprietary software without releasing your source code under AGPLv3, a commercial license must be purchased. Contact [strahinjastojanovic826@gmail.com] for licensing terms.

## Installation

Add `qzip` to your `Cargo.toml`:

```toml
[dependencies]
qzip = "0.6.0"

```
## Quick Start

```rust
use quat_zip::qzip::Qzip;
use quat_zip::error::QzipError;
use quat_zip::async_zip::AsyncQzip;
use quat_zip::quat::Quat;
use quat_zip::serde::BinarySerializable;

fn main() -> Result<(), QzipError> {
    let data = b"Hello, World! Quad-bit compression in action.";

    // 1. Standard compression and decompression
    let compressed = Qzip::compress(data);
    let decompressed = Qzip::decompress(&compressed)?;
    assert_eq!(data.to_vec(), decompressed);

    // 2. Zero-allocation decompression into existing buffer
    let mut buffer = vec![0u8; data.len()];
    let decompressed_bytes = Qzip::decompress_to_slice(&compressed, &mut buffer)?;
    assert_eq!(&buffer[..decompressed_bytes], data);

    // 3. Background thread compression (Zero-dependency std::thread)
    let rx_compressed = AsyncQzip::compress_async(data.to_vec());
    let async_compressed = rx_compressed.recv().unwrap();

    let rx_decompressed = AsyncQzip::decompress_async(async_compressed);
    let async_decompressed = rx_decompressed.recv().unwrap()?;
    assert_eq!(data.to_vec(), async_decompressed);

    // 4. Custom binary serialization for Quats
    let quats = vec![Quat::Q0, Quat::Q1, Quat::Q2, Quat::Q3];
    let bytes = quats.to_bytes();
    let restored_quats = <Vec<Quat>>::from_bytes(&bytes)?;
    assert_eq!(quats, restored_quats);

    println!("All compression, streaming, and serialization tests passed!");
    Ok(())
}

```
## How It Works

1. Deconstruction: Each byte is split into four 2-bit quats (Q0 = 00, Q1 = 01, Q2 = 10, Q3 = 11).
2. RLE Processing: Consecutive identical quats are grouped together (up to 64 repetitions per chunk).
3. Token Encoding: Each chunk is packed into a single byte:
   - Bits 7-6: 2-bit Quat Value (0..3)
   - Bits 5-0: 6-bit Run Length (1..64)

```

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
