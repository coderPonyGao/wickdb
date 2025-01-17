// Copyright 2019 Fullstop000 <fullstop1005@gmail.com>.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// See the License for the specific language governing permissions and
// limitations under the License.

// Copyright (c) 2011 The LevelDB Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file. See the AUTHORS file for names of contributors.

///
/// # Table
///
/// Table is consist of one or more data blocks, an optional filter block
/// a metaindex block, an index block and a table footer. Metaindex block
/// is a special block used to keep parameters of the table, such as filter
/// block name and its block handle. Index block is a special block used to
/// keep record of data blocks offset and length, index block use one as
/// restart interval. The key used by index block are the last key of preceding
/// block, shorter separator of adjacent blocks or shorter successor of the
/// last key of the last block. Filter block is an optional block contains
/// sequence of filter data generated by a filter generator.
///
/// ## Table data structure:
///
/// ```text
///                                                          + optional
///                                                         /
///     +--------------+--------------+--------------+------+-------+-----------------+-------------+--------+
///     | data block 1 |      ...     | data block n | filter block | metaindex block | index block | footer |
///     +--------------+--------------+--------------+--------------+-----------------+-------------+--------+
///
///     Each block followed by a 5-bytes trailer contains compression type and checksum.
///
/// ```
///
/// ## Common Table block trailer:
///
/// ```text
///
///     +---------------------------+-------------------+
///     | compression type (1-byte) | checksum (4-byte) |
///     +---------------------------+-------------------+
///
///     The checksum is a CRC-32 computed using Castagnoli's polynomial. Compression
///     type also included in the checksum.
///
/// ```
///
/// ## Table footer:
///
/// ```text
///
///       +------------------- 40-bytes -------------------+
///      /                                                  \
///     +------------------------+--------------------+------+-----------------+
///     | metaindex block handle / index block handle / ---- | magic (8-bytes) |
///     +------------------------+--------------------+------+-----------------+
///
///     The magic are first 64-bit of SHA-1 sum of "http://code.google.com/p/leveldb/".
///
/// ```
///
/// NOTE: All fixed-length integer are little-endian.
///
///
/// # Block
///
/// Block is consist of one or more key/value entries and a block trailer.
/// Block entry shares key prefix with its preceding key until a restart
/// point reached. A block should contains at least one restart point.
/// First restart point are always zero.
///
/// Block data structure:
///
/// ```text
///       + restart point                 + restart point (depends on restart interval)
///      /                               /
///     +---------------+---------------+---------------+---------------+------------------+----------------+
///     | block entry 1 | block entry 2 |      ...      | block entry n | restarts trailer | common trailer |
///     +---------------+---------------+---------------+---------------+------------------+----------------+
///
/// ```
/// Key/value entry:
///
/// ```text
///               +---- key len ----+
///              /                   \
///     +-------+---------+-----------+---------+--------------------+--------------+----------------+
///     | shared (varint) | not shared (varint) | value len (varint) | key (varlen) | value (varlen) |
///     +-----------------+---------------------+--------------------+--------------+----------------+
///
///     Block entry shares key prefix with its preceding key:
///     Conditions:
///         restart_interval=2
///         entry one  : key=deck,value=v1
///         entry two  : key=dock,value=v2
///         entry three: key=duck,value=v3
///     The entries will be encoded as follow:
///
///       + restart point (offset=0)                                                 + restart point (offset=16)
///      /                                                                          /
///     +-----+-----+-----+----------+--------+-----+-----+-----+---------+--------+-----+-----+-----+----------+--------+
///     |  0  |  4  |  2  |  "deck"  |  "v1"  |  1  |  3  |  2  |  "ock"  |  "v2"  |  0  |  4  |  2  |  "duck"  |  "v3"  |
///     +-----+-----+-----+----------+--------+-----+-----+-----+---------+--------+-----+-----+-----+----------+--------+
///      \                                   / \                                  / \                                   /
///       +----------- entry one -----------+   +----------- entry two ----------+   +---------- entry three ----------+
///
///     The block trailer will contains two restart points:
///
///     +------------+-----------+--------+
///     |     0      |    16     |   2    |
///     +------------+-----------+---+----+
///      \                      /     \
///       +-- restart points --+       + restart points length
///
/// ```
///
/// # Block restarts trailer
///
/// ```text
///
///       +-- 4-bytes --+
///      /               \
///     +-----------------+-----------------+-----------------+------------------------------+
///     | restart point 1 |       ....      | restart point n | restart points len (4-bytes) |
///     +-----------------+-----------------+-----------------+------------------------------+
///
/// ```
///
/// NOTE: All fixed-length integer are little-endian.
///
/// # Filter block
///
/// Filter block consist of one or more filter data and a filter block trailer.
/// The trailer contains filter data offsets, a trailer offset and a 1-byte base Lg.
///
/// Filter block data structure:
///
/// ```text
///
///       + offset 1      + offset 2      + offset n      + trailer offset
///      /               /               /               /
///     +---------------+---------------+---------------+---------+
///     | filter data 1 |      ...      | filter data n | trailer |
///     +---------------+---------------+---------------+---------+
///
/// ```
///
/// Filter block trailer:
///
/// ```text
///
///       +- 4-bytes -+
///      /             \
///     +---------------+---------------+---------------+-------------------------------+------------------+
///     | data 1 offset |      ....     | data n offset | data-offsets length (4-bytes) | base Lg (1-byte) |
///     +---------------+---------------+---------------+-------------------------------+------------------+
///
/// ```
///
/// NOTE: The filter block is not compressed
///
/// # Index block
///
/// Index block consist of one or more block handle data and a common block trailer.
/// The 'separator key' is the key just bigger than the last key in the data block which the 'block handle' pointed to
///
/// ```text
///
///     +---------------+--------------+
///     |      key      |    value     |
///     +---------------+--------------+
///     | separator key | block handle |---- a block handle points a data block starting offset and the its size
///     | ...           | ...          |
///     +---------------+--------------+
///
/// ```
///
/// NOTE: All fixed-length integer are little-endian.
///
/// # Meta block
///
/// This meta block contains a bunch of stats. The key is the name of the statistic. The value contains the statistic.
/// For the current implementation, the meta block only contains the filter meta data:
///
/// ```text
///
///     +-------------+---------------------+
///     |     key     |        value        |
///     +-------------+---------------------+
///     | filter name | filter block handle |
///     +-------------+---------------------+
///
/// ```
///
/// NOTE: All fixed-length integer are little-endian.
pub mod block;
mod filter_block;
pub mod table;

