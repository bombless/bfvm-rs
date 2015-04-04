#![feature(convert)]

/// example code here:
/// ```rust
/// pub fn main() {
///     println!("{:?}", parse_lambda::<mock::Vm>(&mut "`a'".chars()));
/// }
///
/// mod mock {
///     pub struct Vm;
///     #[derive(Debug, Clone)]
///     pub struct Code;
///     impl From<String> for Result<Code, String> {
///         fn from(_: String)->Self {
///             Ok(Code)
///         }
///     }
///     impl super::Vm for Vm {
///         type ByteCode = Code;
///         fn macro_expand<'a>(&mut self, _: &'a str)->Result<Code, super::Signal> {
///             Ok(Code)
///         }
///         fn run(&mut self, _: &Code, _: &Vec<super::Val<Self>>)->Result<super::Val<Self>, String> {
///             Err("mock".to_string())
///         }
///     }
/// }
/// ```
///
use std::rc::Rc;
use std::fmt::{Formatter, Error, Debug, Display};
use std::error::FromError;

pub trait Vm {
    type ByteCode;
    fn macro_expand<'a>(&mut self, &'a str)->Result<Self::ByteCode, Signal>;
    fn run(&mut self, &Self::ByteCode, &Vec<Val<Self>>)->Result<Val<Self>, String>;
}

#[derive(PartialEq, Eq, Debug)]
pub enum Signal {
    Fail(String),
    Continue,
    Quit
}

impl FromError<String> for Signal {
    fn from_error(e: String)->Self {
        Signal::Fail(e)
    }
}

pub enum Val<T: Vm> {
    Str(String),
    If(Rc<Val<T>>, Rc<Val<T>>, Rc<Val<T>>),
    Lambda(T::ByteCode),
    Call(Rc<Val<T>>, Vec<Val<T>>),
    Macro(String),
    Nil
}
use Val::*;

impl<T: Vm> Val<T> {
    pub fn kind(&self)->&'static str {
        match self {
            &Str(_) => "str",
            &If(..) => "if",
            &Lambda(_) => "lambda",
            &Call(..) => "call",
            &Macro(_) => "macro",
            &Nil => "nil"
        }
    }
}

impl<T> From<Rc<Val<T>>> for Val<T> where T: Vm, T::ByteCode: Clone {
    fn from(v: Rc<Val<T>>)->Self {
        From::from(&*v)
    }
}

impl<'a, T> From<&'a Val<T>> for Val<T> where T: Vm, T::ByteCode: Clone {
    fn from(v: &'a Val<T>)->Self {
        match v {
            &Str(ref s) => Str(s.clone()),
            &If(ref p, ref t, ref f) => If(p.clone(), t.clone(), f.clone()),
            &Lambda(ref bc) => Lambda(bc.clone()),
            &Call(ref first, ref args) => Call(first.clone(), {
                args.iter().map(From::from).collect()
            }),
            &Macro(ref s) => Macro(s.clone()),
            &Nil => Nil
        }
    }
}

impl<T> Debug for Val<T> where T: Vm, T::ByteCode: Display {
    fn fmt(&self, f: &mut Formatter)->Result<(), Error> {
        write!(f, "{:?}",  self.to_string())
    }
}

impl<T> Display for Val<T> where T: Vm, T::ByteCode: Display {
        fn fmt(&self, f: &mut Formatter)->Result<(), Error> {
        match self {
            &Nil => write!(f, "nil"),
            &Macro(ref name) => write!(f, "`{}'", name),
            &Str(ref s) => write!(f, "{}", s),
            &If(..) => write!(f, "<if expression>"),
            &Lambda(ref byte_code) => write!(f, "~{}", byte_code),
            &Call(ref byte_code, ref args) => {
                let mut print_args = String::new();
                for i in args {
                    print_args.push_str(&format!("{} ", i))
                }
                write!(f, "({} [ {}])", byte_code, print_args)
            }
        }
    }
}

pub enum ParseResult {
    Char(char),
    Compile(String),
    Eof
}
use ParseResult::*;

impl Debug for ParseResult {
    fn fmt(&self, f: &mut Formatter)->Result<(), Error> {
        match self {
            &Compile(ref s) => write!(f, "failed to compile: {}", s),
            &Char(c) => write!(f,
                               "unexpected character `{}`",
                               c.escape_default().collect::<String>()),
            &Eof => write!(f, "unexpectly terminated")
        }
    }
}

enum CalcResult {
    Msg(String),
    Sgl(Signal)
}
use CalcResult::*;

impl<T> Val<T> where T: Vm, T::ByteCode: Display + Clone {
    fn calc(&self, vm: &mut T)->Result<Val<T>, CalcResult> {
        match self {
            &Nil | &Lambda(_) | &Str(_) => Ok(From::from(self)),
            &Macro(ref name) => match vm.macro_expand(name) {
                Ok(x) => Ok(Lambda(x)),
                Err(err) => Err(Sgl(err))
            },
            &Call(ref first, ref tail) => {
                match first.calc(vm) {
                    Ok(Lambda(ref lambda)) => match vm.run(lambda, tail) {
                        Ok(x) => Ok(x),
                        Err(err) => Err(Msg(format!("runtime error: {}", err)))
                    },
                    Ok(v) => Err(Msg(format!("need callable here, found {} instead", v))),
                    Err(err) => Err(err)
                }
            },
            &If(ref p, ref t, ref f) => {
                let p = match p.calc(vm) {
                    Ok(Nil) => true,
                    Ok(Str(ref s)) if s.is_empty() => true,
                    Ok(_) => false,
                    Err(err) => return Err(err)
                };
                Ok(try!(if p {
                    t.calc(vm)
                } else {
                    f.calc(vm)
                }))
            }
        }
    }
}

