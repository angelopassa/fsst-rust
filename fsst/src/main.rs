use std::{
    cmp::Ordering,
    collections::{BinaryHeap, HashSet},
    fs,
    time::Instant,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut file = fs::read_to_string("tests.nosync/cwida/c_name2")?
        .as_bytes()
        .to_vec();
    let original = file.clone();

    while file.len() < 8_388_608 {
        file.extend(&original);
    }

    //let file = "tumcwitumvldb".as_bytes().to_vec();
    let st = SymbolTable::build(&file);

    let mut start = Instant::now();
    let encoded = st.encode(&file);
    let mut duration = start.elapsed();
    println!(
        "Encoding: {:?} MB/s",
        (file.len() as f32 / (1024. * 1024.)) / duration.as_secs_f32()
    );
    let mut size = "GB";
    let mut div = 1024. * 1024. * 1024.;
    let len = file.len() as f32;
    if len / div < 1. {
        size = "MB";
        div /= 1024.;
    }
    if len / div <= 1. {
        size = "KB";
        div /= 1024.;
    }

    println!(
        "Size Real ({}): {} | Size Encoded ({}): {} | CR: {}",
        size,
        file.len() as f32 / div,
        size,
        encoded.len() as f32 / div,
        file.len() as f32 / encoded.len() as f32
    );

    start = Instant::now();
    let decoded = st.decode(&encoded);
    duration = start.elapsed();
    println!(
        "Decoding: {:?} MB/s",
        (decoded.len() as f32 / (1024. * 1024.)) / duration.as_secs_f32()
    );

    assert_eq!(file, decoded);

    println!(
        "{:?}",
        st.symbols[TABLE_LENGTH..(TABLE_LENGTH + st.n_symbols)]
            .iter()
            .map(|symbol| symbol.iter().map(|&byte| byte as char).collect())
            .collect::<Vec<String>>()
    );

    Ok(())
}

const NR_GENERATION: u8 = 5;
const TABLE_LENGTH: usize = 256;
const SYMBOL_LENGTH: usize = 8;

pub struct SymbolTable {
    n_symbols: usize,
    s_index: [(usize, usize); TABLE_LENGTH],
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
            s_index: [(0, 0); TABLE_LENGTH],
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
        text: &[u8],
    ) {
        let mut current_pos = 0;
        let mut code = self.find_longest_symbol(text);
        let mut prev;
        let mut next_char;

        count1[code] += 1;
        current_pos += self.symbols[code].len();

        if code >= TABLE_LENGTH {
            count1[text[0] as usize] += 1;
        }

        while current_pos < text.len() {
            prev = code;
            code = self.find_longest_symbol(&text[current_pos..]);

            count1[code] += 1;
            count2[prev][code] += 1;

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
            cands.push(HeapPair(gain, self.symbols[code1].clone()));

            for code2 in 0..(TABLE_LENGTH + self.n_symbols) {
                s = self.symbols[code1].clone();
                s.extend(&self.symbols[code2]);
                s = s[..s.len().min(SYMBOL_LENGTH)].to_vec();
                gain = s.len() * count2[code1][code2];
                cands.push(HeapPair(gain, s));
            }
        }

        let mut alredy_in = HashSet::with_capacity(TABLE_LENGTH);
        while new_table.n_symbols < TABLE_LENGTH - 1 {
            let HeapPair(_, sym) = cands.pop().unwrap();
            if !alredy_in.contains(&sym) {
                alredy_in.insert(sym.clone());
                new_table.insert(sym);
            }
        }

        new_table.make_index();

        new_table
    }

    pub fn build(text: &[u8]) -> Self {
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
        println!("Construction in: {} sec.", duration.as_secs_f64());

        st
    }

    fn make_index(&mut self) {
        self.symbols[TABLE_LENGTH..(TABLE_LENGTH + self.n_symbols)].sort_by(|a, b| Ord::cmp(b, a));

        for i in (TABLE_LENGTH..(TABLE_LENGTH + self.n_symbols)).rev() {
            self.s_index[self.symbols[i][0] as usize].0 = i;
        }

        for i in TABLE_LENGTH..(TABLE_LENGTH + self.n_symbols) {
            self.s_index[self.symbols[i][0] as usize].1 = i + 1;
        }
    }

    pub fn find_longest_symbol(&self, text: &[u8]) -> usize {
        let first_char = text[0] as usize;

        let range = self.s_index[first_char];
        for code in range.0..range.1 {
            if text.starts_with(&self.symbols[code]) {
                return code;
            }
        }

        first_char
    }

    pub fn decode(&self, string: &[u8]) -> Vec<u8> {
        let mut out: Vec<u8> = Vec::with_capacity(string.len() * SYMBOL_LENGTH);
        let mut pos = 0;
        while pos < string.len() {
            if string[pos] != 255 {
                out.extend(&self.symbols[TABLE_LENGTH + string[pos] as usize]);
                pos += 1;
            } else {
                out.push(string[pos + 1]);
                pos += 2;
            }
        }

        out
    }

    pub fn encode(&self, string: &[u8]) -> Vec<u8> {
        let mut out = Vec::with_capacity(string.len() / 2);
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

#[derive(Eq, PartialEq)]
struct HeapPair(usize, Vec<u8>);

impl Ord for HeapPair {
    fn cmp(&self, other: &Self) -> Ordering {
        let ord = self.0.cmp(&other.0);
        match ord {
            Ordering::Equal => self.1.len().cmp(&other.1.len()),
            _ => ord,
        }
    }
}

impl PartialOrd for HeapPair {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