use crate::util::coding::{decode_fixed_64, put_fixed_64};
use crate::util::status::{Status, WickErr};
use crate::util::varint::{VarintU64, MAX_VARINT_LEN_U64};

const TABLE_MAGIC_NUMBER: u64 = 0xdb4775248b80fb57;

// 1byte compression type + 4bytes cyc
const BLOCK_TRAILER_SIZE: usize = 5;

// Maximum encoding length of a BlockHandle
const MAX_BLOCK_HANDLE_ENCODE_LENGTH: usize = 2 * MAX_VARINT_LEN_U64;

// Encoded length of a Footer.  Note that the serialization of a
// Footer will always occupy exactly this many bytes.  It consists
// of two block handles and a magic number.
const FOOTER_ENCODED_LENGTH: usize = 2 * MAX_BLOCK_HANDLE_ENCODE_LENGTH + 8;

/// `BlockHandle` is a pointer to the extent of a file that stores a data
/// block or a meta block.
#[derive(Eq, PartialEq, Debug, Clone)]
pub struct BlockHandle {
    offset: u64,
    // NOTICE: the block trailer size is not included
    size: u64,
}

impl BlockHandle {
    pub fn new(offset: u64, size: u64) -> Self {
        Self { offset, size }
    }

    #[inline]
    pub fn set_offset(&mut self, offset: u64) {
        self.offset = offset;
    }

    #[inline]
    pub fn set_size(&mut self, size: u64) {
        self.size = size
    }

    /// Appends varint encoded offset and size into given `dst`
    #[inline]
    pub fn encoded_to(&self, dst: &mut Vec<u8>) {
        VarintU64::put_varint(dst, self.offset);
        VarintU64::put_varint(dst, self.size);
    }

    /// Returns bytes for a encoded BlockHandle
    #[inline]
    pub fn encoded(&self) -> Vec<u8> {
        let mut v = vec![];
        self.encoded_to(&mut v);
        v
    }

