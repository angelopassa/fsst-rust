use std::{collections::BinaryHeap, time::Instant};

use crate::counters::{Counters, TABLE_LENGTH};
use crate::heap::HeapPair;
use crate::lossy_pht::{LossyPHS, TableEntry};
use crate::symbol::{symbol_to_text, Symbol};

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

    fn compress_count(&mut self, text: &[Symbol]) {
        let mut current_char = 1;
        let mut current_8_byte = text[0];
        let mut pointer_succ = 0;
        let mut code = self.find_longest_symbol(&current_8_byte);
        let mut prev;
        let mut next_char;

        self.counters.incr_c1(code);

        /*println!(
            "{:?} {:?} {:?}",
            String::from_utf8(symbol_to_text(&current_8_byte)),
            String::from_utf8(symbol_to_text(&self.symbols[code])),
            String::from_utf8(symbol_to_text(&text[current_char]))
        );*/
        current_8_byte.merge_string(&text[current_char], pointer_succ, self.symbols[code].len);
        //println!("{:?}", String::from_utf8(symbol_to_text(&current_8_byte)));

        pointer_succ += self.symbols[code].len;

        if pointer_succ >= text[current_char].len {
            current_char += 1;
            pointer_succ = 0;
        }

        if code >= TABLE_LENGTH {
            self.counters.incr_c1(text[0].first1byte() as usize);
        }

        while current_char < text.len() {
            assert!(pointer_succ % 8 == 0 && pointer_succ < 64);

            prev = code;
            code = self.find_longest_symbol(&current_8_byte);

            self.counters.incr_c1(code);
            self.counters.incr_c2(prev, code);

            if code >= TABLE_LENGTH {
                next_char = current_8_byte.first1byte() as usize;
                self.counters.incr_c1(next_char);
                self.counters.incr_c2(prev, next_char);
            }

            current_8_byte.merge_string(&text[current_char], pointer_succ, self.symbols[code].len);
            pointer_succ += self.symbols[code].len;

            if pointer_succ >= text[current_char].len {
                current_char += 1;

                if pointer_succ > text[current_char - 1].len && current_char < text.len() {
                    current_8_byte.merge_string(&text[current_char], 0, 0);
                }

                /*if text[current_char - 1].len != 64 {
                    println!("last chunk");
                }*/

                pointer_succ -= text[current_char - 1].len;
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

            length1 = (s1.len / 8) as usize;

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

                if length1 + (s2.len / 8) as usize > SYMBOL_LENGTH {
                    continue;
                }

                let new: Symbol = s1.extend(&s2);
                gain = (new.len / 8) as usize * self.counters.get_from_c2(code1, code2);
                cands.push(HeapPair(gain, new));
            }
        }

        self.clear();

        while !cands.is_empty() && self.n_symbols < TABLE_LENGTH - 1 {
            let HeapPair(_, sym) = cands.pop().unwrap();
            self.insert(sym);
        }
    }

    pub fn build(text: &[Symbol]) -> Self {
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

    pub fn decode(&self, string: &[Symbol]) -> Vec<u8> {
        let mut out = Vec::with_capacity(string.len() * SYMBOL_LENGTH * SYMBOL_LENGTH);
        let mut get_char = false;

        for pos in 0..string.len() {
            for idx in 0..(string[pos].len / 8) {
                let c = ((string[pos].value >> (56 - idx * 8))
                    & (0b00000000_00000000_00000000_11111111)) as u8;
                if get_char {
                    out.push(c);
                    get_char = false;
                } else if c == 255 {
                    get_char = true;
                } else {
                    out.extend(&symbol_to_text(&self.symbols[TABLE_LENGTH + c as usize]));
                }
            }
        }

        out
    }

    pub fn encode(&self, string: &[Symbol]) -> Vec<Symbol> {
        let mut out: Vec<Symbol> = Vec::with_capacity(string.len() / 2);
        let mut pos = 1;
        let mut pos_l = 0;
        let mut current_8_byte = string[0];
        let mut s;
        let mut idx_out = 0;
        out.push(Symbol::new());
        let mut nr_chunk = 0;

        while pos < string.len() {
            if nr_chunk >= 8 {
                nr_chunk = 0;
                idx_out += 1;
                out.push(Symbol::new());
            }

            s = self.find_longest_symbol(&current_8_byte);
            if s >= TABLE_LENGTH {
                out[idx_out].add_char((s - TABLE_LENGTH) as u8);
                nr_chunk += 1;
            } else {
                out[idx_out].add_char(255);

                if nr_chunk >= 8 {
                    nr_chunk = 1;
                    idx_out += 1;
                    out.push(Symbol::new());
                } else {
                    nr_chunk += 2;
                }

                out[idx_out].add_char(current_8_byte.first1byte() as u8);
            }

            current_8_byte.merge_string(&string[pos], pos_l, self.symbols[s].len);
            pos_l += self.symbols[s].len;

            if pos_l >= string[pos].len {
                pos += 1;

                if pos_l > string[pos - 1].len && pos < string.len() {
                    current_8_byte.merge_string(&string[pos], 0, 0);
                }

                /*if string[pos - 1].len != 64 {
                    println!("last chunk");
                }*/

                pos_l -= string[pos - 1].len;
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
