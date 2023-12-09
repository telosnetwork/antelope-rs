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

    /*
    impl Packer for Transaction {
    fn size(&self) -> usize {
        let mut _size: usize = 0;
        _size += self.expiration.size();
        _size += self.ref_block_num.size();
        _size += self.ref_block_prefix.size();
        _size += self.max_net_usage_words.size();
        _size += self.max_cpu_usage_ms.size();
        _size += self.delay_sec.size();
        _size += self.context_free_actions.size();
        _size += self.actions.size();
        _size += self.extension.size();
        return _size;
    }

    fn pack(&self, enc: &mut Encoder) -> usize {
        let pos = enc.get_size();

        self.expiration.pack(enc);
        self.ref_block_num.pack(enc);
        self.ref_block_prefix.pack(enc);
        self.max_net_usage_words.pack(enc);
        self.max_cpu_usage_ms.pack(enc);
        self.delay_sec.pack(enc);
        self.context_free_actions.pack(enc);
        self.actions.pack(enc);
        self.extension.pack(enc);

        enc.get_size() - pos
    }

    fn unpack(&mut self, data: &[u8]) -> usize {
        let mut dec = Decoder::new(data);
        dec.unpack(&mut self.expiration);
        dec.unpack(&mut self.ref_block_num);
        dec.unpack(&mut self.ref_block_prefix);
        dec.unpack(&mut self.max_net_usage_words);
        dec.unpack(&mut self.max_cpu_usage_ms);
        dec.unpack(&mut self.delay_sec);
        dec.unpack(&mut self.context_free_actions);
        dec.unpack(&mut self.actions);
        dec.unpack(&mut self.extension);
        return dec.get_pos();
    }
}
     */

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