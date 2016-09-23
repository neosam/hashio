
use hash::*;
use io::*;
use hashio::*;
use std::io::{Read, Write};
use std::result;
use std::io;
use std::collections::BTreeMap;
use std::rc::Rc;

impl Writable for String {
    fn write_to<W: Write>(&self, write: &mut W) -> result::Result<usize, io::Error> {
        let str_bytes = self.as_bytes();
        let len = usize_to_u32_bytes(str_bytes.len());
        let mut size: usize = 0;
        size += try!(write.write(&len));
        size += try!(write.write(&str_bytes));
        Ok(size)
    }
}
hashable_for_writable!(String);


impl Typeable for String {
    fn type_name() -> String {
        "String".to_string()
    }
    fn type_hash() -> Hash {
        Hash::hash_string("String".to_string())
    }
}

impl HashIOType for String {
    fn childs(&self) -> BTreeMap<String, Box<HashIOType>> {
        BTreeMap::new()
    }

    fn type_hash_obj(&self) -> Hash {
        String::type_hash()
    }
    fn type_name_obj(&self) -> String {
        String::type_name()
    }
}

impl HashIOParse for String {
    fn parse<H, R>(_: &H, read: &mut R, _: &Option<Hash>) -> Result<Rc<Self>>
        where H: HashIO, R: Read {
        let len = try!(read_u32(read));
        let bytes = try!(read_bytes(read, len as usize));
        let res = try!(String::from_utf8(bytes).map_err(|x| HashIOError::ParseError(Box::new(x))));
        Ok(Rc::new(res))
    }
    fn store<H, W>(&self, _: &H, write: &mut W) -> Result<()>
        where H: HashIO, W: Write {
        try!(self.write_to(write));
        Ok(())
    }

    fn unsafe_loader() -> bool {
        true
    }
}