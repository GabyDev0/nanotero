use nanotero_compiler::__private::{LexError, Lexer, Token};
use nanotero_compiler::{ArrayTracker, EvalError, FieldValue, ScopeTracker};

#[test]
fn parser_field() {
    let mut lexer = Lexer::new("Ta: 2\n Te: -1,Tu: false, r: true, P: Infinity, r: NaN")
        .expect("It must support this version");
    let mut scptracker = ScopeTracker::new(&mut lexer, false);
    let result = [
        ("Ta", FieldValue::Literal(Token::Int(2))),
        ("Te", FieldValue::Literal(Token::Int(-1))),
        ("Tu", FieldValue::Literal(Token::Boolean(false))),
        ("r", FieldValue::Literal(Token::Boolean(true))),
        ("P", FieldValue::Literal(Token::Float(f64::INFINITY))),
    ];
    for exp in result {
        let field = scptracker
            .next(&mut lexer)
            .expect("There should be a Field")
            .expect("There should be a Field");
        assert_eq!(field.get_name(&lexer), Some(exp.0), "No match to exp");
        assert_eq!(*field.get_val(), exp.1, "No match to exp");
        let (name, val) = field.tulpe();
        assert_eq!(
            unsafe { name.as_str_unchecked(&lexer) },
            exp.0,
            "No match to exp"
        );
        assert_eq!(val, exp.1, "No match to exp");
    }
    let nan = scptracker
        .next(&mut lexer)
        .expect("There should be a Field")
        .expect("There should be a Field")
        .val();
    match nan {
        FieldValue::Literal(Token::Float(val)) if val.is_nan() => {}
        _ => panic!("It should be NaN"),
    }
    assert!(scptracker.next(&mut lexer).is_none(), "It should be empty");
}
#[test]
fn parser_obj() {
    let mut lexer = Lexer::new("Ta: { Hello: -12, GabyDev0: true, L: nil }")
        .expect("It must support this version");
    let mut scptracket = ScopeTracker::new(&mut lexer, false);
    let result = [
        ("Hello", FieldValue::Literal(Token::Int(-12))),
        ("GabyDev0", FieldValue::Literal(Token::Boolean(true))),
        ("L", FieldValue::Literal(Token::Nil)),
    ];
    let field = scptracket
        .next(&mut lexer)
        .expect("There should be a Field")
        .expect("It shouldn't give an error");
    let (name, val) = field.tulpe();
    assert_eq!(name.as_str(&lexer), Some("Ta"));
    match val {
        FieldValue::Block(mut scope) => {
            for exp in result {
                let field = scope
                    .next(&mut lexer)
                    .expect("There should be a Field")
                    .expect("Debera haber field");
                assert_eq!(field.get_name(&lexer), Some(exp.0), "No match to exp");
                assert_eq!(*field.get_val(), exp.1, "No match to exp");
                let (name, val) = field.tulpe();
                assert_eq!(
                    unsafe { name.as_str_unchecked(&lexer) },
                    exp.0,
                    "No match to exp"
                );
                assert_eq!(val, exp.1, "No match to exp");
            }
            assert_eq!(scope.next(&mut lexer), None, "It should be empty");
            assert_eq!(scope.next(&mut lexer), None, "It should remain empty");
        }
        _ => panic!("Unexpecting {:?}", val),
    }
}

