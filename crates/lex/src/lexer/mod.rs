use std::{marker::PhantomData, ptr::NonNull};
mod string;

use crate::{ast::{Literal, LoomValue, Token}, lexer::string::SimdResult};

/// Errores críticos que pueden ocurrir durante la fase de análisis léxico (Lexing).
///
/// Estos errores representan fallos a nivel de bytes y caracteres antes de que
/// el flujo de tokens llegue al parser.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum LexError {
    /// La cabecera o directiva de versión del archivo `.tero` es inválida o no está soportada.
    InvalidVersion,

    /// Se encontró un carácter inesperado que no pertenece a la gramática del lenguaje.
    InvalidCharacter(char),

    /// Una cadena de texto quedó abierta (falta la comilla de cierre) antes de terminar el archivo o la línea.
    UnterminatedString,
}
/// Lector fuente de Loom
/// Solo acepta versiones 0.x
pub struct IterToken<'a> {
    curr: NonNull<u8>,
    end: NonNull<u8>,
    consum_line: u32,
    line: u32,
    data: PhantomData<&'a str>,
}

/// Wrapper de IterToken que permite un Peek eficiente
pub struct Lexer<'a> {
    iter: IterToken<'a>,
    next_token: Option<Result<Token<'a>, LexError>>
}

impl<'a> IterToken<'a> {
    pub fn new(input: &'a str) -> Self {
        let first = unsafe { input.as_bytes().get_unchecked(0) };
        let end = unsafe { NonNull::from( first ).byte_add(input.len()) };
        let mut iter = Self {
            curr: NonNull::from_ref(first),
            end,
            consum_line: 1,
            line: 1,
            data: PhantomData
        };
        iter.start_trim();
        iter
    }

    #[inline(always)]
    pub(crate) fn len(&self) -> usize {
        (self.end.as_ptr() as usize) - (self.curr.as_ptr() as usize)
    }
    #[inline(always)]
    pub(crate) fn is_simd(&self) -> bool {
        self.is_size(16)
    }
    #[inline(always)]
    pub(crate) fn is_size(&self, b: usize) -> bool {
        ((self.curr.as_ptr() as usize) + b) <= (self.end.as_ptr() as usize)
    }
    #[inline(always)]
    pub(crate) fn is_not_empty(&self) -> bool {
        self.curr < self.end
    }
    #[inline(always)]
    pub(crate) fn is_empty(&self) -> bool {
        self.curr >= self.end
    }
    #[inline(always)]
    fn consume(&mut self, bytes: usize) {
        self.curr = unsafe { self.curr.byte_add(bytes) };
    }
    #[inline(always)]
    fn sub(&mut self, bytes: usize) {
        self.curr = unsafe { self.curr.byte_sub(bytes) };
    }
    #[inline(always)]
    fn get_unchecked(&self) -> u8 {
        unsafe { self.curr.read() }
    }
    #[inline(always)]
    fn get_wide(&self) -> wide::u8x16 {
        wide::u8x16::from(unsafe { *self.curr.cast::<[u8; 16]>().as_ptr() })
    }
    #[inline]
    pub fn start_trim(&mut self) {
        // Definimos los vectores de espacios en blanco ASCII
        let spaces = wide::u8x16::splat(b' ');
        let tabs = wide::u8x16::splat(b'\t');
        let cr = wide::u8x16::splat(b'\r');
        let _ = self.simd(|_, block| {
            // Comparamos los 16 bytes contra los 3 tipos de espacios en blanco en paralelo
            let is_space = block.simd_eq(spaces);
            let is_tab = block.simd_eq(tabs);
            let is_cr = block.simd_eq(cr);

            // Combinamos las máscaras (si es cualquiera de ellos, se vuelve un bit activo)
            let whitespace_mask = is_space | is_tab | is_cr ;
    
            // Convertimos la máscara vectorial a un entero de bits
            let mask: u32 = whitespace_mask.to_bitmask();

            if mask == 0xFFFF {
                // ¡Los 16 bytes completos son espacios en blanco! 
                // Avanzamos el bloque completo en 1 ciclo de reloj.
                SimdResult::Continue
            } else {
                // Encontraste el primer byte que NO es un espacio en blanco.
                // Contamos cuántos bits en uno (espacios) hay desde el inicio antes del primer cero.
                // trailing_ones() te dice exactamente cuántos bytes saltar.
                SimdResult::SkipAndBreak(mask.trailing_ones() as u8)
            }
        });
    
        // cuando quedan menos de 16 bytes:
        self.iter_char(|_, ch| {
            !matches!(ch, b' '|b'\t'|b'\r')
        });
    }

