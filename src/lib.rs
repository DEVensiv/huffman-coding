pub mod bitutils;
mod table;
mod tree;
pub mod window;

use crate::bitutils::Symbol;
use crate::table::Table;
use crate::tree::*;
use crate::window::BitWindow;
use std::collections::HashMap;
use std::error::Error;
use std::io;
use std::io::prelude::*;
use std::io::BufWriter;

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
    let table = Table::from_tree_root(&root).expect("root aint root");
    let mut padding = [0u8];
    input.read_exact(&mut padding)?;
    if input.fill_buf()?.is_empty() {
        return Ok(());
    }
    let padding = padding[0] as usize;
    let mut window: BitWindow<_> = input.into();
    let consume_err_on_read_padding =
        |window: &mut BitWindow<_>, bits: usize, padding: usize| -> Result<_, io::Error> {
            if window.consume(bits)? && window.initialized() < padding {
                Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "Consuming {bits} overlapped with padding",
                ))
            } else {
                Ok(())
            }
        };
    loop {
        let index = window.show_u8() as usize;
        let byte = match table[index] {
            table::Entry::Map { byte, bitlen } => {
                consume_err_on_read_padding(&mut window, bitlen, padding)?;
                byte
            }
            table::Entry::Subtable { offset, bitdepth } => {
                consume_err_on_read_padding(&mut window, 8, padding)?;
                // handle subtable entry
                let index = window.show_exact(bitdepth);
                let entry = table[index + offset];
                let table::Entry::Map { byte, bitlen } = entry else {
                    unimplemented!("dont allow nested subtables");
                };

                consume_err_on_read_padding(&mut window, bitlen, padding)?;
                byte
            }
        };

        debug_assert_eq!(output.write(&[byte])?, 1);
        if window.initialized() == padding {
            output.flush()?;
            return Ok(());
        }
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
