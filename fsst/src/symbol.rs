use std::cmp::min;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Symbol {
    pub value: u64,
    pub len: u32,
}

impl Symbol {
    #[inline]
    pub fn new() -> Self {
        Self { value: 0, len: 0 }
    }

    #[inline]
    pub fn with(value: u64, len: u32) -> Self {
        Self { value, len }
    }

    #[inline]
    pub fn add_char(&mut self, char: u8) {
        self.value |= (char as u64) << (56 - self.len);
        self.len += 8;
    }

    #[inline]
    pub fn starts_with(&self, other: &Self) -> bool {
        return ((self.value ^ other.value) & (u64::MAX << (64 - other.len))) == 0;
    }

    #[inline]
    fn remove_first(&mut self, n: u32) {
        self.value = self.value.checked_shl(n).unwrap_or(0);
        self.len -= n;

        assert!(self.len % 8 == 0);
    }

    #[inline]
    fn add_last(&mut self, other: &Self) {
        self.value |= other.value.checked_shr(self.len).unwrap_or(0);
        self.len += min(64 - self.len, other.len);

        assert!(self.len % 8 == 0);
    }

    #[inline]
    pub fn merge_string(&mut self, other: &Self, pointer: u32, len: u32) {
        self.remove_first(len);
        self.add_last(&Self::with(
            other.value.checked_shl(pointer).unwrap_or(0),
            other.len - pointer,
        ));
    }

    #[inline]
    pub fn extend(&self, other: &Self) -> Self {
        assert!(self.len + other.len <= 64);

        Self::with(self.value | (other.value >> self.len), self.len + other.len)
    }

    #[inline]
    pub fn first3byte(&self) -> u64 {
        self.value >> 40
    }

    #[inline]
    pub fn first2byte(&self) -> u64 {
        self.value >> 48
    }

    #[inline]
    pub fn first1byte(&self) -> u64 {
        assert!(self.len != 0);

        self.value >> 56
    }
}

/*impl Ord for Symbol {
    fn cmp(&self, other: &Self) -> Ordering {
        let (shorter, longer) = if self.len > other.len {
            (other, self)
        } else {
            (self, other)
        };

        for i in 0..(shorter.len / 8) {
            match ((shorter.value << (i * 8)) & (u64::MAX << 56))
                .cmp(&((longer.value << (i * 8)) & (u64::MAX << 56)))
            {
                Ordering::Equal => continue,
                e @ _ => {
                    if shorter == self {
                        return e;
                    } else {
                        return e.reverse();
                    }
                }
            }
        }

        if shorter.len == longer.len {
            Ordering::Equal
        } else if shorter == self {
            Ordering::Less
        } else {
            Ordering::Greater
        }
    }
}

impl PartialOrd for Symbol {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}*/

pub fn text_to_symbols(text: &[u8]) -> Vec<Symbol> {
    let mut s = vec![Symbol::new(); (text.len() as f64 / 8.0).ceil() as usize];
    let mut idx = 0;
    let mut counter = 0;

    for &c in text {
        s[idx].add_char(c);

        counter += 1;

        if counter == 8 {
            counter = 0;
            idx += 1;
        }
    }

    s
}

pub fn symbols_to_text(symbols: &[Symbol]) -> Vec<u8> {
    let mut res = Vec::with_capacity(symbols.len() * 8);

    for &s in symbols {
        res.extend(symbol_to_text(&s));
    }

    res
}

pub fn symbol_to_text(symbol: &Symbol) -> Vec<u8> {
    let mut res = Vec::with_capacity(8);

    for i in 0..(symbol.len / 8) {
        res.push(((symbol.value >> (64 - (i + 1) * 8)) & (u64::MAX >> 56)) as u8);
    }

    res
}
