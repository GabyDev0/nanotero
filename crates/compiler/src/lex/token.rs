use crate::lex::Span;

/// Critical errors that can occur during the lexical analysis (lexing) phase.
///
/// These errors represent failures at the byte and character level before the
/// token stream reaches the parser.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum LexError {
    /// The `.tero` file version header or directive is invalid or not supported.
    InvalidVersion,

    /// An unexpected character was encountered that does not belong to the language grammar.
    InvalidCharacter(char),

    /// A string literal was left open (missing closing quote) before the end of the file or line.
    UnterminatedString,

    /// An unexpected end-of-file was reached before the token could be completed (e.g., while reading a multiline string).
    UnexpectedEOF,

    /// A numeric literal (integer or float) contained digits that overflow the maximum representable value.
    NumericOverflow,
}

/// Represents the individual lexical units (tokens) emitted by the lexer.
///
/// This enum maps syntax control characters to slices of the original text
/// (`&'a str`) to avoid unnecessary memory allocations.
#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    /// A primitive value already processed by the lexer (numbers, booleans, strings, etc.).
    Int(i64),

    /// A floating-point number, which may include a decimal point and/or an exponent.
    Float(f64),

    /// A raw text string, which is taken directly from the source without processing escape sequences.
    RawText(Span),

    /// A text string, which may contain escape sequences (e.g., `\n`, `\"`, etc.).
    Text(Span),
    /// A boolean value, representing either `true` or `false`.
    Boolean(bool),

    /// Represents a `nil` value, indicating the absence of a value or a null state.
    Nil,

    /// A key name or identifier that points to a section of source code.
    Identifier(Span),

    /* Control Characters */
    /// Colon (`:`), used to separate keys from their values.
    Colon,

    /// Comma (`,`), used as an optional delimiter or element separator.
    Comma,

    /// Left brace (`{`), indicates the beginning of an object block.
    LeftBrace,

    /// Right brace (`}`), indicates the end of an object block.
    RightBrace,

    /// Left bracket (`[`), indicates the beginning of an array or list.
    LeftBracket,

    /// Right bracket (`]`), indicates the end of an array or list.
    RightBracket,

    /// Newline (`\n`), used for implicit statement separation.
    Newline,

    /// Subtraction operator or minus sign (`-`), used to denote negative numbers.
    Sub,
}
