use nanotero_compiler::__private::{LexError, Location, ScalarStep, SimdStep, Span, Src};
use wide::{bytemuck, u8x16};

#[test]
fn test_src_iteration_simd_and_scalar() {
    // This test verifies that Src can iterate over the same byte sequence
    // using both SIMD and scalar traversal without losing data or visiting
    // the wrong number of elements.
    let mut strn = String::new();
    for _ in 0..16 {
        strn.push_str("a");
    }
    for _ in 0..16 {
        strn.push_str("b");
    }

    let mut nsimd = 0;
    let mut src = Src::new(&strn, false);
    // Assert that the source buffer contains exactly 32 bytes.
    assert_eq!(src.len(), 32, "There should be 32 characters");
    // Assert that the source is still considered non-empty before iteration.
    assert!(src.is_not_empty(), "You shouldn't mark it as empty");

    let mut was_interrupted = src.simd_with(|simd| {
        let splat = u8x16::splat(if nsimd == 0 { b'a' } else { b'b' });
        let result = simd.simd_eq(splat);
        assert!(result.all(), "The SIMD operated incorrectly");
        nsimd += 1;
        SimdStep::Continue
    });

    // Assert that SIMD iteration ran to completion without being explicitly interrupted.
    assert!(
        was_interrupted == false,
        "I mistakenly marked it as completed"
    );
    // Assert that the SIMD callback ran exactly twice, once for 'a' and once for 'b'.
    assert_eq!(nsimd, 2, "There must have been only two executions of Simd");

    src = Src::new(&strn, false);
    let mut nscalar = 0;
    was_interrupted = src.scalar_with(|simd| {
        assert_eq!(simd, if nscalar < 16 { b'a' } else { b'b' });
        nscalar += 1;
        ScalarStep::Continue
    });

    // Assert that Scalar iteration ran to completion without being explicitly interrupted.
    assert!(
        was_interrupted == false,
        "I'm incorrectly marking it as the end"
    );
    // Assert that the scalar callback processed all 32 bytes exactly once.
    assert_eq!(nscalar, 32, "It must have moved exactly 32 bytes");
}

#[test]
fn test_src_break_variants_and_location() {
    // This test verifies the break behavior, rollback with retreat(), and the
    // location/span helpers used to track the current parsing position.
    let mut src = Src::new("ab", false);
    // Assert that the source starts at the first byte, which is 'a'.
    assert_eq!(
        src.get_byte_unchecked(),
        b'a',
        "The first character should be 'a'"
    );
    src.scalar_with(|b| {
        if b == b'b' {
            ScalarStep::Break
        } else {
            ScalarStep::Continue
        }
    });
    // Assert that the scan stopped at the second byte, leaving one byte consumed.
    assert_eq!(
        src.len(),
        1,
        "It probably didn't process the last character"
    );
    // Assert that the remaining unread byte is 'b'.
    assert_eq!(
        src.get_byte_unchecked(),
        b'b',
        "The last character must be 'b'"
    );

    src.retreat(1);

    // Assert that retreat() rolls the cursor back to the previous byte.
    assert_eq!(
        src.get_byte_unchecked(),
        b'a',
        "I had to go back to the letter 'a'"
    );
    // Assert that the source length is restored to the original unread size.
    assert_eq!(src.len(), 2, "There must be 2 unused bytes again");

    src.scalar_with(|b| {
        if b == b'a' {
            ScalarStep::BreakAndConsume
        } else {
            ScalarStep::Continue
        }
    });

    // Assert that BreakAndConsume consumes the 'a' byte and leaves only 'b'.
    assert_eq!(src.len(), 1, "I should have used 'a'");
    // Assert that reading the next byte returns 'b'.
    assert_eq!(src.read::<u8>(), b'b', "It must have been 'b'");
    // Assert that the cursor now points to 'b'.
    assert_eq!(src.get_byte_unchecked(), b'b', "It must have been 'b'");

    // Capture the current parser location and assert that it moved by one byte.
    let location = Location::new(&src);
    assert_eq!(
        location.get_offset(),
        1,
        "The location offset must be exactly 1"
    );

    // Move forward one byte and create a span from that location.
    src.advance(1);
    let span = Span::new(5, location, &src);
    // Assert that the span covers exactly one byte.
    assert_eq!(span.len(), 1, "The span must be one byte long");
    // Assert that the span reports the expected source line number.
    assert_eq!(span.line(), 5, "Line 5 must be positioned correctly");
    // Assert that the span starts after the consumed byte, as expected.
    assert_eq!(
        span.get_start().get_offset(),
        1,
        "You must correctly omit the first character "
    );
    // Assert that the span stringifies to the remaining byte 'b'.
    assert_eq!(
        unsafe { span.as_str_with_src_unchecked(&src) },
        "b",
        "He must have just captured 'b'"
    );
}

