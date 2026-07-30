use crate::errors::ReaderError::{self, ParseError};

pub struct Reader<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    pub fn take(&mut self, n: usize) -> Result<&'a [u8], ReaderError> {
        let end = self.pos.checked_add(n).ok_or(ParseError("offset overflow"))?;
        let slice = self.data.get(self.pos..end).ok_or(ParseError("unexpected EOF"))?;
        self.pos = end;
        Ok(slice)
    }

    pub fn read_u32(&mut self) -> Result<u32, ReaderError> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }

    pub fn read_bool(&mut self) -> Result<bool, ReaderError> {
        Ok(self.take(1)?[0] != 0)
    }

    pub fn read_string(&mut self) -> Result<&'a str, ReaderError> {
        let len = self.read_u32()? as usize;
        let bytes = self.take(len)?;
        std::str::from_utf8(bytes).map_err(|_| ParseError("invalid utf8 in file name"))
    }
}

// read!(u32, buffer)
