mod pgn_cleaner;
mod pgn_preprocessor;
mod test;

use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;

use pgn_preprocessor::PgnProcessor;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let input = if args.len() > 1 {
        // Read from file
        let path = Path::new(&args[1]);
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut pgn = String::new();
        for line in reader.lines() {
            pgn.push_str(&line?);
            pgn.push(' ');
        }
        pgn
    } else {
        // Read from stdin
        println!("Enter PGN (press Ctrl+D when done):");
        let stdin = io::stdin();
        let mut pgn = String::new();
        for line in stdin.lock().lines() {
            pgn.push_str(&line?);
            pgn.push(' ');
        }
        pgn
    };

    let mut processor = PgnProcessor::new();
    let processed_moves = processor.process_pgn(&input);

    println!("Processed moves:");
    for (i, mv) in processed_moves.iter().enumerate() {
        if i % 2 == 0 {
            print!("{}. ", i / 2 + 1);
        }
        print!("{mv} ");
        if i % 2 == 1 {
            println!();
        }
    }
    if processed_moves.len() % 2 == 1 {
        println!();
    }

    // Write output to file if desired
    if args.len() > 2 {
        let mut output_file = File::create(&args[2])?;
        writeln!(output_file, "{}", processed_moves.join(" "))?;
        println!("Output written to {}", &args[2]);
    }

    Ok(())
}