    /// Decodes a BlockHandle from bytes
    ///
    /// # Error
    ///
    /// If varint decoding fails, return `Status::Corruption` with relative messages
    #[inline]
    pub fn decode_from(src: &[u8]) -> Result<(Self, usize), WickErr> {
        if let Some((offset, n)) = VarintU64::read(src) {
            if let Some((size, m)) = VarintU64::read(&src[n..]) {
                Ok((Self::new(offset, size), m + n))
            } else {
                Err(WickErr::new(Status::Corruption, Some("bad block handle")))
            }
        } else {
            Err(WickErr::new(Status::Corruption, Some("bad block handle")))
        }
    }
}

/// `Footer` encapsulates the fixed information stored at the tail
/// end of every table file.
#[derive(Debug)]
pub struct Footer {
    meta_index_handle: BlockHandle,
    index_handle: BlockHandle,
}

impl Footer {
    #[inline]
    pub fn new(meta_index_handle: BlockHandle, index_handle: BlockHandle) -> Self {
        Self {
            meta_index_handle,
            index_handle,
        }
    }

    /// Decodes a `Footer` from the given `src` bytes and returns the decoded length
    ///
    /// # Error
    ///
    /// Returns `Status::Corruption` when decoding meta index or index handle fails
    ///
    pub fn decode_from(src: &[u8]) -> Result<(Self, usize), WickErr> {
        let magic = decode_fixed_64(&src[FOOTER_ENCODED_LENGTH - 8..]);
        if magic != TABLE_MAGIC_NUMBER {
            return Err(WickErr::new(
                Status::Corruption,
                Some("not an sstable (bad magic number)"),
            ));
        };
        let (meta_index_handle, n) = BlockHandle::decode_from(src)?;
        let (index_handle, m) = BlockHandle::decode_from(&src[n..])?;
        Ok((
            Self {
                meta_index_handle,
                index_handle,
            },
            m + n,
        ))
    }

    /// Encodes footer and returns the encoded bytes
    pub fn encoded(&self) -> Vec<u8> {
        let mut v = vec![];
        self.meta_index_handle.encoded_to(&mut v);
        self.index_handle.encoded_to(&mut v);
        v.resize(2 * MAX_BLOCK_HANDLE_ENCODE_LENGTH, 0);
        put_fixed_64(&mut v, TABLE_MAGIC_NUMBER);
        assert_eq!(
            v.len(),
            FOOTER_ENCODED_LENGTH,
            "[footer] the length of encoded footer is {}, expect {}",
            v.len(),
            FOOTER_ENCODED_LENGTH
        );
        v
    }
}

#[cfg(test)]
mod test_footer {
    use crate::sstable::{BlockHandle, Footer};
    use crate::util::status::Status;
    use std::error::Error;

    #[test]
    fn test_footer_corruption() {
        let footer = Footer::new(BlockHandle::new(300, 100), BlockHandle::new(401, 1000));
        let mut encoded = footer.encoded();
        let last = encoded.last_mut().unwrap();
        *last += 1;
        let r1 = Footer::decode_from(&encoded);
        assert!(r1.is_err());
        let e1 = r1.unwrap_err();
        assert_eq!(e1.status(), Status::Corruption);
        assert_eq!(e1.description(), "not an sstable (bad magic number)");
    }

    #[test]
    fn test_encode_decode() {
        let footer = Footer::new(BlockHandle::new(300, 100), BlockHandle::new(401, 1000));
        let encoded = footer.encoded();
        let (footer, _) = Footer::decode_from(&encoded).expect("footer decoding should work");
        assert_eq!(footer.index_handle, BlockHandle::new(401, 1000));
        assert_eq!(footer.meta_index_handle, BlockHandle::new(300, 100));
    }
}

