#![allow(unused_attributes, unused_features)]
#![feature(convert)]

use std::fmt::{Debug, Formatter, Error};

const E: u8 = 'e' as u8;
const COLON: u8 = ':' as u8;

pub enum ParseResult {
    Char(u8),
    Val(Value),
    Eof
}

impl Debug for ParseResult {
    fn fmt(&self, f: &mut Formatter)->Result<(), Error> {
        match self {
            &ParseResult::Char(ref c) => {
                write!(f, "unexpected character {}", c)
            },
            &ParseResult::Val(ref v) => {
                write!(f, "unexpected value {:?}", v)
            },
            &ParseResult::Eof => {
                write!(f, "unexpected EOF")
            },
        }
    }
}

use ParseResult::*;

#[derive(Debug, PartialEq, Eq)]
pub enum Value {
    ByteString(Vec<u8>),
    Integer(i32),
    List(Vec<Value>),
    Dict(Vec<(Vec<u8>, Value)>)
}


/// See <http://en.wikipedia.org/wiki/Bencode>
pub fn parse(s: &mut Iterator<Item=u8>)->Result<Value, ParseResult> {
    if let Some(b) = s.next() {
        return Ok(match b as char {
            'i' => Value::Integer(try!(parse_integer(s))),
            'l' => Value::List(try!(parse_list(s))),
            'd' => Value::Dict(try!(parse_dict(s))),
            'e' => return Err(Char(E)),
            _ => Value::ByteString(try!(parse_byte_string(b, s)))
        })
    }
    Err(Eof)
}

fn parse_byte_string(b: u8, s: &mut Iterator<Item=u8>)->Result<Vec<u8>, ParseResult> {
    if b == '0' as u8 {
        return match s.next() {
            None => Err(Eof),
            Some(COLON) => Ok(Vec::new()),
            Some(x) => Err(Char(x))
        }
    }
    let mut len = b as u32 - '0' as u32;
    loop {
        let c = s.next();
        match c {
            None => return Err(Eof),
            Some(x) if x >= '0' as u8 && x <= '9' as u8 => {
                len = len * 10 + (x as u32 - '0' as u32)
            },
            Some(COLON) => break,
            Some(x) =>return Err(Char(x))
        }
    }
    let mut ret = Vec::new();
    for _ in (0 .. len) {
        if let Some(b) = s.next() {
            ret.push(b)
        } else {
            return Err(Eof)
        }
    }
    Ok(ret)
}

fn parse_list(s: &mut Iterator<Item=u8>)->Result<Vec<Value>, ParseResult> {
    let mut ret = Vec::new();
    loop {
        match parse(s) {
            Ok(v) => ret.push(v),
            Err(Char(E)) => return Ok(ret),
            Err(err) => return Err(err)
        }
    }
}

fn parse_dict(s: &mut Iterator<Item=u8>)->Result<Vec<(Vec<u8>, Value)>, ParseResult> {
    let mut ret = Vec::new();
    loop {
        let k = match parse(s) {
            Ok(Value::ByteString(v)) => v,
            Ok(v) => return Err(Val(v)),
            Err(Char(E)) => return Ok(ret),
            Err(err) => return Err(err)
        };
        let v = try!(parse(s));
        ret.push((k, v))
    }
}

fn parse_integer(s: &mut Iterator<Item=u8>)->Result<i32, ParseResult> {
    const ZERO: u8 = '0' as u8;
    const MINUS: u8 = '-' as u8;
    let (mut ret, sign) = match s.next() {
        None => return Err(Eof),
        Some(ZERO) => return match s.next() {
            Some(E) => Ok(0),
            Some(x) => Err(Char(x)),
            None => Err(Eof)
        },
        Some(MINUS) => (0, -1),
        Some(x) if x > '0' as u8 && x <= '9' as u8 => {
            (x as i32 - '0' as i32, 1)
        },
        Some(x) => return Err(Char(x))
    };
    loop {
        match s.next() {
            None => return Err(Eof),
            Some(E) => return Ok(sign * ret),
            Some(x) if (x >= '0' as u8 && x <= '9' as u8) && !(ret == 0 && x == '0' as u8) => {
                ret = ret * 10 + x as i32 - '0' as i32
            },
            Some(x) => return Err(Char(x))
        }
    }
}

mod tests;

#[cfg(not(test))]
pub fn main() {
    parse(&mut b"li42elee".iter().cloned()).unwrap();
}
