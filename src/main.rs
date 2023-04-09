use huffman::*;
use std::env;
use std::error::Error;
use std::process::exit;

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args().skip(1);
    if let Some(file) = args.next() {
        if file == "--help" {
            println!("Usage: huffmann <file> [-d]");
            return Ok(());
        }
        if let Some(option) = args.next() {
            if option == "-d" {
                hdecode(file)?;
                return Ok(());
            }
        }
        hencode(file)?;
        Ok(())
    } else {
        eprintln!("[0] Please supply a file argument");
        exit(1);
    }
}