    #[inline(always)]
    pub fn start_tero(&mut self) -> bool {
        self.is_size(4) && 
        unsafe { self.curr.cast::<u32>().read_unaligned() } == u32::from_ne_bytes([b'T',b'E', b'R', b'O'])
    }
    pub fn checkversion(&mut self) -> Result<(), LexError> {
        const TERO: &str = "TERO";
        let curr = self.curr;
        if self.start_tero() {
            self.consume(TERO.len());

            self.iter_char(|_, ch| {
                !matches!(ch, b' '|b'\t'|b'\r')
            });

            if self.is_not_empty() && self.get_unchecked().is_ascii_digit() {
                if self.get_unchecked() != b'0' {
                    return Err(LexError::InvalidVersion);
                }
                self.ignore_line();
                return Ok(());
            }
            self.curr = curr;
        }
        Ok(())
    } 

    #[inline(always)]
    pub(crate) fn next_interal(&mut self) -> Option<Result<Token<'a>, LexError>> {
        while self.is_not_empty() {
            let ch = self.get_unchecked();
            self.consume(1);
            return match ch {
                b'\n' => {
                    self.line += 1;
                    Some(Ok(Token::Newline))
                },
                b'\t' | b' ' | b'\r' => {
                    self.start_trim();
                    continue;
                }
                b'#' => {
                    self.ignore_line();
                    continue;
                }
                b'A'..=b'Z' | b'a'..=b'z' | b'_' => {
                    let chars = self.ident();
                    match chars {
                        "true" => Some(Ok(Token::Value(LoomValue::Raw(Literal::Boolean(true))))),
                        "false" => Some(Ok(Token::Value(LoomValue::Raw(Literal::Boolean(false))))),
                        "nil" => Some(Ok(Token::Value(LoomValue::Raw(Literal::Nil)))),
                        "NaN" => Some(Ok(Token::Value(LoomValue::Raw(Literal::NaN)))),
                        "Infinity" => Some(Ok(Token::Value(LoomValue::Raw(Literal::Infinity)))),
                        _=>Some(Ok(Token::Identifier(chars)))
                    }
                }
                b'0'..=b'9' => {
                    Some(self.as_number())
                }
                b'{' => {
                    Some(Ok(Token::LeftBrace))
                }
                b'}' => {
                    Some(Ok(Token::RightBrace))
                }
                b'[' => {
                    Some(Ok(Token::LeftBracket))
                }
                b']' => {
                    Some(Ok(Token::RightBracket))
                }
                b':' => {
                    Some(Ok(Token::Colon))
                }
                b',' => {
                    Some(Ok(Token::Comma))
                }
                b'-' => {
                    Some(Ok(Token::Sub))
                }
                b'\''| b'"' => {
                    Some(self.string(ch))
                }

                _=> {
                    self.sub(1);
                    
                    Some(Err(LexError::InvalidCharacter( 
                        unsafe { self.as_string().chars().next().unwrap_unchecked() }
                     )))
                }
            }
        }
        None
    }


/*
    #[inline(always)]
    pub fn first_line(&mut self) -> Result<(), LexError> {
        
        Ok(())
    }
*/
}
impl<'a> Iterator for IterToken<'a> {
    type Item = Result<Token<'a>, LexError>;
    fn next(&mut self) -> Option<Self::Item> {
        self.next_interal()
    }
}

impl<'a> Lexer<'a> {
    #[inline]
    pub fn new(input: &'a str) -> Result<Self, LexError> {
        let mut iter = IterToken::new(input);

        iter.checkversion()?;
        //iter.first_line()?;
        let next_token = iter.next();

        Ok(Self {
            iter,
            next_token
        })
    }
    
    pub fn line(&self) -> u32 {
        self.iter.consum_line
    }
    #[inline(always)]
    pub fn next_skip_ln(&mut self) -> Option<Result<Token<'a>, LexError>> {
        loop {
            match self.next() {
                Some(Ok(Token::Newline)) => continue,
                other => return other
            }
        }
    }
    #[inline(always)]
    pub fn peek(&self) -> Option<Result<Token<'a>, LexError>> {
        self.next_token.clone()
    }

}

impl<'a> Iterator for Lexer<'a> {
    type Item = Result<Token<'a>, LexError>;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(Err(_)) = &self.next_token {
            return self.next_token.clone();
        }
        self.iter.consum_line = self.iter.line;
        std::mem::replace(&mut self.next_token, self.iter.next_interal())
    }
}