#[cfg(test)]
mod tests {
    use crate::db::format::{
        InternalKeyComparator, LookupKey, ParsedInternalKey, ValueType, MAX_KEY_SEQUENCE,
    };
    use crate::db::{WickDB, DB};
    use crate::iterator::{EmptyIterator, Iterator};
    use crate::mem::{MemTable, MemoryTable};
    use crate::options::{Options, ReadOptions};
    use crate::sstable::block::*;
    use crate::sstable::table::*;
    use crate::storage::mem::MemStorage;
    use crate::util::comparator::{BytewiseComparator, Comparator};
    use crate::util::slice::Slice;
    use crate::util::status::{Result, Status, WickErr};
    use crate::{WriteBatch, WriteOptions};
    use hashbrown::HashSet;
    use rand::prelude::ThreadRng;
    use rand::Rng;
    use std::cell::Cell;
    use std::cmp::Ordering;
    use std::rc::Rc;
    use std::sync::Arc;

    // Return the reverse of given key
    fn reverse(key: &[u8]) -> Vec<u8> {
        let mut v = Vec::from(key);
        let length = v.len();
        for i in 0..length / 2 {
            v.swap(i, length - i - 1)
        }
        v
    }

    struct ReverseComparator {
        cmp: BytewiseComparator,
    }

    impl ReverseComparator {
        fn new() -> Self {
            Self {
                cmp: BytewiseComparator::new(),
            }
        }
    }

    impl Comparator for ReverseComparator {
        fn compare(&self, a: &[u8], b: &[u8]) -> Ordering {
            self.cmp.compare(&reverse(a), &reverse(b))
        }

        fn name(&self) -> &str {
            "wickdb.ReverseBytewiseComparator"
        }

        fn separator(&self, a: &[u8], b: &[u8]) -> Vec<u8> {
            let s = self.cmp.separator(&reverse(a), &reverse(b));
            reverse(&s)
        }

        fn successor(&self, key: &[u8]) -> Vec<u8> {
            let s = self.cmp.successor(&reverse(key));
            reverse(&s)
        }
    }

    // Helper class for tests to unify the interface between
    // BlockBuilder/TableBuilder and Block/Table
    trait Constructor {
        // Write key/value pairs in `data` into inner data structure ( Block / Table )
        fn finish(&mut self, options: Arc<Options>, data: &[(Vec<u8>, Vec<u8>)]) -> Result<()>;

        // Returns a iterator for inner data structure ( Block / Table )
        fn iter(&self) -> Box<dyn Iterator>;
    }

    struct BlockConstructor {
        block: Block,
        cmp: Arc<dyn Comparator>,
    }

    impl BlockConstructor {
        fn new(cmp: Arc<dyn Comparator>) -> Self {
            Self {
                block: Block::default(),
                cmp,
            }
        }
    }

    impl Constructor for BlockConstructor {
        fn finish(&mut self, options: Arc<Options>, data: &[(Vec<u8>, Vec<u8>)]) -> Result<()> {
            let mut builder =
                BlockBuilder::new(options.block_restart_interval, options.comparator.clone());
            for (key, value) in data {
                builder.add(key.as_slice(), value.as_slice())
            }
            let data = builder.finish();
            let block = Block::new(Vec::from(data))?;
            self.block = block;
            Ok(())
        }

        fn iter(&self) -> Box<dyn Iterator> {
            self.block.iter(self.cmp.clone())
        }
    }

    struct TableConstructor {
        table: Option<Arc<Table>>,
    }

    impl TableConstructor {
        fn new(_cmp: Arc<dyn Comparator>) -> Self {
            Self { table: None }
        }

        #[allow(dead_code)]
        fn approximate_offset_of(&self, key: &[u8]) -> u64 {
            if let Some(t) = &self.table {
                t.approximate_offset_of(key)
            } else {
                0
            }
        }
    }

    impl Constructor for TableConstructor {
        fn finish(&mut self, options: Arc<Options>, data: &[(Vec<u8>, Vec<u8>)]) -> Result<()> {
            let file_name = "test_table";
            let file = options.env.create(file_name)?;
            let mut builder = TableBuilder::new(file, options.clone());
            for (key, value) in data {
                builder
                    .add(key.as_slice(), value.as_slice())
                    .expect("TableBuilder add should work");
            }
            builder
                .finish(false)
                .expect("TableBuilder finish should work");
            let file = options.env.open(file_name)?;
            let file_len = file.len()?;
            let table = Table::open(file, file_len, options.clone())?;
            self.table = Some(Arc::new(table));
            Ok(())
        }

