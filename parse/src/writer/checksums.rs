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
}
