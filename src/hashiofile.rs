extern crate crypto;
extern crate byteorder;

use hash::*;
use hashio::*;
use io::*;
use std::fs::{File, create_dir_all};
use std::path::Path;
use std::fs::rename;
use std::rc::Rc;


/// Structure to store and lead HashIO-able values
#[derive(Clone, Debug, PartialEq)]
pub struct HashIOFile {
    pub base_path: String,
}


impl HashIOFile {
    pub fn new(path: String) -> HashIOFile {
        HashIOFile {
            base_path: path.clone(),
        }
    }

    pub fn directory_for_hash(&self, hash: &Hash) -> String {
        let hash_str = hash.as_string();
        let mut result = String::new();
        result.push_str(&self.base_path);
        result.push('/');
        result.push_str(&hash_str[0..2]);
        result.push('/');
        result
    }

    pub fn filename_for_hash(&self, hash: &Hash) -> String {
        let hash_str = hash.as_string();
        let mut result = self.directory_for_hash(hash);
        result.push_str(&hash_str[2..]);
        result
    }
}

impl HashIO for HashIOFile {
    fn get<T>(&self, hash: &Hash) -> Result<Rc<T>>
                where T: HashIOParse {
        let filename = self.filename_for_hash(hash);
        let mut read = try!(File::open(filename.clone()));
        let mut type_hash: Option<Hash> = None;
        if !T::unsafe_loader() {
            let version = try!(read_u32(&mut read));
            if !T::version_valid(version) {
                // try fallback
                return T::fallback_parse(self, &mut read)
            }
            type_hash = Some(try!(read_hash(&mut read)));
            if !T::type_hash_valid(&type_hash.unwrap()) {
                return Err(HashIOError::TypeError(type_hash.unwrap()))
            }
        }
        let res = try!(T::parse(self, &mut read, &type_hash));
        Ok(res)
    }

    fn put<T>(&self, item: Rc<T>) -> Result<()>
                where T: HashIOParse {

        let hash = item.as_hash();
        let filename = self.filename_for_hash(&hash);

        // First, if the entry already exists, skip the insert because it's already saved.
        if !Path::new(&filename).exists() {
            // First store all childs and their childs.
            // So we make sure that all dependencies are available when the current object has
            // finished writing.
            try!(item.store_childs(self));

            // First write in a slightly modified file which will be renamed when writing was
            // finished.  So we only have valid files or nothing on the expected position but
            // nothing unfinished.
            let tmp_filename = filename.clone() + "_";

            let dir = self.directory_for_hash(&hash);
            try!(create_dir_all(dir));

            // Write is in an extra block to make sure, the write procodure will be completed
            // and the file is closed and completed before we rename the file.
            {
                let mut write = try!(File::create(tmp_filename.clone()));
                if !T::unsafe_loader() {
                    try!(write_u32(1, &mut write));
                    try!(write_hash(&T::type_hash(), &mut write));
                }
                try!(item.store(self, &mut write));
            }
            try!(rename(tmp_filename, filename));
        }
        Ok(())
    }
}


#[cfg(test)]
mod test {
    use super::super::io::*;
    use super::super::hashio::*;
    use std::io::{Read, Write};
    use std::{io, error, fmt};
    use hash::*;
    use std::collections::BTreeMap;
    use std::result;
    use std::rc::Rc;
    use hashiofile::*;

    hashio_type! {
        TestTypeOld {
        } {
            b: String
        }
    }
    hashio_type! {
        TestType {
            [a: u32, read_u32, write_u32]
        } {
            b: String
        }
        fallback => TestTypeOld
    }
    impl From<Rc<TestTypeOld>> for TestType {
        fn from(old: Rc<TestTypeOld>) -> TestType {
            TestType {
                a: 0,
                b: old.b.clone()
            }
        }
    }

    #[test]
    fn testIO() {
        let hash1;
        let hash2;
        {
            let mut hash_io = HashIOFile::new("unittest/hashiofile".to_string());
            let t1 = TestType {
                a: 42,
                b: Rc::new("Hello World".to_string())
            };
            let t2 = TestTypeOld {
                b: Rc::new("Foo".to_string())
            };
            hash1 = t1.as_hash();
            hash2 = t2.as_hash();
            hash_io.put(Rc::new(t1)).unwrap();
            hash_io.put(Rc::new(t2)).unwrap();
        }
        {
            let mut hash_io = HashIOFile::new("unittest/hashiofile".to_string());
            let t1: Rc<TestType> = hash_io.get(&hash1).unwrap();
            assert_eq!(42, t1.a);
            assert_eq!(Rc::new("Hello World".to_string()), t1.b);
            let t2: Rc<TestType> = hash_io.get(&hash2).unwrap();
            assert_eq!(0, t2.a);
            assert_eq!(Rc::new("Foo".to_string()), t2.b);
            let t2: Rc<TestTypeOld> = hash_io.get(&hash2).unwrap();
            assert_eq!(Rc::new("Foo".to_string()), t2.b);            
        }
    }
}