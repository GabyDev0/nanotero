use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque};
use std::hash::Hash;
use std::mem::{MaybeUninit, transmute_copy};
use nanotero_compiler::*;
pub use nanotero_compiler::__private::{unescape_string};
pub use nanotero_macro::TeroDeserialize;
mod error;
pub use error::*;

pub mod eval {
    pub use nanotero_compiler::*;
}

/// A trait for types that can be reconstructed from a Tero data source.
///
/// This trait is typically implemented automatically using `#[derive(TeroDeserialize)]`
/// on structures. The derive macro inspects the struct's fields at compile time 
/// and generates the state-machine logic required to map `FieldValue` variants 
/// into the corresponding Rust types.
///
/// # How it works with the Derive Macro
/// When you apply `#[derive(TeroDeserialize)]` to a struct, the macro ensures 
/// that each named field in your struct also implements `Deserialize`. During 
/// parsing, the generated code will match the incoming keys from the Tero file, 
/// look up their associated `FieldValue` tokens, and recursively invoke 
/// `deserialize_tero` to populate the structure safely.
pub trait Deserialize: Sized {
/// Deserializes the given `FieldValue` into the target Rust type, 
    /// utilizing the `Lexer` if further stream consumption or context is required.
    ///
    /// # Errors
    /// Returns a `EvalError` if the types mismatch (e.g., expecting an `Int` but 
    /// receiving a `Text` literal) or if required data fields are missing.
    fn deserialize_tero(value: FieldValue, lexer: &mut Lexer) -> Result<Self, EvalError>;
}


/// Represents any valid data type in the Tero configuration format v0.1.
///
/// This enum is the final in-memory data structure returned by the parser.
/// It directly contains primitive values or nested collections extracted
/// from the `.tero` file.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// Intentional absence of a value (corresponds to `nil` in Tero).
    Nil,

    /// A boolean value (`true` or `false`).
    Boolean(bool),

    /// A double-precision floating-point number (`f64`), including `NaN` and `Infinity`.
    Float(f64),

    /// A signed 64-bit integer (`i64`).
    Int(i64),

    /// A UTF-8 encoded string.
    String(String),

    /// An indexed and ordered list of [`Value`] elements.
    Array(Vec<Value>),

    /// An ordered collection of key-value pairs, mapped via a [`BTreeMap`].
    Object(BTreeMap<String, Value>),
}

macro_rules! match_literal {
    ($val:expr, $f:expr) => {
        match $val {
            FieldValue::Literal(tk) => ($f)(tk),
            FieldValue::Array(_) => Err(EvalError::UnexpectedToken(Token::LeftBracket)),
            FieldValue::Block(_) => Err(EvalError::UnexpectedToken(Token::LeftBrace))
        }
    };
}
macro_rules! match_block {
    ($val:expr, $f:expr) => {
        match $val {
            FieldValue::Literal(tk) => Err(EvalError::UnexpectedToken(tk)),
            FieldValue::Array(_) => Err(EvalError::UnexpectedToken(Token::LeftBracket)),
            FieldValue::Block(scope) => ($f)(scope)
        }
    };
}
macro_rules! match_array {
    ($val:expr, $f:expr) => {
        match $val {
            FieldValue::Literal(tk) => Err(EvalError::UnexpectedToken(tk)),
            FieldValue::Array(arr) => ($f)(arr),
            FieldValue::Block(_) => Err(EvalError::UnexpectedToken(Token::LeftBrace))
        }
    };
}
macro_rules! impl_integer {
    ($($t:ty),*) => {
        $(
            impl Deserialize for $t {
                fn deserialize_tero(value: FieldValue, _: &mut Lexer) -> Result<Self, EvalError> {
                    match_literal!(value, |tk| match tk {
                        Token::Int(literal) => {
                            let casted = literal as $t;
                            
                            if casted as i64 != literal {
                                return Err(EvalError::Lexical(LexError::NumericOverflow));
                            }
                            
                            Ok(casted)
                        }
                        _ => Err(EvalError::UnexpectedToken(tk)),
                    })
                }
            }
        )*
    };
}
macro_rules! impl_uinteger {
    ($($t:ty),*) => {
        $(
            impl Deserialize for $t {
                fn deserialize_tero(value: FieldValue, _: &mut Lexer) -> Result<Self, EvalError> {
                    match_literal!(value, |tk| match tk {
                        Token::Int(literal) => {
                            if literal < 0 {
                                return Err(EvalError::Lexical(LexError::NumericOverflow));
                            }
                            let casted = literal as $t;
                            
                            if casted as i64 != literal {
                                return Err(EvalError::Lexical(LexError::NumericOverflow));
                            }
                            
                            Ok(casted)
                        }
                        _ => Err(EvalError::UnexpectedToken(tk)),
                    })
                }
            }
        )*
    };
}

