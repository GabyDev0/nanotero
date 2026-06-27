use std::{marker::PhantomData, ptr::NonNull};

use num_traits::Unsigned;

use crate::lex::{LexError, Lexer};
pub trait AsUsize {
    fn as_usize(&self) -> usize;
}

/// Process the source input using SIMD processing with a fallback to scalar processing if SIMD is not possible or if an error occurs during SIMD processing. The provided functions `fs` and `fe` are applied to the source input in their respective processing modes.
macro_rules! process_src_with_simd_fallback {
    ($src:expr, simd: $fs:expr, scalar: $fe:expr) => {
        match $src.simd_with($fs) {
            true => true,
            false => $src.scalar_with($fe),
        }
    };
}

/// Similar to `process_src_with_simd_fallback`, but propagates lexing errors from SIMD processing.
macro_rules! process_src_with_simd_fallback_error {
    ($src:expr, simd: $fs:expr, scalar: $fe:expr) => {
        match $src.simd_with_error($fs) {
            Ok(true) => Ok(true),
            Ok(false) => $src.scalar_with_error($fe),
            Err(e) => Err(e),
        }
    };
}

/// Ignore bytes in `$src` matching any of the provided characters.
/// Uses SIMD first and falls back to scalar scanning when necessary.
macro_rules! ignore_src_chars {
    ($src:expr, $($ch:literal)|+ $(|)?) => {{
        process_src_with_simd_fallback!($src,
            simd: |chunk| {
                let mut mask: u32 = 0;
                $(
                    mask |= chunk.simd_eq(wide::u8x16::splat($ch)).to_bitmask();
                )*

                if mask == 0xffff {
                    SimdStep::Continue
                } else {
                    let offset = mask.trailing_ones() as u8;
                    SimdStep::SkipAndBreak(offset)
                }
            },
            scalar: |byte| match byte {
                $($ch)|* => ScalarStep::Continue,
                _ => ScalarStep::Break,
            }
        )
    }};
}

pub struct Src<'a> {
    start: NonNull<u8>,
    end: NonNull<u8>,
    curr: NonNull<u8>,
    has_simd_padding: bool,
    phantom: std::marker::PhantomData<&'a str>,
}

pub enum SimdStep {
    Continue,
    SkipAndContinue(u8),
    SkipAndBreak(u8),
}

