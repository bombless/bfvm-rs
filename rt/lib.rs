use std::rc::Rc;
use std::fmt::{Formatter, Debug, Display};
use std::fmt::Error as FmtError;

pub trait Vm: {
    type ByteCode;
    type CompileFail;
    type Convert;
    fn macro_expand<'a>(&mut self, _: &'a str)->MacroResult<Self::ByteCode> {
        MacroResult::Err("method `macro_expand` not implemented".to_string())
    }
    fn run(&mut self, _: &Self::ByteCode, _: &Vec<Val<Self>>)->Result<Val<Self>, String> {
            Err("method `run` not implemented".to_string())
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
                args.iter().map(Val::from).collect()
            }),
            &Macro(ref s) => Macro(s.clone()),
            &Nil => Nil
        }
    }
}

impl<T> Debug for Val<T> where T: Vm, T::ByteCode: Display {
    fn fmt(&self, f: &mut Formatter)->Result<(), FmtError> {
        write!(f, "{:?}",  self.to_string())
    }
}

impl<T> Display for Val<T> where T: Vm, T::ByteCode: Display {
    fn fmt(&self, f: &mut Formatter)->Result<(), FmtError> {
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

pub enum Error {
    Char(char),
    Compile(String),
    Eof
}
use Error::Char as UnexpectedChar;
use Error::Compile as CompileError;
use Error::Eof;

impl Debug for Error {
    fn fmt(&self, f: &mut Formatter)->Result<(), FmtError> {
        match self {
            &CompileError(ref s) => write!(f, "failed to compile: {}", s),
            &UnexpectedChar(c) => write!(f,
                               "unexpected character `{}`",
                               c.escape_default().collect::<String>()),
            &Eof => write!(f, "unexpectly terminated")
        }
    }
}

enum CalcResult<T> {
    Ok(T),
    Err(String),
    Quit
}
use CalcResult as Calc;

pub enum MacroResult<T> {
    Ok(T),
    Err(String),
    Continue,
    Quit
}

#[cfg(test)]
impl<T> CalcResult<T> {
    fn unwrap(self)->T {
        match self {
            Calc::Ok(v) => v,
            Calc::Err(err) => panic!("{:?}", err),
            Calc::Quit => panic!("unexpected quit message")
        }
    }
}

impl<T> Val<T> where T: Vm, T::ByteCode: Display + Clone {
    fn calc(&self, vm: &mut T)->CalcResult<Val<T>> {
        match self {
            &Nil | &Lambda(_) | &Str(_) => Calc::Ok(Val::from(self)),
            &Macro(ref name) => match vm.macro_expand(name) {
                MacroResult::Ok(x) => Calc::Ok(Lambda(x)),
                MacroResult::Err(err) => Calc::Err(err),
                MacroResult::Continue => Calc::Ok(Nil),
                MacroResult::Quit => Calc::Quit
            },
            &Call(ref first, ref tail) => {
                match first.calc(vm) {
                    Calc::Ok(Lambda(ref lambda)) => match vm.run(lambda, tail) {
                        Ok(x) => Calc::Ok(x),
                        Err(err) => Calc::Err(format!("runtime error: {}", err))
                    },
                    Calc::Ok(v) => {
                        Calc::Err(format!("callable needed, found {} instead", v))
                    },
                    err @ Calc::Err(_) => err,
                    quit @ Calc::Quit => quit
                }
            },
            &If(ref p, ref t, ref f) => {
                let p = match p.calc(vm) {
                    Calc::Ok(Nil) => false,
                    Calc::Ok(Str(ref s)) if s.is_empty() => false,
                    Calc::Ok(_) => true,
                    err @ Calc::Err(_) => return err,
                    quit @ Calc::Quit => return quit
                };
                if p {
                    t.calc(vm)
                } else {
                    f.calc(vm)
                }
            }
        }
    }
}

fn parse_lambda<T>(s: &mut Iterator<Item=char>)->Result<T::ByteCode, Error>
    where T: Vm,
    T::Convert: From<String>,
    String: From<T::CompileFail>,
    Result<T::ByteCode, T::CompileFail>: From<T::Convert> {
    match parse_str('\'', s) {
        // compile to VM byte code *now*, since we don't do lazy execution
        Ok(s) => match <Result<_, _>>::from(T::Convert::from(s)) {
            Ok(x) => Ok(x),
            Err(err) => Err(CompileError(String::from(err)))
        },
        Err(err) => Err(Error::from(err))
    }
}

fn parse_macro(s: &mut Iterator<Item=char>)->Result<String, Error> {
    parse_str('~', s)
}

fn parse_list<T>(s: &mut Iterator<Item=char>)->Result<Val<T>, Error>
    where T: Vm,
    T::Convert: From<String>,
    String: From<T::CompileFail>,
    Result<T::ByteCode, T::CompileFail>: From<T::Convert> {
    let first = match parse::<T>(s) {
        Ok(x) => x,
        Err(UnexpectedChar(')')) => return Ok(Nil),
        Err(err) => return Err(err)
    };
    let mut ret = Vec::new();
    loop {
        match parse(s) {
            Ok(v) => ret.push(v),
            Err(UnexpectedChar(')')) => return Ok(Call(Rc::new(first), ret)),
            Err(err) => return Err(err)
        }
    }
}

pub fn parse<T>(s: &mut Iterator<Item=char>)->Result<Val<T>, Error>
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
        Some(x) => Err(UnexpectedChar(x)),
    }
}

fn parse_str(delim: char, s: &mut Iterator<Item=char>)->Result<String, Error> {
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
                _ => return Err(UnexpectedChar(c))
            })
        } else if c == '\\' {
            escape = true
        } else {
            ret.push(c)
        }
    }
    Err(Eof)
}

fn parse_if<T>(s: &mut Iterator<Item=char>)->Result<Val<T>, Error>
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
                        Calc::Ok(x) => x.to_string(),
                        Calc::Err(err) => err,
                        Calc::Quit => return
                    })
                }
            },
            Err(Eof) => {
                if !buf.chars().all(char::is_whitespace) {
                    history = buf.clone();
                    continue
                }
            },
            Err(UnexpectedChar(c)) => {
                println!("failed to parse expression: unexpected `{}`",
                         c.escape_default().collect::<String>());
                stdout().flush().unwrap();
                stdout().write(b"\n").unwrap();
                stdout().flush().unwrap()
            },
            Err(CompileError(err)) => {
                println!("illegal lambda literal: {}", err);
            }
        }
        stdout().write(b">").unwrap();
        stdout().flush().unwrap();
        history.clear()
    }
}

#[cfg(test)]
mod tests;