#[test]
fn parser_array() {
    let mut lexer = Lexer::new("Ta [ -12, true\n nil ]").expect("It must support this version");
    let mut scptracket = ScopeTracker::new(&mut lexer, false);
    let result = [
        FieldValue::Literal(Token::Int(-12)),
        FieldValue::Literal(Token::Boolean(true)),
        FieldValue::Literal(Token::Nil),
    ];
    let field = scptracket
        .next(&mut lexer)
        .expect("There should be a Field")
        .expect("It shouldn't give an error");
    //let (name, val) = field.tulpe();
    assert_eq!(field.span().as_str(&lexer), Some("Ta"));
    let val = field.val();
    match val {
        FieldValue::Array(mut scope) => {
            for exp in result {
                let field = scope
                    .next(&mut lexer)
                    .expect("There should be a Field")
                    .expect("It shouldn't give an error");
                assert_eq!(field, exp, "No match to exp");
            }
            assert_eq!(scope.next(&mut lexer), None, "It should be empty");
            assert_eq!(scope.next(&mut lexer), None, "It should remain empty");
        }
        _ => panic!("Unexpecting {:?}", val),
    }
}
fn gen_scope<'a>(code: &'a str) -> (ScopeTracker, Lexer<'a>) {
    let lexer = Lexer::new(code).expect("It must support this version");
    let strk = ScopeTracker::new(&lexer, false);

    (strk, lexer)
}
fn gen_array<'a>(code: &'a str) -> (ArrayTracker, Lexer<'a>) {
    let mut lexer = Lexer::new(code).expect("It must support this version");
    assert_eq!(lexer.next(), Some(Ok(Token::LeftBracket)));
    let strk = ArrayTracker::new(&lexer);

    (strk, lexer)
}
#[test]
fn test_parser_obj_error() {
    let mut lexer = Lexer::new("{").expect("It must support this version");
    assert_eq!(lexer.next(), Some(Ok(Token::LeftBrace)));
    let mut strk = ScopeTracker::new(&lexer, true);

    assert_eq!(
        strk.next(&mut lexer),
        Some(Err(EvalError::Lexical(LexError::UnexpectedEOF)))
    );

    (strk, lexer) = gen_scope("Code::");
    assert_eq!(
        strk.next(&mut lexer),
        Some(Err(EvalError::UnexpectedToken(Token::Colon)))
    );

    (strk, lexer) = gen_scope("Code-");
    assert_eq!(
        strk.next(&mut lexer),
        Some(Err(EvalError::UnexpectedToken(Token::Sub)))
    );

    (strk, lexer) = gen_scope("Code 12");
    assert_eq!(
        strk.next(&mut lexer),
        Some(Err(EvalError::UnexpectedToken(Token::Int(12))))
    );

    (strk, lexer) = gen_scope("Code ,");
    assert_eq!(
        strk.next(&mut lexer),
        Some(Err(EvalError::UnexpectedToken(Token::Comma)))
    );

    (strk, lexer) = gen_scope("Code:");
    assert_eq!(
        strk.next(&mut lexer),
        Some(Err(EvalError::Lexical(LexError::UnexpectedEOF)))
    );

    (strk, lexer) = gen_scope("12:");
    assert_eq!(
        strk.next(&mut lexer),
        Some(Err(EvalError::UnexpectedToken(Token::Int(12))))
    );

    (strk, lexer) = gen_scope("}");
    assert_eq!(
        strk.next(&mut lexer),
        Some(Err(EvalError::UnexpectedToken(Token::RightBrace)))
    );

    (strk, lexer) = gen_scope("C: ñ");
    assert_eq!(
        strk.next(&mut lexer),
        Some(Err(EvalError::Lexical(LexError::InvalidCharacter('ñ'))))
    );

    (strk, lexer) = gen_scope("C: -ñ");
    assert_eq!(
        strk.next(&mut lexer),
        Some(Err(EvalError::Lexical(LexError::InvalidCharacter('ñ'))))
    );

    (strk, lexer) = gen_scope("C: -false");
    assert_eq!(
        strk.next(&mut lexer),
        Some(Err(EvalError::UnexpectedToken(Token::Boolean(false))))
    );

    (strk, lexer) = gen_scope("C: -");
    assert_eq!(
        strk.next(&mut lexer),
        Some(Err(EvalError::Lexical(LexError::UnexpectedEOF)))
    );

    (strk, lexer) = gen_scope("ñ");
    assert_eq!(
        strk.next(&mut lexer),
        Some(Err(EvalError::Lexical(LexError::InvalidCharacter('ñ'))))
    );
}

