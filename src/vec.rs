use hash::*;
use io::*;
use hashio::*;
use std::io::{Read, Write};
use std::result;
use std::io;
use std::collections::BTreeMap;
use std::rc::Rc;
use std::vec::Vec;

impl<T> Writable for Vec<Rc<T>> where T: HashIOParse {
    fn write_to<W: Write>(&self, write: &mut W) -> result::Result<usize, io::Error> {
        try!(write_u32(0, write));
        try!(write_u32(self.len() as u32, write));
        for item in self {
            try!(write_hash(&item.as_hash(), write));
        }
        return Ok(8 + self.len() * 32)
    }
}

impl<T> Hashable for Vec<Rc<T>> where T: HashIOParse {
    fn as_hash(&self) -> Hash {
        self.writable_to_hash()
    }
}

impl<T> Typeable for Vec<Rc<T>> where T: HashIOParse {
    fn type_hash() -> Hash {
        Hash::hash_string(Vec::<Rc<T>>::type_name())
    }

    fn type_name() -> String {
        "Vec<".to_string() + &T::type_name() + ">"
    }
}

impl<T> HashIOType for Vec<Rc<T>> where T: HashIOParse + 'static {
    fn childs(&self) -> BTreeMap<String, Rc<HashIOType>> {
        let mut i = 0;
        let mut res: BTreeMap<String, Rc<HashIOType>> = BTreeMap::new();
        for item in self {
            let i_str = format!("{}", i);
            let boxed_item_object: Rc<HashIOType> = item.clone() as Rc<HashIOType>;
            res.insert(i_str.to_string(), boxed_item_object);
            i += 1;
        }
        res
    }

    fn type_hash_obj(&self) -> Hash {
        Vec::<Rc<T>>::type_hash()
    }

    fn type_name_obj(&self) -> String {
        Vec::<Rc<T>>::type_name()
    }
}

impl<T> HashIOParse for Vec<Rc<T>> where T: HashIOParse + 'static {
    fn parse<H, R>(hash_io: &H, read: &mut R, _: &Option<Hash>)
                -> Result<Rc<Self>> where H: HashIO, R: Read {
        // read and ignore version
        try!(read_u32(read));
        let len = try!(read_u32(read));
        let mut res: Vec<Rc<T>> = Vec::new();
        for _ in 0..len {
            let item_hash = try!(read_hash(read));
            let item: Rc<T> = try!(hash_io.get(&item_hash));
            res.push(item);
        }
        Ok(Rc::new(res))
    }

    fn store<H, W>(&self, _: &H, write: &mut W) -> Result<()> where H: HashIO, W: Write {
        // write version
        try!(write_u32(0, write));
        try!(write_u32(self.len() as u32, write));
        for item in self {
            try!(write_hash(&item.as_hash(), write));
        }
        Ok(())
    }

    fn store_childs<H>(&self, hash_io: &H) -> Result<()>
            where H: HashIO {
        for item in self {
            try!(hash_io.put(item.clone()));
        }
        Ok(())
    }
    fn unsafe_loader() -> bool {
        true
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
    use hashiofile::HashIOFile;
    use std::fs::remove_dir_all;

    hashio_type!{ TestType {
    } {
        a: Vec< Rc<String> >
    }}

    #[test]
    fn test() {
        remove_dir_all("./unittest/vectest/").ok();
        let hash_io = HashIOFile::new("unittest/vectest".to_string());
        let my_vec = vec![Rc::new("a".to_string()), 
                          Rc::new("b".to_string()), 
                          Rc::new("c".to_string())];
        let my_obj = TestType {
            a: Rc::new(my_vec)
        };
        let my_hash = my_obj.as_hash();
        trace!("Insert data");
        hash_io.put(Rc::new(my_obj)).unwrap();

        trace!("Reading data");
        let my_obj: Rc<TestType> = hash_io.get(&my_hash).unwrap();
        let my_vec = my_obj.a.clone();
        assert_eq!(Rc::new("a".to_string()), my_vec[0]);
        assert_eq!(Rc::new("b".to_string()), my_vec[1]);
        assert_eq!(Rc::new("c".to_string()), my_vec[2]);
    }
}