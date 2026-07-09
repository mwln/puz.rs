/// The `.puz` checksum: a modified CRC-16 (rotate-right, then add) applied per
/// byte. See `PUZ.md` §Checksums for the reference algorithm.
pub(crate) fn cksum_region(data: &[u8], mut cksum: u16) -> u16 {
    for &b in data {
        cksum = (cksum >> 1) | ((cksum & 1) << 15);
        cksum = cksum.wrapping_add(b as u16);
    }
    cksum
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
