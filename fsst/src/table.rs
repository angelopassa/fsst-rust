use std::cmp::min;
use std::{collections::BinaryHeap, time::Instant};

use crate::counters::{Counters, TABLE_LENGTH};
use crate::heap::HeapPair;
use crate::lossy_pht::{LossyPHS, TableEntry};
use crate::symbol::Symbol;

const GENERATIONS: [usize; 5] = [8, 38, 68, 98, 128];
const SYMBOL_LENGTH: usize = 8;

pub struct SymbolTable {
    n_symbols: usize,
    symbols_1_byte: [TableEntry; 256],
    symbols_2_byte: Vec<TableEntry>,
    phs: LossyPHS,
    symbols: [Symbol; 2 * TABLE_LENGTH],
    counters: Counters,
}

impl SymbolTable {
    fn new() -> Self {
        let mut symbols = [Symbol::new(); 2 * TABLE_LENGTH];

        for code in 0..TABLE_LENGTH {
            symbols[code].add_char(code as u8);
        }

        Self {
            n_symbols: 0,
            symbols,
            phs: LossyPHS::new(),
            symbols_1_byte: [TableEntry {
                val: 0,
                used: false,
            }; 256],
            symbols_2_byte: vec![
                TableEntry {
                    val: 0,
                    used: false
                };
                65_536
            ],
            counters: Counters::new(),
        }
    }

    fn insert(&mut self, s: Symbol) {
        let idx;

        if s.len == 16 {
            idx = s.first2byte() as usize;
            self.symbols_2_byte[idx].val = TABLE_LENGTH + self.n_symbols;
            self.symbols_2_byte[idx].used = true;
        } else if s.len == 8 {
            idx = s.first1byte() as usize;
            self.symbols_1_byte[idx].val = TABLE_LENGTH + self.n_symbols;
            self.symbols_1_byte[idx].used = true;
        } else if !self.phs.add(s.first3byte(), TABLE_LENGTH + self.n_symbols) {
            return;
        }

        self.symbols[TABLE_LENGTH + self.n_symbols] = s;
        self.n_symbols += 1;
    }

    fn compress_count(&mut self, text: &[u8]) {
        let mut p_start = text.as_ptr();
        let mut p_end;
        let mut prev;
        let mut next_char;
        let mut symbol;

        unsafe {
            p_end = text.as_ptr().add(text.len() - 8);
            symbol = Symbol::with(*(p_start as *const u64), 64);
        }

        let mut code = self.find_longest_symbol(&symbol);

        self.counters.incr_c1(code);

        /*println!(
            "{:?} {:?}",
            String::from_utf8(symbol_to_text(&symbol)),
            String::from_utf8(symbol_to_text(&self.symbols[code])),
        );*/
        unsafe {
            p_start = p_start.add(8);
        }
        //println!("{:?}", String::from_utf8(symbol_to_text(&current_8_byte)));

        if code >= TABLE_LENGTH {
            self.counters.incr_c1(symbol.first1byte() as usize);
        }

        while p_start <= p_end {
            prev = code;

            unsafe {
                symbol = Symbol::with(*(p_start as *const u64), 64);
            }

            code = self.find_longest_symbol(&symbol);

            /*println!(
                "{:?} {:?} {}",
                String::from_utf8(symbol_to_text(&symbol)),
                String::from_utf8(symbol_to_text(&self.symbols[code])),
                self.symbols[code].len / 8
            );*/

            self.counters.incr_c1(code);
            self.counters.incr_c2(prev, code);

            if code >= TABLE_LENGTH {
                next_char = symbol.first1byte() as usize;
                self.counters.incr_c1(next_char);
                self.counters.incr_c2(prev, next_char);
            }

            unsafe {
                p_start = p_start.add(self.symbols[code].len / 8);
            }
        }

        unsafe {
            p_end = text.as_ptr().add(text.len());
        }

        // remaining bytes (less than 8)

        while p_start < p_end {
            prev = code;

            unsafe {
                let offset = 8 * p_end.offset_from(p_start);
                symbol = Symbol::with(*(p_start as *const u64), min(64, offset as usize));
            }

            code = self.find_longest_symbol(&symbol);

            self.counters.incr_c1(code);
            self.counters.incr_c2(prev, code);

            if code >= TABLE_LENGTH {
                next_char = symbol.first1byte() as usize;
                self.counters.incr_c1(next_char);
                self.counters.incr_c2(prev, next_char);
            }

            unsafe {
                p_start = p_start.add(self.symbols[code].len / 8);
            }
        }
    }