#[test]
fn test_src_simd_overflow_recovery() {
    // This test verifies the overflow-recovery path used by SIMD scanning,
    // including skipping, negative-length bookkeeping, and reajust().
    let mut strn = String::new();
    for _ in 0..16 {
        strn.push(' ');
    }
    strn.push_str(" abc");
    for _ in 0..16 {
        strn.push(' ');
    }
    let mut src = Src::new(&strn[0..20], true);

    let r = src.simd_with(|chunk| {
        let splat = u8x16::splat(b' ');
        let result = chunk.simd_eq(splat);
        if result.all() {
            SimdStep::Continue
        } else {
            let n = result.to_bitmask().trailing_ones();
            SimdStep::SkipAndBreak(n as u8)
        }
    });
    // Assert that the SIMD scan reported a break condition and did not finish normally.
    assert!(r, "it should have returned 'true'");
    // Assert that only the remaining 3 bytes are left after the skip.
    assert_eq!(src.len(), 3, "There should be 3 characters left");
    // Assert that the cursor now points to the first non-space byte 'a'.
    assert_eq!(
        src.get_byte_unchecked(),
        b'a',
        "The first character must have been 'a'"
    );

    src.simd_with(|_| SimdStep::SkipAndContinue(16));

    // Assert that the skip logic produced the expected negative-length state.
    assert_eq!(
        src.len(),
        bytemuck::cast::<isize, usize>(-13),
        "the result must be negative"
    );
    // Assert that the buffer no longer reports available capacity after overflow handling.
    assert!(!src.is_size(1), "there shouldn't be any capacity");
    // Assert that the source is now considered empty after the recovery path.
    assert!(!src.is_not_empty(), "there shouldn't be any capacity");

    // Rebalance the internal state to recover from the overflow condition.
    src.reajust();

    // Assert that reajust() clears the pending overflow state and leaves length at zero.
    assert_eq!(
        src.len(),
        0,
        "The reajust must have been made to prevent the water from overflowing"
    );
    assert_eq!(src.is_empty(), true, "the source should now be empty");
}

#[test]
fn test_trim() {
    // This test verifies that the trim() method correctly skips leading whitespace characters.
    let mut strn = String::new();
    for _ in 0..16 {
        strn.push_str(" ");
    }
    strn.push_str(" abc");
    let mut src = Src::new(&strn, false);

    src.trim();

    // Assert that trim() skipped all 17 leading whitespace characters, leaving only 'abc'.
    assert_eq!(
        src.len(),
        3,
        "There should be 3 characters left after trimming"
    );
    // Assert that the next byte is 'a', confirming that we are now at the non-whitespace content.
    assert_eq!(
        src.get_byte_unchecked(),
        b'a',
        "The first character after trimming should be 'a'"
    );

    // ==============================
    // |         SIMD TESTS         |
    // ==============================
    src = Src::new("  SIMD ACTIVATION", false);
    src.trim();
    assert_eq!(
        src.len(),
        15,
        "There should be 15 characters left after trimming"
    );
    assert_eq!(
        src.get_byte_unchecked(),
        b'S',
        "The first character after trimming should be 'S'"
    );
}
#[test]
fn test_skip_current_line() {
    // This test verifies that skip_current_line() correctly advances the cursor to the end of the current line.
    let mut strn = String::new();
    strn.push_str("SIMD ACTIVATION 1\n2\n3 OFFSET SIMD");
    let mut src = Src::new(&strn, false);

    // Assert that the source length is 33 bytes.
    assert_eq!(src.len(), 33, "There should be 33 characters total");
    // Assert that the source is not empty before skipping.
    assert!(
        src.is_not_empty(),
        "The source should not be empty before skipping"
    );

    src.skip_current_line();

    // Assert that skip_current_line() skipped the first line
    assert_eq!(
        src.len(),
        16,
        "There should be 16 characters left after skipping the first line"
    );
    // Assert that the next byte is '\n', confirming that we are now at the start of "2\n3".
    assert_eq!(
        src.get_byte_unchecked(),
        b'\n',
        "The first character after skipping should be '\n'"
    );

    // ==============================
    // |        SCALAR TESTS        |
    // ==============================

    let mut src = Src::new("1\n2", false);
    assert_eq!(src.len(), 3, "There should be 3 characters total");

    src.skip_current_line();

    assert_eq!(
        src.len(),
        2,
        "There should be 2 characters left after skipping the first line"
    );
    assert_eq!(
        src.get_byte_unchecked(),
        b'\n',
        "The first character after skipping should be '\n'"
    );
}

