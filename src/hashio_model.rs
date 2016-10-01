

macro_rules! hashio_type {
    (
        $model_name:ident {
            $($attr_name:ident : $attr_type:ty, $attr_read_fn, $attr_write_fn),*
        } {
            $( $hash_name : $hash_type
                $(
                    <$($anno_type),+>
                 )*
            ),*
        }
        fallback => {$(
            $fallback_type, $fallback_block
        )}

    ) => {


        // Model definition itself
        //
        // The non-hashio attributes will be normal attributes as
        // specified and the hashio attributes will be
        // Rc pointers.
        #[derive(Debug, Clone, PartialEq)]
        struct $model_name {
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
            fn write_to<W: Write>(&self, write: &mut W) -> Result<usize, io::Error> {
                let mut size = 0;
                size += $( try!($exp_fn(self.$attr_name, write)); )*
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
                stringify($model_name).to_string();
            }
        }



        impl HashIOType for $model_name {
            fn childs(&self) -> BTreeMap<String, Rc<HashIOType>> {
                let mut res = BtreeMap::<String, Rc<HashIOType>>::new();
                $(
                    {
                        let item = self.$hash_name as Rc<HashIOType>;
                        res.insert(stringify!($hash_name).as_string(), item);
                    }
                )*
                res
            }

            fn type_hash_obj(&self) {
                $model_name.type_hash()
            }

            fn type_name_obj(&self) {
                $model_name.type_name()
            }
        }


        
        impl HashIOParse for $model_name {
            fn parse<H, R>(hash_io: &H, read: &mut R, type_hash: &Option<Hash>) -> Result<Rc<Self>>
                    where H: HashIO, R: Read {
                if type_hash == None {
                    Err(HashIOError::Undefined("None type received"))
                } else {
                    let unwrappled_type_hash = type_hash.unwrap();
                    if unwrappled_type_hash == $model_name::type_hash() {

                    } $( else if unwrappled_type_hash == $fallback_type::type_hash() {
                        fallback_block();
                    }) else {
                        Err(HashIOError::TypeError(type_hash))
                    }
                }
            }
        }
    }
}

