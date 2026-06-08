use std::collections::BTreeMap;

use lex::{Lexer, Literal, LoomValue, Token};

use crate::values::Value;

mod error;
pub use error::LoomError;
mod text_format;
pub(crate) use text_format::unescape_string;

#[inline(always)]
pub fn parser_string(code: &str) -> Result<Value, LoomError> {
    match Lexer::new(code) {
        Ok(mut lexer) => parser_block_lex(&mut lexer, false),
        Err(err) => Err(LoomError::Lexical(err)),
    }
}

fn parser_block_lex(lex: &mut Lexer, closable: bool) -> Result<Value, LoomError> {
    let mut btree = BTreeMap::new();
    loop {
        match lex.next_skip_ln() {
            // Ignorar cualquier salto de de linea
            Some(Ok(token)) => {
                match token {
                    Token::RightBrace => {
                        // si es un '}' cerramos terminamos solo si closable es true
                        if !closable {
                            return Err(LoomError::UnexpectedToken(String::from('}')));
                        }
                        break;
                    }
                    Token::Comma | Token::Newline => {} // Ignorar Tokens

                    // Si encontramos un identificador creamos un par clave-valor
                    Token::Identifier(ident) => {
                        let value = parser_individual(lex, false)?;
                        btree.insert(String::from(ident), value);
                    }
                    // Cualquier otro token Esta fuera de lugar
                    _ => {
                        return Err(LoomError::UnexpectedToken(token.to_string()));
                    }
                }
            }
            // Devolver los errores del lexer
            Some(Err(err)) => return Err(LoomError::Lexical(err)),

            None => {
                // Termino antes de tiempo?
                if closable {
                    return Err(LoomError::UnexpectedEOF);
                }
                // Si llega aqui retorna con lo que llevaba
                break;
            }
        }
    }
    Ok(Value::Object(btree))
}

fn parser_individual(lex: &mut Lexer, mut values: bool) -> Result<Value, LoomError> {
    loop {
        match lex.next() {
            Some(Ok(token)) => {
                match token {
                    // El colon permite asignar valores primitivos fuera de objetos y array
                    Token::Colon => {
                        if values {
                            // si es true o no permite colon, o ya hubo uno antes que este
                            return Err(LoomError::UnexpectedToken(String::from(":")));
                        }
                        values = true;
                    }
                    // independientemente de values
                    Token::LeftBrace => return parser_block_lex(lex, true),
                    Token::LeftBracket => return parser_array(lex),

                    Token::Sub => {
                        // Solo se permite - si values es true
                        if !values {
                            return Err(LoomError::UnexpectedToken(String::from("-")));
                        }
                        // solo Int y Float permiten el uso de SUB
                        return match lex.next() {
                            Some(Ok(Token::Value(LoomValue::Raw(Literal::Integer(i))))) => {
                                Ok(Value::Integer(-i))
                            }
                            Some(Ok(Token::Value(LoomValue::Raw(Literal::Infinity)))) => {
                                Ok(Value::Float(-f64::INFINITY))
                            }
                            Some(Ok(Token::Value(LoomValue::Raw(Literal::Float(f))))) => {
                                Ok(Value::Float(-f))
                            }
                            Some(Ok(token)) => Err(LoomError::UnexpectedToken(token.to_string())),
                            Some(Err(err)) => Err(LoomError::Lexical(err)),
                            None => Err(LoomError::UnexpectedEOF),
                        };
                    }
                    Token::Value(value) => {
                        if !values {
                            return Err(LoomError::UnexpectedToken(value.to_string()));
                        }
                        return Ok(match value {
                            LoomValue::Raw(Literal::NaN) => Value::Float(f64::NAN),
                            LoomValue::Raw(Literal::Nil) => Value::Nil,
                            LoomValue::Raw(Literal::Infinity) => Value::Float(f64::INFINITY),
                            LoomValue::Raw(Literal::Boolean(b)) => Value::Boolean(b),
                            LoomValue::Raw(Literal::Integer(i)) => Value::Integer(i),
                            LoomValue::Raw(Literal::Float(f)) => Value::Float(f),
                            LoomValue::Raw(Literal::RawText(text)) => {
                                Value::String(String::from(&text[..text.len() - 1]))
                            }
                            LoomValue::Raw(Literal::Text(text)) => {
                                Value::String(unescape_string(text))
                            }
                        });
                    }
                    _ => return Err(LoomError::UnexpectedToken(token.to_string())),
                }
            }
            Some(Err(err)) => return Err(LoomError::Lexical(err)),

            None => return Err(LoomError::UnexpectedEOF),
        }
    }
}

fn parser_array(lex: &mut Lexer) -> Result<Value, LoomError> {
    let mut values = Vec::new();
    loop {
        match lex.peek() {
            Some(Ok(Token::RightBracket)) => {
                lex.next();
                break;
            }
            Some(Ok(Token::Newline | Token::Comma)) => {
                lex.next();
            }
            Some(Ok(_)) => {
                values.push(parser_individual(lex, true)?);
                match lex.peek() {
                    Some(Ok(Token::Comma | Token::Newline)) => {
                        lex.next();
                        continue;
                    }
                    Some(Ok(Token::RightBracket)) => {
                        lex.next();
                        break;
                    }
                    Some(Ok(token)) => return Err(LoomError::UnexpectedToken(token.to_string())),

                    _ => continue,
                }
            }
            Some(Err(err)) => return Err(LoomError::Lexical(err)),
            None => return Err(LoomError::UnexpectedEOF),
        }
    }
    Ok(Value::Array(values))
}
