use std::{ptr::NonNull};

use crate::{IterToken, LexError, ast::{Literal, LoomValue, Token}};
pub enum SimdResult {
    Continue,
    ContinueWithoutSkip,
    SkipAndContinue(u8),
    SkipAndBreak(u8)
}
impl<'a> IterToken<'a> {
    pub(crate) fn to_string(&self, curr: NonNull<u8>) -> &'a str {
        let size = (self.curr.as_ptr() as usize) - (curr.as_ptr() as usize);
        let slice = unsafe { std::slice::from_raw_parts(curr.as_ptr(), size) };

        unsafe { std::str::from_utf8_unchecked(slice) }
    }
    pub(crate) fn as_string(&self) -> &'a str {
        let slice = unsafe { std::slice::from_raw_parts(self.curr.as_ptr(), self.len()) };
        unsafe { std::str::from_utf8_unchecked(slice) }
    }

    #[inline(always)]
    pub(crate) fn simd<F>(&mut self, mut f: F) -> bool
        where
            F: FnMut(&mut Self, wide::u8x16) -> SimdResult,
    {
        while self.is_simd() {
            match f(self, self.get_wide()) {
                SimdResult::Continue => self.consume(16),
                SimdResult::SkipAndContinue(bytes) => self.consume(bytes as usize),
                SimdResult::SkipAndBreak(bytes) => {
                    self.consume(bytes as usize);
                    return true;
                },
                _ => {}
            }
        }
        false
    }
    #[inline(always)]
    pub(crate) fn simd_err<F>(&mut self, mut f: F) -> Result<bool, LexError>
        where
            F: FnMut(&mut Self, wide::u8x16) -> Result<SimdResult, LexError>,
    {
        while self.is_simd() {
            match f(self, self.get_wide())? {
                SimdResult::Continue => self.consume(16),
                SimdResult::SkipAndContinue(bytes) => self.consume(bytes as usize),
                SimdResult::SkipAndBreak(bytes) => {
                    self.consume(bytes as usize);
                    return Ok(true);
                },
                _ => {}
            }
        }
        Ok(false)
    }
    #[inline(always)]
    pub(crate) fn iter_char<F>(&mut self, mut f: F) 
        where
            F: FnMut(&mut Self, u8) -> bool
    {
        while self.is_not_empty() {
            let l = self.get_unchecked();
            if f(self, l) {
                break;
            }
            self.consume(1);
        }
    }
    #[inline(always)]
    pub(crate) fn iter_char_err<F>(&mut self, mut f: F) -> Result<bool, LexError>
        where
            F: FnMut(&mut Self, u8) -> Result<bool, LexError>
    {
        while self.is_not_empty() {
            let l = self.get_unchecked();
            if f(self, l)? {
                return Ok(true)
            }
            self.consume(1);
        }
        Ok(false)
    }
    pub(crate) fn ignore_line(&mut self) {
        let newline = wide::u8x16::splat(b'\n');
        if self.simd(|_, block| {
            let offset: u8 = block.simd_eq(newline).to_bitmask().trailing_zeros() as u8;
            if offset < 16 {
                SimdResult::SkipAndBreak(offset)
            }else{
                SimdResult::Continue
            }
        }) {
            return;
        }
        self.iter_char(|_, ch|{
            if ch == b'\n'{
                true
            }else {
               false 
            }
        });
    }
    pub(crate) fn ident(&mut self) -> &'a str {
        let curr = unsafe { self.curr.sub(1) }; // posicion original (primer caracter)
        let a = wide::u8x16::splat(b'a');
        let z = wide::u8x16::splat(b'z');
        let up_a = wide::u8x16::splat(b'A');
        let up_z = wide::u8x16::splat(b'Z');
        let v0 = wide::u8x16::splat(b'0');
        let v9 = wide::u8x16::splat(b'9');
        let v_ = wide::u8x16::splat(b'_');
        if self.simd(|_, block| {
            let is_lowercase = block.simd_ge(a) & block.simd_le(z);
            let is_uppercase = block.simd_ge(up_a) & block.simd_le(up_z);
            let is_digit     = block.simd_ge(v0) & block.simd_le(v9);
            let is__         = block.simd_eq(v_);
            let result = is_lowercase | is_uppercase | is_digit | is__;
            let mask = result.to_bitmask();
            if mask == 0xFFFF {
                SimdResult::Continue
            }else {
                SimdResult::SkipAndBreak(mask.trailing_ones() as u8)
            }
        }) { return self.to_string(curr) }
        self.iter_char(|_, ch|{
            if !ch.is_ascii_alphanumeric() && ch != b'_' {
                return true;
            }
            false
        });

        self.to_string(curr)
    }
    pub(crate) fn number(&mut self) -> &'a str {
        let curr = unsafe { self.curr.sub(1) }; // posicion original (primer caracter)

        let v0 = wide::u8x16::splat(b'0');
        let v9 = wide::u8x16::splat(b'9');
        let v_ = wide::u8x16::splat(b'_');
        if self.simd(|_, block| {
            let is_digit     = block.simd_ge(v0) & block.simd_le(v9);
            let is__         = block.simd_eq(v_);
            let result = is_digit | is__;
            let mask = result.to_bitmask();
            if mask == 0xFFFF {
                SimdResult::Continue
            }else {
                SimdResult::SkipAndBreak(mask.trailing_ones() as u8)
            }
        }) {
            return self.to_string(curr);
        }
        self.iter_char(|_, ch| {
            if ch.is_ascii_digit() || ch == b'_' {
                false
            }else{
                true
            }
        });
        

        self.to_string(curr)
    }
    pub(crate) fn string(&mut self, limit: u8) -> Result<Token<'a>, LexError> {
        let curr = self.curr;
        let end_str = wide::u8x16::splat(limit);
        let special = wide::u8x16::splat(b'\\');
        let newline = wide::u8x16::splat(b'\n');
        let mut is_special = false;
        if self.simd_err(|lexer, block| {
            let offset_end = block.simd_eq(end_str).to_bitmask().trailing_zeros();  
            let mask_special = block.simd_eq(special).to_bitmask();
            let first_newline = block.simd_eq(newline).to_bitmask().trailing_zeros();
            is_special |= mask_special != 0;
            if offset_end < 16 {
                if first_newline < offset_end {
                    return Err(LexError::UnterminatedString);
                }
                let shift = 16 - offset_end;
                let shifted = (mask_special << shift) as u16;
                let is_scaped = shifted.reverse_bits().trailing_ones() & 0x01 == 1;
                return Ok (if is_scaped {
                    SimdResult::SkipAndContinue((offset_end+1) as u8)
                }else{
                    SimdResult::SkipAndBreak((offset_end+1) as u8)
                });
            }
            if first_newline < 16 {
                Err(LexError::UnterminatedString)
            }else{
                lexer.consume(16usize);
                let is_scaped = ((mask_special as u16).reverse_bits().trailing_ones() & 0x01) == 1;
                if is_scaped & (lexer.is_not_empty() && lexer.get_unchecked() != b'\n') {
                    Ok(SimdResult::SkipAndContinue(1))
                }else{
                    Ok(SimdResult::ContinueWithoutSkip)
                }
            }
            
        })? {
            return match is_special {
                true => Ok(Token::Value(LoomValue::Raw(Literal::Text(self.to_string(curr))))),
                false => Ok(Token::Value(LoomValue::Raw(Literal::RawText(self.to_string(curr))))),
            };
        }

        if self.iter_char_err(|lexer, ch| {
            match ch {
                b'\\' => {
                    is_special = true;
                    lexer.consume(1);
                    if lexer.is_not_empty() && lexer.get_unchecked() != b'\n' {
                        return Ok(false);
                    }
                }
                _ if ch == limit => {
                    lexer.consume(1);
                    return Ok(true);
                }
                _ if ch != b'\n' => {
                    return Ok(false);
                }
                _ =>{}
            }
            Err(LexError::UnterminatedString)
        })?{
            match is_special {
                true => Ok(Token::Value(LoomValue::Raw(Literal::Text(self.to_string(curr))))),
                false => Ok(Token::Value(LoomValue::Raw(Literal::RawText(self.to_string(curr))))),
            }
        }else{
            Err(LexError::UnterminatedString)
        }
    } 
    pub(crate) fn as_number(&mut self) -> Result<Token<'a>, LexError> {
        let mut num: i64 = 0;
        for ch in self.number().as_bytes().iter() {
            if *ch == b'_' {
                continue;
            }
            num = num*10 + (*ch - b'0') as i64
        }       
        if self.is_empty() {
            Ok(Token::Value(LoomValue::Raw(Literal::Integer(num))))
        }else{
            if self.get_unchecked().is_ascii_alphabetic() {
                Err(LexError::InvalidCharacter(unsafe { self.as_string().chars().next().unwrap_unchecked() }))
            }else if self.get_unchecked() == b'.' {
                self.as_f64(num as f64)
            }else{
                Ok(Token::Value(LoomValue::Raw(Literal::Integer(num))))
            }
        }
    }
    pub(crate) fn as_f64(&mut self, mut float: f64) -> Result<Token<'a>, LexError> {
        self.consume(2);
        let mut div = 0.1;
        for ch in self.number().as_bytes().iter() {
            if *ch == b'_' {
                continue;
            }
            float += ((*ch - b'0') as f64) * div;
            div *= 0.1;
        }
        if self.is_not_empty() && self.get_unchecked().is_ascii_alphabetic() {
            Err(LexError::InvalidCharacter(unsafe { self.as_string().chars().next().unwrap_unchecked() }))
        }else{
            Ok(Token::Value(LoomValue::Raw(Literal::Float(float))))
        }
    }
}