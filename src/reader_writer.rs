use std::{
    cmp,
    fs::File,
    io::{self, BufRead, Read, ReadBuf, Seek, SeekFrom, Write},
    mem::MaybeUninit,
};

use crate::parcel;

/// Adapter for file to allow parcels to read/write/seek
pub struct ReaderWriter {
    file: File,
    buf:  Box<[MaybeUninit<u8>]>,
    pos:  usize,
    cap:  usize,
    init: usize,
}

impl ReaderWriter {
    /// Create a reader-writer from a file
    pub fn new(file: File) -> Self {
        Self {
            file,
            buf: Box::new_uninit_slice(8 * 1024),
            pos: 0,
            cap: 0,
            init: 0,
        }
    }

    fn buffer(&self) -> &[u8] {
        unsafe { MaybeUninit::slice_assume_init_ref(&self.buf[self.pos..self.cap]) }
    }
    fn discard_buffer(&mut self) {
        self.pos = 0;
        self.cap = 0;
    }
}

impl parcel::FileBacking for ReaderWriter {}

impl Seek for ReaderWriter {
    fn seek(&mut self, from: SeekFrom) -> Result<u64, io::Error> {
        self.discard_buffer();
        self.file.seek(from)
    }
    fn stream_position(&mut self) -> io::Result<u64> {
        let remainder = (self.cap - self.pos) as u64;
        self.file.stream_position().map(|pos| {
            pos.checked_sub(remainder).expect(
                "overflow when subtracting remaining buffer size from inner stream position",
            )
        })
    }
}
impl Read for ReaderWriter {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.discard_buffer();
        self.file.read(buf)
    }
}
impl BufRead for ReaderWriter {
    fn fill_buf(&mut self) -> std::io::Result<&[u8]> {
        if self.pos >= self.cap {
            debug_assert!(self.pos == self.cap);

            let mut readbuf = ReadBuf::uninit(&mut self.buf);

            // SAFETY: `self.init` is either 0 or set to `readbuf.initialized_len()`
            // from the last time this function was called
            unsafe {
                readbuf.assume_init(self.init);
            }

            self.file.read_buf(&mut readbuf)?;

            self.cap = readbuf.filled_len();
            self.init = readbuf.initialized_len();

            self.pos = 0;
        }
        Ok(self.buffer())
    }

    fn consume(&mut self, amt: usize) {
        self.pos = cmp::min(self.pos + amt, self.cap);
    }
}
impl Write for ReaderWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.discard_buffer();
        self.file.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.discard_buffer();
        self.file.flush()
    }
}