// 🎯 Invocación limpia para todos los enteros con signo pequeños
impl_integer!(i8, i16, i32, i64, i128, isize);
impl_uinteger!(u8, u16, u32, u64, u128, usize);

macro_rules! impl_float {
    ($($t:ty),*) => {
        $(
            impl Deserialize for $t {
                fn deserialize_tero(value: FieldValue, _: &mut Lexer) -> Result<Self, EvalError> {
                    match_literal!(value, |tk| match tk {
                        Token::Int(literal) => Ok(literal as $t),
                        Token::Float(literal) => Ok(literal as $t),
                        
                        _ => Err(EvalError::UnexpectedToken(tk)),
                    })
                }
            }
        )*
    };
}

impl_float!(f32,f64);

impl Deserialize for String {
    fn deserialize_tero(value: FieldValue, lexer: &mut Lexer) -> Result<Self, EvalError> {
        match_literal!(value, |tk| match tk {
            Token::RawText(text) => Ok(text.as_str(lexer).expect("lexer is not the one from FieldValue").to_string() ),
            Token::Text(text) => Ok(unescape_string(text.as_str(lexer).expect("lexer is not the one from FieldValue"))),
            _ => Err(EvalError::UnexpectedToken(tk))
        })
    }
}
impl Deserialize for bool {
    fn deserialize_tero(value: FieldValue, _: &mut Lexer) -> Result<Self, EvalError> {
        match_literal!(value, |tk| match tk {
            Token::Boolean(bl) => Ok(bl),
            _ => Err(EvalError::UnexpectedToken(tk)), 
        })
    }
}

impl<T: Deserialize> Deserialize for Option<T> {
    fn deserialize_tero(value: FieldValue, lexer: &mut Lexer) -> Result<Self, EvalError> {
        match value {
            FieldValue::Literal(Token::Nil) => Ok(None),
            val => Ok(Some(T::deserialize_tero(val, lexer)?))
        }
    }
}
impl<T: Deserialize> Deserialize for Box<T> {
    fn deserialize_tero(value: FieldValue, lexer: &mut Lexer) -> Result<Self, EvalError> {
        Ok(Box::new(T::deserialize_tero(value, lexer)?))
    }
}
impl Deserialize for Box<str> {
    fn deserialize_tero(value: FieldValue, lexer: &mut Lexer) -> Result<Self, EvalError> {
        let string = String::deserialize_tero(value, lexer)?;
        Ok(string.into_boxed_str())
    }
}
impl<'a, T: Deserialize + Clone> Deserialize for Cow<'a, T> {
    fn deserialize_tero(value: FieldValue, lexer: &mut Lexer) -> Result<Self, EvalError> {
        let val = T::deserialize_tero(value, lexer)?;
        Ok(Cow::Owned(val))
    }
}
impl<'a> Deserialize for Cow<'a, str> {
    fn deserialize_tero(value: FieldValue, lexer: &mut Lexer) -> Result<Self, EvalError> {
        let string = String::deserialize_tero(value, lexer)?;
        Ok(Cow::Owned(string))
    }
}
impl<T: Deserialize> Deserialize for BTreeMap<String, T> {
    fn deserialize_tero(value: FieldValue, lexer: &mut Lexer) -> Result<Self, EvalError> {
        match_block!(value, |mut scope: ScopeTracker| {
            let mut map = BTreeMap::new();
            while let Some(field) = scope.next(lexer) {
                let (span, val) = field?.tulpe();
                let strn = span.as_str(lexer).unwrap();
                map.insert(strn.to_string(), T::deserialize_tero(val, lexer)?);
            }
            Ok(map)
        })
    }
}
impl<T: Deserialize> Deserialize for HashMap<String, T> {
    fn deserialize_tero(value: FieldValue, lexer: &mut Lexer) -> Result<Self, EvalError> {
        match_block!(value, |mut scope: ScopeTracker| {
            let mut map = HashMap::new();
            while let Some(field) = scope.next(lexer) {
                let (span, val) = field?.tulpe();
                let strn = span.as_str(lexer).unwrap();
                map.insert(strn.to_string(), T::deserialize_tero(val, lexer)?);
            }
            Ok(map)
        })
    }
}

