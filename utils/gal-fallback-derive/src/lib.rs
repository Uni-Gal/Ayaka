use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::quote;
use syn::{
    parse::{Parse, Parser},
    parse_macro_input, parse_str, Data, DeriveInput, Expr, FieldValue, Fields, Type,
};

#[proc_macro_derive(FallbackSpec)]
pub fn derive_fallback_spec(input: TokenStream) -> TokenStream {
    let struct_input = parse_macro_input!(input as DeriveInput);
    let struct_name = struct_input.ident;
    let vis = struct_input.vis;
    let (fallback_data_declare, data_idents, base_data_idents, some_exact, none_exact, construct) =
        match struct_input.data {
            Data::Struct(data) => match data.fields {
                Fields::Named(fields) => {
                    let fields = fields.named.into_iter().collect::<Vec<_>>();
                    let fallback_data_declare = fields
                        .iter()
                        .map(|field| {
                            let mut field = field.clone();
                            let ty = field.ty.clone();
                            field.ty = Type::parse
                                .parse2(quote! {::gal_fallback::Fallback<#ty>})
                                .unwrap();
                            field
                        })
                        .collect::<Vec<_>>();
                    let data_idents = fields
                        .iter()
                        .map(|field| field.ident.clone().unwrap())
                        .collect::<Vec<_>>();
                    let base_data_idents = fields
                        .iter()
                        .map(|field| {
                            parse_str::<Ident>(&format!("base_{}", field.ident.clone().unwrap()))
                                .expect("Parse base idents failed")
                        })
                        .collect::<Vec<_>>();
                    let some_exact = fields
                        .iter()
                        .map(|field| {
                            parse_str::<Expr>(&format!(
                                "Some(data.{})",
                                field.ident.clone().unwrap()
                            ))
                            .expect("Parse some exact failed")
                        })
                        .collect::<Vec<_>>();
                    let none_exact =
                        std::iter::repeat(parse_str::<Expr>("None").expect("Parse None failed"))
                            .take(fields.len())
                            .collect::<Vec<_>>();
                    let construct = fields
                        .iter()
                        .map(|field| {
                            parse_str::<FieldValue>(&format!(
                                "{0}: ::gal_fallback::Fallback::new({0}, base_{0})",
                                field.ident.clone().unwrap()
                            ))
                            .expect("Parse field value failed")
                        })
                        .collect::<Vec<_>>();
                    (
                        fallback_data_declare,
                        data_idents,
                        base_data_idents,
                        some_exact,
                        none_exact,
                        construct,
                    )
                }
                _ => unimplemented!(),
            },
            _ => unimplemented!(),
        };
    let fallback_struct_name = parse_str::<Ident>(&format!("Fallback{}", struct_name))
        .expect("Parse fallback name failed");
    let output = quote! {
        #[doc(hidden)]
        #vis struct #fallback_struct_name {
            #(#fallback_data_declare ,)*
        }

        impl FallbackSpec for #struct_name {
            type SpecType = #fallback_struct_name;
        }

        impl From<::gal_fallback::Fallback<#struct_name>> for #fallback_struct_name {
            fn from(data: ::gal_fallback::Fallback<#struct_name>) -> Self {
                let (data, base_data) = data.unzip();
                let (#(#data_idents ,)*) = match data {
                    Some(data) => (#(#some_exact ,)*),
                    None => (#(#none_exact ,)*),
                };
                let (#(#base_data_idents ,)*) = match base_data {
                    Some(data) => (#(#some_exact ,)*),
                    None => (#(#none_exact ,)*),
                };
                Self {
                    #(#construct ,)*
                }
            }
        }
    };
    TokenStream::from(output)
}