        fn iter(&self) -> Box<dyn Iterator> {
            match &self.table {
                Some(t) => new_table_iterator(t.clone(), Rc::new(ReadOptions::default())),
                None => Box::new(EmptyIterator::new()),
            }
        }
    }

    // A helper struct to convert user key into lookup key for inner iterator
    struct KeyConvertingIterator {
        inner: Box<dyn Iterator>,
        err: Cell<Option<WickErr>>,
    }

    impl KeyConvertingIterator {
        fn new(iter: Box<dyn Iterator>) -> Self {
            Self {
                inner: iter,
                err: Cell::new(None),
            }
        }
    }

    impl Iterator for KeyConvertingIterator {
        fn valid(&self) -> bool {
            self.inner.valid()
        }

        fn seek_to_first(&mut self) {
            self.inner.seek_to_first()
        }

        fn seek_to_last(&mut self) {
            self.inner.seek_to_last()
        }

        fn seek(&mut self, target: &Slice) {
            let lkey = LookupKey::new(target.as_slice(), MAX_KEY_SEQUENCE);
            self.inner.seek(&lkey.mem_key());
        }

        fn next(&mut self) {
            self.inner.next()
        }

        fn prev(&mut self) {
            self.inner.prev()
        }

        fn key(&self) -> Slice {
            match ParsedInternalKey::decode_from(self.inner.key()) {
                Some(parsed_ikey) => parsed_ikey.user_key.clone(),
                None => {
                    self.err.set(Some(WickErr::new(
                        Status::Corruption,
                        Some("malformed internal key"),
                    )));
                    Slice::from("corrupted key")
                }
            }
        }

        fn value(&self) -> Slice {
            self.inner.value()
        }

        fn status(&mut self) -> Result<()> {
            let err = self.err.take();
            if err.is_none() {
                self.err.set(err);
                self.inner.status()
            } else {
                Err(err.unwrap())
            }
        }
    }

    // A simple wrapper for entries collected in a Vec
    struct EntryIterator {
        current: usize,
        data: Vec<(Vec<u8>, Vec<u8>)>,
        cmp: Arc<dyn Comparator>,
    }

    impl EntryIterator {
        fn new(cmp: Arc<dyn Comparator>, data: Vec<(Vec<u8>, Vec<u8>)>) -> Self {
            Self {
                current: data.len(),
                data,
                cmp,
            }
        }
    }

    impl Iterator for EntryIterator {
        fn valid(&self) -> bool {
            self.current < self.data.len()
        }

        fn seek_to_first(&mut self) {
            self.current = 0
        }

        fn seek_to_last(&mut self) {
            if self.data.is_empty() {
                self.current = 0
            } else {
                self.current = self.data.len() - 1
            }
        }

        fn seek(&mut self, target: &Slice) {
            for (i, (key, _)) in self.data.iter().enumerate() {
                if self.cmp.compare(key.as_slice(), target.as_slice()) != Ordering::Less {
                    self.current = i;
                    return;
                }
            }
            self.current = self.data.len()
        }

        fn next(&mut self) {
            assert!(self.valid());
            self.current += 1
        }

        fn prev(&mut self) {
            assert!(self.valid());
            if self.current == 0 {
                self.current = self.data.len()
            } else {
                self.current -= 1
            }
        }

        fn key(&self) -> Slice {
            if self.valid() {
                Slice::from(self.data[self.current].0.as_slice())
            } else {
                Slice::default()
            }
        }

        fn value(&self) -> Slice {
            if self.valid() {
                Slice::from(self.data[self.current].1.as_slice())
            } else {
                Slice::default()
            }
        }

        fn status(&mut self) -> Result<()> {
            Ok(())
        }
    }

    struct MemTableConstructor {
        inner: MemTable,
    }

    impl MemTableConstructor {
        fn new(cmp: Arc<dyn Comparator>) -> Self {
            let icmp = Arc::new(InternalKeyComparator::new(cmp));
            Self {
                inner: MemTable::new(icmp),
            }
        }
    }

    impl Constructor for MemTableConstructor {
        fn finish(&mut self, _options: Arc<Options>, data: &[(Vec<u8>, Vec<u8>)]) -> Result<()> {
            for (seq, (key, value)) in data.iter().enumerate() {
                self.inner.add(
                    seq as u64 + 1,
                    ValueType::Value,
                    key.as_slice(),
                    value.as_slice(),
                );
            }
            Ok(())
        }

