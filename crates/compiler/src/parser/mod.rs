use crate::{
    LexError,
    lex::{Lexer, Location, Span, Token},
};

mod error;
pub use error::EvalError;
mod text_format;
pub use text_format::unescape_string;

#[derive(Debug, PartialEq)]
/// Tracks the parsing state of a Tero object block.
///
/// `ScopeTracker` records whether the object has a closing brace, whether
/// closure has already been reached, and the location of its opening token.
pub struct ScopeTracker {
    has_closed: bool,
    closed: bool,
    location: Location,
}
#[derive(Debug, PartialEq)]
/// Tracks the parsing state of a Tero array.
///
/// `ArrayTracker` keeps the current position inside the array, whether the
/// array has ended, and whether a value is expected next.
pub struct ArrayTracker {
    location: Location,
    closed: bool,
    has_value: bool,
}

#[derive(Debug, PartialEq)]
pub enum FieldValue {
    Literal(Token),
    Block(ScopeTracker),
    Array(ArrayTracker),
}

#[derive(Debug, PartialEq)]
/// Represents a parsed key-value entry in a Tero object.
///
/// A `Field` contains the original key span and its associated value,
/// which may be a literal, nested object, or nested array.
pub struct Field {
    key: Span,
    value: FieldValue,
    /* value_location: Location */
}

impl ScopeTracker {
    #[inline]
    pub fn new(lex: &Lexer, has_closed: bool) -> Self {
        let mut location = lex.location();
        location.retreat(has_closed as u32);
        Self {
            has_closed,
            closed: false,
            location,
        }
    }
    #[inline(always)]
    fn extract_field(&mut self, lexer: &mut Lexer) -> Result<FieldValue, EvalError> {
        let mut is_colon = false;
        loop {
            match lexer.next_skip_ln() {
                Some(Ok(Token::Colon)) => {
                    if is_colon {
                        return Err(EvalError::UnexpectedToken(Token::Colon));
                    }
                    is_colon = true
                }
                Some(Err(err)) => return Err(EvalError::Lexical(err)),
                Some(Ok(Token::Sub)) => {
                    if !is_colon {
                        return Err(EvalError::UnexpectedToken(Token::Sub));
                    }
                    let prev = lexer.prevlocation();
                    // Only Int and Float allow the use of SUB
                    let r = match lexer.next() {
                        Some(Ok(Token::Int(i))) => Ok(FieldValue::Literal(Token::Int(-i))),
                        Some(Ok(Token::Float(f))) => Ok(FieldValue::Literal(Token::Float(-f))),
                        Some(Ok(token)) => Err(EvalError::UnexpectedToken(token)),
                        Some(Err(err)) => Err(EvalError::Lexical(err)),
                        None => Err(EvalError::Lexical(LexError::UnexpectedEOF)),
                    };
                    if r.is_ok() {
                        lexer.prev_token = prev;
                    }
                    return r;
                }
                Some(Ok(Token::LeftBrace)) => {
                    let mut location = Location::new(&lexer.src);
                    location.retreat(1);
                    return Ok(FieldValue::Block(ScopeTracker {
                        has_closed: true,
                        closed: false,
                        location,
                    }));
                }
                Some(Ok(Token::LeftBracket)) => {
                    let array = ArrayTracker::new(lexer);
                    return Ok(FieldValue::Array(array));
                }

                Some(Ok(tk)) => {
                    return match tk {
                        Token::Int(_)
                        | Token::Float(_)
                        | Token::Boolean(_)
                        | Token::RawText(_)
                        | Token::Text(_)
                        | Token::Nil => {
                            if !is_colon {
                                return Err(EvalError::UnexpectedToken(tk));
                            }
                            Ok(FieldValue::Literal(tk))
                        }
                        _ => Err(EvalError::UnexpectedToken(tk)),
                    };
                }
                None => return Err(EvalError::Lexical(LexError::UnexpectedEOF)),
            }
        }
    }

    /// Advances the object parser to the next field.
    ///
    /// This is the public parser entry point for reading object fields from a
    /// Tero scope. It returns `Some(Ok(Field))` while fields are available,
    /// `Some(Err(_))` on parse failure, and `None` after the object closes.
    pub fn next(&mut self, lexer: &mut Lexer) -> Option<Result<Field, EvalError>> {
        if self.closed {
            return None;
        }
        loop {
            match lexer.next_skip_ln() {
                Some(Ok(Token::Identifier(span))) => match self.extract_field(lexer) {
                    Err(err) => return Some(Err(err)),
                    Ok(val) => {
                        return Some(Ok(Field {
                            key: span,
                            value: val,
                        }));
                    }
                },
                Some(Ok(Token::Comma | Token::Newline)) => {}
                Some(Ok(Token::RightBrace)) => {
                    if !self.has_closed {
                        return Some(Err(EvalError::UnexpectedToken(Token::RightBrace)));
                    }
                    self.closed = true;
                    return None;
                }
                Some(Err(err)) => return Some(Err(EvalError::Lexical(err))),
                Some(Ok(tk)) => return Some(Err(EvalError::UnexpectedToken(tk))),
                None => {
                    if self.has_closed {
                        return Some(Err(EvalError::Lexical(LexError::UnexpectedEOF)));
                    }
                    return None;
                }
            }
        }
    }
}

