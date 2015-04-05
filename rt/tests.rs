#[test]
fn test_basic_lambda() {
    assert_eq!(super::parse_lambda::<Vm>(&mut "'".chars()).unwrap(), "")
}

#[test]
fn test_branches() {
    let vm = super::parse(&mut " ? () 'a' 'b' ".chars()).unwrap();
    assert_eq!(vm.calc(&mut Vm).unwrap().to_string(), r##"b"##)
}

struct Vm;
impl super::Vm for Vm {
    type ByteCode = String;
    type CompileFail = String;
    type Convert = Convert;
}

pub struct Convert(String);

impl From<String> for Convert {
    fn from(v: String)->Self {
        Convert(v)
    }
}

impl From<Convert> for Result<String, String> {
    fn from(v: Convert)->Self {
        Ok(v.0)
    }
}
