mod ast;
mod lexer;

pub use lexer::*;
pub use ast::*;

#[cfg(test)]
mod tests {
    use crate::ast::{Literal, LoomValue, Token};
    use crate::lexer::Lexer;

use super::*;

    #[test]
    fn version_validation() {
        const CODE: &str = "     TERO 0sja\n";
        assert!(CODE.len() < 16, "No estas forzando el uso de bucles");
        let mut lexer = Lexer::new(
            CODE
        ).expect("Debe soportar esa version");
        
        assert!(lexer.next().is_none(), "No deberia generar un nuevo token");

        if let Err(n) = Lexer::new(
            "       TERO 1"
        ) {
            assert!(n == LexError::InvalidVersion, "No esta invalidando correctamente la version");
        }else{
            assert!(false, "No esta validando correctamente la version");
        }
    }
    #[test]
    fn next() {
        const CODE: &str = 
        "      TERO 0asdasd \n\
        NOMBRE: 12 - 13.5 - 'hola' false true NaN nil Infinity";
        const RESULT: [Token<'static>; 12] = [
            Token::Identifier("NOMBRE"), 
            Token::Colon, 
            Token::Value(LoomValue::Raw(Literal::Integer(12))),
            Token::Sub,
            Token::Value(LoomValue::Raw(Literal::Float(13.5))),
            Token::Sub,
            Token::Value(LoomValue::Raw(Literal::RawText("hola'"))),
            Token::Value(LoomValue::Raw(Literal::Boolean(false))),
            Token::Value(LoomValue::Raw(Literal::Boolean(true))),
            Token::Value(LoomValue::Raw(Literal::NaN)),
            Token::Value(LoomValue::Raw(Literal::Nil)),
            Token::Value(LoomValue::Raw(Literal::Infinity)),            
        ];
        let mut lexer = Lexer::new(CODE).expect("Debe soportar esa version");
        for cmptoken in RESULT.iter() {
            let result = lexer.next().expect("Deberia haber token");
            match result {
                Ok(token) => {
                    assert_eq!(token, cmptoken.clone(), "Se esperaba otro token")
                },
                Err(n) => assert!(false, "Error: {:?}", n)
            }
        }
        assert!(lexer.next().is_none(), "Hay otro token al final");
    }

    fn string_validation(cadena: &str) {
        let mut lexer = Lexer::new(cadena).expect("Fallo al crear el lexer");
        let token = lexer.next().expect("debio de crear un token").expect("No debio de encontrar un fallo");
        if let Token::Value(LoomValue::Raw(v)) = token.clone() {
            match v {
                Literal::RawText(c)|Literal::Text(c) => {
                    assert_eq!(c, &cadena[1..], "Cadenas diferentes");
                }
                _=> {
                    assert!(false, "Token incorrecto {:?}", token);
                }
            }
        }else{
            assert!(false, "Token incorrecto {:?}", token);
        }
    }
    #[test]
    fn simple_string_validation() {
        // Verifica si las operaciones SIMD encuentran el limite del string
        string_validation("\"CHAR CONTAINT POSTERIOR 16BYTES\"");

        // Verifica si el bucle char-char encuentra el limite del string
        string_validation("\"CHAR CONTAINT POSTERIOR 17BYTES \"");

        // fuerza solo el uso del bucle char-char
        string_validation("\"WITHOUT SIMD\"");

        // Verifica si el bucle char-char detecta las \
        string_validation(r##""USING COMPLEX STRING\\""##);

        // Verifica si el SIMD detecta las \ 
        // Verifica si el SIMD, omite un caracter si el ultimo caracter del SIMD anterior termina en \
        string_validation(r##""USING COMPLEX STRING\\\\\\\\\\\\\"""##);
        
        // Uso mas complejo, Verifica que ignore las comillas con \
        string_validation(r##""USING\n\" COMPLEX STRING FOR1\"""##);
    }

    #[test]
    fn integral_line() {

        let mut lexer = Lexer::new("\n\n\nA\n\n\nB\n").unwrap();
        
        let a = lexer.next_skip_ln().expect("Deberia haber token").expect("No deberia haber error");
        assert_eq!(lexer.line(), 4, "Error en los saltos de lineas");
        assert_eq!(a, Token::Identifier("A"));
        
        let b = lexer.next_skip_ln().expect("Deberia haber token").expect("No deberia haber error");
        assert_eq!(lexer.line(), 7, "Error en los saltos de lineas");
        assert_eq!(b, Token::Identifier("B"));
        
        let n = lexer.next().expect("Deberia haber token").expect("No deberia haber error");
        assert_eq!(lexer.line(), 8, "Error en los saltos de lineas");
        assert_eq!(n, Token::Newline, "No es un salto de linea");
    }
    #[test]
    fn number_simd() {

        /// Verifica que SIMD + bucle char char funcione bien
        /// los f64 y i64 se extraen de la misma funcion, asi que con validar i64 se valida que funcione bien el f64
        const NUMBER: i64 = 638_419_205_746_139_852;
        let strn = format!("{}", NUMBER);
        let mut lexer = Lexer::new(&strn).unwrap();
        match lexer.next() {
            Some(Ok(Token::Value(LoomValue::Raw(Literal::Integer(v))))) => {
                assert_eq!(v, NUMBER, "Valor incorrecto");
            },
            _ => {
                assert!(false, "El valor deberia ser un numero");
            }
        }
        
    }
}
