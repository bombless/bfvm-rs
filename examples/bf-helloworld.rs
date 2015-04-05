extern crate bf;
use bf::*;
use std::sync::mpsc::channel;
use std::io::{stdout, Write};
use std::thread::spawn;


fn main() {
    let (_, input) = channel();
    let (output, data) = channel();
    let conv = Convert::from("++++++++++[>+++++++>++++++++++>+++>+<<<<-]\
        >++.>+.+++++++..+++.>++.<<+++++++++++++++.\
        >.+++.------.--------.>+.>.");
    let vm = <Result<_, _>>::from(conv).unwrap();
    spawn(move || {
        vm.run(output, input).unwrap();
    });
    loop {
        match data.recv() {
            Ok(x) => {
                stdout().write(&[x]).unwrap();
            },
            Err(_) => {
                break
            }
        }
    }
}
