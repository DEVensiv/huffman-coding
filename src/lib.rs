pub mod bitutils;
mod tree;
#[cfg(feature = "window")]
pub mod window;
use bitutils::Symbol;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::error::Error;
use std::io::BufWriter;
use std::io::prelude::*;
use tree::*;

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
    assert_eq!(output.write(&[9u8 - encoded.bitpos as u8])?, 1);
    output.write_all(&encoded.bytes)?;
    output.flush()?;

    Ok(())
}

pub fn hdecode(mut input: impl BufRead, output: impl Write) -> Result<(), Box<dyn Error>> {
    let mut output = BufWriter::new(output);
    let tree = Tree::try_load(&mut input)?;
    let mut padding = [0u8];
    input.read_exact(&mut padding)?;
    let padding = padding[0] as usize;
    let bits = input.bytes().flat_map(|b| match b {
        Ok(byte) => bitutils::mk_bits(byte),
        Err(_) => Vec::new(),
    });
    let mut node = Walker::No;
    let mut ring_buffer = VecDeque::with_capacity(padding);
    for bit in bits {
        ring_buffer.push_back(bit);
        if ring_buffer.len() != padding {
            continue;
        }
        let bit = ring_buffer.pop_front().unwrap();
        node = match node {
            Walker::No => tree.walk(bit),
            Walker::Next(node) => node.walk(bit),
            Walker::End(key) => {
                assert_eq!(output.write(&[key])?, 1);
                tree.walk(bit)
            }
        };
    }
    match node {
        Walker::End(key) => {
            assert_eq!(output.write(&[key])?, 1);
        }
        _ => Err("decoding failed")?,
    }

    Ok(())
}