    fn make_table(&mut self, sample_frac: usize) {
        let mut cands = BinaryHeap::with_capacity(65_536);
        let mut gain;
        let mut s1;
        let mut s2;
        let mut count;
        let mut length1;

        for code1 in 0..(TABLE_LENGTH + self.n_symbols) {
            count = self.counters.get_from_c1(code1);

            if count < (5 * sample_frac / 128) {
                continue;
            }

            s1 = self.symbols[code1];

            length1 = s1.len / 8;

            gain = length1 * count;

            if code1 < 256 {
                gain *= 8;
            }

            cands.push(HeapPair(gain, s1));

            if sample_frac >= 128 || length1 == SYMBOL_LENGTH {
                continue;
            }

            for code2 in 0..(TABLE_LENGTH + self.n_symbols) {
                s2 = self.symbols[code2];

                if length1 + s2.len / 8 > SYMBOL_LENGTH {
                    continue;
                }

                let new: Symbol = s1.extend(&s2);
                gain = new.len / 8 * self.counters.get_from_c2(code1, code2);
                cands.push(HeapPair(gain, new));
            }
        }

        self.clear();

        while !cands.is_empty() && self.n_symbols < TABLE_LENGTH - 1 {
            let HeapPair(_, sym) = cands.pop().unwrap();
            self.insert(sym);
        }
    }

    pub fn build(text: &[u8]) -> Self {
        let mut st = SymbolTable::new();

        let start = Instant::now();

        for (_, &x) in GENERATIONS.iter().enumerate() {
            st.compress_count(text);
            st.make_table(x);
        }

        let duration = start.elapsed();
        println!("Construction in: {} microsec.", duration.as_micros());

        st
    }

    fn find_longest_symbol(&self, text: &Symbol) -> usize {
        let mut s = self.phs.get(text.first3byte());

        if s.used && text.starts_with(&self.symbols[s.val]) {
            return s.val;
        }

        s = self.symbols_2_byte[text.first2byte() as usize];
        if s.used {
            return s.val;
        }

        s = self.symbols_1_byte[text.first1byte() as usize];
        if s.used {
            return s.val;
        }

        text.first1byte() as usize
    }

    pub fn decode(&self, string: &[u8]) -> Vec<u8> {
        let mut out = Vec::with_capacity(string.len() * SYMBOL_LENGTH);
        let mut p_start: *mut u8 = out.as_mut_ptr();
        let mut size = 0;
        let mut i = 0;

        while i < string.len() {
            if string[i] != 255 {
                unsafe {
                    (p_start as *mut u64)
                        .write_unaligned(self.symbols[TABLE_LENGTH + string[i] as usize].value);
                    p_start = p_start.add(self.symbols[TABLE_LENGTH + string[i] as usize].len / 8);
                    size += self.symbols[TABLE_LENGTH + string[i] as usize].len / 8;
                }

                i += 1;
            } else {
                unsafe {
                    *p_start = string[i + 1];
                    p_start = p_start.add(1);
                    size += 1;
                }

                i += 2;
            }
        }

        unsafe {
            out.set_len(size);
        }

        out
    }

    pub fn encode(&self, string: &[u8]) -> Vec<u8> {
        let mut out: Vec<u8> = Vec::with_capacity(string.len() / 2);

        let mut p_start = string.as_ptr();
        let mut p_end;
        let mut symbol;
        let mut code;

        unsafe {
            p_end = string.as_ptr().add(string.len() - 8);
        }

        while p_start <= p_end {
            unsafe {
                symbol = Symbol::with(*(p_start as *const u64), 64);
            }

            code = self.find_longest_symbol(&symbol);

            if code >= TABLE_LENGTH {
                out.push((code - TABLE_LENGTH) as u8);
            } else {
                out.push(255);
                out.push(symbol.first1byte() as u8);
            }

            unsafe {
                p_start = p_start.add(self.symbols[code].len / 8);
            }
        }

        unsafe {
            p_end = string.as_ptr().add(string.len());
        }

        while p_start < p_end {
            unsafe {
                let offset = 8 * p_end.offset_from(p_start);
                symbol = Symbol::with(*(p_start as *const u64), min(64, offset as usize));
            }

            code = self.find_longest_symbol(&symbol);

            if code >= TABLE_LENGTH {
                out.push((code - TABLE_LENGTH) as u8);
            } else {
                out.push(255);
                out.push(symbol.first1byte() as u8);
            }

            unsafe {
                p_start = p_start.add(self.symbols[code].len / 8);
            }
        }

        out
    }

    fn clear(&mut self) {
        for code in 0..(TABLE_LENGTH + self.n_symbols) {
            let symbol = self.symbols[code];

            if symbol.len == 8 {
                self.symbols_1_byte[symbol.first1byte() as usize] = TableEntry {
                    val: 0,
                    used: false,
                };
            } else if symbol.len == 16 {
                self.symbols_2_byte[symbol.first2byte() as usize] = TableEntry {
                    val: 0,
                    used: false,
                };
            } else {
                self.phs.remove(symbol.first3byte());
            }
        }

        self.counters.clear();

        self.n_symbols = 0;
    }
}
