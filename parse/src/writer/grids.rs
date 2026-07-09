use crate::types::Grid;

/// Serialize the solution grid followed by the player-state (blank) grid.
///
/// `.puz` stores two `width * height` byte blocks back to back: the solution
/// first, then the blank grid. Each cell is a single byte (`.` for a block,
/// `-` for an empty player cell, otherwise the letter).
pub(crate) fn serialize_grids(grid: &Grid) -> Vec<u8> {
    let mut out = Vec::new();
    for row in &grid.solution {
        out.extend_from_slice(row.as_bytes());
    }
    for row in &grid.blank {
        out.extend_from_slice(row.as_bytes());
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
        let bytes = serialize_grids(&grid);
        assert_eq!(bytes, b"ABCD----");
    }

    #[test]
    fn test_serialize_grids_with_blocks() {
        let grid = Grid {
            solution: vec!["A.B".to_string(), "C.D".to_string()],
            blank: vec!["-.-".to_string(), "-.-".to_string()],
        };
        let bytes = serialize_grids(&grid);
        assert_eq!(bytes, b"A.BC.D-.--.-");
    }

    #[test]
    fn test_serialize_grids_order_is_solution_then_blank() {
        // Solution and blank differ so the ordering is observable.
        let grid = Grid {
            solution: vec!["XY".to_string()],
            blank: vec!["--".to_string()],
        };
        assert_eq!(serialize_grids(&grid), b"XY--");
    }
}
