use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Fields};

#[proc_macro_derive(StructPacker)]
pub fn struct_packer_macro(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    // Build the trait implementation
    let name = input.ident;
    let fields = match input.data {
        syn::Data::Struct(s) => match s.fields {
            Fields::Named(fields) => fields.named,
            Fields::Unnamed(fields) => fields.unnamed,
            Fields::Unit => panic!("Unit structs are not supported"),
        },
        _ => panic!("StructPacker can only be derived for structs"),
    };

    let size_fields = fields.iter().map(|f| {
        let field_name = &f.ident;
        quote! {
            _size += self.#field_name.size();
        }
    });

    let pack_fields = fields.iter().map(|f| {
        let field_name = &f.ident;
        quote! {
            self.#field_name.pack(enc);
        }
    });

    let unpack_fields = fields.iter().map(|f| {
        let field_name = &f.ident;
        quote! {
            dec.unpack(&mut self.#field_name);
        }
    });

    let expanded = quote! {
        // Generate the code to be added
        impl Packer for #name {
            fn size(&self) -> usize {
                let mut _size: usize = 0;
                #(#size_fields)*
                _size
            }

            fn pack(&self, enc: &mut Encoder) -> usize {
                let pos = enc.get_size();
                #(#pack_fields)*
                enc.get_size() - pos
            }

            fn unpack(&mut self, data: &[u8]) -> usize {
                let mut dec = Decoder::new(data);
                #(#unpack_fields)*
                dec.get_pos()
            }
        }
    };

    // Return the generated implementation
    TokenStream::from(expanded)
}

#[proc_macro_derive(EnumPacker)]
pub fn enum_packer_macro(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let gen = match input.data {
        syn::Data::Enum(data_enum) => {
            let size_variants = data_enum.variants.iter().enumerate().map(|(_i, variant)| {
                let variant_ident = &variant.ident;
                match &variant.fields {
                    Fields::Unnamed(fields) => {
                        if fields.unnamed.len() != 1 {
                            panic!("Each variant must have exactly one field implementing the Packer trait.");
                        }
                        quote! {
                            #name::#variant_ident(x) => { _size = 1 + x.size(); }
                        }
                    },
                    _ => panic!("Only unnamed fields are supported"),
                }
            });

            let pack_variants = data_enum.variants.iter().enumerate().map(|(i, variant)| {
                let variant_ident = &variant.ident;
                quote! {
                    #name::#variant_ident(x) => {
                        let mut i: u8 = #i as u8;
                        i.pack(enc);
                        x.pack(enc);
                    }
                }
            });

            let unpack_variants = data_enum.variants.iter().enumerate().map(|(i, variant)| {
                let variant_ident = &variant.ident;
                let variant_type = &variant.fields;
                let variant_default = match variant_type {
                    Fields::Unnamed(fields) => {
                        let ty = &fields.unnamed.first().unwrap().ty;
                        quote! {
                            let mut v: #ty = Default::default();
                            dec.unpack(&mut v);
                            *self = #name::#variant_ident(v);
                        }
                    }
                    _ => panic!("Only unnamed fields are supported"),
                };
                quote! {
                    #i => {
                        #variant_default
                    }
                }
            });

            let default_variant = &data_enum.variants[0];
            let default_variant_ident = &default_variant.ident;

            quote! {
                impl Default for #name {
                    #[doc = r""]
                    #[inline]
                    fn default() -> Self {
                        #name::#default_variant_ident(Default::default())
                    }
                }

                impl ::antelope::serializer::Packer for #name {
                    fn size(&self) -> usize {
                        let mut _size: usize = 0;
                        match self {
                            #( #size_variants ),*
                        }
                        _size
                    }

                    fn pack(&self, enc: &mut ::antelope::chain::Encoder) -> usize {
                        let pos = enc.get_size();
                        match self {
                            #( #pack_variants ),*
                        }
                        enc.get_size() - pos
                    }

                    fn unpack<'a>(&mut self, data: &'a [u8]) -> usize {
                        let mut dec = ::antelope::chain::Decoder::new(data);
                        let mut variant_type_index: u8 = 0;
                        dec.unpack(&mut variant_type_index);
                        let variant_type_index = variant_type_index as usize;
                        match variant_type_index {
                            #( #unpack_variants ),*
                            _ => { panic!("bad variant index!"); }
                        }
                        dec.get_pos()
                    }
                }
            }
        }
        _ => panic!("EnumPacker can only be derived for enums"),
    };

    TokenStream::from(gen)
}
