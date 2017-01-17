// Copyright 2017 pdb Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use std::fmt;
use std::io;

/// Represents an offset + size of the source file.
#[derive(Debug,Clone,Copy,Eq,PartialEq)]
pub struct SourceSlice {
    pub offset: u64,
    pub size: usize,
}

/// `Source` provides raw access to a PDB.
///
/// This library is written with zero-copy in mind. `Source`s provide `SourceView`s which need not
/// outlive their parent, supporting implementations of e.g. memory mapped files.
///
/// PDB files are "multi-stream files" (MSF) under the hood. MSFs have various layers of
/// indirection, but ultimately the MSF code asks a `Source` to view a series of { offset, size }
/// records (each a `SourceSlice`) as a contiguous `&[u8]` (provided by `SourceView`).
///
/// The requested offsets will always be aligned to the MSF's page size, which is always a power of
/// two and is usually (but not always) 4 KiB. The requested sizes will also be multiples of the
/// page size, except for the size of the final `SourceSlice`, which may be smaller. PDB files are
/// specified as always being a multiple of the page size, so `Source` implementations are free to
/// e.g. map whole pages and return a sub-slice of the requested length.
///
pub trait Source<'s> : fmt::Debug {
    /// Provides a contiguous view of the source file composed of the requested position(s).
    ///
    /// Note that the SourceView's as_slice() method cannot fail, so `view()` is the time to raise
    /// IO errors.
    fn view(&mut self, slices: &[SourceSlice]) -> Result<Box<SourceView<'s>>, io::Error>;
}

/// An owned, droppable, read-only view of the source file which can be referenced as a byte slice.
pub trait SourceView<'s>: Drop + fmt::Debug {
    fn as_slice<'v>(&'v self) -> &'v [u8];
}

#[derive(Clone)]
struct ReadView {
    bytes: Vec<u8>,
}

impl<'v> fmt::Debug for ReadView {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ReadView({} bytes)", self.bytes.len())
    }
}

impl<'s> SourceView<'s> for ReadView {
    fn as_slice<'v>(&'v self) -> &'v [u8] {
        self.bytes.as_slice()
    }
}

impl<'v> Drop for ReadView {
    fn drop(&mut self) {
        // no-op
    }
}

impl<'s, T> Source<'s> for T where T: io::Read + io::Seek + fmt::Debug + 's {
    fn view(&mut self, slices: &[SourceSlice]) -> Result<Box<SourceView<'s>>, io::Error> {
        let len = slices.iter()
            .fold(0 as usize, |acc,ref s| acc + s.size);

        let mut v = ReadView{
            bytes: Vec::with_capacity(len)
        };
        v.bytes.resize(len, 0);

        {
            let mut bytes = v.bytes.as_mut_slice();
            let mut output_offset: usize = 0;
            for slice in slices {
                self.seek(io::SeekFrom::Start(slice.offset))?;
                self.read_exact(&mut bytes[output_offset .. (output_offset + slice.size)])?;
                output_offset += slice.size;
            }
        }

        Ok(Box::new(v))
    }
}

#[cfg(test)]
mod tests {
    mod read_view {
        use std::io::Cursor;
        use std::io::ErrorKind;
        use source::*;

        #[test]
        fn test_basic_reading() {
            let mut data = vec![0; 4096];
            data[42] = 42;

            let mut source: Box<Source> = Box::new(Cursor::new(data.as_slice()));

            let source_slices = vec![SourceSlice{ offset: 40, size: 4 } ];
            let view = source.view(source_slices.as_slice()).expect("viewing must succeed");
            assert_eq!(&[0u8, 0, 42, 0], view.as_slice());
        }

        #[test]
        fn test_discontinuous_reading() {
            let mut data = vec![0; 4096];
            data[42] = 42;
            data[88] = 88;

            let mut source: Box<Source> = Box::new(Cursor::new(data.as_slice()));

            let source_slices = vec![SourceSlice{ offset: 88, size: 1 }, SourceSlice{ offset: 40, size: 4 } ];
            let view = source.view(source_slices.as_slice()).expect("viewing must succeed");
            assert_eq!(&[88u8, 0, 0, 42, 0], view.as_slice());
        }

        #[test]
        fn test_duplicate_reading() {
            let mut data = vec![0; 4096];
            data[42] = 42;
            data[88] = 88;

            let mut source: Box<Source> = Box::new(Cursor::new(data.as_slice()));

            let source_slices = vec![SourceSlice{ offset: 88, size: 1 }, SourceSlice{ offset: 40, size: 4 }, SourceSlice{ offset: 88, size: 1 } ];
            let view = source.view(source_slices.as_slice()).expect("viewing must succeed");
            assert_eq!(&[88u8, 0, 0, 42, 0, 88], view.as_slice());
        }

        #[test]
        fn test_eof_reading() {
            let data = vec![0; 4096];

            let mut source: Box<Source> = Box::new(Cursor::new(data.as_slice()));

            // one byte is readable, but we asked for two
            let source_slices = vec![SourceSlice { offset: 4095, size: 2 }];
            let r = source.view(source_slices.as_slice());
            match r {
                Ok(_) => panic!("should have failed"),
                Err(e) => {
                    assert_eq!(ErrorKind::UnexpectedEof, e.kind());
                }
            }
        }
    }
}