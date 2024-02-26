pub mod bitutils;
mod error;
mod table;
mod tree;
pub mod window;

use crate::bitutils::Symbol;
pub use crate::error::Error;
use crate::table::Table;
use crate::tree::*;
use crate::window::BitWindow;
use std::collections::HashMap;
use std::io::prelude::*;
use std::io::BufWriter;

pub fn hencode(input: &mut impl Read, output: &mut impl Write) -> Result<(), Error> {
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
    let map = tree.make_conversion_map();

    tree.store(output)?;

    let mut encoded = Symbol {
        bytes: Vec::with_capacity(raw.len()),
        bitpos: 0,
        bytepos: 0,
    };

    for byte in raw.iter() {
        encoded.append_sym(
            map.get(byte)
                .expect("every byte variant in raw must be contained in map"),
        );
    }
    assert_eq!(output.write(&[8u8 - encoded.bitpos as u8])?, 1);
    output.write_all(&encoded.bytes)?;
    output.flush()?;

    Ok(())
}

pub fn hdecode(mut input: impl BufRead, output: impl Write) -> Result<(), Error> {
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
        |window: &mut BitWindow<_>, bits: usize, padding: usize| -> Result<_, Error> {
            if window.consume(bits)? && window.initialized() < padding {
                Err(Error::NoBits)
            } else {
                Ok(())
            }
        };
    loop {
        let index = window.show(8);
        let byte = match table[index] {
            table::Entry::Map { byte, bitlen } => {
                consume_err_on_read_padding(&mut window, bitlen, padding)?;
                byte
            }
            table::Entry::Subtable { offset, bitdepth } => {
                consume_err_on_read_padding(&mut window, 8, padding)?;
                // handle subtable entry
                let index = window.show(bitdepth);
                let entry = table[index + offset];
                let table::Entry::Map { byte, bitlen } = entry else {
                    unimplemented!("dont allow nested subtables");
                };

                consume_err_on_read_padding(&mut window, bitlen, padding)?;
                byte
            }
        };

        assert_eq!(output.write(&[byte])?, 1);
        if window.initialized() == padding {
            output.flush()?;
            return Ok(());
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{hdecode, hencode, Error};
    const RAW: &str = r#"
In computer science and information theory, a Huffman code is a particular type of optimal prefix code that is commonly used for lossless data compression. The process of finding or using such a code is Huffman coding, an algorithm developed by David A. Huffman while he was a Sc.D. student at MIT, and published in the 1952 paper "A Method for the Construction of Minimum-Redundancy Codes".[1]

The output from Huffman's algorithm can be viewed as a variable-length code table for encoding a source symbol (such as a character in a file). The algorithm derives this table from the estimated probability or frequency of occurrence (weight) for each possible value of the source symbol. As in other entropy encoding methods, more common symbols are generally represented using fewer bits than less common symbols. Huffman's method can be efficiently implemented, finding a code in time linear to the number of input weights if these weights are sorted.[2] However, although optimal among methods encoding symbols separately, Huffman coding is not always optimal among all compression methods - it is replaced with arithmetic coding[3] or asymmetric numeral systems[4] if a better compression ratio is required. 
"#;

    fn create_coded() -> Result<Vec<u8>, Error> {
        let raw = RAW;
        let mut out = Vec::new();
        let mut reader = raw.as_bytes();
        hencode(&mut reader, &mut out)?;
        Ok(out)
    }

    #[test]
    fn decode() {
        let coded = create_coded().expect("encoding failed. cannot test decoding");
        println!("created coded: {}", String::from_utf8_lossy(&coded));
        let mut out = Vec::new();
        let mut reader = &coded as &[u8];
        hdecode(&mut reader, &mut out).expect("io err");
        println!("created decoded: {}", String::from_utf8_lossy(&out));

        assert_eq!(RAW.as_bytes(), &out, "decoding yielded incorrect data");
    }

    #[test]
    fn encode() {
        create_coded().unwrap();
    }
}
