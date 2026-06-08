mod parser;
mod values;
pub use parser::*;
pub use values::*;
#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use nanotero_lex::LexError;

    use crate::{
        LoomError,
        Value::{self},
        parser::parser_string,
        unescape_string,
    };
    #[test]
    fn unescape() {
        let fmrt1 = r#"Hello\n\tComer\"""#;
        let rs1 = "Hello\n\tComer\"";
        assert_eq!(
            rs1,
            unescape_string(fmrt1),
            "Funciona mal el unescape_string"
        );
    }

    fn cmp(code: &str, btree: BTreeMap<String, Value>) -> BTreeMap<String, Value> {
        let val = Value::Object(btree);
        let r = parser_string(code).expect("Error");
        assert_eq!(val, r, "No son iguale");
        match val {
            Value::Object(b) => b,
            _ => {
                assert!(false, "Deberia ser Objeto");
                panic!("Deberia ser Objeto");
            }
        }
    }

    #[test]
    fn test_parser() {
        let code1 = "ENTERO: 12, FLOTANTE: -12.3 # Usando comentarios\n CADENABRUTA: \"Hello World\"#Otro comentario para verificar que funciona\n CADENA_COMPLEJA: \"Hello\\tWorld\\nI'm GabyDev0\\nBye\"";
        let mut raiz = BTreeMap::new();
        raiz.insert("ENTERO".to_string(), Value::Integer(12));
        raiz.insert("FLOTANTE".to_string(), Value::Float(-12.3));
        raiz.insert(
            "CADENABRUTA".to_string(),
            Value::String(String::from("Hello World")),
        );
        raiz.insert(
            "CADENA_COMPLEJA".to_string(),
            Value::String(String::from("Hello\tWorld\nI'm GabyDev0\nBye")),
        );
        raiz = cmp(code1, raiz);

        let code2 = "ENTERO: 12\nFLOTANTE: -12.3, CADENABRUTA: \"Hello World\"\n Array[1,2 # Testeo con un comentario\n3]";
        let vec = vec![Value::Integer(1), Value::Integer(2), Value::Integer(3)];
        raiz.remove("CADENA_COMPLEJA");
        raiz.insert("Array".to_string(), Value::Array(vec));

        let obj = cmp(code2, raiz);
        raiz = BTreeMap::new();
        raiz.insert("LOCO".to_string(), Value::Boolean(true));
        raiz.insert("NULL".to_string(), Value::Nil);
        raiz.insert("Inf".to_string(), Value::Float(f64::INFINITY));
        raiz.insert("nah".to_string(), Value::Boolean(false));
        raiz.insert("OBJ".to_string(), Value::Object(obj));

        let code3 = "LOCO: true #explicando\nNULL: nil,Inf: Infinity\nnah:false \
                        OBJ{ \
                            ENTERO: 12, FLOTANTE: -12.3,# Comentarios se practica\n \
                            CADENABRUTA: \"Hello World\" \
                            Array[1 #Comentarios interesantes\n \
                                2,3\n] \
                        }";
        cmp(code3, raiz);
    }

    #[test]
    fn test_error() {
        // 1. Falta cerrar llave
        assert_eq!(
            parser_string("OBJ { ENTERO: 12"),
            Err(LoomError::UnexpectedEOF)
        );

        // 2. Falta cerrar corchete
        assert_eq!(
            parser_string("Array [1, 2, 3"),
            Err(LoomError::UnexpectedEOF)
        );

        // 3. Dos puntos duplicados (vulnerabilidad en tu loop anterior)
        assert_eq!(
            parser_string("ENTERO:: 12"),
            Err(LoomError::UnexpectedToken(String::from(":")))
        );

        // 4. Token inesperado en la raíz
        assert_eq!(
            parser_string("1234"),
            Err(LoomError::UnexpectedToken(String::from("1234")))
        );

        // 5. Token inesperado en anidacion
        assert_eq!(
            parser_string("OBJ { 12"),
            Err(LoomError::UnexpectedToken(String::from("12")))
        );

        // 6. Caracter no esperado
        assert_eq!(
            parser_string("Num1: 12, Num2: ñ"),
            Err(LoomError::Lexical(LexError::InvalidCharacter('ñ')))
        );
    }
}
