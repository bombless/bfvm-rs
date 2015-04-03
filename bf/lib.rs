use std::sync::mpsc::{Sender, Receiver};
use std::str::FromStr;

#[derive(Clone, Debug)]
pub struct Vm(Vec<ByteCode>);

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum ByteCode {
    Lt,
    Gt,
    Plus,
    Minus,
    Dot,
    Comma,
    LeftBracket,
    RightBracket
}

impl Vm {
    pub fn run(self, snd: Sender<u8>, rcv: Receiver<u8>)->Result<(), String> {
        let mut mem: Vec<u8> = vec![ 0 ];
        let vm = self.0;
        let mut pc: usize = 0;
        let mut ptr: usize = 0;
        loop {
            match vm[pc] {
                ByteCode::Gt => {
                    ptr += 1;
                    if mem.len() <= ptr {
                        mem.push(0)
                    }
                },
                ByteCode::Lt => {
                    if ptr == 0 {
                        return Err("illegal pointer movement".to_string())
                    }
                    ptr -= 1
                },
                ByteCode::Plus => {
                    mem[ptr] = ((mem[ptr] as u32 + 1) % 256) as u8;
                },
                ByteCode::Minus => {
                    mem[ptr] = ((mem[ptr] as i32 - 1) % 256) as u8;
                },
                ByteCode::Dot => {
                    //println!("sending {}", mem[ptr]);
                    snd.send(mem[ptr]).unwrap()
                },
                ByteCode::Comma => {
                    mem[ptr] = rcv.recv().unwrap()
                },
                ByteCode::LeftBracket => {
                    if mem[ptr] == 0 {
                        while vm[pc] != ByteCode::RightBracket {
                            pc += 1;
                            if pc == vm.len() {
                                return Err("pc out of range".to_string())
                            }
                        }
                    }
                },
                ByteCode::RightBracket => {
                    if mem[ptr] != 0 {
                        while vm[pc] != ByteCode::LeftBracket {
                            if pc == 0 {
                                return Err("pc out of range".to_string())
                            }
                            pc -= 1
                        }
                    }
                },
            }
            pc += 1;
            if pc == vm.len() {
                break
            }
        }
        Ok(())
    }
}

impl FromStr for Vm {
    type Err = String;
    fn from_str(s: &str)->Result<Vm, String> {
        let mut vec = Vec::new();
        for c in s.chars() {
            vec.push(match c {
                '<' => ByteCode::Lt,
                '>' => ByteCode::Gt,
                '+' => ByteCode::Plus,
                '-' => ByteCode::Minus,
                '.' => ByteCode::Dot,
                ',' => ByteCode::Comma,
                '[' => ByteCode::LeftBracket,
                ']' => ByteCode::RightBracket,
                c => return Err(format!("unexpected character {}", c))
            })
        }
        Ok(Vm(vec))
    }
}
