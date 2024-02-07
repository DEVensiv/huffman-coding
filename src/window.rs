use std::io::BufRead;

pub use crate::error::Error;

/// When constructed via [`From<BufRead>`] will hallucinate a 0 byte if the data source
/// is of lenght 0.
pub struct BitWindow<R> {
    data: R,
    initialized: usize, // number of bits in current that are populated (left to right) -> 0b11100101_xxxxxxxx initialized = 8
    current: usize,
}

/// count of bits in the "current" type
const MAXIBITS: usize = usize::BITS as usize;
/// how many bits to keep in "current" at any time
const READAHEAD: usize = 8;
#[allow(clippy::assertions_on_constants)]
const _: () = assert!(READAHEAD <= (MAXIBITS - 8), "Readahead must be smaller");

/// Alias to u8::BITS as usize
const U8BITS: usize = u8::BITS as usize;

impl<R> BitWindow<R>
where
    R: BufRead,
{
    /// shows the current 8bit window
    ///
    /// padded on the right with 0s if there was insufficient data to fill the window
    pub fn show_u8(&self) -> usize {
        self.show::<8>()
    }

    pub fn show<const BITS: usize>(&self) -> usize {
        self.current >> (MAXIBITS - BITS)
    }

    /// shows a `amt`bit window at the current position
    ///
    /// padded on the right with 0s if there was insufficient data to fill the window
    ///
    /// e.g. if 'amt' is 5 the bits will be layed out like so: "0001_2345" 
    /// where 0s are actual zeros and 1-5 are the 1 starting indecies for the read bits
    pub fn show_exact(&self, amt: usize) -> usize {
        self.current >> (MAXIBITS - amt)
    }

    /// Tells this buffer that `amt` bits have been consumed from the buffer,
    /// so they should no longer be returned by [`show`].
    ///
    /// The `amt` must be `<=` the number of bytes that were initialized
    ///
    /// Attemts to load more bits from the underlying data source if applicable
    ///
    /// # Returns
    /// This method returns `true` if the number of initialized bits drops below 9
    /// AND the underlying data source has reached EOF.
    ///
    /// # Errors
    /// This method returns an I/O error if:
    /// - the underlying reader did so while loading additional bits
    /// - there were less bits present then were tried to consume
    ///
    ///
    /// [`show`]: BitWindow::show
    pub fn consume(&mut self, amt: usize) -> Result<bool, Error> {
        if amt > self.initialized {
            return Err(Error::NoBits);
        }

        self.current <<= amt;
        self.initialized -= amt;
        if self.initialized <= READAHEAD {
            return self.load();
        }
        Ok(false)
    }

    /// This method returns the ammount of initialized bits in the internal buffer
    ///
    /// If this method returns `val < 8` this implied EOF of underlying source since
    /// [`consume`] will always fill up when consumed below 8
    ///
    /// [`consume`]: BitWindow::consume
    pub fn initialized(&self) -> usize {
        self.initialized
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
    fn load(&mut self) -> Result<bool, Error> {
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
        let shift = (MAXIBITS - U8BITS) - self.initialized;
        self.current |= (byte as usize) << shift;
        self.initialized += U8BITS;
    }
}

impl<R> From<R> for BitWindow<R>
where
    R: BufRead,
{
    fn from(mut value: R) -> Self {
        let &initial = value
            .fill_buf()
            .map_err(|_| ())
            .and_then(|buf| buf.first().map_or(Err(()), Ok))
            .unwrap_or(&0);
        value.consume(1);
        BitWindow {
            data: value,
            current: (initial as usize) << (MAXIBITS - U8BITS),
            initialized: U8BITS,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::BufReader;

    use super::*;

    #[test]
    fn show() {
        let data = [0b10011010; 8];
        let data: BufReader<&[u8]> = BufReader::new(&data);
        let reader: BitWindow<BufReader<&[u8]>> = data.into();

        let bits = reader.show_u8();
        assert_eq!(bits, 0b10011010usize);
    }

    #[test]
    fn consume() {
        let data = [0b10011010; 8];
        let data: BufReader<&[u8]> = BufReader::new(&data);
        let mut reader: BitWindow<BufReader<&[u8]>> = data.into();

        reader.consume(4).expect("io err");
        let bits = reader.show_u8();
        assert_eq!(bits, 0b10101001usize);
    }

    #[test]
    fn consume_8() {
        let data = [0b10011010; 8];
        let data: BufReader<&[u8]> = BufReader::new(&data);
        let mut reader: BitWindow<BufReader<&[u8]>> = data.into();

        reader.consume(8).expect("io err");
        let bits = reader.show_u8();
        assert_eq!(bits, 0b10011010);
    }

    #[test]
    fn consume_more() {
        let data = [0b10011010; 8];
        let data: BufReader<&[u8]> = BufReader::new(&data);
        let mut reader: BitWindow<BufReader<&[u8]>> = data.into();

        reader.consume(5).expect("io err");
        reader.consume(6).expect("io err");
        let bits = reader.show_u8();
        assert_eq!(bits, 0b11010100);
    }

    #[test]
    fn consume_last() {
        let data = [0b10011010; 2];
        let data: BufReader<&[u8]> = BufReader::new(&data);
        let mut reader: BitWindow<BufReader<&[u8]>> = data.into();

        reader.consume(5).expect("io err");
        reader.consume(6).expect("io err");
        reader.consume(5).expect("io err");
        let bits = reader.show_u8();
        assert_eq!(bits, 0);
        assert_eq!(reader.initialized, 0);
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

    #[test]
    fn consume_return_not_eof() {
        let data = [0b10011010; 2];
        let data: BufReader<&[u8]> = BufReader::new(&data);
        let mut reader: BitWindow<BufReader<&[u8]>> = data.into();

        let eof = reader.consume(5).expect("io err");
        assert!(!eof);
        assert!(reader.initialized() >= 8);
    }

    #[test]
    fn consume_return_eof() {
        let data = [0b10011010; 2];
        let data: BufReader<&[u8]> = BufReader::new(&data);
        let mut reader: BitWindow<BufReader<&[u8]>> = data.into();

        let eof = reader.consume(8).expect("io err");
        assert!(!eof);
        assert!(reader.initialized() >= 8);
        let eof = reader.consume(1).expect("io err");
        assert!(eof);
        assert!(reader.initialized() == 7);
    }
}
