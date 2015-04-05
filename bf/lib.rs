use std::sync::mpsc::{Sender, Receiver};
use std::fmt::{Formatter, Error, Display};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Vm(Vec<ByteCode>);

impl Display for Vm {
    fn fmt(&self, f: &mut Formatter)->Result<(), Error> {
        for &c in &self.0 {
            try!(write!(f, "{}", c as u8 as char))
        }
        Ok(())
    }
}

impl From<Vm> for Vec<u8> {
    fn from(v: Vm)->Self {
        v.0.into_iter().map(|x| x as u8).collect()
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum ByteCode {
    Lt = b'<',
    Gt = b'>',
    Plus = b'+',
    Minus = b'-',
    Dot = b'.',
    Comma = b',',
    LeftBracket = b'[',
    RightBracket = b']'
}

pub enum Convert {
    Ok(Vm),
    Err(String)
}

impl From<Convert> for Result<Vm, String> {
    fn from(v: Convert)->Self {
        match v {
            Convert::Ok(ok) => Ok(ok),
            Convert::Err(s) => Err(s)
        }
    }
}

impl From<String> for Convert {
    fn from(s: String)->Self {
        From::from(&*s)
    }
}

impl<'a> From<&'a str> for Convert {
    fn from(s: &str)->Self {
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
                    c => return Convert::Err(format!("unexpected character `{}`", c))
                })
            }
            Convert::Ok(Vm(vec))
    }
}


impl Vm {
    pub fn print(s: &[u8])->Vm {
        struct Tracker(u8);
        impl Tracker {
            fn put(&mut self, tar: u8)->Vec<ByteCode> {
                use std::iter::repeat;
                let now = self.0;
                let diff = tar as i32 - now as i32;
                let rep = repeat(if diff < 0 { ByteCode::Minus } else { ByteCode::Plus });
                let mut ret = rep.take(diff.abs() as usize).collect::<Vec<_>>();
                ret.push(ByteCode::Dot);
                self.0 = ((diff + self.0 as i32 + 256) % 256) as u8;
                ret
            }
        }
        let mut tracker = Tracker(0);
        let mut ret = Vec::new();
        for &i in s {
            ret.extend(tracker.put(i))
        }
        Vm(ret)
    }
    pub fn run(&self, snd: Sender<u8>, rcv: Receiver<u8>)->Result<(), String> {
        let mut mem: Vec<u8> = vec![ 0 ];
        let ref vm = self.0;
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
