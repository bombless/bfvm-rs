extern crate bf;
extern crate sexp;
extern crate bencode;


use std::collections::HashMap;
struct Env {
    functions: HashMap<String, Vec<u8>>,
    variables: HashMap<String, lisp::Value>
}
