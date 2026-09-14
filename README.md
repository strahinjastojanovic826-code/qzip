# qzip

`qzip` is a lightweight, zero-dependency data compression library built around **Quad-bit (2-bit) processing** and optimized **Run-Length Encoding (RLE)**. 

Designed to seamlessly integrate with custom low-level ecosystems, `qzip` breaks standard bytes down into 2-bit quats (`00`, `01`, `10`, `11`) and compresses repetitive bit sequences with minimal memory overhead.

## Features

* **Zero Dependencies:** Ultra-fast compilation and zero external crates.
* **Quad-bit Native:** Operates at the 2-bit level (4 quats per byte).
* **6-bit RLE Packing:** Packs quat values and run lengths (up to 64) into compact single-byte tokens.
* **Data Integrity:** Includes a 4-byte magic header (`QZIP`) and length verification to ensure stream safety.

## Installation

Add `qzip` to your `Cargo.toml`:

```toml
[dependencies]
qzip = "0.1.0"

Quick Start

use qzip::{Qzip, QzipError};

```rust
fn main() -> Result<(), QzipError> {
    let data = b"Hello, World! Quad-bit compression in action.";

    // Compress the data
    let compressed = Qzip::compress(data);

    // Decompress back to original bytes
    let decompressed = Qzip::decompress(&compressed)?;

    assert_eq!(data.to_vec(), decompressed);
    Ok(())
}

How It Works

    Deconstruction: Each byte is split into four 2-bit quats (Q0 = 00, Q1 = 01, Q2 = 10, Q3 = 11).

    RLE Processing: Consecutive identical quats are grouped together (up to 64 repetitions per chunk).

    Token Encoding: Each chunk is packed into a single byte:

        Bits 7–6: 2-bit Quat Value (0..3)

        Bits 5–0: 6-bit Run Length (1..64)