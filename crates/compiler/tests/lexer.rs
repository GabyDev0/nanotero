use nanotero_compiler::__private::*;

#[test]
fn lex_version_validation() {
    const CODE: &str = "     Tero 0sja\n";
    assert!(CODE.len() < 16, "No estas forzando el uso de bucles");
    let mut lexer = Lexer::new(CODE).expect("It must support this version");
    assert_eq!(lexer.line(), 1, "Debe ser la primera linea");
    assert_eq!(
        lexer.location(),
        Location::zero(),
        "Debe ser la primera linea"
    );
    assert_eq!(
        lexer.prevlocation(),
        Location::zero(),
        "Debe ser la primera linea"
    );
    assert_eq!(
        lexer.next(),
        Some(Ok(Token::Newline)),
        "Deberia generar un salto de linea como unico token"
    );
    assert!(lexer.next().is_none(), "No deberia generar un nuevo token");

    if let Err(n) = Lexer::new("       Tero 1") {
        assert!(
            n == LexError::InvalidVersion,
            "No esta invalidando correctamente la version"
        );
    } else {
        assert!(false, "No esta validando correctamente la version");
    }
}

fn cmp_eq_token_identifier(tk: Token, s: &str, lexer: &Lexer) {
    match tk {
        Token::Identifier(id) => {
            assert_eq!(
                id.as_str(&lexer),
                Some(s),
                "El identificador no coincide con el esperado"
            );
        }
        _ => panic!("Se esperaba un token de identificador"),
    }
}
#[test]
fn lex_identifier_validation() {
    let mut lexer = Lexer::new(
        "NOMBRE JAJA OTRO_NOMBRE_MUY_LARGO _123NOMBRE GabyDev0 # Ignorando linea epicamente\n",
    )
    .expect("It must support this version");
    let expected = [
        "NOMBRE",
        "JAJA",
        "OTRO_NOMBRE_MUY_LARGO",
        "_123NOMBRE",
        "GabyDev0",
    ];
    let mut peekeable = expected.iter().peekable();
    while let Some(exp) = peekeable.next() {
        let token = lexer
            .next()
            .expect("There should be a token")
            .expect("There shouldn't be an error");
        cmp_eq_token_identifier(token, *exp, &lexer);
        if let Some(expr) = peekeable.peek() {
            cmp_eq_token_identifier(
                lexer
                    .peek()
                    .expect("There should be a token")
                    .expect("There shouldn't be an error"),
                *expr,
                &lexer,
            );
        }
    }
    assert_eq!(
        lexer
            .next()
            .expect("There should be a token")
            .expect("There should be a token"),
        Token::Newline,
        "There should be a line break at the end"
    );
}
#[test]
fn lex_numbers_validation() {
    let mut lexer =
        Lexer::new("123 0.456 100_0_00_00_00_05 0.2_").expect("It must support this version");
    let result = [
        Token::Int(123),
        Token::Float(0.456),
        Token::Int(100000000005),
        Token::Float(0.2),
    ];
    for exp in result {
        let token = lexer
            .next()
            .expect("There should be a token")
            .expect("There shouldn't be an error");
        assert_eq!(token, exp);
    }
    assert_eq!(lexer.next(), None, "There shouldn't be any more tokens");
    lexer = Lexer::new("21983108390182390182309182309812031830012830")
        .expect("It must support this version");
    assert_eq!(
        lexer.next(),
        Some(Err(LexError::NumericOverflow)),
        "The number must exceed the limit"
    );
    assert_eq!(
        lexer.next(),
        Some(Err(LexError::NumericOverflow)),
        "the error must be retained"
    );
}

#[test]
fn lex_string_validation() {
    let mut lexer = Lexer::new(
        "'Hola' 'Z_sssHola222A_../aaa' 'SIMD ACTIVATION\\\\' 'Special\\n' 'SIMD ACTIVATIO\\'A\\\\'",
    )
    .expect("It must support this version");
    let result = [
        (false, "Hola"),
        (false, "Z_sssHola222A_../aaa"),
        (true, "SIMD ACTIVATION\\\\"),
        (true, "Special\\n"),
        (true, "SIMD ACTIVATIO\\'A\\\\"),
    ];
    for exp in result {
        let token = lexer
            .next()
            .expect("There should be a token")
            .expect("There shouldn't be an error");
        match token {
            Token::Text(s) => {
                assert_eq!(exp.0, true, "It should be a Text");
                assert_eq!(unsafe { s.as_str_unchecked(&lexer) }, exp.1);
            }
            Token::RawText(s) => {
                assert_eq!(exp.0, false, "It should be RawText");
                assert_eq!(unsafe { s.as_str_unchecked(&lexer) }, exp.1);
            }
            _ => assert!(false, "A string token was expected"),
        }
    }
    assert!(lexer.next().is_none(), "There shouldn't be any more tokens");
}

#[test]
fn lex_string_error() {
    let mut lexer = Lexer::new("'abc").expect("It must support this version");
    assert_eq!(lexer.next(), Some(Err(LexError::UnterminatedString)));

    lexer = Lexer::new("'abc\\").expect("It must support this version");
    assert_eq!(lexer.next(), Some(Err(LexError::UnexpectedEOF)));

    // =========================
    // |       SIMD TEST       |
    // =========================

    lexer = Lexer::new("'SIMD ACTIVATION\n").expect("It must support this version");
    assert_eq!(lexer.next(), Some(Err(LexError::UnterminatedString)));

    lexer = Lexer::new("'SIMD ACTIVATION\\").expect("It must support this version");
    assert_eq!(lexer.next(), Some(Err(LexError::UnexpectedEOF)));
}

#[test]
fn lex_reset_validation() {
    let mut lexer = Lexer::new("Tero c").expect("It must support this version");
    match lexer.next() {
        Some(Ok(Token::Identifier(span))) => {
            assert_eq!(span.as_str(&lexer), Some("Tero"));
        }
        tk => panic!("This token was not expected: {:?}", tk),
    }
}

#[test]
fn lex_unexpected_character() {
    let mut lexer = Lexer::new("ñ").expect("It must support this version");
    assert_eq!(lexer.next(), Some(Err(LexError::InvalidCharacter('ñ'))));
    assert_eq!(lexer.next(), Some(Err(LexError::InvalidCharacter('ñ')))); // 
}

#[test]
fn lex_simple_tokens() {
    let mut lexer = Lexer::new("-:,[]{}").expect("It must support this version");
    let result = [
        Token::Sub,
        Token::Colon,
        Token::Comma,
        Token::LeftBracket,
        Token::RightBracket,
        Token::LeftBrace,
        Token::RightBrace,
    ];
    for exp in &result {
        assert_eq!(lexer.next(), Some(Ok(exp.clone())));
    }
}
