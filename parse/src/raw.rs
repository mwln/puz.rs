//! Low-level, byte-level readers for `.puz` files.
//!
//! These functions read a `.puz` file's structure directly from its bytes
//! without validating or assembling a [`Puzzle`](crate::Puzzle). They are
//! deliberately lenient: they return whatever structure they can recover, so
//! they remain useful for inspecting or debugging files that fail to parse.
//!
//! For normal use, parse a file into a [`Puzzle`](crate::Puzzle) instead. Reach
//! for this module when you need to see the raw header, grids, string table, or
//! extension-section framing of a file, especially one that does not parse.

// Header field offsets within a `.puz` file.
const OFF_VERSION: usize = 0x18;
const OFF_WIDTH: usize = 0x2C;
const OFF_HEIGHT: usize = 0x2D;
const OFF_NUM_CLUES: usize = 0x2E;
const OFF_BITMASK: usize = 0x30;
const OFF_SCRAMBLED: usize = 0x32;

/// The end of the fixed header; grid data begins here.
pub const HEADER_LEN: usize = 0x34;

/// The fixed-size header fields of a `.puz` file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawHeader {
    /// Grid width in cells (offset 0x2C).
    pub width: u8,
    /// Grid height in cells (offset 0x2D).
    pub height: u8,
    /// Declared clue count (offset 0x2E, little-endian).
    pub num_clues: u16,
    /// Format version string (offset 0x18, NUL-trimmed).
    pub version: String,
    /// Puzzle-type bitmask (offset 0x30, little-endian).
    pub bitmask: u16,
    /// Scrambled tag (offset 0x32, little-endian).
    pub scrambled_tag: u16,
}

/// A cell where the solution and blank grids disagree about a black square.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlackSquareMismatch {
    /// Zero-based row.
    pub row: usize,
    /// Zero-based column.
    pub col: usize,
    /// The byte in the solution grid at this cell.
    pub solution: u8,
    /// The byte in the blank grid at this cell.
    pub blank: u8,
}

/// The two raw grids of a `.puz` file, as flat byte rows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawGrids {
    /// Grid width in cells.
    pub width: usize,
    /// Grid height in cells.
    pub height: usize,
    /// Solution grid rows (each `width` bytes).
    pub solution: Vec<Vec<u8>>,
    /// Blank (player-state) grid rows (each `width` bytes).
    pub blank: Vec<Vec<u8>>,
}

/// The string table of a `.puz` file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawStrings {
    /// Puzzle title.
    pub title: String,
    /// Author line.
    pub author: String,
    /// Copyright line.
    pub copyright: String,
    /// Clues, in file (reading) order.
    pub clues: Vec<String>,
    /// Notes/instructions.
    pub notes: String,
}

/// A single extension section (GRBS, RTBL, GEXT, and so on).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawSection {
    /// The 4-byte section tag.
    pub tag: String,
    /// Byte offset of the tag within the file.
    pub offset: usize,
    /// Declared data length (little-endian u16 after the tag).
    pub length: usize,
    /// Declared section checksum.
    pub checksum: u16,
    /// The section's data bytes (clamped to the end of the file).
    pub data: Vec<u8>,
}

/// Section tags this module recognizes when scanning.
const KNOWN_SECTIONS: [&[u8; 4]; 6] = [b"GRBS", b"RTBL", b"GEXT", b"LTIM", b"RUSR", b"MARK"];

fn u16le(data: &[u8], off: usize) -> u16 {
    u16::from_le_bytes([data[off], data[off + 1]])
}

/// Read the fixed header fields. Returns `None` if the data is shorter than the
/// header.
pub fn read_header(data: &[u8]) -> Option<RawHeader> {
    if data.len() < HEADER_LEN {
        return None;
    }
    let version = String::from_utf8_lossy(&data[OFF_VERSION..OFF_VERSION + 4])
        .trim_end_matches('\0')
        .to_string();
    Some(RawHeader {
        width: data[OFF_WIDTH],
        height: data[OFF_HEIGHT],
        num_clues: u16le(data, OFF_NUM_CLUES),
        version,
        bitmask: u16le(data, OFF_BITMASK),
        scrambled_tag: u16le(data, OFF_SCRAMBLED),
    })
}