        fn iter(&self) -> Box<dyn Iterator> {
            Box::new(KeyConvertingIterator::new(self.inner.iter()))
        }
    }

    struct DBConstructor {
        inner: WickDB,
    }

    impl DBConstructor {
        fn new(cmp: Arc<dyn Comparator>) -> Self {
            let mut options = Options::default();
            options.env = Arc::new(MemStorage::default());
            options.comparator = cmp;
            options.write_buffer_size = 10000; // Something small to force merging
            options.error_if_exists = true;
            let db =
                WickDB::open_db(options, "table_testdb".to_owned()).expect("could not open db");
            Self { inner: db }
        }
    }

    impl Constructor for DBConstructor {
        fn finish(&mut self, _options: Arc<Options>, data: &[(Vec<u8>, Vec<u8>)]) -> Result<()> {
            for (key, value) in data.iter() {
                let mut batch = WriteBatch::new();
                batch.put(key.as_slice(), value.as_slice());
                self.inner
                    .write(WriteOptions::default(), batch)
                    .expect("write batch should work")
            }
            Ok(())
        }

        fn iter(&self) -> Box<dyn Iterator> {
            self.inner.iter(ReadOptions::default())
        }
    }

    struct CommonConstructor {
        constructor: Box<dyn Constructor>,
        // key&value pairs in order
        data: Vec<(Vec<u8>, Vec<u8>)>,
        keys: HashSet<Vec<u8>>,
    }

    impl CommonConstructor {
        fn new(constructor: Box<dyn Constructor>) -> Self {
            Self {
                constructor,
                data: vec![],
                keys: HashSet::new(),
            }
        }
        fn add(&mut self, key: Slice, value: Slice) {
            if !self.keys.contains(key.as_slice()) {
                self.data
                    .push((Vec::from(key.as_slice()), Vec::from(value.as_slice())));
                self.keys.insert(Vec::from(key.as_slice()));
            }
        }

        // Finish constructing the data structure with all the keys that have
        // been added so far.  Returns the keys in sorted order and stores the
        // key/value pairs in `data`
        fn finish(&mut self, options: Arc<Options>) -> Vec<Vec<u8>> {
            let cmp = options.comparator.clone();
            // Sort the data
            self.data
                .sort_by(|(a, _), (b, _)| cmp.compare(a.as_slice(), b.as_slice()));
            let mut res = vec![];
            for (key, _) in self.data.iter() {
                res.push(key.clone())
            }
            self.constructor
                .finish(options, &self.data)
                .expect("constructor finish should be ok");
            res
        }
    }

    struct TestHarness {
        options: Arc<Options>,
        reverse_cmp: bool,
        inner: CommonConstructor,
        rand: ThreadRng,
    }

    impl TestHarness {
        fn new(t: TestType, reverse_cmp: bool, restart_interval: usize) -> Self {
            let mut options = Options::default();
            options.env = Arc::new(MemStorage::default());
            options.block_restart_interval = restart_interval;
            // Use shorter block size for tests to exercise block boundary
            // conditions more
            options.block_size = 256;
            options.paranoid_checks = true;
            if reverse_cmp {
                options.comparator = Arc::new(ReverseComparator::new());
            }
            let constructor: Box<dyn Constructor> = match t {
                TestType::Table => Box::new(TableConstructor::new(options.comparator.clone())),
                TestType::Block => Box::new(BlockConstructor::new(options.comparator.clone())),
                TestType::Memtable => {
                    Box::new(MemTableConstructor::new(options.comparator.clone()))
                }
                TestType::DB => Box::new(DBConstructor::new(options.comparator.clone())),
            };
            TestHarness {
                inner: CommonConstructor::new(constructor),
                reverse_cmp,
                rand: rand::thread_rng(),
                options: Arc::new(options),
            }
        }

        fn add(&mut self, key: &[u8], value: &[u8]) {
            self.inner.add(Slice::from(key), Slice::from(value))
        }

