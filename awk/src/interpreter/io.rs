//
// Copyright (c) 2024 Hemi Labs, Inc.
//
// This file is part of the posixutils-rs project covered under
// the MIT License.  For the full license text, please see the LICENSE
// file in the root directory of this project.
// SPDX-License-Identifier: MIT
//

use std::{
    collections::{hash_map::Entry, HashMap},
    fs::File,
    io::{BufReader, Bytes, Read, Write},
};

pub enum RecordSeparator {
    Char(u8),
    Null,
}

impl TryFrom<String> for RecordSeparator {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let mut iter = value.bytes();
        let result = match iter.next() {
            Some(c) => RecordSeparator::Char(c),
            None => RecordSeparator::Null,
        };
        if iter.next().is_some() {
            Err("the record separator cannot contain more than one characters".to_string())
        } else {
            Ok(result)
        }
    }
}

type ReadResult = Result<u8, String>;

macro_rules! read_iter_next {
    ($iter:expr, $ret:expr) => {
        match $iter.next() {
            Some(byte_result) => byte_result?,
            None => return $ret,
        }
    };
    ($iter:expr) => {
        read_iter_next!($iter, Ok(None))
    };
}

pub trait RecordReader: Iterator<Item = ReadResult> {
    fn is_done(&self) -> bool;

    fn last_byte_read(&self) -> Option<u8>;

    fn read_next_record(&mut self, separator: &RecordSeparator) -> Result<Option<String>, String> {
        if self.is_done() {
            return Ok(None);
        }
        match separator {
            RecordSeparator::Char(sep) => {
                let mut str = String::new();
                let mut next = read_iter_next!(self);
                while next != *sep {
                    str.push(next as char);
                    next = read_iter_next!(self, Ok(Some(str)));
                }
                Ok(Some(str))
            }
            RecordSeparator::Null => {
                let mut next = if let Some(byte) = self.last_byte_read() {
                    byte
                } else {
                    read_iter_next!(self)
                };
                while next.is_ascii_whitespace() {
                    next = read_iter_next!(self);
                }
                let mut str = String::new();
                while next != b'\n' {
                    str.push(next as char);
                    next = read_iter_next!(self, Ok(Some(str)));
                }
                while next.is_ascii_whitespace() {
                    next = read_iter_next!(self, Ok(Some(str)));
                }
                Ok(Some(str))
            }
        }
    }
}

pub struct FileStream {
    bytes: Bytes<BufReader<File>>,
    last_byte_read: Option<u8>,
    is_done: bool,
}

impl FileStream {
    pub fn open(path: &str) -> Result<Self, String> {
        let file = File::open(path).map_err(|e| e.to_string())?;
        let reader = BufReader::new(file);
        Ok(Self {
            bytes: reader.bytes(),
            last_byte_read: None,
            is_done: false,
        })
    }
}

impl Iterator for FileStream {
    type Item = ReadResult;

    fn next(&mut self) -> Option<Self::Item> {
        match self.bytes.next() {
            Some(Ok(byte)) => {
                self.last_byte_read = Some(byte);
                Some(Ok(byte))
            }
            Some(Err(e)) => Some(Err(e.to_string())),
            None => {
                self.is_done = true;
                None
            }
        }
    }
}

impl RecordReader for FileStream {
    fn is_done(&self) -> bool {
        self.is_done
    }

    fn last_byte_read(&self) -> Option<u8> {
        self.last_byte_read
    }
}

pub struct StringRecordReader {
    string: String,
    index: usize,
}

impl<S: Into<String>> From<S> for StringRecordReader {
    fn from(value: S) -> Self {
        Self {
            string: value.into(),
            index: 0,
        }
    }
}

impl Iterator for StringRecordReader {
    type Item = ReadResult;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.string.len() {
            None
        } else {
            let result = self.string.as_bytes()[self.index];
            self.index += 1;
            Some(Ok(result))
        }
    }
}

impl RecordReader for StringRecordReader {
    fn is_done(&self) -> bool {
        self.index == self.string.len()
    }

    fn last_byte_read(&self) -> Option<u8> {
        if self.index == 0 {
            None
        } else {
            Some(self.string.as_bytes()[self.index - 1])
        }
    }
}

pub type EmptyRecordReader = std::iter::Empty<ReadResult>;

impl RecordReader for EmptyRecordReader {
    fn is_done(&self) -> bool {
        true
    }

    fn last_byte_read(&self) -> Option<u8> {
        None
    }
}

#[derive(Default)]
pub struct WriteFiles {
    files: HashMap<String, File>,
}

impl WriteFiles {
    pub fn write(&mut self, filename: &str, contents: &str, append: bool) -> Result<(), String> {
        match self.files.entry(filename.to_string()) {
            Entry::Occupied(mut e) => {
                e.get_mut()
                    .write_all(contents.as_bytes())
                    .map_err(|e| e.to_string())?;
            }
            Entry::Vacant(e) => {
                let mut file = File::options()
                    .write(true)
                    .create(true)
                    .append(append)
                    .open(filename)
                    .map_err(|e| e.to_string())?;
                file.write_all(contents.as_bytes())
                    .map_err(|e| e.to_string())?;
                e.insert(file);
            }
        }
        Ok(())
    }

    pub fn close_file(&mut self, filename: &str) {
        self.files.remove(filename);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn split_records(file_contents: &str, separator: RecordSeparator) -> Vec<String> {
        let mut reader = StringRecordReader::from(file_contents);
        let mut result = Vec::new();
        while let Some(record) = reader.read_next_record(&separator).unwrap() {
            result.push(record);
        }
        result
    }

    #[test]
    fn split_empty_file() {
        assert!(split_records("", RecordSeparator::Null).is_empty());
    }

    #[test]
    fn split_records_with_default_separator() {
        let records = split_records("record1\nrecord2\n  \t\nrecord3\n", RecordSeparator::Null);
        assert_eq!(records, vec!["record1", "record2", "record3"]);
    }

    #[test]
    fn split_records_with_separator_chars() {
        let records = split_records("record1,record2,record3", RecordSeparator::Char(b','));
        assert_eq!(records, vec!["record1", "record2", "record3"]);
    }
}
