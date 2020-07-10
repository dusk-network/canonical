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

// Add a bound `T: HeapSize` to every type parameter T.
fn add_trait_bounds(mut generics: Generics) -> Generics {
    for param in &mut generics.params {
        if let GenericParam::Type(ref mut type_param) = *param {
            type_param.bounds.push(parse_quote!(canon::Canon));
        }
    }
    generics
}

#[proc_macro_derive(Canon)]
pub fn canon_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;

    let generics = add_trait_bounds(input.generics);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let (read, write) = match input.data {
        Data::Struct(ref data) => match data.fields {
            Fields::Named(ref fields) => {
                let write = fields.named.iter().map(|f| {
                    let name = &f.ident;
                    quote_spanned! { f.span() =>
                                     self . #name . write(sink);
                    }
                });

                let read = fields.named.iter().map(|f| {
                    let name = &f.ident;
                    let ty = &f.ty;
                    quote_spanned! { f.span() =>
                        #name : <#ty>::read(source)?,
                    }
                });
                (
                    quote! { Ok(#name { #( #read )* } )},
                    quote! { #( #write )* },
                )
            }
            Fields::Unnamed(ref fields) => {
                let write = fields.unnamed.iter().enumerate().map(|(i, f)| {
                    let i = Literal::usize_unsuffixed(i);
                    quote_spanned! { f.span() =>
                                     self . #i . write(sink);
                    }
                });

                let read = fields.unnamed.iter().map(|f| {
                    let ty = &f.ty;
                    quote_spanned! { f.span() =>
                         <#ty>::read(source)?,
                    }
                });

                (
                    quote! { Ok(#name ( #( #read )* ) )},
                    quote! { #( #write )* },
                )
            }
            Fields::Unit => (quote! { Ok(Self) }, quote! { () }),
        },
        Data::Enum(ref data) => {
            if data.variants.len() > 256 {
                unimplemented!(
                    "More than 256 enum variants is not supported at the time."
                )
            }

            let mut reads = vec![];
            let mut writes = vec![];

            for (i, v) in data.variants.iter().enumerate() {
                let tag = Literal::u8_suffixed(i as u8);
                let ident = &v.ident;

                match v.fields {
                    Fields::Unit => {
                        reads.push(quote! { #tag => Ok( #name :: #ident ), });
                        writes.push(
                            quote! { #name :: #ident => #tag . write(sink), },
                        )
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
                            quote_spanned! { f.span() => #ident . write(sink); }
                        });

                        reads.push(
                            quote! { #tag => Ok( #name :: #ident ( #( #fields_read ),* ) ) , }
                        );

                        writes.push(
                            quote! { #name :: #ident ( #( #fields_bind ),* ) =>
                            { #tag . write(sink); #( #fields_assign )* } },
                        );
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
                            quote_spanned! { f.span() => #ident . write(sink); }
                        });

                        reads.push(
                            quote! { #tag => Ok( #name :: #ident { #( #fields_read ),* } ) , }
                        );

                        writes.push(
                            quote! { #name :: #ident { #( #fields_bind ),* } =>
                            { #tag . write(sink); #( #fields_assign )* } },
                        );
                    }
                }
            }

            (
                quote! {
                    let tag = u8::read(source)?;
                    match tag {
                        #( #reads )*
                        _ => Err(InvalidEncoding)
                    }
                },
                quote! {
                    match self {
                        #( #writes )*
                    }
                },
            )
        }
        Data::Union(_) => unimplemented!("Union types are not derivable"),
    };

    let output = quote! {
        impl #impl_generics Canon for #name #ty_generics #where_clause {
            fn write(&self, sink: &mut impl canon::Sink) {
                #write
            }

            fn read(source: &mut impl canon::Source)
                    -> Result<Self, canon::InvalidEncoding> {
                #read
            }
        }
    };

    proc_macro::TokenStream::from(output)
}
