/// RollingSum keeps an accumulated checksum of neighboring range of bytes, which can roll
/// byte by byte within the limits of block size
pub struct RollingSum {
    /// Sum of all the block values
    pub r1: u32,
    /// Sum of all r1 calculated in consecutive iterations
    pub r2: u32,
    /// Number of bytes included in the summed r1, r2 variables
    pub l: u32,
}

impl Default for RollingSum {
    fn default() -> Self {
        Self::new()
    }
}

impl RollingSum {
    // In rsync it is 1 << 16 for digesting speed,
    // whereas adler-32 uses 65521 (the largest prime number smaller than 2^16)
    const MODULO: u32 = 1 << 16;

    pub fn new() -> Self {
        Self { r1: 0, r2: 0, l: 0 }
    }

    /// Returns aggregated rolling checksum for current state
    pub fn digest(&self) -> u32 {
        // If we used different modulo, we would have here r = r1 + (r2 * MODULO).
        // Because MODULO is 1 << 16 we can left shift bits also here.
        self.r1 + (self.r2 << 16)
    }

    /// Append a slice of bytes to the current rolling checksum state
    pub fn update(&mut self, buffer: &[u8]) {
        let mut a: u32 = 0;
        let mut b: u32 = 0;
        let len = buffer.len() as u32;

        buffer.iter().enumerate().for_each(|(index, byte)| {
            a += *byte as u32;
            b += (*byte as u32) * (len - (index as u32));
        });

        self.r1 = (self.r1.wrapping_add(a)) % RollingSum::MODULO;
        self.r2 = (self.r2.wrapping_add(b)) % RollingSum::MODULO;
        self.l = (self.l.wrapping_add(len)) % RollingSum::MODULO;
    }

    /// Roll forward the window. Remove one byte from the beginning
    /// of the window and add one byte at the end of the window (if provided)
    pub fn roll_fw(&mut self, prev: u8, next: Option<u8>) {
        self.r1 = (self
            .r1
            .wrapping_sub(prev as u32)
            .wrapping_add(next.map_or(0, u32::from)))
            % RollingSum::MODULO;
        self.r2 = (self
            .r2
            .wrapping_sub(self.l * (prev as u32))
            .wrapping_add(self.r1))
            % RollingSum::MODULO;
        if next.is_none() {
            self.l = self.l.wrapping_sub(1);
        }
    }
}

/// Calculate rolling checksum (weak hash) for given chunk of data
pub fn chunk_rollsum(chunk: &[u8]) -> u32 {
    let mut rolling_sum = RollingSum::new();
    rolling_sum.update(chunk);
    rolling_sum.digest()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn default_constructed() {
        let rs = RollingSum::new();
        assert_eq!(rs.l, 0);
        assert_eq!(rs.r1, 0);
        assert_eq!(rs.r2, 0);
        assert_eq!(rs.digest(), 0u32);
    }

    #[test]
    pub fn rollsum() {
        let mut rs = RollingSum::new();
        rs.update(vec![1, 2, 3, 4].as_slice());
        assert_eq!(rs.l, 4);
        assert_eq!(rs.digest(), 1310730);

        // [1, 2, 3, 4, 5, 6, 7, 8]
        rs.update(vec![5, 6, 7, 8].as_slice());
        assert_eq!(rs.l, 8);
        assert_eq!(rs.digest(), 5242916);

        // Test rolling forward
        //
        // [2, 3, 4, 5, 6, 7, 8, 9]
        rs.roll_fw(1, Some(9));
        assert_eq!(rs.l, 8);
        assert_eq!(rs.digest(), 7602220);

        // Roll forward more
        //
        // [5, 6, 7, 8, 9, 10, 11]
        rs.roll_fw(2, Some(10));
        rs.roll_fw(3, Some(11));
        rs.roll_fw(4, None);
        assert_eq!(rs.l, 7);
        assert_eq!(rs.digest(), 13893688);
    }

    #[test]
    pub fn update() {
        let mut rs = RollingSum::new();

        let mut vec: Vec<u8> = Vec::with_capacity(80);
        for i in 0..vec.capacity() {
            vec.push(i as u8);
        }

        rs.update(vec.as_slice());
        assert_eq!(rs.digest(), 1296567384);
    }

    #[test]
    pub fn rollsum_from_chunk() {
        let vec: Vec<u8> = vec![5; 20];
        assert_eq!(chunk_rollsum(vec.as_slice()), 68812900);
    }
}
