#[macro_use]
mod src;
mod token;
pub use src::*;
pub use token::*;

pub struct Lexer<'a> {
    pub(crate) src: Src<'a>,
    ntoken: Option<Result<Token, LexError>>,
    end_token: Location,
    pub(crate) prev_token: Location,
    nline: u32, // The line number corresponding to `ntoken`.
    line: u32,  // The current line number in the source, updated as we consume tokens.
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Result<Self, LexError> {
        let mut lexer = Self {
            src: Src::new(input, false),
            ntoken: None,
            nline: 1,
            line: 1,
            end_token: Location::zero(),
            prev_token: Location::zero(),
        };
        lexer.src.trim();
        lexer.checkversion()?;
        lexer.ntoken = lexer.next_internal();
        Ok(lexer)
    }
    fn as_string(&self) -> &'a str {
        let start = self.src.curr();
        let len = self.src.end().as_usize() - start.as_usize();

        let slice = unsafe { std::slice::from_raw_parts(start.as_ptr(), len) };
        unsafe { std::str::from_utf8_unchecked(slice) }
    }
    fn next_internal(&mut self) -> Option<Result<Token, LexError>> {
        while self.src.is_not_empty() {
            let ch = self.src.get_byte_unchecked();
            self.src.advance(1);
            return match ch {
                b'\n' => {
                    self.nline += 1;
                    Some(Ok(Token::Newline))
                }
                b' ' | b'\t' | b'\r' | b'\0' => {
                    self.src.trim();
                    continue;
                }
                b'0'..=b'9' => Some(self.number(ch)),

                b'-' => Some(Ok(Token::Sub)),

                b',' => Some(Ok(Token::Comma)),

                b':' => Some(Ok(Token::Colon)),

                b'{' => Some(Ok(Token::LeftBrace)),

                b'}' => Some(Ok(Token::RightBrace)),

                b'[' => Some(Ok(Token::LeftBracket)),

                b']' => Some(Ok(Token::RightBracket)),

                b'\'' | b'"' => Some(self.next_string(ch)),

                b'a'..=b'z' | b'A'..=b'Z' | b'_' => Some(Ok(self.identifier())),
                b'#' => {
                    self.src.skip_current_line();
                    continue;
                }
                _ => {
                    self.src.retreat(1);
                    let strn = self.as_string();

                    Some(Err(LexError::InvalidCharacter(unsafe {
                        strn.chars().next().unwrap_unchecked()
                    })))
                }
            };
        }
        None
    }

    /// Number parsing
    #[inline(always)]
    fn number_calc<F: FnMut(u8) -> Result<(), LexError>>(
        &mut self,
        mut f: F,
    ) -> Result<(), LexError> {
        let n_ = b'_'.wrapping_sub(b'0');
        let n0 = wide::u8x16::splat(b'0');
        let n9 = wide::u8x16::splat(9);
        if self.src.simd_with_error(|_, chunk| {
            let sub = chunk - n0;
            let result = sub.simd_le(n9) | sub.simd_eq(n_);
            let len_num = result.to_bitmask().trailing_ones();
            let array = sub.as_array();
            for (i, v) in array.iter().enumerate() {
                if i >= len_num as usize {
                    break;
                }
                if (*v) == n_ {
                    continue;
                }
                f(*v)?
            }
            if len_num == 16 {
                Ok(SimdStep::Continue)
            } else {
                Ok(SimdStep::SkipAndBreak(len_num as u8))
            }
        })? {
            return Ok(());
        }

        self.src.scalar_with_error(|_, n| {
            let c = n.wrapping_sub(b'0');
            if c <= 9 {
                f(c)?;
                Ok(ScalarStep::Continue)
            } else if c == n_ {
                Ok(ScalarStep::Continue)
            } else {
                Ok(ScalarStep::Break)
            }
        })?;
        Ok(())
    }

    fn number(&mut self, ch: u8) -> Result<Token, LexError> {
        let mut value: i64 = (ch - b'0') as i64;
        self.number_calc(|n| {
            let temp = value.wrapping_mul(10).wrapping_add(n as i64);
            if temp < value {
                Err(LexError::NumericOverflow)
            } else {
                value = temp;
                Ok(())
            }
        })?;
        Ok(self.flotable(value))
    }

    #[inline(always)]
    fn flotable(&mut self, num: i64) -> Token {
        match self.src.get_byte() {
            Some(b'.')
                if self.src.is_size(2)
                    && self.src.read::<u16>() != u16::from_ne_bytes([b'.', b'.']) =>
            {
                self.src.advance(1);
                self.next_float(num as f64)
            }
            _ => Token::Int(num),
        }
    }
    /// Process the numeric computation by parsing the source bytes from
    #[inline(always)]
    fn next_float(&mut self, mut num: f64) -> Token {
        let mut mul = 0.1;
        let _ = self.number_calc(|n| {
            num += (n as f64) * mul;
            mul *= 0.1;
            Ok(())
        });
        Token::Float(num)
    }
    #[inline(always)]
    pub fn identifier(&mut self) -> Token {
        let mut location = Location::new(&self.src);
        location.retreat(1);

        let nspace = b'z' - b'a';
        let wa = wide::u8x16::splat(b'a');
        let wspace = wide::u8x16::splat(nspace);
        let wa_uppercase = wide::u8x16::splat(b'A');
        let w_ = wide::u8x16::splat(b'_');
        let w0 = wide::u8x16::splat(b'0');

        process_src_with_simd_fallback!(self.src,
            simd: |chunk| {
                let result = (chunk - wa).simd_le(wspace) | (chunk - wa_uppercase).simd_le(wspace) | (chunk - w0).simd_le(wide::u8x16::splat(9)) | chunk.simd_eq(w_);
                let mask = result.to_bitmask();

                if mask == 0xFFFF {
                    SimdStep::Continue
                } else {
                    SimdStep::SkipAndBreak(mask.trailing_ones() as u8)
                }
            },
            scalar: |ch| {
                match ch {
                    b'a'..=b'z' | b'A'..=b'Z' | b'_' | b'0'..=b'9' => ScalarStep::Continue,
                    _ => ScalarStep::Break
                }
            }
        );
        let end = Location::new(&self.src);

        let span = Span::new_from_end(self.nline, location, end);
        let strn = unsafe { span.as_str_unchecked(&self) };

        // Match the identifier to a keyword
        match strn {
            "true" => Token::Boolean(true),
            "false" => Token::Boolean(false),
            "NaN" => Token::Float(f64::NAN),
            "Infinity" => Token::Float(f64::INFINITY),
            "nil" => Token::Nil,
            _ => Token::Identifier(span),
        }
    }

    pub fn next_string(&mut self, closed: u8) -> Result<Token, LexError> {
        let location = Location::new(&self.src);
        let mut is_special = false;

        let wclosed = wide::u8x16::splat(closed);
        let wbackslash = wide::u8x16::splat(b'\\');
        let wnewline = wide::u8x16::splat(b'\n');

        if process_src_with_simd_fallback_error!(self.src,
            simd: |src, chunk| {
                // Simd operations
                let rclosed = (chunk.simd_eq(wclosed).to_bitmask() as u16).trailing_zeros() as u16;
                let rbackslash = chunk.simd_eq(wbackslash).to_bitmask() as u16;
                let nbackslash = rbackslash.trailing_zeros() as u16;
                let rnewline = (chunk.simd_eq(wnewline).to_bitmask() as u16).trailing_zeros() as u16;

                // Backslash handling?
                is_special |= nbackslash < rclosed;

                if rnewline < rclosed {
                    return Err(LexError::UnterminatedString);
                }
                return if rclosed == 16 {
                    // Advance 16 or 17 bytes
                    let is_next = rbackslash.reverse_bits().trailing_ones() & 0x1 != 0; // impar backslash
                    match is_next {
                        true if src.is_size(17) => src.advance(1),
                        false => {},
                        _ => return Err(LexError::UnexpectedEOF)
                    }
                    Ok(SimdStep::Continue)
                }else {
                    let shift = 16 - rclosed; // inverse
                    let shifted = (rbackslash.wrapping_shl(shift as u32)) as u16;
                    // check whether there is an odd number of backslashes when reading from right to left
                    let is_scaped = shifted.reverse_bits().trailing_ones() & 0x01 == 1;

                    return Ok (if is_scaped {
                        SimdStep::SkipAndContinue((rclosed+1) as u8) // to close the quotation marks
                    }else{
                        SimdStep::SkipAndBreak((rclosed+1) as u8) // close the closing quotation marks
                    });
                }

            },
            scalar: |src, ch| {
                match ch {
                    b'\\' => {
                        if src.is_size(2) {
                            src.advance(1);
                            is_special = true;
                            Ok(ScalarStep::Continue)
                        }else{
                            Err(LexError::UnexpectedEOF)
                        }
                    },
                    _ if ch == closed => Ok(ScalarStep::BreakAndConsume),
                    _ => Ok(ScalarStep::Continue)

                }
            }
        )? {
            let mut end = Location::new(&self.src);
            end.retreat(1);
            let span = Span::new_from_end(self.nline, location, end);
            Ok(if is_special {
                Token::Text(span)
            } else {
                Token::RawText(span)
            })
        } else {
            Err(LexError::UnterminatedString)
        }
    }

    #[inline(always)]
    fn start_tero(&mut self) -> bool {
        self.src.is_size(4)
            && unsafe { self.src.curr().cast::<u32>().read_unaligned() }
                == u32::from_ne_bytes([b'T', b'e', b'r', b'o'])
    }

    /// Skip Tokens newline
    #[inline]
    pub fn next_skip_ln(&mut self) -> Option<Result<Token, LexError>> {
        loop {
            return match self.next() {
                Some(Ok(Token::Newline)) => continue,
                tk => tk,
            };
        }
    }
    #[inline(always)]
    pub fn peek(&self) -> Option<Result<Token, LexError>> {
        self.ntoken.clone()
    }
    fn checkversion(&mut self) -> Result<(), LexError> {
        if self.start_tero() {
            self.src.advance(4);
            // Skip whitespaces
            self.src.scalar_with(|chunk| {
                if matches!(chunk, b' ' | b'\t' | b'\r') {
                    ScalarStep::Continue
                } else {
                    ScalarStep::Break
                }
            });
            // Check version number
            match self.src.get_byte() {
                Some(b'0') => {
                    self.src.skip_current_line();
                    Ok(())
                }
                Some(ch) if ch.is_ascii_digit() => Err(LexError::InvalidVersion),
                _ => {
                    self.src.reset(); // abort the "Tero" read
                    Ok(())
                } // No version info, reset to start for normal lexing
            }
        } else {
            Ok(())
        }
    }
    #[inline(always)]
    pub fn location(&self) -> Location {
        self.end_token
    }
    #[inline(always)]
    pub fn prevlocation(&self) -> Location {
        self.prev_token
    }
    #[inline(always)]
    pub fn line(&self) -> u32 {
        self.line
    }
    #[inline(always)]
    pub fn gen_error(&self, loc: Location, line: u32) -> Option<(u32, String)> {
        self.src.gen_error(line, loc)
    }
    #[inline(always)]
    pub fn gen_error_with_span(&self, span: Span) -> Option<(u32, String)> {
        self.src.gen_error(span.line(), span.get_start())
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Result<Token, LexError>;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(Err(_)) = &self.ntoken {
            return self.ntoken.clone();
        }
        self.line = self.nline;
        self.prev_token = self.end_token;
        self.end_token = Location::new(&self.src);
        let tk = self.next_internal();
        std::mem::replace(&mut self.ntoken, tk)
    }
}
