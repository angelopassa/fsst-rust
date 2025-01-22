const HASH_TABLE_SIZE: usize = 2048;

#[derive(Clone, Copy)]
pub struct TableEntry {
    pub val: usize,
    pub used: bool,
}

pub struct LossyPHS {
    pub table: [TableEntry; HASH_TABLE_SIZE],
}

impl LossyPHS {
    pub fn new() -> Self {
        Self {
            table: [TableEntry {
                val: 0,
                used: false,
            }; HASH_TABLE_SIZE],
        }
    }

    #[inline]
    pub fn add(&mut self, entry: u64, code: usize) -> bool {
        let idx = hash(entry) & (HASH_TABLE_SIZE as u64 - 1);

        if self.table[idx as usize].used {
            false
        } else {
            self.table[idx as usize].val = code;
            self.table[idx as usize].used = true;
            true
        }
    }

    #[inline]
    pub fn get(&self, entry: u64) -> TableEntry {
        let idx = hash(entry) & (HASH_TABLE_SIZE as u64 - 1);

        self.table[idx as usize]
    }

    #[inline]
    pub fn remove(&mut self, entry: u64) {
        let idx = hash(entry) & (HASH_TABLE_SIZE as u64 - 1);

        self.table[idx as usize] = TableEntry {
            val: 0,
            used: false,
        };
    }
}

#[inline]
pub fn hash(value: u64) -> u64 {
    value.wrapping_mul(2971215073) ^ value.wrapping_shr(15)
}
