pub const TABLE_LENGTH: usize = 256;
const COUNTER1_LENGTH: usize = 2 * TABLE_LENGTH;
const COUNTER2_LENGTH: usize = COUNTER1_LENGTH * COUNTER1_LENGTH;

pub struct Counters {
    pub counter1: Vec<usize>,
    pub counter2: Vec<usize>,
    bitmap1: [u64; COUNTER1_LENGTH / 64],
    bitmap2: [u64; COUNTER2_LENGTH / 64],
}

impl Counters {
    pub fn new() -> Self {
        let mut c1 = Vec::with_capacity(COUNTER1_LENGTH);
        let mut c2 = Vec::with_capacity(COUNTER2_LENGTH);

        unsafe {
            c1.set_len(COUNTER1_LENGTH);
            c2.set_len(COUNTER2_LENGTH);
        }

        Self {
            counter1: c1,
            counter2: c2,
            bitmap1: [0; COUNTER1_LENGTH / 64],
            bitmap2: [0; COUNTER2_LENGTH / 64],
        }
    }

    pub fn incr_c1(&mut self, idx: usize) {
        if self.is_set_c1(idx) {
            self.counter1[idx] += 1;
        } else {
            self.counter1[idx] = 1;
            self.set_c1(idx);
        }
    }

    pub fn incr_c2(&mut self, idx1: usize, idx2: usize) {
        assert!(idx1 < COUNTER1_LENGTH && idx2 < COUNTER1_LENGTH);

        let idx = idx1 * COUNTER1_LENGTH + idx2;

        if self.is_set_c2(idx) {
            self.counter2[idx] += 1;
        } else {
            self.counter2[idx] = 1;
            self.set_c2(idx);
        }
    }

    #[inline]
    fn is_set_c1(&self, idx: usize) -> bool {
        assert!(idx < COUNTER1_LENGTH);

        let bucket = idx % (COUNTER1_LENGTH / 64);

        return self.bitmap1[bucket] & (1 << (idx % 64)) != 0;
    }

    #[inline]
    fn set_c1(&mut self, idx: usize) {
        assert!(idx < COUNTER1_LENGTH);

        let bucket = idx % (COUNTER1_LENGTH / 64);

        self.bitmap1[bucket] += 1 << (idx % 64);
    }

    #[inline]
    fn is_set_c2(&self, idx: usize) -> bool {
        assert!(idx < COUNTER2_LENGTH);

        let bucket = idx % (COUNTER2_LENGTH / 64);

        return self.bitmap2[bucket] & (1 << (idx % 64)) != 0;
    }

    #[inline]
    fn set_c2(&mut self, idx: usize) {
        assert!(idx < COUNTER2_LENGTH);

        let bucket = idx % (COUNTER2_LENGTH / 64);

        self.bitmap2[bucket] += 1 << (idx % 64);
    }

    #[inline]
    pub fn get_from_c1(&self, idx: usize) -> usize {
        assert!(idx < COUNTER1_LENGTH);

        self.counter1[idx]
    }

    #[inline]
    pub fn get_from_c2(&self, idx1: usize, idx2: usize) -> usize {
        assert!(idx1 < COUNTER1_LENGTH && idx2 < COUNTER1_LENGTH);

        self.counter2[idx1 * COUNTER1_LENGTH + idx2]
    }

    pub fn clear(&mut self) {
        for i in 0..self.bitmap1.len() {
            self.bitmap1[i] = 0;
        }

        for i in 0..self.bitmap2.len() {
            self.bitmap2[i] = 0;
        }
    }
}
