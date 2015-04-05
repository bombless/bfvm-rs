use std::rc::Rc;
use std::fmt::{Formatter, Error, Debug, Display};

pub trait Vm: {
    type ByteCode;
    type CompileFail;
    type Convert;
    fn macro_expand<'a>(&mut self, _: &'a str)->Result<Self::ByteCode, Signal> {
        Err(Signal::Smoke)
    }
    fn run(&mut self, _: &Self::ByteCode, _: &Vec<Val<Self>>)->Result<Val<Self>, String> {
            Err("method `run` not implemented".to_string())
    }
}

#[derive(PartialEq, Eq, Debug)]
pub enum Signal {
    Fail(String),
    Smoke,
    Continue,
    Quit
}

impl From<String> for Signal {
    fn from(e: String)->Self {
        Signal::Fail(e)
    }
}

pub enum Val<T: Vm + ?Sized> {
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
        fn filter(input: &str, delim: char)->String {
            let mut ret = String::new();
            for c in input.chars() {
                if c == delim {
                    ret.push_str(&format!("\\{}", c))
                } else {
                    ret.push(c)
                }
            }
            ret
        }
        match self {
            &Nil => write!(f, "nil"),
            &Macro(ref name) => write!(f, "@{}~", name),
            &Str(ref s) => write!(f, "{}", s),
            &If(..) => write!(f, "<if expression>"),
            &Lambda(ref byte_code) => write!(f, "`{}'", filter(&byte_code.to_string(), '\'')),
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

pub enum ParseError {
    Char(char),
    Compile(String),
    Eof
}
use ParseError::*;

impl Debug for ParseError {
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

#[derive(Debug)]
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
                Err(Signal::Continue) => Ok(Nil),
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
                    Ok(Nil) => false,
                    Ok(Str(ref s)) if s.is_empty() => false,
                    Ok(_) => true,
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

fn parse_lambda<T>(s: &mut Iterator<Item=char>)->Result<T::ByteCode, ParseError>
    where T: Vm,
    T::Convert: From<String>,
    String: From<T::CompileFail>,
    Result<T::ByteCode, T::CompileFail>: From<T::Convert> {
    match parse_str('\'', s) {
        Ok(s) => match From::from(From::from(s)) {
            Ok(x) => Ok(x),
            Err(err) => Err(Compile(From::from(err)))
        },
        Err(err) => Err(From::from(err))
    }
}

fn parse_macro(s: &mut Iterator<Item=char>)->Result<String, ParseError> {
    parse_str('~', s)
}

fn parse_list<T>(s: &mut Iterator<Item=char>)->Result<Val<T>, ParseError>
    where T: Vm,
    T::Convert: From<String>,
    String: From<T::CompileFail>,
    Result<T::ByteCode, T::CompileFail>: From<T::Convert> {
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

pub fn parse<T>(s: &mut Iterator<Item=char>)->Result<Val<T>, ParseError>
    where T: Vm,
    T::Convert: From<String>,
    String: From<T::CompileFail>,
    Result<T::ByteCode, T::CompileFail>: From<T::Convert> {
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

fn parse_str(delim: char, s: &mut Iterator<Item=char>)->Result<String, ParseError> {
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

fn parse_if<T>(s: &mut Iterator<Item=char>)->Result<Val<T>, ParseError>
    where T: Vm,
    T::Convert: From<String>,
    String: From<T::CompileFail>,
    Result<T::ByteCode, T::CompileFail>: From<T::Convert> {
    let (p, t, f) = (try!(parse(s)), try!(parse(s)), try!(parse(s)));
    Ok(If(Rc::new(p), Rc::new(t), Rc::new(f)))
}

pub fn repl<T>(vm: &mut T) where
    T: Vm,
    T::Convert: From<String>,
    String: From<T::CompileFail>,
    Result<T::ByteCode, T::CompileFail>: From<T::Convert>,
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
                        },
                        Err(Sgl(Signal::Smoke)) => "broken implementation".to_string()
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
