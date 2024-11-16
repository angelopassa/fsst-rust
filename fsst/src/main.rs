use std::{collections::BinaryHeap, vec};

fn main() {
    let mut st = SymbolTable::build(&"tumcwitumvldb".chars().collect());
    //println!("{:?}", st.symbols.split_off(255));
}

const NR_GENERATION: u8 = 5;
const TABLE_LENGTH: usize = 256;
const SYMBOL_LENGTH: usize = 8;

pub struct SymbolTable {
    n_symbols: u8,
    s_index: Vec<u16>,
    symbols: Vec<Vec<char>>,
}

impl SymbolTable {
    fn new() -> Self {
        let mut symbols = vec![vec!['\"'; SYMBOL_LENGTH]; 2 * TABLE_LENGTH];
        for code in 0..TABLE_LENGTH {
            symbols[code] = (char::from_u32(code as u32))
                .unwrap()
                .to_string()
                .chars()
                .collect();
        }
        Self {
            n_symbols: 0,
            s_index: vec![u16::MAX; TABLE_LENGTH + 1],
            symbols,
        }
    }

    fn insert(&mut self, s: Vec<char>) {
        self.symbols[TABLE_LENGTH + self.n_symbols as usize] = s.clone();
        self.n_symbols += 1;
    }

    fn compress_count(&self, count1: &mut Vec<u64>, count2: &mut Vec<Vec<u64>>, text: &Vec<char>) {
        println!("{:?}", self.s_index);
        let mut current_pos = 0;
        let mut code = self.find_longest_symbol(text) as usize;
        println!(
            "{} {:?} {}",
            code,
            self.symbols[code],
            self.symbols[code].len()
        );
        let mut prev;
        let mut next_char;

        current_pos += self.symbols[code as usize].len();
        count1[code] += 1;
        while current_pos < text.len() {
            prev = code;
            code = self.find_longest_symbol(&text[current_pos..text.len()].to_vec()) as usize;

            println!(
                "{} {} {:?}",
                code,
                char::from_u32(code as u32).unwrap(),
                self.symbols[code]
            );
            count1[code] += 1;
            println!("{}", count1[code]);
            count2[prev][code] += 1;

            if code >= TABLE_LENGTH {
                //maybe also for the first
                next_char = text[current_pos] as usize;
                count1[next_char] += 1;
                count2[prev][next_char] += 1;
            }

            current_pos += self.symbols[code as usize].len();
        }
    }

    fn make_table(&self, count1: Vec<u64>, count2: Vec<Vec<u64>>) -> Self {
        let mut new_table = SymbolTable::new();
        let mut cands = BinaryHeap::new();
        let mut gain;
        let mut s;

        for code1 in 0..TABLE_LENGTH + self.n_symbols as usize {
            gain = self.symbols[code1].len() * count1[code1] as usize;
            cands.push((gain, self.symbols[code1].clone()));
            for code2 in 0..TABLE_LENGTH + self.n_symbols as usize {
                s = self.symbols[code1].clone();
                s.extend(&self.symbols[code2]);
                s = s.into_iter().take(SYMBOL_LENGTH).collect();
                gain = s.len() * count2[code1][code2] as usize;
                cands.push((gain, s.clone()));
            }
        }

        //println!("{:?}", cands);
        while (new_table.n_symbols as usize) < TABLE_LENGTH - 1 {
            let (zero, uno) = cands.pop().unwrap();
            println!("{} {:?}", zero, uno);
            new_table.insert(uno);
        }

        new_table.make_index();

        println!("{:?}", new_table.symbols);

        new_table
    }

    pub fn build(text: &Vec<char>) -> Self {
        let mut st = SymbolTable::new();

        for i in 0..3 {
            println!("Iteration Nr. {}", i + 1);
            let mut count1: Vec<u64> = vec![0; 2 * TABLE_LENGTH];
            let mut count2: Vec<Vec<u64>> = vec![vec![0; 2 * TABLE_LENGTH]; 2 * TABLE_LENGTH];
            st.compress_count(&mut count1, &mut count2, text);
            st = st.make_table(count1, count2);
        }

        st
    }

    /*
        Method for sorting the symbols in the real table.
        Puts in `s_index[x]` the index of first symbol in `symbols` which starts with `x`
    */
    fn make_index(&mut self) {
        self.symbols[TABLE_LENGTH..TABLE_LENGTH + self.n_symbols as usize]
            .sort_by(|a, b| Ord::cmp(b, a));
        println!("{:?}", self.symbols.clone().split_off(TABLE_LENGTH - 1));

        for i in (TABLE_LENGTH..(TABLE_LENGTH + (self.n_symbols as usize))).rev() {
            self.s_index[self.symbols[i][0] as usize] = i as u16;
        }

        self.s_index[TABLE_LENGTH] = TABLE_LENGTH as u16 + (self.n_symbols as u16);
    }

    pub fn find_longest_symbol(&self, text: &Vec<char>) -> u16 {
        let first_char = text[0] as usize;

        for code in self.s_index[first_char]..self.s_index[first_char - 1] {
            if text.starts_with(&self.symbols[code as usize]) {
                return code;
            }
        }

        return first_char as u16;
    }
}
