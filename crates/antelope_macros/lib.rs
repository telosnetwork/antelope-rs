use proc_macro::{TokenStream};
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Fields};

#[proc_macro_derive(StructPacker)]
pub fn your_macro_name_derive(input: TokenStream) -> TokenStream {
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