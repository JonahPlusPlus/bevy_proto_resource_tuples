use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{Ident, Index};

fn get_idents(fmt_string: fn(usize) -> String, count: usize) -> Vec<Ident> {
    (0..count)
        .map(|i| Ident::new(&fmt_string(i), Span::call_site()))
        .collect::<Vec<Ident>>()
}

#[proc_macro]
pub fn impl_resource_apis(_input: TokenStream) -> TokenStream {
    let mut tokens = TokenStream::new();
    let max_types = 16;
    let types = get_idents(|i| format!("P{i}"), max_types);

    for i in 1..=max_types {
        let ty = &types[0..i];
        let indices = (0..i).map(Index::from).collect::<Vec<_>>();
        tokens.extend(TokenStream::from(quote! {
            impl<#(#ty: Resource + FromWorld,)*> InitResources for (#(#ty,)*) {
                type IDS = [ComponentId; #i];

                fn init_resources(world: &mut World) -> Self::IDS {
                    [#(world.init_resource::<#ty>(),)*]
                }
            }

            impl<#(#ty: Resource,)*> InsertResources for (#(#ty,)*) {
                fn insert_resources(self, world: &mut World) {
                    #(world.insert_resource(self.#indices);)*
                }
            }
        }));
    }

    tokens
}