        fn test_forward_scan(&self, expected: &[(Vec<u8>, Vec<u8>)]) {
            let mut iter = self.inner.constructor.iter();
            assert!(
                !iter.valid(),
                "iterator should be invalid after being initialized"
            );
            iter.seek_to_first();
            for (key, value) in expected.iter() {
                assert_eq!(
                    format_kv(key.clone(), value.clone()),
                    format_entry(iter.as_ref())
                );
                iter.next();
            }
            assert!(
                !iter.valid(),
                "iterator should be invalid after yielding all entries"
            );
        }

        fn test_backward_scan(&self, expected: &[(Vec<u8>, Vec<u8>)]) {
            let mut iter = self.inner.constructor.iter();
            assert!(
                !iter.valid(),
                "iterator should be invalid after being initialized"
            );
            iter.seek_to_last();
            for (key, value) in expected.iter().rev() {
                assert_eq!(
                    format_kv(key.clone(), value.clone()),
                    format_entry(iter.as_ref())
                );
                iter.prev();
            }
            assert!(
                !iter.valid(),
                "iterator should be invalid after yielding all entries"
            );
        }

        fn test_random_access(&mut self, keys: &[Vec<u8>], expected: Vec<(Vec<u8>, Vec<u8>)>) {
            let mut iter = self.inner.constructor.iter();
            assert!(
                !iter.valid(),
                "iterator should be invalid after being initialized"
            );
            let mut expected_iter = EntryIterator::new(self.options.comparator.clone(), expected);
            for _ in 0..1000 {
                match self.rand.gen_range(0, 5) {
                    // case for `next`
                    0 => {
                        if iter.valid() {
                            iter.next();
                            expected_iter.next();
                            if iter.valid() {
                                assert_eq!(
                                    format_entry(iter.as_ref()),
                                    format_entry(&expected_iter)
                                );
                            } else {
                                assert_eq!(iter.valid(), expected_iter.valid());
                            }
                        }
                    }
                    // case for `seek_to_first`
                    1 => {
                        iter.seek_to_first();
                        expected_iter.seek_to_first();
                        if iter.valid() {
                            assert_eq!(format_entry(iter.as_ref()), format_entry(&expected_iter));
                        } else {
                            assert_eq!(iter.valid(), expected_iter.valid());
                        }
                    }
                    // case for `seek`
                    2 => {
                        let rkey = random_seek_key(keys, self.reverse_cmp);
                        let key = Slice::from(rkey.as_slice());
                        iter.seek(&key);
                        expected_iter.seek(&key);
                        if iter.valid() {
                            assert_eq!(format_entry(iter.as_ref()), format_entry(&expected_iter));
                        } else {
                            assert_eq!(iter.valid(), expected_iter.valid());
                        }
                    }
                    // case for `prev`
                    3 => {
                        if iter.valid() {
                            iter.prev();
                            expected_iter.prev();
                            if iter.valid() {
                                assert_eq!(
                                    format_entry(iter.as_ref()),
                                    format_entry(&expected_iter)
                                );
                            } else {
                                assert_eq!(iter.valid(), expected_iter.valid());
                            }
                        }
                    }
                    // case for `seek_to_last`
                    4 => {
                        iter.seek_to_last();
                        expected_iter.seek_to_last();
                        if iter.valid() {
                            assert_eq!(format_entry(iter.as_ref()), format_entry(&expected_iter));
                        } else {
                            assert_eq!(iter.valid(), expected_iter.valid());
                        }
                    }
                    _ => { /* ignore */ }
                }
            }
        }

        fn do_test(&mut self) {
            let keys = self.inner.finish(self.options.clone());
            let expected = self.inner.data.clone();
            self.test_forward_scan(&expected);
            self.test_backward_scan(&expected);
            self.test_random_access(&keys, expected);
        }
    }

    #[inline]
    fn format_kv(key: Vec<u8>, value: Vec<u8>) -> String {
        unsafe {
            format!(
                "'{}->{}'",
                String::from_utf8_unchecked(key),
                String::from_utf8_unchecked(value)
            )
        }
    }

    // Return a String represents current entry of the given iterator
    #[inline]
    fn format_entry(iter: &dyn Iterator) -> String {
        format!("'{:?}->{:?}'", iter.key(), iter.value())
    }

