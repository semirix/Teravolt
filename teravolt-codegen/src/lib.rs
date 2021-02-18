use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(TeravoltPacket)]
pub fn packet(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let ident = input.ident.clone();

    let expanded = quote! {
        use crate::Packet;
        use std::any::TypeId;
        use std::convert::{TryFrom, Into};

        impl TeravoltPacket for #ident {
            fn as_packet(&self) -> Packet {
                Packet::new::<#ident>(self.clone())
            }

            fn id() -> TypeId {
                TypeId::of::<#ident>()
            }
        }
    };

    let tokens = TokenStream::from(expanded);

    //panic!(tokens.to_string());

    tokens
}