#[test]
fn test_parser_array_error() {
    let (mut arrtrk, mut lexer) = gen_array("[ñ]");
    assert_eq!(
        arrtrk.next(&mut lexer),
        Some(Err(EvalError::Lexical(LexError::InvalidCharacter('ñ'))))
    );

    (arrtrk, lexer) = gen_array("[12 14]");
    assert_eq!(
        arrtrk.next(&mut lexer),
        Some(Ok(FieldValue::Literal(Token::Int(12))))
    );
    assert_eq!(
        arrtrk.next(&mut lexer),
        Some(Err(EvalError::UnexpectedToken(Token::Int(14))))
    );

    (arrtrk, lexer) = gen_array("[12 {]");
    assert_eq!(
        arrtrk.next(&mut lexer),
        Some(Ok(FieldValue::Literal(Token::Int(12))))
    );
    assert_eq!(
        arrtrk.next(&mut lexer),
        Some(Err(EvalError::UnexpectedToken(Token::LeftBrace)))
    );

    (arrtrk, lexer) = gen_array("[12 []");
    assert_eq!(
        arrtrk.next(&mut lexer),
        Some(Ok(FieldValue::Literal(Token::Int(12))))
    );
    assert_eq!(
        arrtrk.next(&mut lexer),
        Some(Err(EvalError::UnexpectedToken(Token::LeftBracket)))
    );

    (arrtrk, lexer) = gen_array("[12 }]");
    assert_eq!(
        arrtrk.next(&mut lexer),
        Some(Ok(FieldValue::Literal(Token::Int(12))))
    );
    assert_eq!(
        arrtrk.next(&mut lexer),
        Some(Err(EvalError::UnexpectedToken(Token::RightBrace)))
    );

    (arrtrk, lexer) = gen_array("[12 -]");
    assert_eq!(
        arrtrk.next(&mut lexer),
        Some(Ok(FieldValue::Literal(Token::Int(12))))
    );
    assert_eq!(
        arrtrk.next(&mut lexer),
        Some(Err(EvalError::UnexpectedToken(Token::Sub)))
    );

    (arrtrk, lexer) = gen_array("[-ñ]");
    assert_eq!(
        arrtrk.next(&mut lexer),
        Some(Err(EvalError::Lexical(LexError::InvalidCharacter('ñ'))))
    );

    (arrtrk, lexer) = gen_array("[-12]");
    assert_eq!(
        arrtrk.next(&mut lexer),
        Some(Ok(FieldValue::Literal(Token::Int(-12))))
    );
    assert_eq!(arrtrk.next(&mut lexer), None);

    (arrtrk, lexer) = gen_array("[-]");
    assert_eq!(
        arrtrk.next(&mut lexer),
        Some(Err(EvalError::UnexpectedToken(Token::RightBracket)))
    );

    (arrtrk, lexer) = gen_array("[-");
    assert_eq!(
        arrtrk.next(&mut lexer),
        Some(Err(EvalError::Lexical(LexError::UnexpectedEOF)))
    );

    (arrtrk, lexer) = gen_array("[");
    assert_eq!(
        arrtrk.next(&mut lexer),
        Some(Err(EvalError::Lexical(LexError::UnexpectedEOF)))
    );
}

#[test]
fn test_parser_array() {
    let (mut arrtrk, mut lexer) = gen_array("[{}]");
    match arrtrk.next(&mut lexer) {
        Some(Ok(FieldValue::Block(_))) => {}
        v => panic!("Unexpected: {:?}", v),
    }
    (arrtrk, lexer) = gen_array("[[]]");
    match arrtrk.next(&mut lexer) {
        Some(Ok(FieldValue::Array(_))) => {}
        v => panic!("Unexpected: {:?}", v),
    }
}
