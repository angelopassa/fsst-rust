use std::{env, fs, time::Instant};

mod counters;
mod heap;
mod lossy_pht;
mod symbol;
mod table;
use table::SymbolTable;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    println!("{:?}", args);
    let file = fs::read_to_string(&args[1])?.as_bytes().to_vec();
    let mut start = Instant::now();
    let all_speed = Instant::now();
    let st = SymbolTable::build(&file);

    let mut end = start.elapsed();
    println!(
        "Building speed: {} MB/s",
        (file.len() as f64 / 1024. / 1024.) / end.as_secs_f64()
    );
    start = Instant::now();
    let encoded = st.encode(&file);
    end = start.elapsed();
    println!(
        "Compression speed: {} MB/s",
        (file.len() as f64 / 1024. / 1024.) / end.as_secs_f64()
    );
    println!("cs: {}", end.as_micros());

    //start = Instant::now();
    let decoded = st.decode(&encoded);
    end = start.elapsed();
    println!(
        "Decompression speed: {} MB/s",
        ((encoded.len() * 8) as f64 / 1024. / 1024.) / end.as_secs_f64()
    );
    let c = all_speed.elapsed();
    println!(
        "All: {} MB/s",
        (file.len() as f64 / 1024. / 1024.) / c.as_secs_f64()
    );

    println!("{}", file.len() as f32 / encoded.len() as f32);

    let _ = fs::write("output.txt", decoded);

    Ok(())
}
