/// The 8-byte mask string for the "masked" checksums (spells "ICHEATED").
const MASK: &[u8; 8] = b"ICHEATED";

/// The `.puz` checksum: a modified CRC-16 (rotate-right, then add) applied per
/// byte. See `PUZ.md` §Checksums for the reference algorithm.
pub(crate) fn cksum_region(data: &[u8], mut cksum: u16) -> u16 {
    for &b in data {
        cksum = (cksum >> 1) | ((cksum & 1) << 15);
        cksum = cksum.wrapping_add(b as u16);
    }
    cksum
}

/// The four component checksums of a `.puz` file, in the order the format
/// masks them: header (CIB), solution, fill (player grid), and text.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Components {
    pub(crate) header: u16,
    pub(crate) solution: u16,
    pub(crate) fill: u16,
    pub(crate) text: u16,
}

impl Components {
    /// The CIB (header) checksum, over the 8 header bytes at 0x2C..0x34
    /// (width, height, num_clues LE, bitmask LE, scrambled-tag LE).
    pub(crate) fn cib(&self) -> u16 {
        self.header
    }

    /// The overall/global file checksum (stored at 0x00): header, then the
    /// solution grid, the fill grid, and the text region, chained. Extensions
    /// are not included.
    pub(crate) fn global(&self, solution: &[u8], fill: &[u8], text: &[u8]) -> u16 {
        let mut c = self.header;
        c = cksum_region(solution, c);
        c = cksum_region(fill, c);
        cksum_region(text, c)
    }

    /// The 8 "masked" checksum bytes stored at 0x10..0x18. Each component's low
    /// and high bytes are XORed with the corresponding byte of "ICHEATED".
    pub(crate) fn masked(&self) -> [u8; 8] {
        let lows = [self.header, self.solution, self.fill, self.text];
        let mut out = [0u8; 8];
        for (i, c) in lows.iter().enumerate() {
            out[i] = (*c as u8) ^ MASK[i]; // low byte ^ "ICHE"
            out[i + 4] = ((*c >> 8) as u8) ^ MASK[i + 4]; // high byte ^ "ATED"
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cksum_region_empty_returns_seed() {
        assert_eq!(cksum_region(&[], 0), 0);
        assert_eq!(cksum_region(&[], 0x1234), 0x1234);
    }

    #[test]
    fn test_cksum_region_single_byte() {
        // seed 0, byte 0x01: rotate(0)=0, +1 => 1
        assert_eq!(cksum_region(&[0x01], 0), 1);
    }

    #[test]
    fn test_cksum_region_known_vector() {
        // Hand-computed against PUZ.md's reference algorithm, seed 0:
        //   0x01: rot(0)=0,      +1 => 0x0001
        //   0x02: rot(1)=0x8000, +2 => 0x8002
        //   0x03: rot(0x8002)=0x4001, +3 => 0x4004
        assert_eq!(cksum_region(&[0x01, 0x02, 0x03], 0), 0x4004);
    }

    #[test]
    fn test_cksum_region_seed_chaining() {
        // Feeding the checksum of region A as the seed for region B must equal
        // checksumming the concatenation in one pass. The composite checksums
        // in Task 7 chain regions with a running seed, so this property must
        // hold.
        let a = [0x10u8, 0x20, 0x30];
        let b = [0x40u8, 0x50];
        let chained = cksum_region(&b, cksum_region(&a, 0));
        let concat: Vec<u8> = a.iter().chain(b.iter()).copied().collect();
        assert_eq!(chained, cksum_region(&concat, 0));
    }

    #[test]
    fn test_cksum_region_wraps_at_u16() {
        // High bytes accumulate and wrap without panicking (wrapping_add).
        let data = [0xFFu8; 8];
        // Just assert it computes a value (no overflow panic) and is stable.
        assert_eq!(cksum_region(&data, 0), cksum_region(&data, 0));
    }
}
