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

    pub struct Convert;

    impl Display for Code {
        fn fmt(&self, f: &mut Formatter)->Result<(), Error> {
            write!(f, "Code")
        }
    }

    impl From<String> for Convert {
        fn from(_: String)->Self {
            Convert
        }
    }

    impl From<Convert> for Result<Code, String> {
        fn from(_: Convert)->Self {
            Ok(Code)
        }
    }

    impl ::rt::Vm for Vm {
        type ByteCode = Code;
        type Convert = Convert;
        type CompileFail = String;
    }
}
