use super::*;

trait ToValue {
    fn to_value(self)->Value;
}

impl ToValue for i32 {
    fn to_value(self)->Value {
        Value::Integer(self)
    }
}

impl ToValue for Vec<Value> {
    fn to_value(self)->Value {
        Value::List(self)
    }
}

impl<'a> ToValue for &'a [u8] {
    fn to_value(self)->Value {
        Value::ByteString(Vec::new() + self)
    }
}

impl ToValue for Vec<(Vec<u8>, Value)> {
    fn to_value(self)->Value {
        Value::Dict(self)
    }
}

#[cfg(test)]
fn eq<T: ToValue>(lhs: &[u8], rhs: T) {
    assert_eq!(parse(&mut lhs.iter().cloned()).unwrap(), ToValue::to_value(rhs))
}

#[test]
fn test_integer() {
    eq(b"i18e", 18);
    eq(b"i-18e", -18)
}

#[test]
fn test_list() {
    let item_1 = Value::Integer(42);
    let item_2 = Value::List(Vec::new());
    let list = vec![ item_1, item_2 ];
    eq(b"li42elee", list)
}

#[test]
fn test_dict() {
    let first = (Vec::new() + b"1", Value::Integer(2));
    let second = (Vec::new() + b"2", Value::Integer(3));
    let third = (Vec::new() + b"3", Value::Integer(5));
    let code = b"d1:1i2e1:2i3e1:3i5ee";
    eq(code, vec![ first, second, third ])
}


#[test]
fn test_byte_string() {
    eq(b"5:hello", &b"hello"[..])
}
