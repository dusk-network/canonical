// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

//! Derives for `Canon` trait for rust types

#![deny(missing_docs)]

use proc_macro2::{Ident, Literal};
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{
    parse_macro_input, parse_quote, Data, DeriveInput, Fields, GenericParam,
    Generics,
};

const FIELD_NAMES: [&str; 16] = [
    "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o",
    "p",
];

#[allow(unused)] // FIXME
                 // Add a bound `X: Canon` to every type parameter X.
fn add_trait_bounds(mut generics: Generics) -> Generics {
    for param in &mut generics.params {
        if let GenericParam::Type(ref mut type_param) = *param {
            type_param.bounds.push(parse_quote!(canonical::Canon));
        }
    }
    generics
}

#[proc_macro_derive(Canon)]
/// Derive macro that implements the serialization method for a type
pub fn canon_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident.clone();

    let (_, ty_generics, where_clause) = input.generics.split_for_impl();

    let (read, write, length) = match input.data {
        Data::Struct(ref data) => match data.fields {
            Fields::Named(ref fields) => {
                let read = fields.named.iter().map(|f| {
                    let name = &f.ident;
                    let ty = &f.ty;
                    quote_spanned! { f.span() =>
                                     #name : <#ty>::read(source)?,
                    }
                });

                let write = fields.named.iter().map(|f| {
                    let name = &f.ident;
                    quote_spanned! { f.span() =>
                                     Canon::write(&self . #name, sink);
                    }
                });

                let length = fields.named.iter().map(|f| {
                    let name = &f.ident;
                    quote_spanned! { f.span() =>
                                     + Canon::encoded_len(& self.#name)
                    }
                });

                (
                    quote! { Ok(#name { #( #read )* } )},
                    quote! { #( #write )* },
                    quote! { #( #length )* },
                )
            }
            Fields::Unnamed(ref fields) => {
                let read = fields.unnamed.iter().map(|f| {
                    let ty = &f.ty;
                    quote_spanned! { f.span() =>
                                     <#ty>::read(source)?,
                    }
                });

                let write = fields.unnamed.iter().enumerate().map(|(i, f)| {
                    let i = Literal::usize_unsuffixed(i);
                    quote_spanned! { f.span() =>
                                     Canon::write(&self . #i, sink);
                    }
                });

                let length = fields.unnamed.iter().enumerate().map(|(i, f)| {
                    let i = Literal::usize_unsuffixed(i);
                    quote_spanned! { f.span() =>
                                     + Canon::encoded_len(& self.#i)
                    }
                });

                (
                    quote! { Ok(#name ( #( #read )* ) )},
                    quote! { #( #write )* },
                    quote! { #( #length )* },
                )
            }
            Fields::Unit => {
                (quote! { Ok(Self) }, quote! { () }, quote! { + 0 })
            }
        },
        Data::Enum(ref data) => {
            if data.variants.len() > 256 {
                unimplemented!(
                    "More than 256 enum variants is not supported at the time."
                )
            }

            let mut reads = vec![];
            let mut writes = vec![];
            let mut lengths = vec![];

            for (i, v) in data.variants.iter().enumerate() {
                let tag = Literal::u8_suffixed(i as u8);
                let ident = &v.ident;

                match v.fields {
                    Fields::Unit => {
                        reads.push(quote! { #tag => Ok( #name :: #ident ), });
                        writes.push(
                            quote! { #name :: #ident => Canon::write(& #tag, sink), },
                        );
                        lengths.push(quote! { #name :: #ident => 1, });
                    }
                    Fields::Unnamed(ref fields) => {
                        let fields_read = fields.unnamed.iter().map(|f| {
                            let ty = &f.ty;
                            quote_spanned! { f.span() =>
                                             <#ty>::read(source)?
                            }
                        });
                        let fields_bind =
                            fields.unnamed.iter().enumerate().map(|(i, f)| {
                                let ident =
                                    Ident::new(FIELD_NAMES[i], f.span());
                                quote_spanned! { f.span() => #ident }
                            });

                        let fields_assign = fields.unnamed.iter().enumerate().map(|(i, f)| {
                            let ident = Ident::new(FIELD_NAMES[i], f.span());
                            quote_spanned! { f.span() => Canon::write(#ident, sink); }
                        });

                        let fields_lengths = fields.unnamed.iter().enumerate().map(|(i, f)| {
                            let ident = Ident::new(FIELD_NAMES[i], f.span());
                            quote_spanned! { f.span() => + Canon::encoded_len(#ident)}
                        });

                        let fields_bind2 = fields_bind.clone();

                        reads.push(
                            quote! { #tag => Ok( #name :: #ident ( #( #fields_read ),* ) ) , },
                        );

                        writes.push(quote! { #name :: #ident ( #( #fields_bind ),* ) =>
                                              { Canon::write(& #tag, sink); #( #fields_assign )* } });

                        lengths.push(quote! { #name :: #ident ( #( #fields_bind2 ),* ) => {
                            1 #( #fields_lengths )*
                        },
                        });
                    }
                    Fields::Named(ref fields) => {
                        let fields_read = fields.named.iter().map(|f| {
                            let ty = &f.ty;
                            let ident = &f.ident;
                            quote_spanned! { f.span() =>
                                             #ident : <#ty>::read(source)?
                            }
                        });
                        let fields_bind = fields.named.iter().map(|f| {
                            let ident = &f.ident;
                            quote_spanned! { f.span() => #ident }
                        });

                        let fields_assign = fields.named.iter().map(|f| {
                            let ident = &f.ident;
                            quote_spanned! { f.span() => Canon::write(#ident, sink); }
                        });

                        let fields_lengths = fields.named.iter().map(|f| {
                            let ident = &f.ident;
                            quote_spanned! { f.span() => + Canon::encoded_len(#ident) }
                        });

                        let fields_bind2 = fields_bind.clone();

                        reads.push(
                            quote! { #tag => Ok( #name :: #ident { #( #fields_read ),* } ) , },
                        );

                        writes.push(quote! { #name :: #ident { #( #fields_bind ),* } =>
                                              { Canon::write(& #tag, sink); #( #fields_assign )* } });

                        lengths.push(quote! { #name :: #ident { #( #fields_bind2 ),* } => {
                            1 #( #fields_lengths )*
                        },
                        });
                    }
                }
            }

            (
                quote! {
                    let tag = u8::read(source)?;
                    match & tag {
                        #( #reads )*
                        _ => Err(canonical::CanonError::InvalidEncoding)
                    }
                },
                quote! {
                    match self {
                        #( #writes )*
                    }
                },
                quote! {
                    + match & self {
                        #( #lengths )*
                    }
                },
            )
        }
        Data::Union(_) => unimplemented!("Union types are not derivable"),
    };

    let output = quote! {
        impl #ty_generics canonical::Canon for #name #ty_generics #where_clause {
            fn write(&self, sink: &mut canonical::Sink) {
                #write
                ;
            }

            fn read(source: &mut canonical::Source)
                    -> Result<Self, canonical::CanonError> {
                #read
            }

            fn encoded_len(&self) -> usize {
                0 #length
            }
        }
    };

    println!("{:?}", output.to_string());

    proc_macro::TokenStream::from(output)
}
