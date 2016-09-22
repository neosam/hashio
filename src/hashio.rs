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
use std::vec::Vec;
use std::path::Path;
use std::fs::rename;
use std::result;


/// Default error type for HashIO.
#[derive(Debug)]
pub enum HashIOError {
    Undefined(String),
    VersionError(u32),
    TypeError(Hash),
    IOError(io::Error),
    ParseError(Box<error::Error>)
}
type Result<T> = result::Result<T, HashIOError>;

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
    fn store(&self) -> ();
    fn unsafe_loader(&self) -> bool {
        false
    }
    fn version_valid(&self, version: i32) -> bool {
        version == 1
    }
    fn type_hash_valid(&self, hash: &Hash) -> bool {
        false
    }
    fn childs(&self) -> BTreeMap<String, Box<HashIOType>>;

    fn type_hash_obj(&self) -> Hash;
    fn type_name_obj(&self) -> String;
}

pub trait HashIOParse: HashIOType + Typeable {
     fn parse<H>(hash_io: &H, write: &Write) -> Box<Self>
        where H: HashIO;
}

pub trait HashIO {
    fn get<T>(&self, hash: &Hash) -> T
                where T: HashIOParse;
    fn put<T>(&self, item: &T) -> ()
                where T: HashIOParse;
}