/// Read the solution and blank grids. Returns `None` if the data is too short
/// to hold the header and both `width * height` grids.
pub fn read_grids(data: &[u8]) -> Option<RawGrids> {
    let header = read_header(data)?;
    let width = header.width as usize;
    let height = header.height as usize;
    let board = width * height;
    let sol_end = HEADER_LEN + board;
    let fill_end = sol_end + board;
    if fill_end > data.len() {
        return None;
    }
    let rows = |bytes: &[u8]| -> Vec<Vec<u8>> {
        if width == 0 {
            return Vec::new();
        }
        bytes.chunks(width).map(<[u8]>::to_vec).collect()
    };
    Some(RawGrids {
        width,
        height,
        solution: rows(&data[HEADER_LEN..sol_end]),
        blank: rows(&data[sol_end..fill_end]),
    })
}

/// A numbered cell: a grid position that starts an across and/or down word.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NumberedCell {
    /// The clue number assigned to this cell.
    pub number: u16,
    /// Zero-based row.
    pub row: usize,
    /// Zero-based column.
    pub col: usize,
    /// Whether this cell starts an across word.
    pub across: bool,
    /// Whether this cell starts a down word.
    pub down: bool,
}

impl RawGrids {
    /// Whether a cell is playable: a letter/digit or the open-cell byte `-`.
    /// A `.` (black square) or anything else is not playable.
    fn is_playable(&self, row: usize, col: usize) -> bool {
        matches!(
            self.solution.get(row).and_then(|r| r.get(col)),
            Some(&b) if b == b'-' || b.is_ascii_alphanumeric()
        )
    }

    fn is_black(&self, row: usize, col: usize) -> bool {
        self.solution.get(row).and_then(|r| r.get(col)) == Some(&b'.')
    }

    /// Compute clue numbering by walking the solution grid in reading order,
    /// using the standard rule: a cell starts an across word when it and the
    /// cell to its right are playable and it is at the left edge or preceded by
    /// a black square (and symmetrically for down words). Each numbered cell
    /// increments the running number once.
    ///
    /// This is the same rule the parser uses to assign clue numbers, exposed
    /// here on raw bytes so tools can show it for files that fail to parse.
    pub fn clue_numbers(&self) -> Vec<NumberedCell> {
        let mut out = Vec::new();
        let mut number = 0u16;
        for row in 0..self.height {
            for col in 0..self.width {
                if !self.is_playable(row, col) {
                    continue;
                }
                let starts_across =
                    (col == 0 || self.is_black(row, col - 1)) && self.is_playable(row, col + 1);
                let starts_down =
                    (row == 0 || self.is_black(row - 1, col)) && self.is_playable(row + 1, col);
                if starts_across || starts_down {
                    number += 1;
                    out.push(NumberedCell {
                        number,
                        row,
                        col,
                        across: starts_across,
                        down: starts_down,
                    });
                }
            }
        }
        out
    }

    /// The number of across and down clue slots the grid implies.
    pub fn clue_counts(&self) -> (usize, usize) {
        let numbers = self.clue_numbers();
        let across = numbers.iter().filter(|n| n.across).count();
        let down = numbers.iter().filter(|n| n.down).count();
        (across, down)
    }

    /// Cells where exactly one grid marks a black square (`.`).
    ///
    /// A well-formed puzzle has none; mismatches indicate a non-standard grid
    /// (for example, a theme glyph placed in a playable cell).
    pub fn black_square_mismatches(&self) -> Vec<BlackSquareMismatch> {
        let mut out = Vec::new();
        for (r, (sol_row, blank_row)) in self.solution.iter().zip(&self.blank).enumerate() {
            for (c, (&s, &b)) in sol_row.iter().zip(blank_row).enumerate() {
                if (s == b'.') != (b == b'.') {
                    out.push(BlackSquareMismatch {
                        row: r,
                        col: c,
                        solution: s,
                        blank: b,
                    });
                }
            }
        }
        out
    }
}

/// Read the string table (title, author, copyright, clues, notes).
///
/// Reads `num_clues` clue strings as declared in the header. Returns `None` if
/// the data is too short to reach the string section.
pub fn read_strings(data: &[u8]) -> Option<RawStrings> {
    let header = read_header(data)?;
    let board = (header.width as usize) * (header.height as usize);
    let start = HEADER_LEN + 2 * board;
    if start > data.len() {
        return None;
    }

    let mut offset = start;
    let title = next_string(data, &mut offset);
    let author = next_string(data, &mut offset);
    let copyright = next_string(data, &mut offset);
    let clues = (0..header.num_clues)
        .map(|_| next_string(data, &mut offset))
        .collect();
    let notes = next_string(data, &mut offset);

    Some(RawStrings {
        title,
        author,
        copyright,
        clues,
        notes,
    })
}

