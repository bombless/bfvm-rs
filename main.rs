#![feature(convert, collections)]

extern crate bf;
extern crate rt;
extern crate bencode;

use std::sync::mpsc::channel;
use std::default::Default;
use std::fmt::{Formatter, Error, Display};

#[derive(Default)]
struct BfVm {
    log_macros: Vec<(String, bf::Vm)>,
    log_calls: Vec<(bf::Vm, Vec<rt::Val<BfVm>>, rt::Val<BfVm>)>
}

impl Display for BfVm {
    fn fmt(&self, f: &mut Formatter)->Result<(), Error> {
        let cnt_macros = self.log_macros.len();
        if cnt_macros == 0 {
            try!(write!(f, "no log for macros\n"));
        } else {
            try!(write!(f, "log for macros: ({} entries)\n", cnt_macros));
            for &(ref k, ref v) in &self.log_macros {
                try!(write!(f, "!{}={}\n", k, v));
            }
        }
        let cnt_calls = self.log_calls.len();
        if cnt_calls == 0 {
            try!(write!(f, "no log for calls\n"));
        } else {
            try!(write!(f, "log for calls: ({} entries)\n", cnt_calls));
            for &(ref code, ref args, ref rslt) in &self.log_calls {
                try!(write!(f, "`{:?}'\n", code.to_string()));
                for (idx, arg) in (1 ..).zip(args.iter()) {
                    try!(write!(f, "arg{}: {:?}\n", idx, arg.to_string()));
                }
                try!(write!(f, "result: {:?}", rslt));
            }
        }
        Ok(())
    }
}

impl rt::Vm for BfVm {
    type ByteCode = bf::Vm;
    fn macro_expand<'a>(&mut self, id: &'a str)->Result<bf::Vm, rt::Signal> {
        let ret = match id {
            "greeting" => Ok(try!(From::from(
                "++++++++++[>+++++++>++++++++++>+++>+<<<<-]>++.>+.+++++++..+++."))),
            "A" => Ok(bf::Vm::print(b"1:A")),
            "log" => {
                let log = self.to_string();
                Ok(bf::Vm::print(&bencode::byte_string(log.as_bytes())))
            },
            "help" => {
                Err(rt::Signal::Continue)
            }
            "quit" => {
                Err(rt::Signal::Quit)
            },
            x => if let Ok(idx) = x.parse::<u8>() {
                if let Some(entry) = self.log_macros.get(idx as usize) {
                    println!("{:?}", entry)
                } else {
                    println!("no macro access log entry for index {}", idx);
                    println!("type `(@log~)` for log overview")
                }
                return Err(rt::Signal::Continue)
            } else if let (Some(&b'#'), Ok(idx)) =
                (x.as_bytes().first(), x.chars().skip(1).collect::<String>().parse::<u8>()) {
                if let Some(entry) = self.log_macros.get(idx as usize) {
                    println!("{:?}", entry)
                } else {
                    println!("no function call log entry for index #{}", idx);
                    println!("type `(@log~)` for log overview")
                }
                return Err(rt::Signal::Continue)
            } else {
                return Err(rt::Signal::Fail(format!("failed to expand macro `{}`",
                                                    id.escape_default())))
            }
        };
        if let Ok(ref ok) = ret {
            self.log_macros.push((id.to_string(), ok.clone()));
            println!("{} entries for macro now", self.log_macros.len())
        }
        ret
    }
    fn run(&mut self, code: &bf::Vm, args: &Vec<rt::Val<Self>>)->Result<rt::Val<Self>, String> {
        let (output, data) = channel();
        let (arg_stream, input) = channel();
        arg_stream.send(b'l').unwrap();
        for i in args {
            for b in utils::rt2bencode(From::from(i)) {
                arg_stream.send(b).unwrap()
            }
        }
        arg_stream.send(b'e').unwrap();

        if let Err(err) = code.run(output, input) {
            return Err(format!("failed to start vm: {:?}", err))
        }
        let mut ret = Vec::new();
        while let Ok(n) = data.recv() {
            //println!("line {}, receiving 0x{:X}", line!(), n);
            ret.push(n)
        }
        match bencode::parse(&mut ret.iter().cloned()) {
            Ok(s) => {
                self.log_calls.push((code.clone(),
                                     args.iter().map(From::from).collect(),
                                     utils::bencode2rt(s.clone())));
                println!("{} entries for function call now", self.log_calls.len());
                Ok(utils::bencode2rt(s))
            },
            Err(err) => {
                let fmt = utils::pretty(&ret);
                Err(format!("broken return value {:?}, {:?}", fmt, err))
            }
        }
    }
}

fn main() {
    rt::repl::<BfVm>(&mut Default::default());
}

mod utils;
