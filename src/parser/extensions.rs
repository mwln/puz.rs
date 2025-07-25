use super::io::find_section;
use crate::{
    error::{PuzError, PuzWarning},
    types::{Extensions, Rebus},
};
use std::collections::HashMap;

/// Information about extra sections in the .puz file
#[derive(Debug)]
#[allow(clippy::upper_case_acronyms)]
enum ExtraSection {
    GRBS,
    RTBL,
    GEXT,
}

const EXTRA_SECTIONS: [(&str, ExtraSection); 3] = [
    ("GRBS", ExtraSection::GRBS),
    ("RTBL", ExtraSection::RTBL),
    ("GEXT", ExtraSection::GEXT),
];

/// Parse extension sections with recovery for non-critical failures
pub(crate) fn parse_extensions_with_recovery(
    data: &[u8],
    width: u8,
    height: u8,
) -> Result<(Extensions, Vec<PuzWarning>), PuzError> {
    let mut rebus = None;
    let mut circles = None;
    let mut given = None;
    let mut warnings = Vec::new();

    for (section_name, section_type) in &EXTRA_SECTIONS {
        match find_section(data, section_name) {
            Ok(Some(section_data)) => {
                match section_type {
                    ExtraSection::GRBS => {
                        // Validate GRBS section size first
                        let expected_size = (width as usize) * (height as usize);
                        if section_data.len() != expected_size {
                            warnings.push(PuzWarning::SkippedExtension {
                                section: "GRBS".to_string(),
                                reason: format!(
                                    "Size mismatch: expected {} bytes, got {}",
                                    expected_size,
                                    section_data.len()
                                ),
                            });
                            continue;
                        }

                        match find_section(data, "RTBL") {
                            Ok(Some(rtbl_data)) => {
                                match parse_rebus(&section_data, &rtbl_data, width, height) {
                                    Ok(parsed_rebus) => rebus = Some(parsed_rebus),
                                    Err(e) => warnings.push(PuzWarning::SkippedExtension {
                                        section: "GRBS/RTBL".to_string(),
                                        reason: format!("Failed to parse rebus data: {}", e),
                                    }),
                                }
                            }
                            Ok(None) => warnings.push(PuzWarning::SkippedExtension {
                                section: "GRBS".to_string(),
                                reason:
                                    "RTBL section not found - rebus requires both GRBS and RTBL"
                                        .to_string(),
                            }),
                            Err(e) => warnings.push(PuzWarning::SkippedExtension {
                                section: "GRBS".to_string(),
                                reason: format!("Failed to read RTBL section: {}", e),
                            }),
                        }
                    }
                    ExtraSection::GEXT => {
                        // Validate GEXT section size first
                        let expected_size = (width as usize) * (height as usize);
                        if section_data.len() != expected_size {
                            warnings.push(PuzWarning::SkippedExtension {
                                section: "GEXT".to_string(),
                                reason: format!(
                                    "Size mismatch: expected {} bytes, got {}",
                                    expected_size,
                                    section_data.len()
                                ),
                            });
                        } else {
                            match parse_gext(&section_data, width, height) {
                                Ok((parsed_circles, parsed_given)) => {
                                    circles = parsed_circles;
                                    given = parsed_given;
                                }
                                Err(e) => warnings.push(PuzWarning::SkippedExtension {
                                    section: "GEXT".to_string(),
                                    reason: format!("Failed to parse GEXT data: {}", e),
                                }),
                            }
                        }
                    }
                    ExtraSection::RTBL => {
                        // Handled with GRBS
                    }
                }
            }
            Ok(None) => {
                // Section not present - this is normal, not a warning
            }
            Err(e) => warnings.push(PuzWarning::SkippedExtension {
                section: section_name.to_string(),
                reason: format!("Failed to read section: {}", e),
            }),
        }
    }

    Ok((
        Extensions {
            rebus,
            circles,
            given,
        },
        warnings,
    ))
}

/// Parse rebus data from GRBS and RTBL sections
fn parse_rebus(
    grbs_data: &[u8],
    rtbl_data: &[u8],
    width: u8,
    height: u8,
) -> Result<Rebus, PuzError> {
    let grid_size = (width as usize) * (height as usize);
    if grbs_data.len() != grid_size {
        return Err(PuzError::SectionSizeMismatch {
            section: "GRBS".to_string(),
            expected: grid_size,
            found: grbs_data.len(),
        });
    }

    // Parse GRBS grid
    let grid = grbs_data
        .chunks(width as usize)
        .map(|chunk| chunk.to_vec())
        .collect();

    // Parse RTBL table using proper character encoding
    let rtbl_str = super::io::decode_puz_string(rtbl_data)?;
    let mut table = HashMap::new();

    for entry in rtbl_str.split(';') {
        if entry.trim().is_empty() {
            continue; // Skip empty entries
        }
        if let Some(colon_pos) = entry.find(':') {
            let key_str = entry[..colon_pos].trim();
            let value = entry[colon_pos + 1..].trim().to_string();
            if let Ok(key) = key_str.parse::<u8>() {
                table.insert(key, value);
            }
        }
    }

    Ok(Rebus { grid, table })
}

/// Type alias for the complex return type of GEXT parsing
type GextResult = (Option<Vec<Vec<bool>>>, Option<Vec<Vec<bool>>>);

/// Parse GEXT section for circles and given squares
fn parse_gext(data: &[u8], width: u8, height: u8) -> Result<GextResult, PuzError> {
    let grid_size = (width as usize) * (height as usize);
    if data.len() != grid_size {
        return Err(PuzError::SectionSizeMismatch {
            section: "GEXT".to_string(),
            expected: grid_size,
            found: data.len(),
        });
    }

    let mut has_circles = false;
    let mut has_given = false;
    let mut circles = vec![vec![false; width as usize]; height as usize];
    let mut given = vec![vec![false; width as usize]; height as usize];

    for (i, &byte) in data.iter().enumerate() {
        let row = i / (width as usize);
        let col = i % (width as usize);

        if byte & 0x80 != 0 {
            // Circled/shaded square
            circles[row][col] = true;
            has_circles = true;
        }

        if byte & 0x40 != 0 {
            // Contents were given
            given[row][col] = true;
            has_given = true;
        }
    }

    Ok((
        if has_circles { Some(circles) } else { None },
        if has_given { Some(given) } else { None },
    ))
}
