use std::io::{Cursor, Read, Seek, SeekFrom, Write};

/// A cursor for reading and writing data from subsections of
pub struct MemCursor<T: AsRef<[u8]>> {
    cursor: Cursor<T>,
    offset: u64,
}

impl<T: AsRef<[u8]>> MemCursor<T> {
    pub fn new(inner: T, offset: u64) -> Self {
        Self {
            cursor: Cursor::new(inner),
            offset,
        }
    }
}

impl<T: AsRef<[u8]>> Seek for MemCursor<T> {
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        let new_pos = match pos {
            SeekFrom::Current(current) => SeekFrom::Current(current),
            SeekFrom::Start(start) => SeekFrom::Start(start - self.offset),
            SeekFrom::End(end) => SeekFrom::End(end),
        };
        self.cursor.seek(new_pos).map(|to| to + self.offset)
    }
}

impl<T: AsRef<[u8]>> Read for MemCursor<T> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.cursor.read(buf)
    }
}

impl<'a> Write for MemCursor<&'a mut [u8]> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.cursor.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl<'a> Write for MemCursor<&'a mut Vec<u8>> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.cursor.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl Write for MemCursor<Vec<u8>> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.cursor.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl Write for MemCursor<Box<[u8]>> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.cursor.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
