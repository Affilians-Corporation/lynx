extern crate proc_macro;

use proc_macro::TokenStream;
use quote::*;
use std::sync::atomic::{AtomicU32, Ordering};
use syn::{parse_macro_input, Data, DeriveInput};


static NEXT_ID: AtomicU32 = AtomicU32::new(1);

#[proc_macro_derive(Component)]
pub fn component_derive(inp: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(inp as DeriveInput);
    let struct_name = &input.ident;
    let generics = &input.generics;
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);

    let (types, output_types, dismember_body) = match &input.data {
        Data::Struct(data) => match &data.fields {
            syn::Fields::Named(f) => {
                let field_types: Vec<_> = f.named.iter().map(|field| &field.ty).collect();
                let output = quote! {
                    (#(<#field_types as Component>::DismemberedOutput),*)
                };
                let body = f.named.iter().map(|f| {
                    let name = &f.ident;
                    quote! { self.#name.dismember() }
                });
                (
                    field_types,
                    output,
                    quote! { (#(#body),*) },
                )
            }
            syn::Fields::Unnamed(f) => {
                let field_types: Vec<_> = f.unnamed.iter().map(|f| &f.ty).collect();
                let output = quote! {
                    (#(<#field_types as Component>::DismemberedOutput),*)
                };
                let body = (0..field_types.len()).map(|i| {
                    let idx = syn::Index::from(i);
                    quote! { self.#idx.dismember() }
                });
                (
                    field_types,
                    output,
                    quote! { (#(#body),*) },
                )
            }
            syn::Fields::Unit => (
                Vec::new(),
                quote! { () },
                quote! { () },
            ),
        },
        _ => unimplemented!(),
    };

    let derive = quote! {
        use lynx_traits::*;

        impl #generics Component for #struct_name #generics {
            type DismemberedOutput = #output_types;
            const COUNT: usize = 0 #(+ <#types as Component>::COUNT)*;

            fn dismember(self) -> Self::DismemberedOutput {
                #dismember_body
            }

            fn dismembered_type_count() -> u32 {
                Self::COUNT as u32
            }
            fn sizes() -> &'static [usize] {
                static SIZES: std::sync::OnceLock<&'static [usize]> = std::sync::OnceLock::new();
                SIZES.get_or_init(|| {
                    let computed: Vec<usize> = match std::mem::size_of::<Self>() {
                        0 => vec![0 as usize],
                        _ => {
                            let slices: Vec<&'static [usize]> = vec![
                                #( <#types as Component>::sizes() ),*
                            ];

                            slices.into_iter().flatten().copied().collect()
                        }
                    };
                    Box::leak(computed.into_boxed_slice())
                })
            }
            fn id() -> u32{
                #id
            }
        }
    };

    TokenStream::from(derive)
}

#[proc_macro_derive(Signature)]
pub fn derive_signature(input: TokenStream) -> TokenStream {
    let inp = parse_macro_input!(input as DeriveInput);
    let ident = &inp.ident;
    let generics = &inp.generics;
    let (types, fields) = match &inp.data {
        Data::Struct(data) => {
            match &data.fields {
                syn::Fields::Named(f) => f.named.iter().map(|f| (f.clone().ty, f.clone().ident.unwrap())).collect::<(Vec<_>, Vec<_>)>(),
                syn::Fields::Unnamed(f) => f.unnamed.iter().map(|f| (f.clone().ty, f.clone().ident.unwrap())).collect(),
                _ => panic!("Unit Structs may not be inserted")
            }
        }, _ => unimplemented!()
    };

    let insert = quote! {
        #(archetype.insert_component::<#types>(&self.#fields).unwrap();)*
        archetype.set_entity_count(archetype.get_entity_count() + 1);
        if archetype.column_must_resize() {
            let entity_count = archetype.get_entity_count().clone();
            #(
                let sizes = <#types as Component>::sizes();
                for (index, value) in sizes.iter().enumerate() {
                    archetype.get_mut::<#types>(index).unwrap()
                                                      .resize_bytes(entity_count as usize * value,
                                                                    (entity_count as usize * value) * 2);
                }
            )*
        }
    };

    let create = quote! {
        #(archetype.initialize_column::<#types>();)*
    };

    let output = quote! {
        impl #generics Signature for #ident #generics {
            #[inline(always)]
            fn insert_components(&self, archetype: &mut impl Archetype) {
                #insert
            }

            fn create(archetype: &mut impl Archetype) {
                if archetype.get_entity_count() == 0 {
                    #create
                }
            }

            fn gen_ids() -> &'static [u32] {
                static IDS: std::sync::OnceLock<&'static [u32]> = std::sync::OnceLock::new();
                IDS.get_or_init(|| {
                    let mut ids = Vec::new();
                    #(
                        ids.push(<#types as Component>::id());
                        for i in 0..<#types as Component>::COUNT - 1 {
                            ids.push(0);
                        }
                    )*
                    Box::leak(ids.into_boxed_slice())
                })
            }

            fn bulk(&self, archetype: &mut impl Archetype, times: usize) {
                todo!()
            }
        }
    };

    TokenStream::from(output)
}