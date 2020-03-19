use crate::args;
use crate::utils::{check_reserved_name, get_crate_name};
use inflector::Inflector;
use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Error, Result};

pub fn generate(enum_args: &args::Enum, input: &DeriveInput) -> Result<TokenStream> {
    let crate_name = get_crate_name(enum_args.internal);
    let attrs = &input.attrs;
    let vis = &input.vis;
    let ident = &input.ident;
    let e = match &input.data {
        Data::Enum(e) => e,
        _ => return Err(Error::new_spanned(input, "It should be a enum")),
    };

    let gql_typename = enum_args.name.clone().unwrap_or_else(|| ident.to_string());
    check_reserved_name(&gql_typename, enum_args.internal)?;

    let desc = enum_args
        .desc
        .as_ref()
        .map(|s| quote! { Some(#s) })
        .unwrap_or_else(|| quote! {None});

    let mut enum_items = Vec::new();
    let mut items = Vec::new();
    let mut schema_enum_items = Vec::new();

    for variant in &e.variants {
        if !variant.fields.is_empty() {
            return Err(Error::new_spanned(
                &variant,
                format!(
                    "Invalid enum variant {}.\nGraphQL enums may only contain unit variants.",
                    variant.ident
                ),
            ));
        }

        let item_ident = &variant.ident;
        let mut item_args = args::EnumItem::parse(&variant.attrs)?;
        let gql_item_name = item_args
            .name
            .take()
            .unwrap_or_else(|| variant.ident.to_string().to_screaming_snake_case());
        let item_deprecation = item_args
            .deprecation
            .as_ref()
            .map(|s| quote! { Some(#s) })
            .unwrap_or_else(|| quote! {None});
        let item_desc = item_args
            .desc
            .as_ref()
            .map(|s| quote! { Some(#s) })
            .unwrap_or_else(|| quote! {None});
        enum_items.push(&variant.ident);
        items.push(quote! {
            #crate_name::EnumItem {
                name: #gql_item_name,
                value: #ident::#item_ident,
            }
        });
        schema_enum_items.push(quote! {
            enum_items.insert(#gql_item_name, #crate_name::registry::EnumValue {
                name: #gql_item_name,
                description: #item_desc,
                deprecation: #item_deprecation,
            });
        });
    }

    let expanded = quote! {
        #(#attrs)*
        #[derive(Copy, Clone, Eq, PartialEq, Debug)]
        #vis enum #ident {
            #(#enum_items),*
        }

        impl #crate_name::EnumType for #ident {
            fn items() -> &'static [#crate_name::EnumItem<#ident>] {
                &[#(#items),*]
            }
        }

        impl #crate_name::Type for #ident {
            fn type_name() -> std::borrow::Cow<'static, str> {
                std::borrow::Cow::Borrowed(#gql_typename)
            }

            fn create_type_info(registry: &mut #crate_name::registry::Registry) -> String {
                registry.create_type::<Self, _>(|registry| {
                    #crate_name::registry::Type::Enum {
                        name: #gql_typename.to_string(),
                        description: #desc,
                        enum_values: {
                            let mut enum_items = std::collections::HashMap::new();
                            #(#schema_enum_items)*
                            enum_items
                        },
                    }
                })
            }
        }

        impl #crate_name::InputValueType for #ident {
            fn parse(value: &#crate_name::Value) -> Option<Self> {
                #crate_name::EnumType::parse_enum(value)
            }
        }

        #[#crate_name::async_trait::async_trait]
        impl #crate_name::OutputValueType for #ident {
            async fn resolve(value: &Self, _: &#crate_name::ContextSelectionSet<'_>) -> #crate_name::Result<serde_json::Value> {
                #crate_name::EnumType::resolve_enum(value)
            }
        }
    };
    Ok(expanded.into())
}