    fn random_seek_key(keys: &[Vec<u8>], reverse_cmp: bool) -> Vec<u8> {
        if keys.is_empty() {
            b"foo".to_vec()
        } else {
            let mut rnd = rand::thread_rng();
            let result = keys.get(rnd.gen_range(0, keys.len())).unwrap();
            match rnd.gen_range(0, 3) {
                1 => {
                    // Attempt to return something smaller than an existing key
                    let mut cloned = result.clone();
                    if !cloned.is_empty() && *cloned.last().unwrap() > 0u8 {
                        let last = cloned.last_mut().unwrap();
                        *last -= 1
                    }
                    cloned
                }
                2 => {
                    // Return something larger than an existing key
                    let mut cloned = result.clone();
                    if reverse_cmp {
                        cloned.insert(0, 0)
                    } else {
                        cloned.push(0);
                    }
                    cloned
                }
                _ => result.clone(), // Return an existing key
            }
        }
    }

    enum TestType {
        Table,
        Block,
        Memtable,
        #[allow(dead_code)]
        DB, // TODO: Enable DB test util fundamental components are stable
    }

    fn new_test_suits() -> Vec<TestHarness> {
        let mut tests = vec![
            (TestType::Table, false, 16),
            (TestType::Table, false, 1),
            (TestType::Table, false, 1024),
            (TestType::Table, true, 16),
            (TestType::Table, true, 1),
            (TestType::Table, true, 1024),
            (TestType::Block, false, 16),
            (TestType::Block, false, 1),
            (TestType::Block, false, 1024),
            (TestType::Block, true, 16),
            (TestType::Block, true, 1),
            (TestType::Block, true, 1024),
            // Restart interval does not matter for memtables
            (TestType::Memtable, false, 16),
            (TestType::Memtable, true, 16),
            // Do not bother with restart interval variations for DB
            // (TestType::DB, false, 16),
            // (TestType::DB, true, 16),
        ];
        let mut results = vec![];
        for (t, reverse_cmp, restart_interval) in tests.drain(..) {
            results.push(TestHarness::new(t, reverse_cmp, restart_interval));
        }
        results
    }

    fn random_key(length: usize) -> Vec<u8> {
        let chars = vec![
            '0', '1', 'a', 'b', 'c', 'd', 'e', '\u{00fd}', '\u{00fe}', '\u{00ff}',
        ];
        let mut rnd = rand::thread_rng();
        let mut result = vec![];
        for _ in 0..length {
            let i = rnd.gen_range(0, chars.len());
            let v = chars.get(i).unwrap();
            let mut buf = vec![0; v.len_utf8()];
            v.encode_utf8(&mut buf);
            result.append(&mut buf);
        }
        result
    }

    fn random_value(length: usize) -> Vec<u8> {
        let mut result = vec![0u8; length];
        let mut rnd = rand::thread_rng();
        for i in 0..length {
            let v = rnd.gen_range(0, 96);
            result[i] = v as u8;
        }
        result
    }

    #[test]
    fn test_empty_harness() {
        for mut test in new_test_suits().drain(..) {
            test.do_test()
        }
    }

    #[test]
    fn test_simple_empty_key() {
        for mut test in new_test_suits().drain(..) {
            test.add(b"", b"v");
            test.do_test();
        }
    }

    #[test]
    fn test_single_key() {
        for mut test in new_test_suits().drain(..) {
            test.add(b"abc", b"v");
            test.do_test();
        }
    }

    #[test]
    fn test_mutiple_key() {
        for mut test in new_test_suits().drain(..) {
            test.add(b"abc", b"v");
            test.add(b"abcd", b"v");
            test.add(b"ac", b"v2");
            test.do_test();
        }
    }

    #[test]
    fn test_special_key() {
        for mut test in new_test_suits().drain(..) {
            test.add(b"\xff\xff", b"v");
            test.do_test();
        }
    }

    #[test]
    fn test_randomized_key() {
        let mut rnd = rand::thread_rng();
        for mut test in new_test_suits().drain(..) {
            for _ in 0..1000 {
                let key = random_key(rnd.gen_range(1, 10));
                let value = random_value(rnd.gen_range(1, 5));
                test.add(&key, &value);
            }
            test.do_test();
        }
    }
}
