extern crate lisp;
use lisp::*;

fn main() {
    let val_1 = Value::Number(42);
    println!("val_1 = {:?}", val_1);
    let val_2 = Value::cons(val_1, Value::Nil);
    println!("val_2 = {:?}", val_2);
    let val_3 = Value::car(val_2.clone()).unwrap();
    let val_4 = Value::cdr(val_2).unwrap();
    println!("(car val_2) = {:?}, (cdr val_2) = {:?}", val_3, val_4)
}
