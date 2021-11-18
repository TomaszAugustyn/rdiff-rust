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

    pub fn digest(&self) -> u32 {
        // If we used different modulo, we would have here r = r1 + (r2 * MODULO).
        // Because MODULO is 1 << 16 we can left shift bits also here.
        self.r1 + (self.r2 << 16)
    }

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

pub fn chunk_rollsum(chunk: &[u8]) -> u32 {
    let mut rolling_sum = RollingSum::new();
    rolling_sum.update(chunk);
    rolling_sum.digest()
}
