//! Inspect the extension sections of one or more `.puz` files.
//!
//! Dumps every 4-byte section tag found after the puzzle body, its declared
//! length, and (for GRBS) a summary of the rebus-marked cells. Use it to
//! understand files that warn during parsing, e.g. "GRBS without RTBL".
//!
//! Usage:
//!     cargo run --release --example inspect_sections -- <FILE> [<FILE> ...]

use std::path::PathBuf;

fn main() {
    let files: Vec<PathBuf> = std::env::args().skip(1).map(PathBuf::from).collect();
    if files.is_empty() {
        eprintln!("usage: inspect_sections <FILE> [<FILE> ...]");
        std::process::exit(2);
    }

    for path in &files {
        let data = match std::fs::read(path) {
            Ok(d) => d,
            Err(e) => {
                println!("READ-ERR {}: {e}", path.display());
                continue;
            }
        };

        println!("=== {} ({} bytes) ===", path.display(), data.len());

        if data.len() < 0x34 {
            println!("  too short for a header");
            continue;
        }
        let width = data[0x2C] as usize;
        let height = data[0x2D] as usize;
        let board = width * height;
        println!("  grid {width}x{height} = {board} cells");

        // Body: header (0x34) + solution (board) + fill (board) + strings.
        // Extension sections follow the strings. Rather than track string
        // lengths, scan the whole file for known 4-byte tags and report each,
        // then verify the section framing (len + checksum + data).
        let known = [b"GRBS", b"RTBL", b"GEXT", b"LTIM", b"RUSR", b"MARK"];
        let mut found_any = false;
        let mut i = 0usize;
        while i + 8 <= data.len() {
            let tag = &data[i..i + 4];
            if known.iter().any(|k| *k == tag) {
                let len = u16::from_le_bytes([data[i + 4], data[i + 5]]) as usize;
                let cksum = u16::from_le_bytes([data[i + 6], data[i + 7]]);
                let start = i + 8;
                let end = (start + len).min(data.len());
                let tag_str = std::str::from_utf8(tag).unwrap_or("????");
                println!(
                    "  section {tag_str} @0x{i:X} len={len} cksum=0x{cksum:04X} \
                     (data 0x{start:X}..0x{end:X})"
                );
                found_any = true;

                match tag {
                    b"GRBS" => {
                        let body = &data[start..end];
                        let nz: Vec<(usize, usize, u8)> = body
                            .iter()
                            .enumerate()
                            .filter(|(_, &v)| v != 0)
                            .map(|(idx, &v)| (idx / width, idx % width, v))
                            .collect();
                        println!(
                            "    GRBS: {} marked cell(s){}",
                            nz.len(),
                            if body.len() == board {
                                String::new()
                            } else {
                                format!(" (len {} != board {board}!)", body.len())
                            }
                        );
                        if !nz.is_empty() {
                            println!("    marked (row,col,key): {nz:?}");
                        }
                    }
                    b"RTBL" | b"RUSR" => {
                        let body = &data[start..end];
                        println!("    body: {:?}", String::from_utf8_lossy(body));
                    }
                    b"GEXT" => {
                        let body = &data[start..end];
                        let nz = body.iter().filter(|&&b| b != 0).count();
                        println!("    GEXT: {nz} nonzero flag cell(s)");
                    }
                    _ => {}
                }

                // Advance past this section's framing + data to avoid
                // re-matching bytes inside the data.
                i = end.max(i + 1);
            } else {
                i += 1;
            }
        }

        if !found_any {
            println!("  no extension sections found");
        }
        println!();
    }
}
