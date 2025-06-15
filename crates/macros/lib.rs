use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Fields, Error};

#[proc_macro_derive(StructPacker)]
pub fn struct_packer_macro(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let fields = match &input.data {
        syn::Data::Struct(s) => match &s.fields {
            Fields::Named(f)   => &f.named,
            Fields::Unnamed(f) => &f.unnamed,
            Fields::Unit       => {
                return Error::new_spanned(
                    name,
                    "Unit structs are not supported",
                )
                .to_compile_error()
                .into();
            }
        },
        _ => {
            return Error::new_spanned(
                name,
                "StructPacker can only be derived for structs",
            )
            .to_compile_error()
            .into();
        }
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
            dec.unpack(&mut self.#field_name)?;
        }
    });

    let expanded = quote! {
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

            fn unpack(&mut self, data: &[u8]) -> Result<usize, PackerError> {
                let mut dec = Decoder::new(data);
                #(#unpack_fields)*
                Ok(dec.get_pos())
            }
        }
    };

    TokenStream::from(expanded)
}


#[proc_macro_derive(EnumPacker)]
pub fn enum_packer_macro(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let result = (|| -> Result<proc_macro2::TokenStream, Error> {
        // Validate that we are deriving for an enum
        let data_enum = match &input.data {
            syn::Data::Enum(e) => e,
            _ => {
                return Err(Error::new_spanned(
                    &input.ident,
                    "EnumPacker can only be derived for enums",
                ));
            }
        };

        // Per-variant validation
        for variant in &data_enum.variants {
            match &variant.fields {
                Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {}
                Fields::Unnamed(_) => {
                    return Err(Error::new_spanned(
                        variant,
                        "Each variant must have exactly one field implementing the Packer trait",
                    ));
                }
                _ => {
                    return Err(Error::new_spanned(
                        variant,
                        "Only unnamed tuple-style variants are supported",
                    ));
                }
            }
        }

        let name = &input.ident;

        // Generate code now that we know the input is valid.
        let size_variants = data_enum.variants.iter().map(|variant| {
            let ident = &variant.ident;
            quote! {
                #name::#ident(x) => { _size = 1 + x.size(); }
            }
        });

        let pack_variants = data_enum.variants.iter().enumerate().map(|(i, variant)| {
            let ident = &variant.ident;
            quote! {
                #name::#ident(x) => {
                    let mut i: u8 = #i as u8;
                    i.pack(enc);
                    x.pack(enc);
                }
            }
        });

        let unpack_variants = data_enum
            .variants
            .iter()
            .enumerate()
            .map(|(i, variant)| {
                let ident = &variant.ident;

                let ty = match &variant.fields {
                    Fields::Unnamed(fields) => &fields.unnamed[0].ty,
                    _ => unreachable!(), // already rejected above
                };

                quote! {
                    #i => {
                        let mut v: #ty = Default::default();
                        dec.unpack(&mut v)?;
                        *self = #name::#ident(v);
                    }
                }
            });

        let default_variant_ident = &data_enum.variants[0].ident;

        Ok(quote! {
            impl Default for #name {
                #[inline]
                fn default() -> Self {
                    #name::#default_variant_ident(Default::default())
                }
            }

            impl ::antelope::serializer::Packer for #name {
                fn size(&self) -> usize {
                    let mut _size: usize = 0;
                    match self {
                        #(#size_variants),*
                    }
                    _size
                }

                fn pack(&self, enc: &mut ::antelope::serializer::Encoder) -> usize {
                    let pos = enc.get_size();
                    match self {
                        #(#pack_variants),*
                    }
                    enc.get_size() - pos
                }

                fn unpack<'a>(
                    &mut self,
                    data: &'a [u8],
                ) -> Result<usize, ::antelope::serializer::packer::PackerError> {
                    let mut dec = ::antelope::serializer::Decoder::new(data);
                    let mut variant_type_index: u8 = 0;
                    dec.unpack(&mut variant_type_index)?;
                    let variant_type_index = variant_type_index as usize;
                    match variant_type_index {
                        #(#unpack_variants),*
                        _ => {
                            return Err(::antelope::packer_error!(
                                "bad variant index: {}",
                                variant_type_index
                            ))
                        }
                    }
                    Ok(dec.get_pos())
                }
            }
        })
    })();

    // Convert Result to TokenStream, producing a nice compile-time error on failure.
    match result {
        Ok(tokens) => tokens.into(),
        Err(e) => e.to_compile_error().into(),
    }
}
