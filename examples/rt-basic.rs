#![feature(convert)]

extern crate rt;
use rt::*;
use std::rc::Rc;

fn main() {
    let val_1 = Val::Str::<mock::Vm>("abc".to_string());
    println!("val_1 = {:?}", val_1);
    let val_2 = Val::Nil;
    println!("val_2 = {:?}", val_2);
    let val_3 = Val::Str("cef".to_string());
    let val_4 = Val::If(Rc::new(val_1), Rc::new(val_2), Rc::new(val_3));
    println!("val_4 = {}", val_4)
}
mod mock {
    use std::fmt::{Formatter, Error, Display};
    pub struct Vm;
    #[derive(Debug, Clone)]
    pub struct Code;

    impl Display for Code {
        fn fmt(&self, f: &mut Formatter)->Result<(), Error> {
            write!(f, "Code")
        }
    }

    impl From<String> for Result<Code, String> {
        fn from(_: String)->Self {
            Ok(Code)
        }
    }
    impl ::rt::Vm for Vm {
        type ByteCode = Code;
        fn macro_expand<'a>(&mut self, _: &'a str)->Result<Code, ::rt::Signal> {
            Ok(Code)
        }
        fn run(&mut self, _: &Code, _: &Vec<::rt::Val<Self>>)->Result<::rt::Val<Self>, String> {
            Err("mock".to_string())
        }
    }
}
