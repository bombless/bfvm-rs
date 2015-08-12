use std::fmt::{Debug, Formatter, Error};

pub fn object<'a>(kind: &'a str, content: &'a [u8])->Vec<u8> {
    let mut ret = Vec::new();
    ret.push(b'd');
    ret.extend(byte_string(kind.as_bytes()));
    ret.extend(byte_string(content));
    ret.push(b'e');
    ret
}

pub fn byte_string(s: &[u8])->Vec<u8> {
    let mut ret = Vec::new();
    let len = s.len();
    ret.extend(format!("{}", len).bytes());
    ret.push(b':');
    ret.extend(s.iter().cloned());
    ret
}

pub enum ParseError {
    Char(u8),
    Val(Value),
    Eof
}

impl Debug for ParseError {
    fn fmt(&self, f: &mut Formatter)->Result<(), Error> {
        match self {
            &ParseError::Char(ref c) => {
                let fmt = match std::char::from_u32(*c as u32) {
                    Some(x) => x.escape_default().collect(),
                    None => format!("0x{:02X}", c)
                };
                write!(f, "unexpected character `{}`", fmt)
            },
            &ParseError::Val(ref v) => {
                write!(f, "unexpected value {:?}", v)
            },
            &ParseError::Eof => {
                write!(f, "unexpected EOF")
            },
        }
    }
}

use ParseError::*;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Value {
    ByteString(Vec<u8>),
    Integer(i32),
    List(Vec<Value>),
    Dict(Vec<(Vec<u8>, Value)>)
}

use Value::*;

impl From<Value> for Vec<u8> {
    fn from(v: Value)->Self {
        v.into_bytes()
    }
}

impl From<ParseError> for String {
    fn from(r: ParseError)->Self {
        format!("{:?}", r)
    }
}

impl Value {
    fn into_bytes(self)->Vec<u8> {
        match self {
            ByteString(v) => byte_string(&v),
            Integer(v) => format!("i{}e", v).into_bytes(),
            List(v) => {
                let mut ret = Vec::new();
                ret.push(b'l');
                for i in v {
                    ret.extend(i.into_bytes())
                }
                ret.push(b'e');
                ret
            },
            Dict(v) => {
                let mut ret = Vec::new();
                ret.push(b'd');
                for (k, v) in v {
                    ret.extend(byte_string(&k));
                    ret.extend(v.into_bytes())
                }
                ret.push(b'e');
                ret
            }
        }
    }
}

/// See <http://en.wikipedia.org/wiki/Bencode>
pub fn parse(s: &mut Iterator<Item=u8>)->Result<Value, ParseError> {
    if let Some(b) = s.next() {
        return Ok(match b as char {
            'i' => Integer(try!(parse_integer(s))),
            'l' => List(try!(parse_list(s))),
            'd' => Dict(try!(parse_dict(s))),
            'e' => return Err(Char(b'e')),
            _ => ByteString(try!(parse_byte_string(b, s)))
        })
    }
    Err(Eof)
}

fn parse_byte_string(b: u8, s: &mut Iterator<Item=u8>)->Result<Vec<u8>, ParseError> {
    if b == b'0' {
        return match s.next() {
            None => Err(Eof),
            Some(b':') => Ok(Vec::new()),
            Some(_) => Err(Char(b))
        }
    }
    else if b < b'0' || b > b'9' {
        return Err(Char(b))
    }
    let mut len = b as u32 - '0' as u32;
    loop {
        match s.next() {
            None => return Err(Eof),
            Some(x) if x >= b'0' && x <= b'9' => {
                len = len * 10 + (x as u32 - '0' as u32)
            },
            Some(b':') => break,
            Some(x) => return Err(Char(x))
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

fn parse_list(s: &mut Iterator<Item=u8>)->Result<Vec<Value>, ParseError> {
    let mut ret = Vec::new();
    loop {
        match parse(s) {
            Ok(v) => ret.push(v),
            Err(Char(b'e')) => return Ok(ret),
            Err(err) => return Err(err)
        }
    }
}

fn parse_dict(s: &mut Iterator<Item=u8>)->Result<Vec<(Vec<u8>, Value)>, ParseError> {
    let mut ret = Vec::new();
    loop {
        let k = match parse(s) {
            Ok(ByteString(v)) => v,
            Ok(v) => return Err(Val(v)),
            Err(Char(b'e')) => return Ok(ret),
            Err(err) => return Err(err)
        };
        let v = try!(parse(s));
        ret.push((k, v))
    }
}

fn parse_integer(s: &mut Iterator<Item=u8>)->Result<i32, ParseError> {
    let (mut ret, sign) = match s.next() {
        None => return Err(Eof),
        Some(b'0') => return match s.next() {
            Some(b'e') => Ok(0),
            Some(x) => Err(Char(x)),
            None => Err(Eof)
        },
        Some(b'-') => (0, -1),
        Some(x) if x > b'0' && x <= b'9' => {
            (x as i32 - '0' as i32, 1)
        },
        Some(x) => return Err(Char(x))
    };
    loop {
        match s.next() {
            None => return Err(Eof),
            Some(b'e') => return Ok(sign * ret),
            Some(x) if (x >= b'0' && x <= b'9') && !(ret == 0 && x == b'0') => {
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
