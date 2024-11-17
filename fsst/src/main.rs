use std::{collections::BinaryHeap, fs, time::Instant};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file = fs::read_to_string("text.txt")?;

    SymbolTable::build(&file.as_bytes().to_vec());

    Ok(())
}

//FIND_LONG problem

const NR_GENERATION: u8 = 5;
const TABLE_LENGTH: usize = 256;
const SYMBOL_LENGTH: usize = 8;

pub struct SymbolTable {
    n_symbols: usize,
    s_index: [usize; TABLE_LENGTH],
    symbols: Vec<Vec<u8>>,
}

impl SymbolTable {
    fn new() -> Self {
        let mut symbols = Vec::with_capacity(2 * TABLE_LENGTH);
        for code in 0..TABLE_LENGTH {
            symbols.push(Vec::from([code as u8]));
        }

        for _ in TABLE_LENGTH..2 * TABLE_LENGTH {
            symbols.push(Vec::from([0]));
        }

        Self {
            n_symbols: 0,
            s_index: [usize::MAX; TABLE_LENGTH],
            symbols,
        }
    }

    fn insert(&mut self, s: Vec<u8>) {
        self.symbols[TABLE_LENGTH + self.n_symbols] = s;
        self.n_symbols += 1;
    }

    fn compress_count(
        &self,
        count1: &mut [usize; 2 * TABLE_LENGTH],
        count2: &mut [[usize; 2 * TABLE_LENGTH]; 2 * TABLE_LENGTH],
        text: &Vec<u8>,
    ) {
        let mut current_pos = 0;
        let mut code = self.find_longest_symbol(text);
        let mut prev;
        let mut next_char;

        count1[code] += 1;
        current_pos += self.symbols[code].len();
        //print!("pos: {} char_n: {} ", current_pos, code);
        //println!("char: {:?}", String::from_utf8(self.symbols[code].to_vec()));

        if code >= TABLE_LENGTH {
            count1[text[0] as usize] += 1;
        }

        while current_pos < text.len() {
            prev = code;
            code = self.find_longest_symbol(&text[current_pos..]);
            //println!("prev: {} code: {}", prev, code);

            count1[code] += 1;
            count2[prev][code] += 1;

            //print!("pos: {} char_n: {} ", current_pos, code);
            //println!("char: {:?}", String::from_utf8(self.symbols[code].to_vec()));
            if code >= TABLE_LENGTH {
                next_char = text[current_pos] as usize;
                count1[next_char] += 1;
                count2[prev][next_char] += 1;
            }

            current_pos += self.symbols[code].len();
        }
    }

    fn make_table(
        self,
        count1: [usize; 2 * TABLE_LENGTH],
        count2: [[usize; 2 * TABLE_LENGTH]; 2 * TABLE_LENGTH],
    ) -> Self {
        let mut new_table = SymbolTable::new();
        let mut cands = BinaryHeap::with_capacity(TABLE_LENGTH);
        let mut gain;
        let mut s;

        for code1 in 0..(TABLE_LENGTH + self.n_symbols) {
            gain = self.symbols[code1].len() * count1[code1];
            cands.push((gain, self.symbols[code1].clone()));

            for code2 in 0..(TABLE_LENGTH + self.n_symbols) {
                s = self.symbols[code1].clone();
                s.extend(&self.symbols[code2]);
                s = s[..s.len().min(SYMBOL_LENGTH)].to_vec();
                gain = s.len() * count2[code1][code2];
                cands.push((gain, s));
            }
        }

        while new_table.n_symbols < TABLE_LENGTH - 1 {
            new_table.insert(cands.pop().unwrap().1);
        }

        new_table.make_index();

        /*println!(
            "{:?}",
            new_table
                .symbols
                .iter()
                .map(|el| el.iter().map(|byte| *byte as char).collect())
                .collect::<Vec<String>>()
        );*/

        new_table
    }

    pub fn build(text: &Vec<u8>) -> Self {
        let start = Instant::now();
        let mut st = SymbolTable::new();

        for i in 0..NR_GENERATION {
            println!("Iteration Nr. {}", i + 1);
            let mut count1: [usize; 2 * TABLE_LENGTH] = [0; 2 * TABLE_LENGTH];
            let mut count2: [[usize; 2 * TABLE_LENGTH]; 2 * TABLE_LENGTH] =
                [[0; 2 * TABLE_LENGTH]; 2 * TABLE_LENGTH];
            st.compress_count(&mut count1, &mut count2, text);
            st = st.make_table(count1, count2);
        }

        let duration = start.elapsed();
        println!("Construction in: {}", duration.as_secs_f64());

        st
    }

    /*
        Method for sorting the symbols in the real table.
        Puts in `s_index[x]` the index of first symbol in `symbols` which starts with `x`
    */
    fn make_index(&mut self) {
        self.symbols[TABLE_LENGTH..(TABLE_LENGTH + self.n_symbols)].sort_by(|a, b| Ord::cmp(b, a));

        for i in (TABLE_LENGTH..(TABLE_LENGTH + self.n_symbols)).rev() {
            self.s_index[self.symbols[i][0] as usize] = i;
        }
    }

    pub fn find_longest_symbol(&self, text: &[u8]) -> usize {
        let first_char = text[0] as usize;

        for code in self.s_index[first_char]..self.s_index[first_char - 1] {
            println!("{} {:?}", code, self.symbols[code as usize]);
            if text.starts_with(&self.symbols[code as usize]) {
                return code as usize;
            }
        }

        first_char
    }

    pub fn decode(&self, string: &Vec<u8>) -> Vec<u8> {
        let mut out: Vec<u8> = Vec::with_capacity(TABLE_LENGTH);
        let mut pos = 0;
        while pos < string.len() {
            if string[pos] != 255 {
                out.extend(&self.symbols[TABLE_LENGTH + pos]);
                pos += 1;
            } else {
                out.push(string[pos + 1]);
                pos += 2;
            }
        }

        out
    }

    pub fn endcode(&self, string: &Vec<u8>) -> Vec<u8> {
        let mut out = Vec::with_capacity(TABLE_LENGTH);
        let mut pos = 0;
        let mut s;

        while pos < string.len() {
            s = self.find_longest_symbol(&string[pos..]);
            if s >= TABLE_LENGTH {
                out.push((s - TABLE_LENGTH) as u8);
            } else {
                out.push(255);
                out.push(string[pos]);
            }

            pos += self.symbols[s].len();
        }

        out
    }
}
