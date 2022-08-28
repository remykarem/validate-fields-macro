#![allow(dead_code)]

use validate_fields_macro::validate_fields;


#[validate_fields]
struct Person {
    name: String,
    age: u8,
    email: String,
}

#[validate_fields]
struct Credentials<'a> {
    private_key: &'a str,
}

#[validate_fields]
struct Client<'a> {
    url: &'a str,
    path: &'a str,
}

fn main() {
    println!("Hello, world!");
}