macro_rules! impl_array {
    ($f:expr, $collection:ty $(,$extra_bounds:path)*) => {
        impl<T: Deserialize $(+ $extra_bounds)*> Deserialize for $collection {
            fn deserialize_tero(value: FieldValue, lexer: &mut Lexer) -> Result<Self, EvalError> {
                match_array!(value, |mut arr: ArrayTracker| {
                    let mut vec = <$collection>::new();
                    while let Some(el) = arr.next(lexer) {
                        ($f)(&mut vec, T::deserialize_tero(el?, lexer)?);
                    }
                    Ok(vec)
                })
            }
        }
        
    };
}
impl_array!(|vec: &mut Vec<T>, t| vec.push(t), Vec<T>);
impl_array!(|set: &mut HashSet<T>, t| { set.insert(t); }, HashSet<T>, Hash,Eq);
impl_array!(|set: &mut BTreeSet<T>, t| { set.insert(t); }, BTreeSet<T>, Ord);
impl_array!(|deque: &mut VecDeque<T>, t| deque.push_back(t), VecDeque<T>);

impl<const SIZE: usize, T: Deserialize> Deserialize for [T; SIZE] {
    fn deserialize_tero(value: FieldValue, lexer: &mut Lexer) -> Result<Self, EvalError> {
        match_array!(value, |mut arr: ArrayTracker| {
            let mut vec = Vec::new();
            while let Some(el) = arr.next(lexer) {
                vec.push(T::deserialize_tero(el?, lexer)?);
            }
            if vec.len() < SIZE {
                return Err(EvalError::MissingField(format!("[{}]", vec.len()).into_boxed_str()));
            }
            let mut arr: [MaybeUninit<T>; SIZE] = unsafe { MaybeUninit::uninit().assume_init() };
            for (i, element) in vec.into_iter().enumerate() {
                if i >= SIZE {
                    break;
                }
                arr[i] = MaybeUninit::new(element);
            }
            Ok(unsafe { transmute_copy::<[MaybeUninit<T>; SIZE], [T; SIZE]>(&arr) })
        })
    }
}

impl Deserialize for Value {
    fn deserialize_tero(value: FieldValue, lexer: &mut Lexer) -> Result<Self, EvalError> {
        match value {
            FieldValue::Array(_) => Ok(Value::Array(Vec::<Value>::deserialize_tero(value, lexer)?)),
            FieldValue::Block(_) => Ok(Value::Object(BTreeMap::deserialize_tero(value, lexer)?)),
            FieldValue::Literal(Token::Boolean(bl)) => Ok(Value::Boolean(bl)),
            FieldValue::Literal(Token::Int(i)) => Ok(Value::Int(i)),
            FieldValue::Literal(Token::Float(f)) => Ok(Value::Float(f)),
            FieldValue::Literal(Token::RawText(_)|Token::Text(_)) => Ok(Value::String(String::deserialize_tero(value, lexer)?)),
            FieldValue::Literal(Token::Nil) => Ok(Value::Nil),
            FieldValue::Literal(tk) => Err(EvalError::UnexpectedToken(tk))
        }
    }
}
/*
macro_rules!  {
    () => {
        
    };
}*/

/// Parses a text string in Tero format and deserializes it into any structure that implements [`Deserialize`] (such as types deriving [`TeroDeserialize`]).
///
/// # Errors
///
/// Returns a [`TeroError`] if the text contains lexical or syntax errors
/// that do not comply with the Tero v0.1 standard.
///
/// # Example
///
/// ```rust
/// 
/// let code = "config { active: true }";
/// let result = nanotero::from_str::<nanotero::Value>(code);
/// assert!(result.is_ok());
/// ```
#[inline(always)]
pub fn from_str<T: Deserialize>(code: &str) -> Result<T, TeroError> {
    let mut lexer = Lexer::new(code).map_err(|_| TeroError::invalid_version())?;
    let scope = ScopeTracker::new(&lexer, false);
    let field = FieldValue::Block(scope);

    T::deserialize_tero(field, &mut lexer).map_err(|err| TeroError::from_eval(err, &lexer))
}