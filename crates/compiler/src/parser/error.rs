use crate::{LexError, Span, Token};

#[derive(Debug, PartialEq)]
pub enum EvalError {
    Lexical(LexError),
    UnexpectedToken(Token),
    DuplicateKey(Span),
    MissingField(Box<str>),
    /*TypeError {expected: String, found:String},
    MissingField*/
}
