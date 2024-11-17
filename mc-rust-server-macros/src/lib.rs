extern crate proc_macro;

use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(McProtocol, attributes(protocol_read, protocol_write))]
pub fn proc_macro_protocol(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    mc_protocol::proc_macro_protocol(parse_macro_input!(input as DeriveInput)).into()
}
mod mc_protocol;