fn parse_lambda<T>(s: &mut Iterator<Item=char>)->Result<T::ByteCode, ParseResult>
    where T: Vm, Result<T::ByteCode, String>: From<String> {
    match parse_str('\'', s) {
        Ok(s) => match From::from(s) {
            Ok(x) => Ok(x),
            Err(err) => Err(Compile(err))
        },
        Err(err) => Err(err)
    }
}

fn parse_macro(s: &mut Iterator<Item=char>)->Result<String, ParseResult> {
    parse_str('~', s)
}

fn parse_list<T>(s: &mut Iterator<Item=char>)->Result<Val<T>, ParseResult>
    where T: Vm, Result<T::ByteCode, String>: From<String> {
    let first = match parse::<T>(s) {
        Ok(x) => x,
        Err(Char(')')) => return Ok(Nil),
        Err(err) => return Err(err)
    };
    let mut ret = Vec::new();
    loop {
        match parse(s) {
            Ok(v) => ret.push(v),
            Err(Char(')')) => return Ok(Call(Rc::new(first), ret)),
            Err(err) => return Err(err)
        }
    }
}

pub fn parse<T>(s: &mut Iterator<Item=char>)->Result<Val<T>, ParseResult>
    where T: Vm, Result<T::ByteCode, String>: From<String> {
    match s.next() {
        None => Err(Eof),
        Some(' ') | Some('\t') | Some('\r') | Some('\n') => parse(s),
        Some(x) if x == '\'' || x == '"' => Ok(Str(try!(parse_str(x, s)))),
        Some('?') => Ok(try!(parse_if(s))),
        Some('`') => Ok(Lambda(try!(parse_lambda::<T>(s)))),
        Some('(') => parse_list(s),
        Some('@') => Ok(Macro(try!(parse_macro(s)))),
        Some(x) => Err(Char(x)),
    }
}

fn parse_str(delim: char, s: &mut Iterator<Item=char>)->Result<String, ParseResult> {
    let mut escape = false;
    let mut ret = String::new();
    while let Some(c) = s.next() {
        if c == delim && !escape {
            return Ok(ret)
        }
        else if escape {
            escape = false;
            ret.push(match c {
                'r' => '\r',
                'n' => '\n',
                't' => '\t',
                '\'' => '\'',
                '"' => '"',
                '\\' => '\\',
                c if c == delim => delim,
                _ => return Err(Char(c))
            })
        } else if c == '\\' {
            escape = true
        } else {
            ret.push(c)
        }
    }
    Err(Eof)
}

fn parse_if<T>(s: &mut Iterator<Item=char>)->Result<Val<T>, ParseResult>
    where T: Vm, Result<T::ByteCode, String>: From<String> {
    let (p, t, f) = (try!(parse(s)), try!(parse(s)), try!(parse(s)));
    Ok(If(Rc::new(p), Rc::new(t), Rc::new(f)))
}

pub fn repl<T>(vm: &mut T) where
    T: Vm,
    Result<T::ByteCode, String>: From<String>,
    T::ByteCode: Display + Clone {
    use std::io::{stdin, stdout, Write};
    let mut history = String::new();
    stdout().write(b">").unwrap();
    stdout().flush().unwrap();
    loop {
        if !history.is_empty() {
            stdout().write(b"> ... ").unwrap();
            stdout().flush().unwrap();
        }
        let mut line = String::new();
        stdin().read_line(&mut line).unwrap();
        let mut buf = history.clone();
        buf.push_str(&line);
        let s = &mut buf.chars();
        match parse::<T>(s) {
            Ok(ref x) => {
                let unexpected = s.find(|&x| match x {
                    '\t' | '\r' | '\n' | ' ' => false,
                    _ => true
                });
                if let Some(c) = unexpected {
                    println!("error: unexpected `{}`", c.escape_default().collect::<String>());
                } else {
                    println!("{}", match x.calc(vm) {
                        Ok(x) => format!("{}", x),
                        Err(Msg(msg)) => format!("failed to calculate: {}", msg),
                        Err(Sgl(Signal::Quit)) => return,
                        Err(Sgl(Signal::Continue)) => String::new(),
                        Err(Sgl(Signal::Fail(err))) => {
                            format!("{}", err)
                        }
                    })
                }
            },
            Err(Eof) => {
                if !buf.chars().all(char::is_whitespace) {
                    history = buf.clone();
                    continue
                }
            },
            Err(Char(c)) => {
                println!("failed to parse expression: unexpected `{}`",
                         c.escape_default().collect::<String>());
                stdout().flush().unwrap();
                stdout().write(b"\n").unwrap();
                stdout().flush().unwrap()
            },
            Err(Compile(err)) => {
                println!("compile error: {}", err);
            }
        }
        stdout().write(b">").unwrap();
        stdout().flush().unwrap();
        history.clear()
    }
}

#[cfg(test)]
mod tests;
