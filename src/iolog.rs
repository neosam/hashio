//! Implementation of the hashable log which uses a HashIO in the background.

/*extern crate time;

use hash::*;
use hashio::*;
use io::*;
use logger::*;
use std::io;
use std::io::{Write, Read};
use std::fs::{File};
use self::time::{now};
use iolog_1::{IOLog1, IOLogItem1};
use hashio_1::{HashIO1, HashIOImpl1};

impl From<HashIOError> for LogError {
    fn from(hash_io_error: HashIOError) -> LogError {
        LogError::CustomError(format!("LogError::HashIOError: {}", hash_io_error))
    }
}

pub struct IOLogItem<T>
        where T: Hashtype,
              HashIO: HashIOImpl<T> {
    pub parent_hash: Hash,
    pub item: T
}

impl<T> Writable for IOLogItem<T>
        where T: Hashtype,
              HashIO: HashIOImpl<T> {
    fn write_to<W: Write>(&self, write: &mut W) -> Result<usize, io::Error> {
        let mut size = 0;
        size += try!(write_u32(1, write)); // write version
        size += try!(write_hash(&IOLogItem::<T>::type_hash(), write));
        size += try!(write_hash(&self.parent_hash, write));
        try!(write_hash(&self.item.as_hash(), write));
        size += 32;
        Ok(size)
    }
}
impl<T> Hashable for IOLogItem<T>
        where T: Hashtype,
              HashIO: HashIOImpl<T> {
    fn as_hash(&self) -> Hash {
        self.writable_to_hash()
    }
}

impl<T> Typeable for IOLogItem<T>
        where T: Hashtype,
              HashIO: HashIOImpl<T> {
    fn type_hash() -> Hash {
        let mut byte_gen: Vec<u8> = Vec::new();
        let id = String::from("IOLogItem");
        let id_bytes = id.as_bytes();
        byte_gen.extend_from_slice(&*Hash::hash_bytes(id_bytes).get_bytes());
        byte_gen.extend_from_slice(&*T::type_hash().get_bytes());
        let hash = Hash::hash_bytes(byte_gen.as_slice());
        hash
    }

    fn type_name() -> String {
        let mut res = String::from("IOLogItem<");
        res.push_str(&T::type_name());
        res.push_str(">");
        res
    }
}
impl<T> Hashtype for IOLogItem<T>
        where T: Hashtype,
              HashIO: HashIOImpl<T> {}

impl<T> HashIOImpl<IOLogItem<T>> for HashIO
        where T: Hashtype,
              HashIO: HashIOImpl<T> {
    fn receive_hashable<R>(&self, read: &mut R, _: &Hash) -> Result<IOLogItem<T>, HashIOError>
            where R: Read {
        let version = try!(read_u32(read));
        if version < 1 {
            return Err(HashIOError::VersionError(version))
        }
        let type_hash = try!(read_hash(read));
        if type_hash != IOLogItem::<T>::type_hash() {
            return Err(HashIOError::TypeError(type_hash))
        }
        let parent_hash = try!(read_hash(read));
        let item;
        {
            let hash_val = try!(read_hash(read));
            item = try!(self.get(&hash_val));
        }
        Ok(IOLogItem {
            parent_hash: parent_hash,
            item: item
        })
    }

    fn store_childs(&self, hashable: &IOLogItem<T>) -> Result<(), HashIOError> {
        try!(self.put(&hashable.item));
        Ok(())
    }

    fn store_hashable<W>(&self, hashable: &IOLogItem<T>, write: &mut W) -> Result<(), HashIOError>
            where W: Write {
        try!(hashable.write_to(write));
        Ok(())
    }


}

pub struct IOLog<T>
        where T: Hashtype,
              HashIO: HashIOImpl<T> {
    pub head: Option<IOLogItem<T>>,
    pub hashio: HashIO
}

impl<T> IOLog<T>
        where T: Hashtype,
              HashIO: HashIOImpl<T> {
    pub fn write_head(&self) -> Result<(), io::Error> {
        if self.head.is_some() {
            let now = time::now();
            let hashio = &self.hashio;
            let timestamp = format!("{}/head-{}", hashio.base_path, now.rfc3339());
            let hash = self.head.as_ref().unwrap().as_hash();
            let filename =  format!("{}/head", hashio.base_path);
            let mut file = try!(File::create(filename));
            try!(write_hash(&hash, &mut file));
            let mut backup = try!(File::create(timestamp));
            try!(write_hash(&hash, &mut backup));
        }
        Ok(())
    }
}

impl<T> Log for IOLog<T>
        where T: Hashtype,
              HashIO: HashIOImpl<T> {
    type Item = T;

    /// Add new entry to the log
    fn push(&mut self, hashable: T) -> Hash {
        let new_head = IOLogItem {
            parent_hash: match &self.head {
                &Option::None => Hash::None,
                &Option::Some(ref parent_item) => parent_item.as_hash()
            },
            item: hashable
        };
        let parent_hash = new_head.parent_hash.clone();
        match self.hashio.put::<IOLogItem<T>>(&new_head) {
            Ok(_) => (),
            Err(_) => return Hash::None
        }
        let hash = new_head.as_hash();
        self.head = Some(new_head);
        match self.write_head() {
            Ok(_) => (),
            Err(_) => { return Hash::None }
        };
        if hash == parent_hash {
            warn!("hash equals parent hash\n");
        }
        hash
    }


    /// Head hash
    fn head_hash(&self) -> Option<Hash> {
        match &self.head {
            &Option::None => Option::None,
            &Option::Some(ref item) => Some(item.as_hash())
        }
    }

    /// Get the parent hash of the given hash.
    ///
    /// If the given hash is the first entry without a successor, it returns
    /// None, otherwise it returns the hash wrapped in Option::Some.
    ///
    /// # Errors
    /// Throws an error if an entry of the hash was not found.
    fn parent_hash(&self, hash: Hash) -> Result<Option<Hash>, LogError> {
        let item: IOLogItem<T> = try!(self.hashio.get::<IOLogItem<T>>(&hash));
        if item.parent_hash == hash {
            warn!("parent_hash detected redundancy\n");
        }
        let res = Ok(match item.parent_hash {
            Hash::None => Option::None,
            _ => Option::Some(item.parent_hash)
        });
        res
    }

    /// Get the borrowed entry of the given hash
    ///
    /// # Errors
    /// Throws an error if an entry of the hash was not found.
    fn get(&self, hash: Hash) -> Result<Self::Item, LogError> {
        let item: IOLogItem<T> = try!(self.hashio.get::<IOLogItem<T>>(&hash));
        Ok(item.item)
    }

    // Set defferent head
    fn reset_head(&mut self, hash: &Hash) -> Result<(), LogError> {
        let item: IOLogItem<T> = try!(self.hashio.get::<IOLogItem<T>>(&hash));
        self.head = Some(item);
        Ok(())
    }
}

impl<T> IOLog<T>
        where T: Hashtype,
            HashIO: HashIOImpl<T> {
    pub fn new(path: String) -> IOLog<T> {
        let hashio = HashIO::new(path.clone());
        let filename = format!("{}/head", path.clone());
        let hash = match File::open(filename) {
            Ok(mut file) => read_hash(&mut file).unwrap_or(Hash::None),
            Err(_) => Hash::None
        };
        let head = match hash {
            Hash::None => Option::None,
            _ => hashio.get::<IOLogItem<T>>(&hash).ok()
        };
        IOLog{
            head: head,
            hashio: HashIO::new(path)
        }
    }
}

impl<T, U> From<IOLogItem1<T>> for IOLogItem<U>
        where T: Hashable, U: Hashtype,
              HashIO1: HashIOImpl1<T>,
              HashIO: HashIOImpl<U>,
              U: From<T> {
    fn from(f: IOLogItem1<T>) -> IOLogItem<U> {
        IOLogItem {
            parent_hash: f.parent_hash,
            item: U::from(f.item)
        }
    }
}

impl<T, U> From<IOLog1<T>> for IOLog<U>
        where T: Hashable, U: Hashtype,
                  HashIO1: HashIOImpl1<T>,
                  HashIO: HashIOImpl<U>,
                  U: From<T> {
    fn from(f: IOLog1<T>) -> IOLog<U> {
        IOLog {
            head: f.head.map(| x | IOLogItem::from(x) ),
            hashio: HashIO::new(f.hashio.base_path)
        }
    }
}


#[cfg(test)]
mod test {
    use super::super::hash::*;
    use super::super::hashio::*;
    use super::super::io::*;
    use super::super::logger::*;
    use super::*;
    use std::io::{Read, Write};
    use std::io;
    use std::fs::remove_dir_all;
    use super::super::hashio_1;

    tbd_model!(A, [
        [a: u8, write_u8, read_u8]
     ], [
        [b: String]
     ]);

    #[test]
    fn test() {
        remove_dir_all("unittest/logtest").ok();
        let mut log = IOLog::<A>::new("unittest/logtest".to_string());
        // make sure the log is empty
        assert_eq!(None, log.head_hash());

        let one = A{a: 1, b: "one".to_string()};
        let two = A{a: 2, b: "two".to_string()};
        let hash_one = log.push(one.clone());
        let hash_two = log.push(two.clone());

        print!("Hash written: {}\n", hash_one.as_string());
        print!("Hash written: {}\n", hash_two.as_string());
        let one_ref: A = log.get(hash_one).unwrap();
        let two_ref: A = log.get(hash_two).unwrap();
        assert_eq!(one, one_ref);
        assert_eq!(two, two_ref);

        // Verify if reloading works correcty
        println!("Verify reloading\n");
        let log2 = IOLog::<A>::new("unittest/logtest".to_string());
        let two_ref2: A = log.get(log2.head_hash().unwrap()).unwrap();
        assert_eq!(two, two_ref2);

        println!("Log3");
        let log3 = IOLog::<A>::new("unittest/logtest".to_string());
        assert_eq!(Ok(Some(hash_one)), log3.parent_hash(hash_two));

        let mut hash_iter = LogIteratorHash::from_log(&log3);
        print!("Hash two\n");
        assert_eq!(Some(hash_two), hash_iter.next());
        print!("Hash one\n");
        assert_eq!(Some(hash_one), hash_iter.next());
        assert_eq!(None, hash_iter.next());

        let mut iter = LogIteratorRef::from_log(&log3);
        assert_eq!(Some(two), iter.next());
        assert_eq!(Some(one), iter.next());
        assert_eq!(None, iter.next());
    }
}
*/