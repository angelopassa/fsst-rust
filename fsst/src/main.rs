use std::io::Write;
use std::time::Duration;
use std::{env, fs, time::Instant};

mod counters;
mod heap;
mod lossy_pht;
mod symbol;
mod table;
use table::SymbolTable;

/*
    Compression: cargo run --release file_in file_out
*/
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let mut start;
    let mut end;

    let file = fs::read_to_string(&args[1])?;
    let mut output = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&args[2])?;

    let lines = file.lines().map(|line| line.as_bytes()).collect::<Vec<_>>();

    start = Instant::now();
    let st = SymbolTable::build(&lines);
    end = Instant::now().duration_since(start);
    println!(
        "Building speed: {} MB/s",
        (file.len() as f64 / 1024. / 1024.) / end.as_secs_f64()
    );

    end = Duration::ZERO;
    let mut size = 0;
    let mut buffer = Vec::with_capacity(8 * 1024 * 1024);
    let mut buffer_out = Vec::with_capacity(file.len());
    for line in &lines {
        start = Instant::now();
        st.encode(line, &mut buffer);
        end += Instant::now().duration_since(start);
        size += buffer.len();
        st.decode(&buffer, &mut buffer_out);
        writeln!(output, "{}", String::from_utf8(buffer_out.to_vec())?)?;
        buffer.clear();
        buffer_out.clear();
    }

    println!(
        "Compression speed: {} MB/s",
        (file.len() as f64 / 1024. / 1024.) / end.as_secs_f64()
    );

    println!(
        "Compression Ratio: {}",
        lines.iter().map(|x| x.len()).sum::<usize>() as f32 / size as f32
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[test]
    fn table_and_compression() -> Result<(), Box<dyn std::error::Error>> {
        let dir = fs::read_dir("tests.nosync/cwida")?;
        let mut results = String::from(
            "FILE NAME \t| TABLE CONSTRUCTION SPEED | COMPRESSION SPEED | COMPRESSION RATIO\n",
        );

        for tests in dir {
            let mut start;
            let mut end;
            let tests_unw = tests.unwrap();
            let filename = &tests_unw.file_name().into_string().unwrap();
            println!("File: {}", filename);
            results.push_str(&filename);
            results.push_str("|");

            let file = fs::read_to_string(tests_unw.path())?;

            let lines = file.lines().map(|line| line.as_bytes()).collect::<Vec<_>>();

            start = Instant::now();
            let st = SymbolTable::build(&lines);
            end = Instant::now().duration_since(start);
            results.push_str(&(file.len() as f64 / 1024. / 1024. / end.as_secs_f64()).to_string());
            results.push_str("|");

            let mut size = 0;
            let mut buffer = Vec::with_capacity(8 * 1024 * 1024);

            for line in lines.iter() {
                buffer.clear();
                start = Instant::now();
                st.encode(line, &mut buffer);
                end += Instant::now().duration_since(start);
                size += buffer.len();
            }

            results.push_str(&(file.len() as f64 / 1024. / 1024. / end.as_secs_f64()).to_string());
            results.push('|');

            let cr = lines.iter().map(|x| x.len()).sum::<usize>() as f32 / size as f32;
            results.push_str(&cr.to_string());
            results.push('\n');
        }

        let _ = fs::write("results.txt", results);

        Ok(())
    }

    #[test]
    fn decompression() -> Result<(), Box<dyn std::error::Error>> {
        let dir = fs::read_dir("tests.nosync/cwida")?;
        let mut results = String::from("FILE NAME \t| DECOMPRESSION SPEED\n");

        for tests in dir {
            let mut start;
            let mut end;
            let tests_unw = tests.unwrap();
            let filename = &tests_unw.file_name().into_string().unwrap();
            println!("File: {}", filename);
            results.push_str(&filename);
            results.push('|');

            let file = fs::read_to_string(tests_unw.path())?;

            let lines = file.lines().map(|line| line.as_bytes()).collect::<Vec<_>>();

            let st = SymbolTable::build(&lines);

            let mut size = 0;
            let mut time = Duration::ZERO;
            let mut buffer_enc = Vec::with_capacity(8 * 1024 * 1024);
            let mut buffer_dec = Vec::with_capacity(file.len());

            for line in lines.iter() {
                buffer_enc.clear();
                st.encode(line, &mut buffer_enc);
                buffer_dec.clear();
                start = Instant::now();
                st.decode(&buffer_enc, &mut buffer_dec);
                end = Instant::now().duration_since(start);
                size += buffer_dec.len();
                time += end;
            }

            results.push_str(&(size as f64 / 1024. / 1024. / time.as_secs_f64()).to_string());
            results.push('\n');
        }

        let _ = fs::write("decomp.txt", results);

        Ok(())
    }
}
