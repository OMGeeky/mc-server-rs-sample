use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::parse::Parse;
use syn::{Data, DeriveInput, Lit, Token};

pub fn proc_macro_protocol(input: DeriveInput) -> TokenStream {
    match input.data {
        Data::Enum(e) => {
            let variants = e.variants.into_iter();
            let mut variant_data = vec![];
            for v in variants {
                let attr = v.attrs.into_iter().find_map(|a| {
                    if a.path().is_ident("protocol_read") {
                        Some(a.parse_args::<ProtocolAttribute>().ok()?)
                    } else {
                        None
                    }
                });
                if let Some(attr) = attr {
                    variant_data.push(ProtocolVariant {
                        attr,
                        ident: v.ident,
                    })
                }
            }

            let variant_match = variant_data.into_iter().map(|v| {
                let state = v.attr.state;
                let id = v.attr.packet_id;
                let name = v.ident;
                quote! {(#state, #id)=>Self::#name (read_protocol_data(stream).await?)}
            });
            let variants_match = quote! {#(#variant_match),*};

            let enum_name = input.ident;
            quote! {
                #[automatically_derived]
                impl #enum_name {
                    pub async fn read_protocol_data<T:AsyncRead + AsyncWrite + Unpin>(
                        protocol_id: VarInt,
                        connection_state: ConnectionState,
                        stream: &mut RWStreamWithLimit<'_, T>,
                    )-> Result<IncomingPackageContent, String> {
                        Ok(match (connection_state, protocol_id.to_rs()){
                            #variants_match,
                            (other_state, other_id) => {
                                return Err(format!("Unrecognized protocol+state combination: {:?}/{}", other_state, other_id));
                            }
                        })
                    }
                }
            }
        }
        _ => {
            panic!("This macro is only supported for Enums")
        }
    }
}

#[derive(Debug)]
struct ProtocolVariant {
    attr: ProtocolAttribute,
    ident: Ident,
}
#[derive(Debug)]
struct ProtocolAttribute {
    state: syn::ExprPath,
    packet_id: Lit,
}
impl Parse for ProtocolAttribute {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let state = input.parse()?;
        input.parse::<Token![,]>()?;
        let packet_id = input.parse()?;

        Ok(ProtocolAttribute { state, packet_id })
    }
}
