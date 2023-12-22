use std::fmt::Display;

use crate::{bitutils::Symbol, tree::Tree};

#[derive(Clone, Copy, Debug)]
pub enum Entry {
    Map { byte: u8, bitlen: usize },
    Subtable { offset: usize, bitdepth: usize },
}

#[derive(Debug)]
pub struct Table {
    table: Vec<Entry>,
}

#[derive(Debug)]
struct Conversion {
    byte: u8,
    representation: Symbol,
}

#[derive(Debug)]
struct SubtableSize {
    root_id: u8,
    bitdepth: usize,
}

impl Table {
    fn new() -> Table {
        Table {
            table: vec![
                Entry::Subtable {
                    offset: 0,
                    bitdepth: 0
                };
                256
            ],
        }
    }

    /// # Returns
    /// [None] if `root` was not of type [Tree::Root]
    pub fn from_tree_root(root: &Tree) -> Option<Self> {
        let conversion_map = root.make_conversion_map()?;

        let mut table = Table::new();
        let mut subtables = Vec::new();
        for (byte, symbol) in conversion_map {
            let conversion = Conversion {
                byte,
                representation: symbol,
            };
            match conversion.representation.bytepos {
                0 => fill_symbol(&mut table, conversion),
                1 => subtables.push(conversion),
                _ => {
                    eprintln!("cannot build huffman table");
                    return None;
                }
            }
        }

        let subtable_sizes = subtable_sizes(&subtables);
        for conversion in subtables {
            assert!(conversion.representation.bytepos == 1);
            let size = subtable_sizes
                .iter()
                .find(|size| size.root_id == conversion.representation.bytes[0])
                .expect("should always find something");
            fill_subtable(&mut table, size, conversion);
        }

        Some(table)
    }
}

fn fill_subtable(table: &mut Table, subtable: &SubtableSize, conversion: Conversion) {
    let offset = match table.table[subtable.root_id as usize] {
        Entry::Subtable {
            offset: 0,
            bitdepth: 0,
        } => {
            let start = table.table.len();
            let subtable_len = 1 << subtable.bitdepth;
            for _ in 0..subtable_len {
                table.table.push(Entry::Subtable {
                    offset: 0,
                    bitdepth: 0,
                });
            }
            table.table[subtable.root_id as usize] = Entry::Subtable {
                offset: start,
                bitdepth: subtable.bitdepth,
            };
            start
        }
        Entry::Subtable {
            offset: index,
            bitdepth,
        } => {
            assert_eq!(bitdepth, subtable.bitdepth, "cannot happen");
            index
        }
        Entry::Map { byte: _, bitlen: _ } => panic!("there should be a subtable entry"),
    };

    // local index
    let index = conversion.representation.bytes[1] as usize >> (8 - subtable.bitdepth);
    // global index
    let index = offset + index;

    let inflation = 1 << (subtable.bitdepth - conversion.representation.bitpos);
    for pos in index..index + inflation {
        table.table[pos] = Entry::Map {
            byte: conversion.byte,
            bitlen: 8 + conversion.representation.bitpos,
        }
    }
}

/// `conversions` are all conversions that require more then 8 bits to encode
fn subtable_sizes(conversions: &[Conversion]) -> Vec<SubtableSize> {
    conversions
        .iter()
        .fold(Vec::new(), |mut acc: Vec<SubtableSize>, entry| {
            match acc
                .iter_mut()
                .find(|size| size.root_id == entry.representation.bytes[0])
            {
                Some(size) => {
                    size.bitdepth = size.bitdepth.max(entry.representation.bitpos);
                    acc
                }
                None => {
                    acc.push(SubtableSize {
                        root_id: entry.representation.bytes[0],
                        bitdepth: entry.representation.bitpos,
                    });
                    acc
                }
            }
        })
}

fn fill_symbol(table: &mut Table, conversion: Conversion) {
    let index = conversion.representation.bytes[0] as usize;
    let inflation = 1 << (8 - conversion.representation.bitpos);
    for pos in index..index + inflation {
        table.table[pos] = Entry::Map {
            byte: conversion.byte,
            bitlen: conversion.representation.bitpos,
        }
    }
}

impl Display for Table {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for i in 0..256 {
            let line = self.table[i];
            match line {
                Entry::Map { byte, bitlen } => {
                    writeln!(f, "{i:08b}: byte={byte}, takes {bitlen} bits")?;
                }
                Entry::Subtable { offset, bitdepth } => {
                    let len = 1 << bitdepth;
                    writeln!(f, "{i:08b}: {bitdepth}bit subtable at {offset}")?;
                    for j in offset..(offset + len) {
                        let Entry::Map { byte, bitlen } = self.table[j] else {
                            panic!("no subtables in subtables")
                        };
                        // local index (shifted to most significant bits for visual clarity)
                        let index = (j - offset) << (8 - bitdepth);
                        writeln!(f, "\t\t{index:08b}: byte={byte}, takes {bitlen} bits")?;
                    }
                }
            }
        }
        Ok(())
    }
}

