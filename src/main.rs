use huffman::*;
use std::env;
use std::fs::OpenOptions;
use std::io::BufReader;
use std::process::exit;

fn main() -> Result<(), Error> {
    let mut args = env::args().skip(1);
    if let Some(arg) = args.next() {
        if arg == "--help" {
            println!("Usage: huffmann <file> [-d]");
            return Ok(());
        }
        let mut input = OpenOptions::new().read(true).open(&arg)?;

        if let Some(option) = args.next() {
            if option == "-d" {
                let mut output = OpenOptions::new()
                    .create(true)
                    .write(true)
                    .open(&arg[..arg.len() - 4])?;

                let mut input = BufReader::new(input);
                hdecode(&mut input, &mut output)?;
                return Ok(());
            }
        }

        let mut output = OpenOptions::new()
            .create(true)
            .write(true)
            .open(format!("{}.rxc", arg))?;
        hencode(&mut input, &mut output)?;
        Ok(())
    } else {
        eprintln!("[0] Please supply a file argument");
        exit(1);
    }
}
