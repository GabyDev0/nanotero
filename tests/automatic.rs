use std::collections::{BTreeMap, HashMap};

use nanotero::{TeroDeserialize, from_str, TeroError};

#[derive(PartialEq, Debug, TeroDeserialize)]
struct Example {
    u1: u8,
    u2: u16,
    u3: u32,
    u4: u64,
    u5: u128,
    u6: usize,
    i1: i8,
    i2: i16,
    i3: i32,
    i4: i64,
    i5: i128,
    i6: isize,
    f1: f32,
    f2: f64,

    s1: String,
    s2: String,
    op: Option<String>,
    op2: Option<String>,

    b: bool
}
impl Example {
    fn new() -> Self{
        Self {
            u1: 255,
            u2: 35000,
            u3: u32::MAX,
            u4: u32::MAX as u64,
            u5: 0,
            u6: 0,
            i1: -1,
            i2: -126,
            i3: 0,
            i4: -45000,
            i5: 100,
            i6: 12,
            f1: 0.0,
            f2: 0.0,
            s1: String::from("Hello World"),
            s2: String::from("\tHello\nWorld"),
            op: None,
            op2: None,
            b: false
        }
    }
    fn code() -> String{
        let n = Self::new();
        format!(
            "i1: {}\ni2: {},i3: {},i4: {}\ni5: {},i6: {},u1: {}\nu2: {},u3: {},u4: {}\nu5: {},u6: {}, \
            f1: {},f2: {}\ns1: {:?},\ns2: {:?},\n\nop: nil\nb: {}\n\n",
            n.i1, n.i2, n.i3, n.i4, n.i5, n.i6,
            n.u1, n.u2, n.u3, n.u4, n.u5, n.u6,
            n.f1, n.f2, n.s1, n.s2, n.b
        )
    }
}

#[derive(Debug, TeroDeserialize)]
pub struct OverFlow {
    #[tero(name = "v")]
    _v: u8,

    #[tero(name = "i")]
    _i: i8
}

#[test]
fn primitive_type() -> Result<(), TeroError>{
    let exp = Example::new();
    let code = Example::code();
    let result = from_str(&code)?;
    assert_eq!(exp, result); 
    Ok(())
}
#[test]
fn btree_map() -> Result<(), TeroError> {
    let mut exp = BTreeMap::new();
    exp.insert(String::from("hey"), Some(12));
    exp.insert(String::from("hey2"), Some(25));
    exp.insert(String::from("hey3"), Some(-5));
    exp.insert(String::from("hey4"), None);
    let result = from_str("hey: 12, hey2: 25\n hey3: -5, hey4: nil")?;
    assert_eq!(exp, result);
    Ok(())
}
#[test]
fn hash_map() -> Result<(), TeroError> {
    let mut exp = HashMap::new();
    exp.insert(String::from("hey"), Some(12));
    exp.insert(String::from("hey2"), Some(25));
    exp.insert(String::from("hey3"), Some(-5));
    exp.insert(String::from("hey4"), None);
    let result = from_str("hey: 12, hey2: 25\n hey3: -5, hey4: nil")?;
    assert_eq!(exp, result);
    Ok(())
}


#[test]
fn overflow() {
    match from_str::<OverFlow>("v: 256, i: 0") {
        Err(TeroError::NumericOverflow { found: _ }) => {},
        tk => panic!("Unexpected: {:?}", tk)
    }
    match from_str::<OverFlow>("v: -1, i: 0") {
        Err(TeroError::NumericOverflow { found: _ }) => {},
        tk => panic!("Unexpected: {:?}", tk)
    }
    match from_str::<OverFlow>("v: 0, i: 128") {
        Err(TeroError::NumericOverflow { found: _ }) => {},
        tk => panic!("Unexpected: {:?}", tk)
    }
    match from_str::<OverFlow>("v: 0, i: -129") {
        Err(TeroError::NumericOverflow { found: _ }) => {},
        tk => panic!("Unexpected: {:?}", tk)
    }
}

#[derive(Debug, TeroDeserialize)]
struct TestVecAndFix {
    v: Vec<[u8; 3]>
}
#[test]
fn test_vec() -> Result<(), TeroError> {
    let result = from_str::<TestVecAndFix>("v: [[1, 2, 3], [4, 5, 6]]")?;
    assert_eq!(result.v, vec![[1, 2, 3], [4, 5, 6]]);
    Ok(())
}

#[test]
fn found_error() {
    match from_str::<OverFlow>("v: -1, i: -1") {
        Err(TeroError::NumericOverflow { found: result }) => {
            assert_eq!(&result, "1 | v: -1, i: -1\n        ^ Numeric overflow");
        },
        r => panic!("unexpected value: {:?}", r)
    }
    match from_str::<OverFlow>("v: 1, v: 1") {
        Err(TeroError::DuplicateKey { found: result }) => {
            assert_eq!(&result, "1 | v: 1, v: 1\n            ^ Duplicate key");
        }
        r => panic!("unexpected value: {:?}", r)
    }
    match from_str::<OverFlow>("v: '") {
        Err(TeroError::UnterminatedString { found: result }) => {
            assert_eq!(&result, "1 | v: '\n        ^ the string end is missing");
        }
        r => panic!("unexpected value: {:?}", r)
    }
    match from_str::<OverFlow>("v: ") {
        Err(TeroError::UnexpectedEOF { found: result }) => {
            assert_eq!(&result, "1 | v: \n        ^ The file ended abruptly.");
        }
        r => panic!("unexpected value: {:?}", r)
    }
    match from_str::<OverFlow>("v: ñ") {
        Err(TeroError::InvalidCharacter { found: result }) => {
            assert_eq!(&result, "1 | v: ñ\n        ^ Invalid character: 'ñ'");
        }
        r => panic!("unexpected value: {:?}", r)
    }
    match from_str::<OverFlow>("v: 'hello'") {
        Err(TeroError::UnexpectedToken { found: result }) => {
            assert_eq!(&result, "1 | v: 'hello'\n        ^ Unexpected token: RawText(Span { start: Location(4), end: 9, line: 1 })");
        }
        r => panic!("unexpected value: {:?}", r)
    }
    match from_str::<OverFlow>("v: 12") {
        Err(TeroError::MissingField { found: result }) => {
            assert_eq!(&result, "1 | v: 12\n          ^ Missing field: \"i\"");
        }
        r => panic!("unexpected value: {:?}", r)
    }
    match from_str::<OverFlow>("Tero 1") {
        Err(TeroError::InvalidVersion) => {
        }
        r => panic!("unexpected value: {:?}", r)
    }
}