pub enum ScalarStep {
    Continue,
    Break,
    BreakAndConsume,
}
impl<'a> Src<'a> {
    pub fn new(input: &'a str, has_simd_padding: bool) -> Self {
        let first = unsafe { &*input.as_bytes().as_ptr() };
        let end = unsafe { NonNull::from(first).byte_add(input.len()) };
        Self {
            start: NonNull::from_ref(first),
            end,
            curr: NonNull::from_ref(first),
            has_simd_padding,
            phantom: std::marker::PhantomData,
        }
    }
    #[inline(always)]
    pub(crate) fn reset(&mut self) {
        self.curr = self.start;
    }
    /// Return the length of the remaining input
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.end.as_usize().wrapping_sub(self.curr.as_usize())
    }

    #[inline(always)]
    pub fn curr(&self) -> NonNull<u8> {
        self.curr
    }
    #[inline(always)]
    pub fn end(&self) -> NonNull<u8> {
        self.end
    }

    /// Return number of bytes remaining in the source input
    #[inline(always)]
    pub fn read<T: Unsigned>(&self) -> T {
        debug_assert!(self.is_size(std::mem::size_of::<T>()), "Out of range");
        unsafe { self.curr.cast().read_unaligned() }
    }

    /// Check if there are at least `b` bytes remaining
    #[inline(always)]
    pub fn is_size(&self, b: usize) -> bool {
        (self.curr.as_usize() + b) <= self.end.as_usize()
    }

    /// Check if there are at least 16 bytes remaining for SIMD processing
    #[inline(always)]
    pub fn can_simd(&self) -> bool {
        // We need to evaluate whether "|" performs better than a "||"
        (self.is_size(16)) | (self.has_simd_padding & self.is_not_empty())
    }

    /// Check if there is any input left to process
    #[inline(always)]
    pub fn is_not_empty(&self) -> bool {
        self.curr < self.end
    }

    /// Check if there is no input left to process
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.curr >= self.end
    }

    /// Return SIMD chunk of 16 bytes from the current position
    #[inline(always)]
    pub fn get_wide(&self) -> wide::u8x16 {
        wide::u8x16::from(unsafe { *self.curr.cast::<[u8; 16]>().as_ptr() })
    }

    /// Advance the current position by `bytes` bytes
    #[inline(always)]
    pub fn advance(&mut self, bytes: usize) {
        debug_assert!(
            self.is_size(bytes),
            "Attempting to advance beyond the end of input"
        );
        self.curr = unsafe { self.curr.byte_add(bytes) };
    }
    fn advance_unchecked(&mut self, bytes: usize) {
        debug_assert!(
            bytes <= 16,
            "advance_unchecked should only be used for small skips ({})",
            bytes
        );
        self.curr = unsafe { self.curr.byte_add(bytes) };
    }

    /// Subtract `bytes` bytes from the current position
    #[inline(always)]
    pub fn retreat(&mut self, bytes: usize) {
        debug_assert!(
            self.curr.as_usize() >= self.start.as_usize() + bytes,
            "Attempting to retreat before the start of input"
        );
        self.curr = unsafe { self.curr.byte_sub(bytes) };
    }

    /// Return the byte at the current position and advance by one byte
    #[inline(always)]
    pub fn get_byte(&mut self) -> Option<u8> {
        if self.is_not_empty() {
            let byte = unsafe { self.curr.read() };
            Some(byte)
        } else {
            None
        }
    }

    /// Return the byte at the current position without advancing
    #[inline(always)]
    pub fn get_byte_unchecked(&self) -> u8 {
        debug_assert!(
            self.is_not_empty(),
            "Attempting to read byte from empty input"
        );
        unsafe { self.curr.read() }
    }

    /// Check if there are at least `bytes` bytes of capacity left in the source input
    #[inline(always)]
    pub fn reajust(&mut self) {
        if !self.is_not_empty() {
            self.curr = self.end;
        }
    }

    /// Begin a SIMD processing loop, applying the provided function `f` to each 16-byte
    /// chunk until the end of input or until `f` indicates to break
    #[inline(always)]
    pub fn simd_with<F: FnMut(wide::u8x16) -> SimdStep>(&mut self, mut f: F) -> bool {
        while self.can_simd() {
            match f(self.get_wide()) {
                SimdStep::Continue => self.advance_unchecked(16),
                SimdStep::SkipAndContinue(skip) => {
                    self.advance_unchecked(skip as usize);
                    continue;
                }
                SimdStep::SkipAndBreak(skip) => {
                    self.advance_unchecked(skip as usize);
                    return true;
                }
            }
        }
        false
    }

    /// Begin a SIMD processing loop with error handling, applying the provided function `f` to each 16-byte chunk until the end of input or until `f` indicates to break or returns an error
    #[inline(always)]
    pub fn simd_with_error<F: FnMut(&mut Self, wide::u8x16) -> Result<SimdStep, LexError>>(
        &mut self,
        mut f: F,
    ) -> Result<bool, LexError> {
        while self.can_simd() {
            match f(self, self.get_wide())? {
                SimdStep::Continue => self.advance_unchecked(16),
                SimdStep::SkipAndContinue(skip) => {
                    self.advance_unchecked(skip as usize);
                    continue;
                }
                SimdStep::SkipAndBreak(skip) => {
                    self.advance_unchecked(skip as usize);
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    /// Begin a scalar processing loop, applying the provided function `f` to each byte until the end of input or until `f` indicates to break
    #[inline(always)]
    pub fn scalar_with<F: FnMut(u8) -> ScalarStep>(&mut self, mut f: F) -> bool {
        while let Some(ch) = self.get_byte() {
            match f(ch) {
                ScalarStep::Continue => self.advance(1),
                ScalarStep::Break => return true,
                ScalarStep::BreakAndConsume => {
                    self.advance(1);
                    return true;
                }
            }
        }
        false
    }
    /// Begin a scalar processing loop with error handling, applying the provided function `f` to each byte until the end of input or until `f` indicates to break or returns an error
    #[inline(always)]
    pub fn scalar_with_error<F: FnMut(&mut Self, u8) -> Result<ScalarStep, LexError>>(
        &mut self,
        mut f: F,
    ) -> Result<bool, LexError> {
        while self.is_not_empty() {
            match f(self, self.get_byte_unchecked())? {
                ScalarStep::Continue => self.advance(1),
                ScalarStep::Break => return Ok(true),
                ScalarStep::BreakAndConsume => {
                    self.advance(1);
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }
    pub fn ignore_src_condition<S, F>(&mut self, simd_op: S, scalar_op: F) -> bool
    where
        S: Fn(wide::u8x16) -> wide::u8x16,
        F: Fn(u8) -> bool,
    {
        process_src_with_simd_fallback!(self,
            simd: |chunk| {
                let mask = simd_op(chunk).to_bitmask();
                if mask == 0xFFFF {
                    SimdStep::Continue
                } else {
                    let offset = mask.trailing_ones() as u8;
                    SimdStep::SkipAndBreak(offset)
                }
            },
            scalar: |byte| if scalar_op(byte) { ScalarStep::Continue } else { ScalarStep::Break }
        )
    }

    pub fn trim(&mut self) {
        ignore_src_chars!(self, b' ' | b'\t' | b'\r' | b'\0');
    }
    pub fn skip_current_line(&mut self) {
        self.ignore_src_condition(
            |chunk| chunk.simd_ne(wide::u8x16::splat(b'\n')),
            |byte| byte != b'\n',
        );
    }

    #[inline(always)]
    pub fn offset(&self, p: NonNull<u8>) -> u32 {
        debug_assert!(
            p.as_usize() >= self.start.as_usize(),
            "position to be calculated, which is less than the starting pointer"
        );
        (p.as_usize() - self.start.as_usize()) as u32
    }
    #[inline(always)]
    fn ptr_from_location(&self, l: Location) -> Option<NonNull<u8>> {
        //debug_assert!((self.start.as_usize() + l.0 as usize) < self.end().as_usize(), "location overfloow");
        let mut calc = self.start.as_usize() + l.0 as usize;
        if calc > self.end().as_usize() {
            return None;
        } else if (calc == self.end().as_usize()) & (self.start.as_usize() < self.end().as_usize())
        {
            calc -= 1;
        }
        Some(unsafe { NonNull::new_unchecked(calc as *mut u8) })
    }
    /// find the first line break or go back to display a 32-character snippet
    #[inline(always)]
    pub fn prevleft(&self, mut start: NonNull<u8>) -> (bool, u32, NonNull<u8>) {
        let mut left = 0;
        // move backwards in 16-byte blocks
        /*while start.as_usize() >= self.start.as_usize() + 16 {
            let chunk = unsafe { wide::u8x16::from(start.byte_sub(16).cast::<[u8; 16]>().read()) };
            // search for the first line break from right to left
            let mask = (chunk.simd_ne(wide::u8x16::splat(b'\n')).to_bitmask() as u16).reverse_bits().trailing_ones();
            if mask + left > 32 {
                let offset = 32 - left;
                start = unsafe { start.byte_sub(offset as usize) };
                return (true, 32, start);
            }else if mask < 16 {
                left += mask;
                start = unsafe { start.byte_sub(mask as usize) };
                return (false, left, start);
            }else{
                left += 16;
                start = unsafe { start.byte_sub(16) };
            }
        }*/
        while start.as_usize() >= self.start.as_usize() {
            let ch = unsafe { start.read() };
            if left == 32 {
                while start.as_usize() > self.start.as_usize()
                    && (unsafe { start.read() } & 0xC0) == 0x80
                {
                    start = unsafe { start.byte_sub(1) };
                }
                return (true, 32, start);
            } else if ch == b'\n' {
                left -= 1;
                start = unsafe { start.byte_add(1) };
                return (false, left, start);
            } else {
                if (ch & 0xC0) != 0x80 {
                    left += 1;
                }
            }
            start = unsafe { start.byte_sub(1) };
        }
        return (false, left, start);
    }
    pub fn gen_error(&self, line: u32, start: Location) -> Option<(u32, String)> {
        let curr = self.ptr_from_location(start)?;
        let mut src = Src {
            start: self.start,
            end: self.end,
            curr: curr,
            has_simd_padding: false,
            phantom: PhantomData,
        };
        let mut right = 0;
        let mut right_continue = false;
        /*process_src_with_simd_fallback!(src,
            simd: |chunk| {
                let mask = chunk.simd_ne(wide::u8x16::splat(b'\n')).to_bitmask().trailing_ones();
                if right + mask > 32 {
                    let offset = 32 - right;
                    right = 32;
                    right_continue = true;
                    SimdStep::SkipAndBreak(offset as u8)
                }else if mask < 16 {
                    right += mask;
                    SimdStep::SkipAndBreak(mask as u8)
                }else{
                    right += mask;
                    SimdStep::Continue
                }
            },
            scalar: |ch| {

            }
        );*/
        let _ = src.scalar_with(|ch| {
            if right == 32 {
                right_continue = true;
                return ScalarStep::Break;
            }
            if ch == b'\n' {
                ScalarStep::Break
            } else {
                if (ch & 0xC0) != 0x80 {
                    right += 1;
                }
                ScalarStep::Continue
            }
        });
        let end = src.curr();

        let (left_continue, mut left, mut start) = self.prevleft(unsafe { curr.byte_add(1) });
        // overfloow
        if start.as_usize() < self.start.as_usize() {
            start = self.start;
        }
        let mut string = String::new();
        let strn = line.to_string();
        left += (strn.len() as u32) + 3;
        string.push_str(&strn);
        string.push_str(" | ");
        if left_continue {
            string.push_str("...");
            left += 3;
        }
        let span = Span {
            start: Location((start.as_usize() - self.start.as_usize()) as u32),
            end: (end.as_usize() - self.start.as_usize()) as u32,
            line,
        };
        string.push_str(span.as_str_with_src_unchecked(self));

        if right_continue {
            string.push_str("...");
        }

        Some((left, string))
    }
}

/// Position in the source code, represented as a byte offset from the start of the input.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub struct Location(u32);

/// Range in the source code, defined by a starting `Location` and an ending byte offset, along with the line number for error reporting.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Span {
    start: Location,
    end: u32,
    line: u32,
}

impl Location {
    #[inline(always)]
    pub fn new(src: &Src) -> Self {
        let offset = src.curr.as_usize() - src.start.as_usize();
        Self(offset as u32)
    }
    #[inline(always)]
    pub fn zero() -> Self {
        Self(0)
    }
    /// Get the current byte offset as a `u32`
    #[inline(always)]
    pub fn get_offset(&self) -> u32 {
        self.0
    }
    /// Advance the location by `bytes` bytes
    #[inline(always)]
    pub unsafe fn advance(&mut self, bytes: u32) {
        self.0 += bytes;
    }
    /// Retreat the location by `bytes` bytes
    #[inline(always)]
    pub fn retreat(&mut self, bytes: u32) {
        self.0 -= bytes;
    }
}
impl Span {
    /// Create a new `Span` from a starting `Location` and the current position in the source, calculating the end offset and line number as needed.
    #[inline(always)]
    pub fn new(line: u32, start: Location, src: &Src) -> Self {
        let offset = src.curr.as_usize() - src.start.as_usize();
        debug_assert!(
            offset >= start.get_offset() as usize,
            "Span end offset must be greater than or equal to start offset"
        );
        debug_assert!(
            offset <= src.end.as_usize() - src.start.as_usize(),
            "Span end offset is out of bounds of the source input"
        );

        Self {
            start,
            end: offset as u32,
            line,
        }
    }
    /// Create a new `Span` from a starting `Location` and an explicit ending `Location`, calculating the end offset and line number as needed.
    #[inline(always)]
    pub fn new_from_end(line: u32, start: Location, end: Location) -> Self {
        debug_assert!(
            end.get_offset() >= start.get_offset(),
            "Span end offset must be greater than or equal to start offset"
        );

        Self {
            start,
            end: end.get_offset(),
            line,
        }
    }
    /// Get the length of the span in bytes
    #[inline(always)]
    pub fn len(&self) -> u32 {
        self.end - self.start.get_offset()
    }
    #[inline(always)]
    pub fn line(&self) -> u32 {
        self.line
    }

    #[inline(always)]
    pub fn get_start(&self) -> Location {
        self.start
    }

    #[inline(always)]
    pub fn get_end(&self) -> u32 {
        self.end
    }

    #[inline(always)]
    pub fn as_ptr(offset: u32, src: &Src) -> NonNull<u8> {
        unsafe { src.start.byte_add(offset as usize) }
    }

    #[inline(always)]
    pub fn is_valid_with_src(&self, src: &Src) -> bool {
        (self.start.get_offset() <= self.end) && (self.end <= src.end.as_usize() as u32)
    }

    ///
    #[inline(always)]
    pub fn is_valid(&self, lex: &Lexer) -> bool {
        self.is_valid_with_src(&lex.src)
    }
    /// Convert the `Span` into a string slice by calculating the start and end pointers based on the offsets and the original source.
    #[inline(always)]
    pub fn as_str_with_src_unchecked<'a>(&self, src: &'a Src) -> &'a str {
        let start_ptr = Self::as_ptr(self.start.get_offset(), src);
        let end_ptr = Self::as_ptr(self.end, src);
        debug_assert!(
            self.is_valid_with_src(src),
            "Span end offset is out of bounds of the source input"
        );

        let len = end_ptr.as_usize() - start_ptr.as_usize();
        unsafe {
            std::str::from_utf8_unchecked(std::slice::from_raw_parts(start_ptr.as_ptr(), len))
        }
    }

    /// Convert the `Span` into a string slice using the original source from the provided `Lexer`, which contains the `Src`. This is a convenience method to avoid having to pass the `Src` separately when you have access to the `Lexer`.
    #[inline(always)]
    pub fn as_str<'a>(&self, lexer: &'a Lexer) -> Option<&'a str> {
        if self.is_valid(lexer) {
            Some(self.as_str_with_src_unchecked(&lexer.src))
        } else {
            None
        }
    }

    #[inline(always)]
    pub unsafe fn as_str_unchecked<'a>(&self, lexer: &'a Lexer) -> &'a str {
        self.as_str_with_src_unchecked(&lexer.src)
    }
}

impl AsUsize for NonNull<u8> {
    #[inline(always)]
    fn as_usize(&self) -> usize {
        self.as_ptr() as usize
    }
}
