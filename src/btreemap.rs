use hash::*;
use io::*;
use hashio::*;
use std::io::{Read, Write};
use std::result;
use std::io;
use std::collections::BTreeMap;
use std::rc::Rc;

impl<T,U> Writable for BTreeMap<Rc<T>, Rc<U>> 
            where T: HashIOParse, U: HashIOParse {
    fn write_to<W: Write>(&self, write: &mut W) -> result::Result<usize, io::Error> {
        try!(write_u32(0, write));
        try!(write_u32(self.len() as u32, write));
        for (key, item) in self {
            try!(write_hash(&key.as_hash(), write));
            try!(write_hash(&item.as_hash(), write));
        }
        return Ok(8 + self.len() * 64)
    }
}

impl<T, U> Hashable for BTreeMap<Rc<T>, Rc<U>> 
        where T: HashIOParse, U: HashIOParse {
    fn as_hash(&self) -> Hash {
        self.writable_to_hash()
    }
}

impl<T, U> Typeable for BTreeMap<Rc<T>, Rc<U>> 
        where T: HashIOParse, U: HashIOParse {
    fn type_hash() -> Hash {
        Hash::hash_string(BTreeMap::<Rc<T>, Rc<U>>::type_name())
    }

    fn type_name() -> String {
        "BTreeMap<".to_string() + &T::type_name() + "," +
                &U::type_name() + ">"
    }
}

impl<T, U> HashIOType for BTreeMap<Rc<T>, Rc<U>> 
        where T: HashIOParse + 'static, U: HashIOParse + 'static {
    fn childs(&self) -> BTreeMap<String, Rc<HashIOType>> {
        let mut res: BTreeMap<String, Rc<HashIOType>> = BTreeMap::new();
        for (key, item) in self {
            let key_str = format!("{:?}", key);
            let boxed_item_object: Rc<HashIOType> = item.clone() as Rc<HashIOType>;
            res.insert(key_str.to_string(), boxed_item_object);
        }
        res
    }

    fn type_hash_obj(&self) -> Hash {
        BTreeMap::<Rc<T>, Rc<U>>::type_hash()
    }

    fn type_name_obj(&self) -> String {
        BTreeMap::<Rc<T>, Rc<U>>::type_name()
    }
}

impl<T, U> HashIOParse for BTreeMap<Rc<T>, Rc<U>> 
            where T: HashIOParse + Ord + 'static, U: HashIOParse + 'static {
    fn parse<H, R>(hash_io: &H, read: &mut R, _: &Option<Hash>)
                -> Result<Rc<Self>> where H: HashIO, R: Read {
        // read and ignore version
        try!(read_u32(read));
        let len = try!(read_u32(read));
        let mut res: BTreeMap<Rc<T>, Rc<U>> = BTreeMap::new();
        for _ in 0..len {
            let key_hash = try!(read_hash(read));
            let key: Rc<T> = try!(hash_io.get(&key_hash));
            let val_hash = try!(read_hash(read));
            let val: Rc<U> = try!(hash_io.get(&val_hash));
            res.insert(key, val);
        }
        Ok(Rc::new(res))
    }

    fn store<H, W>(&self, _: &H, write: &mut W) -> Result<()> where H: HashIO, W: Write {
        // write version
        try!(write_u32(0, write));
        try!(write_u32(self.len() as u32, write));
        for (key, item) in self {
            try!(write_hash(&key.as_hash(), write));
            try!(write_hash(&item.as_hash(), write));
        }
        Ok(())
    }

    fn store_childs<H>(&self, hash_io: &H) -> Result<()>
            where H: HashIO {
        for (key, item) in self {
            try!(hash_io.put(key.clone()));
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
        a: BTreeMap< Rc<String>, Rc<String> >
    }}

    #[test]
    fn test() {
        remove_dir_all("./unittest/btreemaptest/").ok();
        let hash_io = HashIOFile::new("unittest/btreemaptest".to_string());
        let mut btreemap: BTreeMap<Rc<String>, Rc<String>> = BTreeMap::new();
        btreemap.insert(Rc::new("a".to_string()), Rc::new("1".to_string()));
        btreemap.insert(Rc::new("b".to_string()), Rc::new("2".to_string()));
        btreemap.insert(Rc::new("c".to_string()), Rc::new("3".to_string()));

        
        let my_obj = TestType {
            a: Rc::new(btreemap)
        };
        let my_hash = my_obj.as_hash();
        trace!("Insert data");
        
        hash_io.put(Rc::new(my_obj)).unwrap();

        trace!("Reading data");
        let my_obj: Rc<TestType> = hash_io.get(&my_hash).unwrap();
        let my_btree = my_obj.a.clone();
        assert_eq!(&Rc::new("1".to_string()), my_btree.get(&Rc::new("a".to_string())).unwrap());
        assert_eq!(&Rc::new("2".to_string()), my_btree.get(&Rc::new("b".to_string())).unwrap());
        assert_eq!(&Rc::new("3".to_string()), my_btree.get(&Rc::new("c".to_string())).unwrap());
    }
}