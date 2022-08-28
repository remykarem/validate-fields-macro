#![feature(proc_macro_diagnostic)]

extern crate proc_macro;
#[macro_use]
extern crate quote;
extern crate serde_yaml;
extern crate syn;

use std::collections::{BTreeMap, HashSet};
use std::fs::File;

use proc_macro::TokenStream;
use syn::spanned::Spanned;
use syn::{Fields, Ident, Item, ItemStruct};

const PATH: &str = "application.yml";
type Config = BTreeMap<String, BTreeMap<String, String>>;

#[proc_macro_attribute]
pub fn validate_fields(_metadata: TokenStream, input: TokenStream) -> TokenStream {
    // Parse the `TokenStream` into a syntax tree, specifically an `Item`. An `Item` is a
    // syntax item that can appear at the module level i.e. a function definition, a struct
    // or enum definition, etc.
    let item = syn::parse(input).expect("failed to parse input into `syn::Item`");

    // Match on the parsed item and respond accordingly.
    match item {
        // If the attribute was applied to a struct, we're going to do some more work
        // to figure out if there's a field named "bees". It's important to take a reference
        // to `struct_item`, otherwise you partially move `item`.
        Item::Struct(ref struct_item) => {
            light_it_up(struct_item);
        }

        // If the attribute was applied to any other kind of item, we want to generate a
        // compiler error.
        _ => {
            item.span().unwrap().warning("uhmm");
        }
    }

    // Use `quote` to convert the syntax tree back into tokens so we can return them. Note
    // that the tokens we're returning at this point are still just the input, we've simply
    // converted it between a few different forms.
    let output = quote! { #item };

    output.into()
}

/// Generate fun compiler errors.
fn light_it_up(struct_: &ItemStruct) {
    // Open the application.yml file
    let file = if let Ok(file) = File::open(PATH) {
        file
    } else {
        struct_
            .ident
            .span()
            .unstable()
            .error(format!("No {} found", PATH))
            .emit();
        return;
    };

    // Parse the application.yml file
    let mut config: Config = if let Ok(config) = serde_yaml::from_reader(file) {
        config
    } else {
        struct_
            .ident
            .span()
            .unstable()
            .error(format!("Error parsing {}", PATH))
            .emit();
        return;
    };

    // Check for key in application.yml based on name of the identifier of the struct
    let key = struct_.ident.to_string().to_lowercase();
    let source_keys: HashSet<String> = if let Some(k) = config.remove(&key) {
        k.into_keys().collect()
    } else {
        struct_
            .ident
            .span()
            .unstable()
            .error(format!("Cannot find key `{}`", key))
            .emit();
        return;
    };

    // Transform
    let target_keys: HashSet<&Ident> = if let Fields::Named(ref fields) = struct_.fields {
        fields
            .named
            .iter()
            .map(|named| named.ident.as_ref().unwrap())
            .collect()
    } else {
        struct_
            .ident
            .span()
            .unstable()
            .error("Huhh what you trying to do")
            .emit();
        return;
    };

    // Underline properties that do not exist in the application.yml
    target_keys
        .iter()
        .filter(|&ident| !source_keys.contains(&ident.to_string()))
        .for_each(|&ident| {
            ident
                .span()
                .unstable()
                .error("Property does not exist")
                .emit()
        });

    // Complain if properties that exist in the application.yml
    // do not exist in the struct definition
    let target_keys: HashSet<String> = target_keys.into_iter().map(|k| k.to_string()).collect();
    let mut missing_properties: Vec<String> = source_keys
        .into_iter()
        .filter(|source_key| !target_keys.contains(source_key))
        .collect();
    missing_properties.sort_unstable();
    let msg = format!(
        "Missing the following properties: {}",
        missing_properties.join(", ")
    );
    if !missing_properties.is_empty() {
        struct_.ident.span().unstable().warning(msg).emit();
    }
}
