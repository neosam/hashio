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
use std::collections::BTreeMap;
use std::result;
use std::rc::Rc;
use std::fmt::Debug;

/// Default error type for HashIO.
#[derive(Debug)]
pub enum HashIOError {
    Undefined(String),
    VersionError(u32),
    TypeError(Hash),
    IOError(io::Error),
    ParseError(Box<error::Error>),
    FallbackNotSupported
}
pub type Result<T> = result::Result<T, HashIOError>;

impl fmt::Display for HashIOError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            HashIOError::Undefined(ref msg) => write!(f, "Undefined error: {}", msg),
            HashIOError::VersionError(version) => write!(f, "Unsupported version: {}", version),
            HashIOError::TypeError(ref hash) => write!(f, "Unexpected type: {}", hash.as_string()),
            HashIOError::IOError(ref err) => write!(f, "IOError: {}", err),
            HashIOError::ParseError(ref err) => write!(f, "Parse error: {}", err),
            HashIOError::FallbackNotSupported => write!(f, "Fallback is not supported")
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
            HashIOError::ParseError(ref err) => err.description(),
            HashIOError::FallbackNotSupported => "Fallback is not supported"
        }
    }
}
impl From<io::Error> for HashIOError {
    fn from(err: io::Error) -> HashIOError {
        HashIOError::IOError(err)
    }
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


/// HashIO trait used to identify trait object types.
///
/// In order to do analytics on an abstract level, this
/// trait provides all required information.  It knows its
/// type name, its type type hash and its hashio children.
pub trait HashIOType: Hashable + Debug {
    fn childs(&self) -> BTreeMap<String, Rc<HashIOType>> {
        BTreeMap::new()
    }

    fn type_hash_obj(&self) -> Hash;
    fn type_name_obj(&self) -> String;
}



/// Complete HashIO type which is able to be stored and
/// read from hashio implementations.
pub trait HashIOParse: HashIOType + Typeable {
    fn parse<H, R>(hash_io: &H, read: &mut R, type_hash: &Option<Hash>) -> Result<Rc<Self>>
        where H: HashIO, R: Read;
    fn store<H, W>(&self, hash_io: &H, write: &mut W) -> Result<()>
        where H: HashIO, W: Write;
    fn store_childs<H>(&self, _: &H) -> Result<()>
        where H: HashIO {
        Ok(())
    }
    fn fallback_parse<H, R>(_: &H, _: &mut R) -> Result<Rc<Self>>
            where H: HashIO, R: Read {
        Err(HashIOError::FallbackNotSupported)
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


/// HashIO implementations control the IO itself.
pub trait HashIO {
    fn get<T>(&self, hash: &Hash) -> Result<Rc<T>>
                where T: HashIOParse;
    fn put<T>(&self, item: Rc<T>) -> Result<()>
                where T: HashIOParse;
}




