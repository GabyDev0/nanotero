use nanotero_compiler::{EvalError, LexError, Lexer, Location, Span};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TeroError {
    #[error("The file ended abruptly. \n{found}")]
    UnexpectedEOF { found: String },

    #[error("Invalid character. \n{found}")]
    InvalidCharacter { found: String },

    #[error("Unterminated String. \n{found}")]
    UnterminatedString { found: String },
    
    #[error("Numeric Overfloow. \n{found}")]
    NumericOverflow { found: String },

    #[error("Duplicate key. \n{found}")]
    DuplicateKey { found: String },

    #[error("Unexpected Token. \n{found}")]
    UnexpectedToken { found: String },

    #[error("Missing field. \n{found}")]
    MissingField { found: String },

    #[error("supports version 0")]
    InvalidVersion,
}


#[inline(always)]
fn get_string_err(lexer: &Lexer, loc: Location, line: u32, first: u32) -> String {
    let (left, mut string) = lexer.gen_error(loc, line).unwrap();
    string.push('\n');
    for _ in first..left {
        string.push(' ');
    }
    string.push('^');
    string
}

#[inline(always)]
fn get_string_err_with_span(lexer: &Lexer, span: Span, first: u32) -> String {
    get_string_err(lexer, span.get_start(), span.line(), first)
}

impl TeroError {
    pub fn from_lexical(value: LexError, lexer: &Lexer) -> Self {
        match value {
            LexError::InvalidVersion => {
                Self::InvalidVersion
            }
            LexError::UnexpectedEOF => {
                let mut string = get_string_err(lexer, lexer.location(), lexer.line(),0);
                string.push_str(" The file ended abruptly.");
                Self::UnexpectedEOF { found: string }
            }
            LexError::InvalidCharacter(ch) => {
                let mut string = get_string_err(lexer, lexer.location(), lexer.line(),0);
                
                string.push_str(" Invalid character: '");
                string.push(ch);
                string.push_str("'");

                Self::InvalidCharacter { found: string }
            }
            LexError::UnterminatedString => {
                let mut string = get_string_err(lexer, lexer.location(), lexer.line(),0);
                string.push_str(" the string end is missing");

                Self::UnterminatedString { found: string }
            }
            LexError::NumericOverflow => {
                let mut string = get_string_err(lexer, lexer.prevlocation(), lexer.line(),0);
                string.push_str(" Numeric overflow");
                Self::NumericOverflow { found: string }
            }
        }
    }
    pub fn from_eval(value: EvalError, lexer: &Lexer) -> Self {
        match value {
            EvalError::Lexical(lex) => Self::from_lexical(lex, lexer),
            EvalError::DuplicateKey(span) => {
                let mut string = get_string_err_with_span(lexer, span, 0);
                string.push_str(" Duplicate key");
                Self::DuplicateKey { found: string }
            }
            EvalError::UnexpectedToken(tk) => {
                let mut string = get_string_err(lexer, lexer.prevlocation(), lexer.line(), 0);
                string.push_str(&format!(" Unexpected token: {:?}", tk));
                Self::UnexpectedToken { found: string }
            }
            EvalError::MissingField(field) => {
                let mut string = get_string_err(lexer, lexer.prevlocation(), lexer.line(), 0);
                string.push_str(&format!(" Missing field: {:?}", field));
                Self::MissingField { found: string }
            }
        }
    }
    pub fn invalid_version() -> Self {
        Self::InvalidVersion
    }
}
