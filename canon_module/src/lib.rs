#[cfg(not(feature = "host"))]
mod passthrough {
    #[macro_export]
    macro_rules! module {
        // Opening case

        ( impl $name:ident {
            $($rest:tt)*
        } ) => {
            // MODULE PRELUDE

            use canonical::Canon as _;

            type BS = canonical::BridgeStore<canonical::Id32>;

            fn query(bytes: &mut [u8; 1024 * 64]) -> Result<(), <BS as canonical::Store>::Error> {
                let store = BS::default();
                let mut source = canonical::ByteSource::new(&bytes[..], store.clone());

                let query_tag = u16::read(&mut source)?;

                Ok(())
            }

            #[no_mangle]
            fn q(bytes: &mut [u8; 1024 * 64]) {
                match query(bytes) {
                    Ok(_) => (),
                    Err(_) => todo!(),
                }
            }

            #[no_mangle]
            fn t(bytes: &mut [u8; 1024 * 64]) {
                todo!("borf")
            }

            impl $name {
                module! { ( 0 ) ; $($rest)* }
            }
        };

        // Transactions

        ( $count:tt ; pub fn $name:ident ( & mut $self:ident $(, $arg:ident : $ty:ty)* ) -> $ret:ty {
            $($body:tt)*
        }

          $($rest:tt)*

        ) => {
            pub fn $name( & mut $self  $(, $arg : $ty)* ) -> $ret {
                $($body)*
            }

            module! { ($count + 1) ; $($rest)* }
        };

        // Queries

        ( $count:tt ; pub fn $name:ident ( & $self:ident $(, $arg:ident : $ty:ty)* ) $( -> $ret:ty )? {
            $($body:tt)*
        }

          $($rest:tt)*

        ) => {
            pub fn $name( & $self  $(, $arg : $ty)* ) $( -> $ret)? {
                $($body)*
            }

            module! { ( $count + 1) ; $($rest)* }
        };

        // Static methods

        ( $count:tt ; pub fn $name:ident ( $($arg:ident : $ty:ty),* ) $( -> $ret:ty )? {
            $($body:tt)*
        }

          $($rest:tt)*

        ) => {
            pub fn $name( $( $arg : $ty)* ) $( -> $ret)? {
                $($body)*
            }

            module! { $count ; $($rest)* }
        };

        // Empty case, fin.
        ( _ ; ) => ();
    }
}

#[cfg(feature = "host")]
mod query {
    pub use canonical_host::Query;

    #[macro_export]
    macro_rules! module {
        // Opening case

        ( impl $name:ident {
            $($rest:tt)*
        } ) => {
            impl $name {
                module! { ( 0 ) ; $($rest)* }
            }
        };

        // Transactions

        ( $count:tt ; pub fn $name:ident ( & mut $self:ident $(, $arg:ident : $ty:ty)* ) -> $ret:ty {
            $($body:tt)*
        }

          $($rest:tt)*

        ) => {
            pub fn $name( $($arg : $ty),* ) -> canonical_host::Transaction < ( $($ty),* ) , $ret > {
                canonical_host::Transaction::new( ( $($arg),*) )
            }

            module! { ( $count + 1) ; $($rest)* }
        };

        // Queries

        ( $count:tt ; pub fn $name:ident ( & $self:ident $(, $arg:ident : $ty:ty)* ) -> $ret:ty {
            $($body:tt)*
        }

          $($rest:tt)*

        ) => {
            pub fn $name( $($arg : $ty),* ) -> canonical_host::Query < ( $($ty),* ) , $ret > {
                canonical_host::Query::new( ( $($arg),*) )
            }

            module! { ( $count + 1 ) ; $($rest)* }
        };

        // Static methods

        ( pub fn $name:ident ( $($arg:ident : $ty:ty),* ) $( -> $ret:ty )? {
            $($body:tt)*
        }

          $($rest:tt)*

        ) => {
            pub fn $name( $( $arg : $ty)* ) $( -> $ret)? {
                $($body)*
            }

            module! { $count ; $($rest)* }
        };

        // Empty case, fin.
        ( _ ; ) => ();
    }
}
