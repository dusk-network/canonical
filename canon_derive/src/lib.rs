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
    parse_macro_input, parse_quote, punctuated::Punctuated, Data, DeriveInput,
    Fields, GenericParam, Generics, Path, PathArguments, PathSegment,
    TraitBound, TraitBoundModifier, Type, TypeParam, TypeParamBound,
    WherePredicate,
};

const FIELD_NAMES: [&str; 16] = [
    "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o",
    "p",
];

// Add a bound `T: Canon` to every type parameter T.
fn add_trait_bounds(mut generics: Generics, bound: Ident) -> Generics {
    for param in &mut generics.params {
        if let GenericParam::Type(ref mut type_param) = *param {
            type_param
                .bounds
                .push(parse_quote!(canonical::Canon< #bound >));
        }
    }
    generics
}

// Add a __S : canocical::Store to the bounds.
fn add_store_bound(mut generics: Generics) -> Generics {
    let mut segments = Punctuated::new();
    segments.push(PathSegment {
        ident: Ident::new("canonical", generics.span()),
        arguments: PathArguments::None,
    });
    segments.push(PathSegment {
        ident: Ident::new("Store", generics.span()),
        arguments: PathArguments::None,
    });

    let store_trait = TypeParamBound::Trait(TraitBound {
        modifier: TraitBoundModifier::None,
        lifetimes: None,
        path: Path {
            leading_colon: None,
            segments,
        },
        paren_token: None,
    });
    let mut bounds = Punctuated::new();

    // Insert at beginning, to not trip up const generics.
    //
    // Note that lifetimes should technically go at the beginning,
    // but the derive macro does not support types with lifetimes anyway.

    bounds.insert(0, store_trait);

    let param = TypeParam {
        attrs: vec![],
        ident: Ident::new("__S", generics.span()),
        colon_token: None,
        bounds,
        eq_token: None,
        default: None,
    };

    generics.params.insert(0, GenericParam::Type(param));
    generics
}

// Finds the type parameter that is bound to Store
fn mentions_store(generics: &Generics) -> Option<Ident> {
    for param in &generics.params {
        if let GenericParam::Type(type_param) = param {
            for bound in &type_param.bounds {
                if let TypeParamBound::Trait(trait_bound) = bound {
                    if let Some(PathSegment { ident, .. }) =
                        trait_bound.path.segments.last()
                    {
                        if ident == "Store" {
                            return Some(type_param.ident.clone());
                        }
                    }
                }
            }
        }
    }

    if let Some(where_clause) = &generics.where_clause {
        for pred in &where_clause.predicates {
            if let WherePredicate::Type(bound) = pred {
                for b in &bound.bounds {
                    if let TypeParamBound::Trait(traitbound) = b {
                        if let Some(PathSegment { ident, .. }) =
                            traitbound.path.segments.last()
                        {
                            if ident == "Store" {
                                if let Type::Path(type_path) = &bound.bounded_ty
                                {
                                    if type_path.path.segments.len() == 1 {
                                        // Single type before, like [S]: Store

                                        let segment = type_path
                                            .path
                                            .segments
                                            .first()
                                            .expect("len > 0");

                                        return Some(segment.ident.clone());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

#[proc_macro_derive(Canon)]
/// Derive macro that implements the serialization method for a type
pub fn canon_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident.clone();

    let store_bound_name = mentions_store(&input.generics);

    let generics = add_trait_bounds(
        input.generics.clone(),
        store_bound_name
            .clone()
            .unwrap_or_else(|| Ident::new("__S", input.span())),
    );

    let (_, ty_generics, where_clause) = generics.split_for_impl();

    // if the bounds already contain a Store bound, don't add it again.

    let impl_generics = if store_bound_name.is_none() {
        add_store_bound(generics.clone())
    } else {
        generics.clone()
    };

    let __s =
        store_bound_name.unwrap_or_else(|| Ident::new("__S", input.span()));

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
                                     Canon::< #__s >::write(&self . #name, sink) ?;
                    }
                });

                let length = fields.named.iter().map(|f| {
                    let name = &f.ident;
                    quote_spanned! { f.span() =>
                                     + Canon::<#__s>::encoded_len(& self.#name)
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
                                     Canon::<#__s>::write(&self . #i, sink) ?;
                    }
                });

                let length = fields.unnamed.iter().enumerate().map(|(i, f)| {
                    let i = Literal::usize_unsuffixed(i);
                    quote_spanned! { f.span() =>
                                     + Canon::<#__s>::encoded_len(& self.#i)
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
                            quote! { #name :: #ident => Canon::<#__s>::write(& #tag, sink) ?, },
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
                            quote_spanned! { f.span() => Canon::<#__s>::write(#ident, sink)?; }
                        });

                        let fields_lengths = fields.unnamed.iter().enumerate().map(|(i, f)| {
                            let ident = Ident::new(FIELD_NAMES[i], f.span());
                            quote_spanned! { f.span() => + Canon::<#__s>::encoded_len(#ident)}
                        });

                        let fields_bind2 = fields_bind.clone();

                        reads.push(
                            quote! { #tag => Ok( #name :: #ident ( #( #fields_read ),* ) ) , },
                        );

                        writes.push(quote! { #name :: #ident ( #( #fields_bind ),* ) =>
                        { Canon::<#__s>::write(& #tag, sink)?; #( #fields_assign )* } });

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
                            quote_spanned! { f.span() => Canon::<#__s>::write(#ident, sink)?; }
                        });

                        let fields_lengths = fields.named.iter().map(|f| {
                            let ident = &f.ident;
                            quote_spanned! { f.span() => + Canon::<#__s>::encoded_len(#ident) }
                        });

                        let fields_bind2 = fields_bind.clone();

                        reads.push(
                            quote! { #tag => Ok( #name :: #ident { #( #fields_read ),* } ) , },
                        );

                        writes.push(quote! { #name :: #ident { #( #fields_bind ),* } =>
                        { Canon::<#__s>::write(& #tag, sink)?; #( #fields_assign )* } });

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
                        _ => Err(canonical::InvalidEncoding.into())
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
        impl #impl_generics canonical::Canon<#__s> for #name #ty_generics #where_clause {
            fn write(&self, sink: &mut impl canonical::Sink < #__s >)
              -> Result<(), #__s::Error> {
                #write
                ;
                Ok(())
            }

            fn read(source: &mut impl canonical::Source < #__s >)
                    -> Result<Self, #__s::Error> {
                #read
            }

            fn encoded_len(&self) -> usize {
                0 #length
            }
        }
    };

    proc_macro::TokenStream::from(output)
}
