extern crate bf;
use bf::*;
use std::sync::mpsc::channel;
use std::str::FromStr;
use std::io::{stdout, Write};
use std::thread::spawn;
fn main() {
    let (_, input) = channel();
    let (output, data) = channel();
    let vm = FromStr::from_str("++++++++++[>+++++++>++++++++++>+++>+<<<<-]\
        >++.>+.+++++++..+++.>++.<<+++++++++++++++.\
        >.+++.------.--------.>+.>.");
    //println!("{:?}", vm);
    let vm: Vm = vm.unwrap();
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
