use std::io::{self, BufRead, BufReader, Error};

#[derive(PartialEq, Debug)]
pub enum Estimate {
    Exact(usize),
    AtLeast(usize),
}

pub struct BitWindow<T> {
    data: BufReader<T>,
    initialized: usize, // number of bits in current that are populated (left to right) -> 0b11100101_xxxxxxxx initialized = 8
    current: u16,
}

impl<T> BitWindow<T>
where
    T: BufRead,
{
    /// shows the current 8bit window
    ///
    /// padded on the right with 0s if there was insufficient data to fill the window
    pub fn show(self) -> u8 {
        (self.current >> 8) as u8
    }

    /// shows a `amt`bit window at the current position
    ///
    /// padded on the right with 0s if there was insufficient data to fill the window
    ///
    /// there are `8 - amt` 0 bits before the data
    pub fn show_exact(self, amt: usize) -> u8 {
        (self.current >> (16 - amt)) as u8
    }

    /// Tells this buffer that `amt` bits have been consumed from the buffer,
    /// so they should no longer be returned by [`show`].
    ///
    /// The `amt` must be `<=` the number of bytes that were initialized
    ///
    /// Attemts to load more bits from the underlying data source if applicable
    ///
    /// # Returns
    /// This method returns `true` when the underlying data source has reached EOF.
    ///
    /// [`show`]: BitWindow::show
    pub fn consume(&mut self, amt: usize) -> Result<bool, io::Error> {
        debug_assert!(amt <= 8);
        debug_assert!(amt <= self.initialized);
        self.current <<= amt;
        self.initialized -= amt;
        if self.initialized <= 8 {
            return self.load();
        }
        Ok(false)
    }

    /// loads another byte into `current`
    ///
    /// # Safety
    /// To ensure safe operation this method SHALL NOT be called when `self.initialized > 8`
    /// called when `self.initialized <= 8`
    ///
    /// # Returns
    /// This method returns `true` when the underlying data source has reached EOF.
    /// In this case no bits have been loaded into `self.current`
    ///
    /// # Errors
    /// This method returns an I/O error if the underlaying data source produced
    /// one during read.
    /// In this case no bits have been loaded into `self.current`
    fn load(&mut self) -> Result<bool, io::Error> {
        let data = self.data.fill_buf()?;
        match data.first() {
            Some(&byte) => {
                self.append_byte(byte);
                self.data.consume(1);
                Ok(false)
            }
            None => Ok(true),
        }
    }

    /// Appends `byte` to `self.current`
    ///
    /// Appending is done with bit accuracy, meaning the user has to make sure
    /// that `self.initialized <= 8`.
    ///
    /// # Safety
    /// This function produces undefined behavior when called while `self.initialized > 8`
    fn append_byte(&mut self, byte: u8) {
        let shift = 8 - self.initialized;
        self.current |= (byte as u16) << shift;
        self.initialized += 8;
    }
}

impl<T> From<T> for BitWindow<T>
where
    T: std::io::Read,
{
    fn from(value: T) -> Self {
        BufReader::new(value).into()
    }
}

impl<T> From<BufReader<T>> for BitWindow<T>
where
    T: std::io::Read,
{
    fn from(mut value: BufReader<T>) -> Self {
        let &initial = value
            .fill_buf()
            .and_then(|buf| buf.first().map_or(Err(Error::other("")), Ok))
            .unwrap_or(&0);
        BitWindow {
            data: value,
            current: (initial as u16) << 8,
            initialized: 8,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn show() {
        let data = [0b10011010; 8];
        let data: BufReader<&[u8]> = BufReader::new(&data);
        let reader: BitWindow<BufReader<&[u8]>> = data.into();

        let bits = reader.show();
        assert_eq!(bits, 0b10011010u8);
    }

    #[test]
    fn consume() {
        let data = [0b10011010; 8];
        let data: BufReader<&[u8]> = BufReader::new(&data);
        let mut reader: BitWindow<BufReader<&[u8]>> = data.into();

        reader.consume(4).expect("io err");
        let bits = reader.show();
        assert_eq!(bits, 0b10101001u8);
    }

    #[test]
    fn consume_8() {
        let data = [0b10011010; 8];
        let data: BufReader<&[u8]> = BufReader::new(&data);
        let mut reader: BitWindow<BufReader<&[u8]>> = data.into();

        reader.consume(8).expect("io err");
        let bits = reader.show();
        assert_eq!(bits, 0b10011010);
    }

    #[test]
    fn consume_more() {
        let data = [0b10011010; 8];
        let data: BufReader<&[u8]> = BufReader::new(&data);
        let mut reader: BitWindow<BufReader<&[u8]>> = data.into();

        reader.consume(5).expect("io err");
        reader.consume(6).expect("io err");
        let bits = reader.show();
        assert_eq!(bits, 0b11010100);
    }

    #[test]
    fn show_exact() {
        let data = [0b10011010; 8];
        let data: BufReader<&[u8]> = BufReader::new(&data);
        let mut reader: BitWindow<BufReader<&[u8]>> = data.into();

        reader.consume(5).expect("io err");
        let bits = reader.show_exact(5);
        assert_eq!(bits, 0b00001010);
    }
}
