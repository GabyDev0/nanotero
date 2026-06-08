use nanotero_lex::LexError;

#[derive(Debug, PartialEq)]
pub enum LoomError {
    Lexical(LexError),
    UnexpectedEOF,
    UnexpectedToken(String),
    /*TypeError {expected: String, found:String},
    MissingField*/
}
