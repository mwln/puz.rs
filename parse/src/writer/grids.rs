use crate::types::Grid;

/// Serialize the solution grid followed by the player-state (blank) grid.
///
/// `.puz` stores two `width * height` byte blocks back to back: the solution
/// first, then the blank grid. Each cell is a single byte (`.` for a block,
/// `-` for an empty player cell, otherwise the letter).
///
/// Diagramless puzzles store black squares as `:` (0x3A) instead of `.`
/// (0x2E). When `diagramless` is set, every `.` in both grids is emitted as
/// `:`, matching the on-disk convention the parser detects.
pub(crate) fn serialize_grids(grid: &Grid, diagramless: bool) -> Vec<u8> {
    let mut out = Vec::new();
    for row in grid.solution.iter().chain(grid.blank.iter()) {
        for &b in row.as_bytes() {
            out.push(if diagramless && b == b'.' { b':' } else { b });
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_grids_concatenates_rows() {
        let grid = Grid {
            solution: vec!["AB".to_string(), "CD".to_string()],
            blank: vec!["--".to_string(), "--".to_string()],
        };
        let bytes = serialize_grids(&grid, false);
        assert_eq!(bytes, b"ABCD----");
    }

    #[test]
    fn test_serialize_grids_with_blocks() {
        let grid = Grid {
            solution: vec!["A.B".to_string(), "C.D".to_string()],
            blank: vec!["-.-".to_string(), "-.-".to_string()],
        };
        let bytes = serialize_grids(&grid, false);
        assert_eq!(bytes, b"A.BC.D-.--.-");
    }

    #[test]
    fn test_serialize_grids_order_is_solution_then_blank() {
        // Solution and blank differ so the ordering is observable.
        let grid = Grid {
            solution: vec!["XY".to_string()],
            blank: vec!["--".to_string()],
        };
        assert_eq!(serialize_grids(&grid, false), b"XY--");
    }

    #[test]
    fn test_serialize_grids_non_square() {
        // width (3) != height (2): guards against row/col transposition bugs.
        let grid = Grid {
            solution: vec!["ABC".to_string(), "DEF".to_string()],
            blank: vec!["---".to_string(), "---".to_string()],
        };
        let bytes = serialize_grids(&grid, false);
        assert_eq!(bytes, b"ABCDEF------");
        assert_eq!(bytes.len(), 3 * 2 * 2); // width*height, two grids
    }

    #[test]
    fn test_serialize_grids_single_cell() {
        let grid = Grid {
            solution: vec!["A".to_string()],
            blank: vec!["-".to_string()],
        };
        assert_eq!(serialize_grids(&grid, false), b"A-");
    }

    #[test]
    fn test_serialize_grids_diagramless_emits_colon() {
        // With diagramless set, every '.' black square becomes ':' in both grids.
        let grid = Grid {
            solution: vec!["A.B".to_string(), "C.D".to_string()],
            blank: vec!["-.-".to_string(), "-.-".to_string()],
        };
        let bytes = serialize_grids(&grid, true);
        assert_eq!(bytes, b"A:BC:D-:--:-");
    }

    #[test]
    fn test_serialize_grids_diagramless_false_unchanged() {
        // Without the flag, '.' stays '.' (byte-identical to the normal path).
        let grid = Grid {
            solution: vec!["A.B".to_string()],
            blank: vec!["-.-".to_string()],
        };
        assert_eq!(serialize_grids(&grid, false), b"A.B-.-");
    }
}
