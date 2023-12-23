pub mod bitutils;
pub mod window;
mod tree;
use bitutils::Symbol;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::error::Error;
use std::fs;
use std::fs::OpenOptions;
use std::io::prelude::*;
use tree::*;

use crate::table::Table;

#[cfg(feature = "table")]
mod table;

pub fn hencode(file: String) -> Result<(), Box<dyn Error>> {
    let raw = fs::read(&file)?;
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
    let file = format!("{}.rxc", file);

    let mut file = OpenOptions::new().create(true).write(true).open(&file)?;

    tree.store(&mut file)?;

    let mut encoded = Symbol {
        bytes: Vec::with_capacity(raw.len()),
        bitpos: 0,
        bytepos: 0,
    };

    for byte in raw.iter() {
        encoded.append_sym(map.get(byte).ok_or("byte vector creation failed")?);
    }
    assert_eq!(file.write(&[9u8 - encoded.bitpos as u8])?, 1);
    file.write_all(&encoded.bytes)?;
    file.flush()?;

    Ok(())
}

pub fn hdecode(file: String) -> Result<(), Box<dyn Error>> {
    let mut source = OpenOptions::new().read(true).open(&file)?;
    let tree = Tree::try_load(&mut source)?;
    let mut target = OpenOptions::new()
        .create(true)
        .write(true)
        .open(&file[..file.len() - 4])?;
    let mut padding = [0u8];
    source.read_exact(&mut padding)?;
    let padding = padding[0] as usize;
    let bits = source.bytes().flat_map(|b| match b {
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
                assert_eq!(target.write(&[key])?, 1);
                tree.walk(bit)
            }
        };
    }
    match node {
        Walker::End(key) => {
            assert_eq!(target.write(&[key])?, 1);
        }
        _ => Err("decoding failed")?,
    }

    Ok(())
}
