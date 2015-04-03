
use super::{parse_sexp, Value};
#[test]
fn test() {
    let sexp = parse_sexp(&mut b"()".iter().cloned().peekable());
    //let sexp = sexp.unwrap();
    //assert_eq!(sexp, Value::Nil)
}
