

macro_rules! hashio_type {
    
        ($model_name:ident {
            $([$attr_name:ident : $attr_type:ty, $attr_read_fn:ident, $attr_write_fn:ident]),*
        } {
            $( $hash_name:ident : $hash_type:ident
                $(
                    <$($anno_type:ident),+>
                 )*
            ),*
        }
        $(fallback => $fallback_type:ident)*
        $(plain_fallback => $plain_fallback_fn:ident)*

        ) => {


        // Model definition itself
        //
        // The non-hashio attributes will be normal attributes as
        // specified and the hashio attributes will be
        // Rc pointers.
        #[derive(Debug, Clone, PartialEq)]
        pub struct $model_name {
            $(pub $attr_name: $attr_type,)*
            $(pub $hash_name: Rc<$hash_type $(<$($anno_type),+>)*),*>
        }

        // Make it writeable so the model is able to write into
        // a write stream which in turn can be turned into a
        // hashable again
        //
        // Use the exp_fn for the non hashio attributes.  Only the 
        // hash of the HashIO attributes will be stored
        impl Writable for $model_name {
            fn write_to<W: Write>(&self, write: &mut W) -> result::Result<usize, io::Error> {
                let mut size = 0;
                size += $( try!($attr_write_fn(self.$attr_name, write)); )*
                $(
                    try!(write_hash(&self.$hash_name.as_hash(), write));
                    size += 32;
                )*
                Ok(size)
            }
        }
        hashable_for_writable!($model_name);


        // Make the type able to represent itself 
        impl Typeable for $model_name {
            fn type_hash() -> Hash {
                let mut byte_gen: Vec<u8> = Vec::new();
                $(
                    {
                        let type_string = stringify!($attr_type);
                        let type_bytes = type_string.as_bytes();
                        let type_hash = Hash::hash_bytes(type_bytes);
                        byte_gen.extend_from_slice(&*type_hash.get_bytes());
                    };
                )*
                $(
                    {
                        let type_hash: Hash = $hash_type$(::<$($anno_type),+>)*::type_hash();
                        byte_gen.extend_from_slice(&*type_hash.get_bytes());
                    };
                )*
                Hash::hash_bytes(byte_gen.as_slice())
            }

            fn type_name() -> String {
                stringify!($model_name).to_string()
            }
        }



        impl HashIOType for $model_name {
            fn childs(&self) -> BTreeMap<String, Rc<HashIOType>> {
                let mut res = BTreeMap::<String, Rc<HashIOType>>::new();
                $(
                    {
                        let item = self.$hash_name.clone() as Rc<HashIOType>;
                        res.insert(stringify!($hash_name).to_string(), item);
                    }
                )*
                res
            }

            fn type_hash_obj(&self) -> Hash {
                $model_name::type_hash()
            }

            fn type_name_obj(&self) -> String {
                $model_name::type_name()
            }
        }



        impl HashIOParse for $model_name {
            fn parse<H, R>(hash_io: &H, read: &mut R, type_hash: &Option<Hash>) -> Result<Rc<Self>>
                    where H: HashIO, R: Read {
                if *type_hash == None {
                    Err(HashIOError::Undefined("None type received".to_string()))
                } else {
                    let unwrappled_type_hash = type_hash.unwrap();
                    if unwrappled_type_hash == $model_name::type_hash() {
                        $(
                            let $attr_name: $attr_type = try!($attr_read_fn(read));
                        )*
                        $(
                            let $hash_name: Rc<$hash_type> = {
                                let hash = try!(read_hash(read));
                                try!(hash_io.get(&hash))
                            };
                        )*
                        Ok(Rc::new($model_name {
                            $($attr_name: $attr_name,)*
                            $($hash_name: $hash_name),*
                        }))
                    } $( else if unwrappled_type_hash == $fallback_type::type_hash() {
                        let fallback_obj = try!($fallback_type::parse(hash_io, read, type_hash));
                        Ok(Rc::new($model_name::from(fallback_obj)))
                    })* else {
                        Err(HashIOError::TypeError(*type_hash.as_ref().unwrap()))
                    }
                }
            }

            fn store<H, W>(&self, _: &H, write: &mut W) -> Result<()> 
                    where H: HashIO, W: Write {
                try!(self.write_to(write));
                Ok(())
            }

            fn store_childs<H>(&self, hash_io: &H) -> Result<()> 
                    where H: HashIO {
                $(
                    try!(hash_io.put(self.$hash_name.clone()));
                )*
                Ok(())
            }

            $(fn fallback_parse<H, R>(hash_io: &H, read: &mut R) -> Result<Rc<Self>>
                    where H: HashIO, R: Read {
                $plain_fallback_fn(hash_io, read)
            })*

            fn type_hash_valid(hash: &Hash) -> bool {
                if *hash == $model_name::type_hash() {
                    true
                } $(else if *hash == $fallback_type::type_hash() {
                    true
                })* else {
                    false
                }
            }            
        }
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


    hashio_type! {
        TestTypeOld {
        } {
            a: String
        }
    }
    hashio_type! {
        TestType {
            [x: u32, read_u32, write_u32]
        } {
            a: String
        }
        fallback => TestTypeOld
        plain_fallback => plain_fallback
    }

    impl From<Rc<TestTypeOld>> for TestType {
        fn from(old: Rc<TestTypeOld>) -> TestType {
            TestType {
                x: 0,
                a: old.a.clone()
            }
        }
    }
    fn plain_fallback<H, R>(hash_io: &H, read: &mut R) -> Result<Rc<TestType>>
            where H: HashIO, R: Read {
        let x = try!(read_u32(read));
        Ok(Rc::new(TestType {
            x: x,
            a: Rc::new("".to_string())
        }))
    }


    #[test]
    fn test() {
        let test_obj = Rc::new(TestType {
            x: 1,
            a: Rc::new("abc".to_string())
        });
        assert_eq!(TestType::type_hash().as_string(), test_obj.type_hash_obj().as_string());
        assert_eq!("TestType".to_string(), TestType::type_name());
        assert_eq!("TestType".to_string(), test_obj.type_name_obj());
    }
}