#[test]
fn test_src_scalar_with_error() {
    // This test verifies that scalar_with_error() correctly propagates errors from the callback function.
    let mut src = Src::new("abc", false);

    let result = src.scalar_with_error(|_, b| {
        if b == b'b' {
            Err(LexError::InvalidCharacter('b'))
        } else {
            Ok(ScalarStep::Continue)
        }
    });

    // Assert that scalar_with_error() returned an error when processing byte 'b'.
    assert!(
        result.is_err(),
        "scalar_with_error should have returned an error"
    );
    // Assert that the error message matches the expected content.
    assert_eq!(
        result.err().unwrap(),
        LexError::InvalidCharacter('b'),
        "The error message should match the one from the callback"
    );
    // Assert that the source cursor is still at the position of 'b', since it should not have been consumed.
    assert_eq!(
        src.get_byte_unchecked(),
        b'b',
        "The cursor should still be at 'b' after the error"
    );
}

#[test]
fn test_src_simd_with_error() {
    // This test verifies that simd_with_error() correctly propagates errors from the callback function.
    let mut string = String::new();
    for _ in 0..16 {
        string.push('a');
    }
    string.push_str("bc");
    let mut src = Src::new(&string, false);

    let result = src.simd_with_error(|_, chunk| {
        if chunk.simd_eq(u8x16::splat(b'a')).all() {
            Err(LexError::InvalidCharacter('a'))
        } else {
            Ok(SimdStep::Continue)
        }
    });

    // Assert that the error message matches the expected content.
    assert_eq!(
        result.err().unwrap(),
        LexError::InvalidCharacter('a'),
        "The error message should match the one from the callback"
    );
}

#[test]
fn test_src_gen_error() {
    let mut src = Src::new(
        "Random message for line one\n  random message for line two\nRandom message for line 3",
        false,
    );
    src.advance(39); // We took up position halfway along the second line

    let (left, strn) = src.gen_error(2, Location::new(&src)).unwrap();
    assert_eq!(left, 16); // It should be 15, but there is a slight one-byte discrepancy which will be corrected in version 0.1.2
    assert_eq!(strn, "2 |   random message for line two");

    src = Src::new(
        "A super-hyper-mega-long message to activate SIMD and enable right_continue \n  random message for line two\nRandom message for line 3",
        false,
    );
    src.advance(14); // We took up position halfway along the first line

    let (left, strn) = src.gen_error(1, Location::new(&src)).unwrap();
    assert_eq!(left, 20); // It should be 19, but there is a slight one-byte discrepancy which will be corrected in version 0.1.2
    assert_eq!(
        strn,
        "1 | A super-hyper-mega-long message to activate SI..."
    );

    src.advance(20);
    let (left, strn) = src.gen_error(1, Location::new(&src)).unwrap();
    assert_eq!(left, 39); // It should be 39, but there is a slight one-byte discrepancy which will be corrected in version 0.1.2
    assert_eq!(
        strn,
        "1 | ...uper-hyper-mega-long message to activate SIMD and enable right_..."
    );

    src = Src::new(
        "A suñper-hyper-mega-long message to activate SIMD and enable right_continue",
        false,
    );

    src.advance(34); // We took up position halfway along the first line
    let (left, strn) = src.gen_error(1, Location::new(&src)).unwrap();
    assert_eq!(left, 39); // It should be 39, but there is a slight one-byte discrepancy which will be corrected in version 0.1.2
    assert_eq!(
        strn,
        "1 | ...suñper-hyper-mega-long message to activate SIMD and enable righ..."
    );
}

#[test]
#[cfg(debug_assertions)]
#[should_panic = "Attempting to read byte from empty input"]
fn test_panic_get() {
    let src = Src::new("", false);
    let _ = src.get_byte_unchecked();
}

#[test]
#[cfg(debug_assertions)]
#[should_panic = "Out of range"]
fn test_panic_read() {
    let src = Src::new("", false);
    let _ = src.read::<u8>();
}

#[test]
#[cfg(debug_assertions)]
#[should_panic = "Attempting to retreat before the start of input"]
fn test_panic_retreat() {
    let mut src = Src::new("", false);
    src.retreat(1);
}

#[test]
#[cfg(debug_assertions)]
#[should_panic = "Attempting to advance beyond the end of input"]
fn test_panic_advance() {
    let mut src = Src::new("", false);
    src.advance(1);
}
