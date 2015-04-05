use ::rt::Val as RtVal;
use ::rt::Vm;
use ::bencode::{object, Value, byte_string};
use ::std::fmt::Display;
use std::rc::Rc;

pub fn pretty(s: &[u8])->String {
    use std::char::from_u32;
    let mut fmt = String::new();
    for &i in s {
        match from_u32(i as u32) {
            Some(x) => fmt.push(x),
            None => fmt.push_str(&format!("0x{:02X}", i))
        }
    }
    fmt
}

pub fn rt2bencode<T>(v: RtVal<T>)->Vec<u8>
    where T: Vm,
          Vec<u8>: From<T::ByteCode>,
          RtVal<T>: From<Rc<RtVal<T>>> {
    match v {
        RtVal::Str(s) => byte_string(s.as_bytes()),
        RtVal::If(p, t, f) => {
            let mut ret = vec![ b'l' ];
            ret.extend(rt2bencode(RtVal::from(p)));
            ret.extend(rt2bencode(RtVal::from(t)));
            ret.extend(rt2bencode(RtVal::from(f)));
            ret.push(b'e');
            object("if", &ret)
        },
        RtVal::Lambda(code) => <Vec<_>>::from(code),
        RtVal::Call(fst, args) => {
            let mut ret = vec![ b'l' ];
            ret.extend(rt2bencode(RtVal::from(fst)));
            for i in args {
                ret.extend(rt2bencode(i));
            }
            ret.push(b'e');
            object("call", &ret)
        },
        RtVal::Macro(s) => object("macro", s.as_bytes()),
        RtVal::Nil => b"0:".iter().cloned().collect()
    }
}

pub fn bencode2rt<T>(v: Value)->RtVal<T>
    where T: Vm, T::ByteCode: Display {
    match v {
        Value::Dict(mut v) => {
            if v.len() != 1 {
                return RtVal::Nil
            }
            let (kind, v) = v.pop().unwrap();
            let kind = match ::std::str::from_utf8(&kind) {
                Ok(x) => x,
                _ => return RtVal::Nil
            };
            match (kind, v) {
                ("str", Value::ByteString(v)) => match String::from_utf8(v) {
                    Ok(x) => RtVal::Str(x),
                    _ => RtVal::Nil
                },
                _ => RtVal::Nil
            }
        },
        Value::ByteString(s) => match String::from_utf8(s) {
            Ok(s) => RtVal::Str(s),
            _ => RtVal::Nil
        },
        _ => RtVal::Nil
    }
}
