use crate::{
    checksums::cksum_region,
    error::PuzError,
    types::Extensions,
};

/// GEXT bit flags (mirrors `parser::extensions::parse_gext`).
const GEXT_CIRCLED: u8 = 0x80;
const GEXT_GIVEN: u8 = 0x40;

/// Serialize the extension sections implied by `extensions`, in the order the
/// parser looks for them: GRBS + RTBL (rebus), then GEXT (circles/given).
///
/// Each section is framed as: 4-byte ASCII name, 2-byte little-endian data
/// length, 2-byte data checksum, the data, and a trailing NUL byte. Only
/// sections with content are emitted; a puzzle with no extensions produces no
/// bytes.
pub(crate) fn serialize_extensions(
    extensions: &Extensions,
    width: u8,
    height: u8,
) -> Result<Vec<u8>, PuzError> {
    let mut out = Vec::new();

    if let Some(rebus) = &extensions.rebus {
        // GRBS: width*height bytes, one rebus key per cell (0 = none).
        let grbs = flatten_u8_grid(&rebus.grid, width, height, "GRBS")?;
        write_section(&mut out, b"GRBS", &grbs);

        // RTBL: "key:value;" entries, keys ascending for determinism.
        let mut keys: Vec<&u8> = rebus.table.keys().collect();
        keys.sort_unstable();
        let mut rtbl = String::new();
        for k in keys {
            // Keys are right-justified to width 2 in real files (e.g. " 1:FOO;").
            let value = &rebus.table[k];
            rtbl.push_str(&format!("{k:>2}:{value};"));
        }
        write_section(&mut out, b"RTBL", rtbl.as_bytes());
    }

    // GEXT: width*height bitmask bytes; emit if any circle/given is set.
    if extensions.circles.is_some() || extensions.given.is_some() {
        let gext = build_gext(extensions, width, height)?;
        write_section(&mut out, b"GEXT", &gext);
    }

    Ok(out)
}

/// Append one framed section: name, LE length, data checksum, data, NUL.
fn write_section(out: &mut Vec<u8>, name: &[u8; 4], data: &[u8]) {
    out.extend_from_slice(name);
    out.extend_from_slice(&(data.len() as u16).to_le_bytes());
    out.extend_from_slice(&cksum_region(data, 0).to_le_bytes());
    out.extend_from_slice(data);
    out.push(0);
}

/// Flatten a `height`-row, `width`-column `u8` grid into row-major bytes,
/// validating the dimensions.
fn flatten_u8_grid(
    grid: &[Vec<u8>],
    width: u8,
    height: u8,
    section: &str,
) -> Result<Vec<u8>, PuzError> {
    if grid.len() != height as usize || grid.iter().any(|r| r.len() != width as usize) {
        return Err(PuzError::SectionSizeMismatch {
            section: section.to_string(),
            expected: width as usize * height as usize,
            found: grid.iter().map(|r| r.len()).sum(),
        });
    }
    Ok(grid.iter().flatten().copied().collect())
}

/// Build the GEXT bitmask grid from the circles/given boolean grids.
fn build_gext(extensions: &Extensions, width: u8, height: u8) -> Result<Vec<u8>, PuzError> {
    let (w, h) = (width as usize, height as usize);
    let mut bytes = vec![0u8; w * h];

    let mut apply = |grid: &Vec<Vec<bool>>, flag: u8, name: &str| -> Result<(), PuzError> {
        if grid.len() != h || grid.iter().any(|r| r.len() != w) {
            return Err(PuzError::SectionSizeMismatch {
                section: name.to_string(),
                expected: w * h,
                found: grid.iter().map(|r| r.len()).sum(),
            });
        }
        for (row, cells) in grid.iter().enumerate() {
            for (col, &set) in cells.iter().enumerate() {
                if set {
                    bytes[row * w + col] |= flag;
                }
            }
        }
        Ok(())
    };

    if let Some(circles) = &extensions.circles {
        apply(circles, GEXT_CIRCLED, "GEXT circles")?;
    }
    if let Some(given) = &extensions.given {
        apply(given, GEXT_GIVEN, "GEXT given")?;
    }

    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Rebus;
    use std::collections::HashMap;

    fn no_ext() -> Extensions {
        Extensions {
            rebus: None,
            circles: None,
            given: None,
        }
    }

    #[test]
    fn test_no_extensions_produces_no_bytes() {
        assert!(serialize_extensions(&no_ext(), 2, 2).unwrap().is_empty());
    }

    #[test]
    fn test_gext_circles_framing_and_flag() {
        let mut e = no_ext();
        e.circles = Some(vec![vec![true, false], vec![false, false]]);
        let bytes = serialize_extensions(&e, 2, 2).unwrap();

        // name
        assert_eq!(&bytes[0..4], b"GEXT");
        // length = 4 (2x2)
        assert_eq!(u16::from_le_bytes([bytes[4], bytes[5]]), 4);
        // data starts at 8 (after name+len+cksum)
        assert_eq!(&bytes[8..12], &[GEXT_CIRCLED, 0, 0, 0]);
        // trailing NUL
        assert_eq!(*bytes.last().unwrap(), 0);
    }

    #[test]
    fn test_gext_combines_circle_and_given_in_one_byte() {
        let mut e = no_ext();
        e.circles = Some(vec![vec![true]]);
        e.given = Some(vec![vec![true]]);
        let bytes = serialize_extensions(&e, 1, 1).unwrap();
        // single data byte carries both flags
        assert_eq!(bytes[8], GEXT_CIRCLED | GEXT_GIVEN);
    }

    #[test]
    fn test_rebus_emits_grbs_then_rtbl() {
        let mut e = no_ext();
        let mut table = HashMap::new();
        table.insert(1u8, "HEART".to_string());
        e.rebus = Some(Rebus {
            grid: vec![vec![0, 1], vec![0, 0]],
            table,
        });
        let bytes = serialize_extensions(&e, 2, 2).unwrap();
        assert_eq!(&bytes[0..4], b"GRBS");
        // GRBS data (4 bytes) at offset 8, then NUL, then RTBL
        assert_eq!(&bytes[8..12], &[0, 1, 0, 0]);
        let rtbl_pos = bytes.windows(4).position(|w| w == b"RTBL").unwrap();
        assert_eq!(&bytes[rtbl_pos..rtbl_pos + 4], b"RTBL");
    }

    #[test]
    fn test_gext_dimension_mismatch_errors() {
        let mut e = no_ext();
        e.circles = Some(vec![vec![true, false]]); // 1 row, expected 2
        assert!(matches!(
            serialize_extensions(&e, 2, 2).unwrap_err(),
            PuzError::SectionSizeMismatch { .. }
        ));
    }
}