impl ArrayTracker {
    pub fn new(lexer: &Lexer) -> Self {
        let mut location = lexer.location();
        location.retreat(1);
        Self {
            location,
            closed: false,
            has_value: true,
        }
    }

    /// Advances the array parser to the next element.
    ///
    /// This is the public parser entry point for reading array elements from a
    /// Tero array. It returns `Some(Ok(FieldValue))` while values are available,
    /// `Some(Err(_))` on parse failure, and `None` after the array closes.
    pub fn next(&mut self, lexer: &mut Lexer) -> Option<Result<FieldValue, EvalError>> {
        if self.closed {
            return None;
        }
        loop {
            match lexer.next() {
                Some(Ok(Token::Comma | Token::Newline)) => {
                    self.has_value = true;
                }
                Some(Ok(Token::LeftBrace)) => {
                    if !self.has_value {
                        return Some(Err(EvalError::UnexpectedToken(Token::LeftBrace)));
                    }
                    self.has_value = false;
                    let sub_obj = ScopeTracker::new(lexer, true);
                    return Some(Ok(FieldValue::Block(sub_obj)));
                }
                Some(Ok(Token::LeftBracket)) => {
                    if !self.has_value {
                        return Some(Err(EvalError::UnexpectedToken(Token::LeftBracket)));
                    }
                    self.has_value = false;
                    let sub_arr = ArrayTracker::new(lexer);
                    return Some(Ok(FieldValue::Array(sub_arr)));
                }
                Some(Ok(Token::RightBracket)) => {
                    self.closed = true;
                    return None;
                }
                Some(Ok(Token::Sub)) => {
                    let prev = lexer.prevlocation();
                    if !self.has_value {
                        return Some(Err(EvalError::UnexpectedToken(Token::Sub)));
                    }
                    self.has_value = false;
                    // Only Int and Float allow the use of SUB
                    let r = Some(match lexer.next() {
                        Some(Ok(Token::Int(i))) => Ok(FieldValue::Literal(Token::Int(-i))),
                        Some(Ok(Token::Float(f))) => Ok(FieldValue::Literal(Token::Float(-f))),
                        Some(Ok(token)) => Err(EvalError::UnexpectedToken(token)),
                        Some(Err(err)) => Err(EvalError::Lexical(err)),
                        None => Err(EvalError::Lexical(LexError::UnexpectedEOF)),
                    });
                    if r.is_some() {
                        lexer.prev_token = prev;
                    }
                    return r;
                }
                Some(Ok(tk)) => match tk {
                    Token::Int(_)
                    | Token::Float(_)
                    | Token::Boolean(_)
                    | Token::Nil
                    | Token::Text(_)
                    | Token::RawText(_) => {
                        if !self.has_value {
                            return Some(Err(EvalError::UnexpectedToken(tk)));
                        }
                        self.has_value = false;
                        return Some(Ok(FieldValue::Literal(tk)));
                    }
                    _ => return Some(Err(EvalError::UnexpectedToken(tk))),
                },
                Some(Err(err)) => return Some(Err(EvalError::Lexical(err))),
                None => return Some(Err(EvalError::Lexical(LexError::UnexpectedEOF))),
            }
        }
    }
}

impl Field {
    pub fn span(&self) -> Span {
        self.key
    }
    pub fn val(self) -> FieldValue {
        self.value
    }
    pub fn get_name<'a>(&self, lex: &'a Lexer) -> Option<&'a str> {
        self.key.as_str(lex)
    }
    pub fn get_val(&self) -> &FieldValue {
        &self.value
    }
    pub fn tulpe(self) -> (Span, FieldValue) {
        (self.key, self.value)
    }
}
impl FieldValue {
    pub fn as_token(&self) -> Token {
        match self {
            FieldValue::Array(_) => Token::LeftBracket,
            FieldValue::Block(_) => Token::LeftBrace,
            FieldValue::Literal(tk) => tk.clone(),
        }
    }
}
