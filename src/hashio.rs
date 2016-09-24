//! IO operations for hashable objects.
//!
//! # Usage
//! The main point here is to create objecs which can iteract with the HashIO
//! object.  These objects should be able to (de)serialize themselves, the children.
//! To do this, the HashIOImpl trait can be implemented and to be able to do this,
//! several the type must provide some features:
//!
//! * Hashable:  Create a hash which represents its content.
//! * Typeable:  Create a hash which represents its type.
//! * It should store a version number and the type hash and check against it when loading.
//!
//! This sounds complicated and that's why there are marcos which implement
//! everything required.


extern crate crypto;
extern crate byteorder;

use std::io::{Read, Write};
use std::{io, error, fmt};
use hash::*;
use io::*;
use std::fs::{File, create_dir_all};
use std::collections::BTreeMap;
use std::path::Path;
use std::fs::rename;
use std::result;
use std::rc::Rc;


/// Default error type for HashIO.
#[derive(Debug)]
pub enum HashIOError {
    Undefined(String),
    VersionError(u32),
    TypeError(Hash),
    IOError(io::Error),
    ParseError(Box<error::Error>)
}
pub type Result<T> = result::Result<T, HashIOError>;

impl fmt::Display for HashIOError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            HashIOError::Undefined(ref msg) => write!(f, "Undefined error: {}", msg),
            HashIOError::VersionError(version) => write!(f, "Unsupported version: {}", version),
            HashIOError::TypeError(ref hash) => write!(f, "Unexpected type: {}", hash.as_string()),
            HashIOError::IOError(ref err) => err.fmt(f),
            HashIOError::ParseError(ref err) => write!(f, "Parse error: {}", err)
        }
    }
}
impl error::Error for HashIOError {
    fn description(&self) -> &str {
        match *self {
            HashIOError::Undefined(ref msg) => msg,
            HashIOError::VersionError(_) => "Unsupported version",
            HashIOError::TypeError(_) => "Unexpected type",
            HashIOError::IOError(ref err) => err.description(),
            HashIOError::ParseError(ref err) => err.description()
        }
    }
}
impl From<io::Error> for HashIOError {
    fn from(err: io::Error) -> HashIOError {
        HashIOError::IOError(err)
    }
}


/// Structure to store and lead HashIO-able values
#[derive(Clone, Debug, PartialEq)]
pub struct HashIOFile {
    pub base_path: String,
}

/// Allows a type to identify itself.
pub trait Typeable {
    /// Identifies the type using a unique hash value.
    fn type_hash() -> Hash;

    /// Identifies the type name
    ///
    /// This is very handy to create error messages.
    fn type_name() -> String;
}

pub trait HashIOType: Hashable {
    fn childs(&self) -> BTreeMap<String, Rc<HashIOType>> {
        BTreeMap::new()
    }

    fn type_hash_obj(&self) -> Hash;
    fn type_name_obj(&self) -> String;
}

pub trait HashIOParse: HashIOType + Typeable {
    fn parse<H, R>(hash_io: &H, read: &mut R, type_hash: &Option<Hash>) -> Result<Rc<Self>>
        where H: HashIO, R: Read;
    fn store<H, W>(&self, hash_io: &H, write: &mut W) -> Result<()>
        where H: HashIO, W: Write;
    fn store_childs<H>(&self, _: &H) -> Result<()>
        where H: HashIO {
        Ok(())
    }

    fn unsafe_loader() -> bool {
        false
    }
    fn version_valid(version: u32) -> bool {
        version == 1
    }
    fn type_hash_valid(_: &Hash) -> bool {
        false
    }
}

pub trait HashIO {
    fn get<T>(&self, hash: &Hash) -> Result<Rc<T>>
                where T: HashIOParse;
    fn put<T>(&self, item: &T) -> Result<()>
                where T: HashIOParse;
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
                return Err(HashIOError::VersionError(version))
            }
            type_hash = Some(try!(read_hash(&mut read)));
            if !T::type_hash_valid(&type_hash.unwrap()) {
                return Err(HashIOError::TypeError(type_hash.unwrap()))
            }
        }
        let res = try!(T::parse(self, &mut read, &type_hash));
        Ok(res)
    }

    fn put<T>(&self, item: &T) -> Result<()>
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