/// Read a NUL-terminated string starting at `*offset`, advancing past the NUL.
///
/// Bytes are decoded lossily as UTF-8; a missing terminator reads to the end.
fn next_string(data: &[u8], offset: &mut usize) -> String {
    let start = (*offset).min(data.len());
    let mut end = start;
    while end < data.len() && data[end] != 0 {
        end += 1;
    }
    let s = String::from_utf8_lossy(&data[start..end]).into_owned();
    *offset = (end + 1).min(data.len().saturating_add(1));
    s
}

/// Scan the file for known extension sections and return them in file order.
///
/// Each section's framing is `tag (4 bytes)`, `length (u16 LE)`,
/// `checksum (u16 LE)`, then `length` data bytes. Data is clamped to the end of
/// the file so a bad length can't over-read.
pub fn scan_sections(data: &[u8]) -> Vec<RawSection> {
    let mut sections = Vec::new();
    let mut i = 0usize;
    while i + 8 <= data.len() {
        let tag = &data[i..i + 4];
        if KNOWN_SECTIONS.iter().any(|k| k.as_slice() == tag) {
            let length = u16le(data, i + 4) as usize;
            let checksum = u16le(data, i + 6);
            let start = i + 8;
            let end = (start + length).min(data.len());
            sections.push(RawSection {
                tag: String::from_utf8_lossy(tag).into_owned(),
                offset: i,
                length,
                checksum,
                data: data[start..end].to_vec(),
            });
            // Advance past the framing + data so we don't re-match inside data.
            i = end.max(i + 1);
        } else {
            i += 1;
        }
    }
    sections
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a minimal `.puz` byte buffer for tests: a header with the given
    /// dimensions/clue count, two grids, then a string table.
    fn build(
        width: u8,
        height: u8,
        solution: &[u8],
        blank: &[u8],
        clues: &[&str],
        bitmask: u16,
    ) -> Vec<u8> {
        let mut out = vec![0u8; HEADER_LEN];
        out[OFF_VERSION..OFF_VERSION + 4].copy_from_slice(b"1.3\0");
        out[OFF_WIDTH] = width;
        out[OFF_HEIGHT] = height;
        out[OFF_NUM_CLUES..OFF_NUM_CLUES + 2].copy_from_slice(&(clues.len() as u16).to_le_bytes());
        out[OFF_BITMASK..OFF_BITMASK + 2].copy_from_slice(&bitmask.to_le_bytes());
        out.extend_from_slice(solution);
        out.extend_from_slice(blank);
        let z = |s: &str, out: &mut Vec<u8>| {
            out.extend_from_slice(s.as_bytes());
            out.push(0);
        };
        z("Title", &mut out);
        z("Author", &mut out);
        z("(c)", &mut out);
        for c in clues {
            z(c, &mut out);
        }
        z("Notes", &mut out);
        out
    }

    #[test]
    fn test_read_header_fields() {
        let data = build(2, 2, b"AB.D", b"--.-", &["a", "b", "c"], 0x0401);
        let h = read_header(&data).unwrap();
        assert_eq!(h.width, 2);
        assert_eq!(h.height, 2);
        assert_eq!(h.num_clues, 3);
        assert_eq!(h.version, "1.3");
        assert_eq!(h.bitmask, 0x0401);
        assert_eq!(h.scrambled_tag, 0x0000);
    }

    #[test]
    fn test_read_header_too_short() {
        assert!(read_header(&[0u8; 10]).is_none());
    }

    #[test]
    fn test_read_grids_shapes_rows() {
        let data = build(2, 2, b"AB.D", b"--.-", &["a"], 0x0001);
        let g = read_grids(&data).unwrap();
        assert_eq!(g.width, 2);
        assert_eq!(g.height, 2);
        assert_eq!(g.solution, vec![b"AB".to_vec(), b".D".to_vec()]);
        assert_eq!(g.blank, vec![b"--".to_vec(), b".-".to_vec()]);
    }

    #[test]
    fn test_black_square_mismatches_none_when_consistent() {
        let data = build(2, 2, b"AB.D", b"--.-", &["a"], 0x0001);
        let g = read_grids(&data).unwrap();
        assert!(g.black_square_mismatches().is_empty());
    }

    #[test]
    fn test_black_square_mismatches_detects_disagreement() {
        // Solution has '.' at (1,0) but blank has '-' there.
        let data = build(2, 2, b"AB.D", b"----", &["a"], 0x0001);
        let g = read_grids(&data).unwrap();
        let m = g.black_square_mismatches();
        assert_eq!(m.len(), 1);
        assert_eq!(
            m[0],
            BlackSquareMismatch {
                row: 1,
                col: 0,
                solution: b'.',
                blank: b'-',
            }
        );
    }

    #[test]
    fn test_clue_numbers_simple_open_grid() {
        // 2x2 open grid: (0,0) starts A+D #1, (0,1) starts D #2, (1,0) starts A #3.
        let data = build(2, 2, b"ABCD", b"----", &["a"], 0x0001);
        let g = read_grids(&data).unwrap();
        let nums = g.clue_numbers();
        assert_eq!(nums.len(), 3);
        assert_eq!(
            nums[0],
            NumberedCell {
                number: 1,
                row: 0,
                col: 0,
                across: true,
                down: true
            }
        );
        assert_eq!(
            nums[1],
            NumberedCell {
                number: 2,
                row: 0,
                col: 1,
                across: false,
                down: true
            }
        );
        assert_eq!(
            nums[2],
            NumberedCell {
                number: 3,
                row: 1,
                col: 0,
                across: true,
                down: false
            }
        );
        assert_eq!(g.clue_counts(), (2, 2));
    }

    #[test]
    fn test_clue_numbers_ignores_length_one_slots() {
        // (0,1) is a lone playable cell between blacks: no across, no down word.
        // Row 0: A . B  Row 1: . . .  (only (0,0) and (0,2) are isolated too)
        let data = build(3, 1, b"A.B", b"-.-", &["a"], 0x0001);
        let g = read_grids(&data).unwrap();
        // No cell has a right/down neighbor playable, so no numbered cells.
        assert!(g.clue_numbers().is_empty());
        assert_eq!(g.clue_counts(), (0, 0));
    }

    #[test]
    fn test_read_strings_lists_clues() {
        let data = build(2, 2, b"AB.D", b"--.-", &["one", "two", "three"], 0x0001);
        let s = read_strings(&data).unwrap();
        assert_eq!(s.title, "Title");
        assert_eq!(s.author, "Author");
        assert_eq!(s.copyright, "(c)");
        assert_eq!(s.clues, vec!["one", "two", "three"]);
        assert_eq!(s.notes, "Notes");
    }

    #[test]
    fn test_scan_sections_finds_framed_section() {
        // Header + 2x2 grids, then a GEXT section framed as tag/len/cksum/data.
        let mut data = build(2, 2, b"AB.D", b"--.-", &["a"], 0x0001);
        data.extend_from_slice(b"GEXT");
        data.extend_from_slice(&4u16.to_le_bytes()); // length
        data.extend_from_slice(&0xABCDu16.to_le_bytes()); // checksum
        data.extend_from_slice(&[0x80, 0, 0, 0]); // one circled cell

        let sections = scan_sections(&data);
        let gext = sections
            .iter()
            .find(|s| s.tag == "GEXT")
            .expect("GEXT found");
        assert_eq!(gext.length, 4);
        assert_eq!(gext.checksum, 0xABCD);
        assert_eq!(gext.data, vec![0x80, 0, 0, 0]);
    }

    #[test]
    fn test_scan_sections_clamps_bad_length() {
        // A section claiming more data than the file holds must not over-read.
        let mut data = build(2, 2, b"AB.D", b"--.-", &["a"], 0x0001);
        data.extend_from_slice(b"GRBS");
        data.extend_from_slice(&9999u16.to_le_bytes()); // absurd length
        data.extend_from_slice(&0u16.to_le_bytes());
        data.extend_from_slice(&[0, 0]); // only 2 bytes of data present

        let sections = scan_sections(&data);
        let grbs = sections
            .iter()
            .find(|s| s.tag == "GRBS")
            .expect("GRBS found");
        assert_eq!(grbs.length, 9999);
        assert_eq!(grbs.data.len(), 2, "data must be clamped to file end");
    }
}
