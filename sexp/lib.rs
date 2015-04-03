use std::iter::Peekable;
#[derive(Clone, PartialEq, Eq)]
pub enum Value {
    List(Vec<Value>),
    Number(i32),
    Str(String),
    Atom(String),
    Nil
}
pub use Value::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseResult {
    Char(u8),
    Eof
}
use ParseResult::*;


const RIGHT_PAREN: u8 = ')' as u8;
const LEFT_PAREN: u8 = '(' as u8;
const SEMI_COLON: u8 = ';' as u8;

fn parse_space<T: Iterator<Item=u8>>(s: &mut Peekable<T>)->Result<u8, ParseResult> {
    while let Some(b) = s.next() {
        println!("{}", line!());
        match b as char {
            ' ' | '\t' | '\r' | '\n' => (),
            _ => return Ok(b)
        }
    }
    Err(Eof)
}


fn parse_list<T: Iterator<Item=u8>>(s: &mut Peekable<T>)->Result<Value, ParseResult> {
    let fst = match parse_sexp(s) {
        Err(Char(RIGHT_PAREN)) => return Ok(Nil),
        Err(err) => return Err(err),
        Ok(x) => x
    };
    let b = try!(parse_space(s));
    if b == ')' as u8 {
        return Ok(List(vec![ fst ]))
    }
    let mut list = vec![ fst ];
    let s = &mut Some(b).into_iter().chain(s).peekable();
    loop {
        match parse_sexp(s) {
            Err(Char(RIGHT_PAREN)) => return Ok(List(list)),
            Ok(x) => list.push(x),
            Err(err) => return Err(err)
        }
    }
}


fn parse_string<T: Iterator<Item=u8>>(delim: u8, s: &mut Peekable<T>)->Result<Value, ParseResult> {
    use std::char::from_u32;
    let mut escape = false;
    let mut ret = String::new();
    while let Some(x) = s.next() {
        if x == delim && !escape { return Ok(Str(ret)) }
        else if escape {
            ret.push(match from_u32(x as u32) {
                Some('\'') => '\'',
                Some('"') => '"',
                Some('t') => '\t',
                Some('r') => '\r',
                Some('n') => '\n',
                Some('\\') => '\\',
                _ => return Err(Char(x))
            });
            escape = false
        } else if x == '\\' as u8 {
            escape = true
        } else {
            match from_u32(x as u32) {
                None => return Err(Char(x)),
                Some(x) => ret.push(x)
            }
        }
    }
    Err(Eof)
}

fn parse_atom<T: Iterator<Item=u8>>(s: &mut Peekable<T>)->Result<Value, ParseResult> {
    let mut ret = String::new();
    loop {
        match s.peek() {
            Some(&x) if (x >= 'a' as u8 && x <= 'z' as u8) ||
                    (x >= 'A' as u8 && x <= 'Z' as u8) => {
                ret.push(x as char)
            },
            _ => return Ok(Str(ret))
        }
        s.next();
    }
}

pub fn parse_sexp<T: Iterator<Item=u8>>(s: &mut Peekable<T>)->Result<Value, ParseResult> {
    Ok(match try!(parse_space(s)) {
        LEFT_PAREN => try!(parse_list(s)),
        RIGHT_PAREN => return Err(Char(RIGHT_PAREN)),
        SEMI_COLON => {
            while let Some(x) = s.next() {
                if x == '\n' as u8 {
                    break
                }
            }
            try!(parse_sexp(s))
        },
        x if x == '\'' as u8 || x == '"' as u8 => try!(parse_string(x, s)),
        x => try!(parse_atom(&mut [x].iter().cloned().chain(s).peekable()))
    })
}

#[cfg(test)]
mod tests;

fn main() {
    parse_sexp(&mut b"()".iter().cloned().peekable()).unwrap();
}
