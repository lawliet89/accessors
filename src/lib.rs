//! Support for `#[derive(accesors)]`.  Based on the [example code][] for
//! syn.
//!
//! [example code]: https://github.com/dtolnay/syn

#![feature(proc_macro, proc_macro_lib)]

// I threw this code together in just a few minutes, and it could use a
// good refactoring once I figure out the basic ideas.  Do not use use this
// as an example of good style.

extern crate proc_macro;
#[macro_use]
extern crate quote;
extern crate syn;

use proc_macro::TokenStream;
use std::collections::BTreeMap;

#[proc_macro_derive(getters, attributes(getters))]
pub fn derive_getters(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input(&input.to_string()).unwrap();
    let expanded = expand_getters(ast);
    expanded.to_string().parse().unwrap()
}

fn expand_getters(ast: syn::MacroInput) -> quote::Tokens {
    // println!("Defining getters for: {:#?}", ast);

    let name = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics
        .split_for_impl();

    let getters = match ast.body {
        syn::Body::Struct(syn::VariantData::Struct(ref fields)) => {
            fields.iter().map(|f| {
                let field_name = f.ident.as_ref().unwrap();
                let field_ty = &f.ty;

                quote! {
                    pub fn #field_name(&self) -> &#field_ty {
                        &self.#field_name
                    }
                }
            })
        }
        _ => panic!("#[derive(getters)] can only be used with braced structs"),
    };

    quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            #(#getters)*
        }
    }
}

#[proc_macro_derive(setters, attributes(setters))]
pub fn derive_setters(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input(&input.to_string()).unwrap();
    let expanded = expand_setters(ast);
    // println!("Expanded: {}", expanded.to_string());
    expanded.to_string().parse().unwrap()
}

fn expand_setters(ast: syn::MacroInput) -> quote::Tokens {
    // println!("Defining setters for: {:#?}", ast);

    let mut setters_attrs = ast.attrs.iter().filter(|a| a.name() == "setters");
    let config = config_from(&mut setters_attrs, &["into"]);
    // println!("Config: {:#?}", &config);
    let into_default = syn::Lit::Bool(false);
    let into = match *config.get("into").unwrap_or(&into_default) {
        syn::Lit::Bool(b) => b,
        ref val => panic!("'into' must be a boolean value, not {:?}", val),
    };

    let name = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics
        .split_for_impl();

    let setters = match ast.body {
        syn::Body::Struct(syn::VariantData::Struct(ref fields)) => {
            fields.iter().map(|f| {
                let field_name = f.ident.as_ref().unwrap();
                let field_ty = &f.ty;

                let set_fn_name: syn::Ident = format!("set_{}", field_name).into();
                if into {
                    quote! {
                        pub fn #set_fn_name<T>(&mut self, value: T)
                            where T: Into<#field_ty>
                        {
                            self.#field_name = value.into();
                        }
                    }
                } else {
                    quote! {
                        pub fn #set_fn_name(&mut self, value: #field_ty) {
                            self.#field_name = value;
                        }
                    }
                }
            })
        }
        _ => panic!("#[derive(setters)] can only be used with braced structs"),
    };

    quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            #(#setters)*
        }
    }
}

fn config_from(attrs: &mut Iterator<Item = &syn::Attribute>,
               keys: &[&str])
               -> BTreeMap<String, syn::Lit> {
    let mut result = BTreeMap::new();
    while let Some(attr) = attrs.next() {
        if let syn::MetaItem::List(_, ref args) = attr.value {
            for arg in args {
                if let syn::NestedMetaItem::MetaItem(ref meta_item) = *arg {
                    let name = meta_item.name();
                    if !keys.contains(&name) {
                        panic!("'{}' in {:?} is not a known attribute", name, attr);
                    }
                    match *meta_item {
                        syn::MetaItem::Word(_) => {
                            result.insert(name.to_owned(), syn::Lit::Bool(true));
                        }
                        syn::MetaItem::NameValue(_, ref value) => {
                            result.insert(name.to_owned(), value.to_owned());
                        }
                        _ => panic!("can't parse '{:?}'", &arg),
                    }
                } else {
                    panic!("'{:?}' in {:?} is not a known attribute", arg, attr);
                }
            }
        } else {
            panic!("{:?} must be a key-value attribute", attr);
        }
    }
    result
}
