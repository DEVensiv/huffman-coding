pub mod bitutils;
mod tree;
pub mod window;
use bitutils::Symbol;
use std::collections::HashMap;
use std::error::Error;
use std::io;
use std::io::prelude::*;
use std::io::BufWriter;
use tree::*;
use window::BitWindow;

#[cfg(feature = "table")]
mod table;

#[cfg(feature = "table")]
use crate::table::Table;

pub fn hencode(input: &mut impl Read, output: &mut impl Write) -> Result<(), Box<dyn Error>> {
    let mut raw = Vec::new();
    input.read_to_end(&mut raw)?;
    let mut map = HashMap::new();
    for byte in raw.iter() {
        let entry = map.entry(byte).or_insert(0usize);
        *entry += 1;
    }
    let freq: Vec<Tree> = map
        .into_iter()
        .map(|(&key, value)| Tree::Leaf(key, value))
        .collect();

    let tree = Tree::mktree(freq);
    let map = tree.make_conversion_map().ok_or("map creation failed")?;

    tree.store(output)?;

    let mut encoded = Symbol {
        bytes: Vec::with_capacity(raw.len()),
        bitpos: 0,
        bytepos: 0,
    };

    for byte in raw.iter() {
        encoded.append_sym(map.get(byte).ok_or("byte vector creation failed")?);
    }
    assert_eq!(output.write(&[8u8 - encoded.bitpos as u8])?, 1);
    output.write_all(&encoded.bytes)?;
    output.flush()?;

    Ok(())
}

pub fn hdecode(mut input: impl BufRead, output: impl Write) -> Result<(), Box<dyn Error>> {
    let mut output = BufWriter::new(output);
    let root = Tree::try_load(&mut input)?;
    let mut padding = [0u8];
    input.read_exact(&mut padding)?;
    if input.fill_buf()?.is_empty() {
        return Ok(());
    }
    let padding = padding[0] as usize;
    let mut window: BitWindow<_> = input.into();
    let mut walker = root.walk(window.show_exact(1) != 0);
    window.consume(1)?;
    loop {
        walker = match walker {
            Walker::Next(node) => {
                let bit = window.show_exact(1) != 0;
                // check if smaller then padding since if its equal to padding `bit` is still data
                if window.consume(1)? && window.initialized() < padding {
                    return Err(Box::new(io::Error::other(
                        "Read into padding, invalid encoding",
                    )));
                }
                node.walk(bit)
            }
            Walker::End(key) => {
                debug_assert_eq!(output.write(&[key])?, 1);
                // if `initialized == padding` that means EOF and no more bits (since padding < 8)
                if window.initialized() == padding {
                    output.flush()?;
                    return Ok(());
                }
                // read another bit if we are not at EOF
                let bit = window.show_exact(1) != 0;
                // dont need to check for "read into padding" since we know there is at least one
                // more bit because of the EOF check above
                window.consume(1)?;
                root.walk(bit)
            }
        };
    }
}

#[cfg(test)]
mod tests {
    use std::{
        error::Error,
        fs::{self, OpenOptions},
        io::{BufReader, Read, Seek},
    };
    use tempfile::tempfile;

    use crate::{hdecode, hencode};
    const RAW: &str = "flake.lock";
    const CODED: &str = "flake.lock.rxc";

    fn create_decoded() -> Result<(), Box<dyn Error>> {
        let raw = OpenOptions::new().read(true).open(RAW)?;
        let _ = fs::remove_file(CODED);
        let mut out = OpenOptions::new().write(true).create(true).open(CODED)?;
        let mut reader = BufReader::new(raw);
        hencode(&mut reader, &mut out)
    }

    #[test]
    fn decode() {
        create_decoded().expect("encoding failed. cannot test decoding");

        let mut out = tempfile().expect("temfile err");
        let raw = OpenOptions::new().read(true).open(CODED).expect("file err");
        let mut reader = BufReader::new(raw);
        hdecode(&mut reader, &mut out).expect("io err");

        out.seek(std::io::SeekFrom::Start(0))
            .expect("could not seek tmpfile");
        let mut raw = OpenOptions::new().read(true).open(RAW).expect("cant read");
        let mut raw_data = Vec::new();
        raw.read_to_end(&mut raw_data).expect("cant read");
        let mut out_data = Vec::new();
        out.read_to_end(&mut out_data).expect("cant read tmpfile");

        assert_eq!(raw_data, out_data, "decoding yielded incorrect data");
    }

    #[test]
    fn encode() {
        let mut out = tempfile().expect("temfile err");
        let raw = OpenOptions::new().read(true).open(RAW).expect("file err");
        let mut reader = BufReader::new(raw);
        hencode(&mut reader, &mut out).expect("io err");
    }
}
