use std::panic::PanicHookInfo;

use crate::alphabet::{Alphabet, Symbol};
use biterator::Bit::{self, One, Zero};

struct Encoder<S, A, const BITS_OF_PRECISION: u32>
where
    S: Symbol,
    A: Alphabet<S = S>,
{
    alphabet: A,
}

/// Errors that can occur while encoding
#[derive(thiserror::Error, Debug, PartialEq)]
enum EncodeError {
    #[error("Stream not terminated by EOF symbol")]
    UnterminatedStream,
}

enum EncoderOutputState {
    /// The initial state, before encoding has begun
    Initial,
    /// State at the top of the loop iterating over input symbols.
    TopOfSymbolLoop,
    /// State at the top of the loop performing rescaling when the a-b interval
    /// is entirely below or above the midpoint.
    TopOfRescaleLoop,
    /// State when the a-b interval falls into the left half of the 0-whole
    /// interval, and needs to be scaled to straddle the midpoint.
    BLessThanHalf,
    /// State when the a-b interval falls into the right half of the 0-whole
    /// interval, and needs to be scaled to straddle the midpoint.
    AGreaterThanHalf,
    /// State after the loop iterating over input symbols has terminated.
    AfterSymbolLoop,
    /// State at end of algorithm, when the final a value is less than or equal
    /// to quarter
    ALessThanEqQuarter,
    /// State at the end of the algorithm, when the final a value is greater
    /// than quarter.
    AGreaterThanQuarter,
    /// Final state, when encoding has finished.
    Final,
}

struct EncoderOutput<'e, S, A, I, const BITS_OF_PRECISION: u32>
where
    S: Symbol,
    A: Alphabet<S = S>,
    I: Iterator<Item = S>,
{
    input: I,
    encoder: &'e Encoder<S, A, BITS_OF_PRECISION>,

    state: EncoderOutputState,
    a: usize,
    b: usize,
    w: usize,
    s: usize,
    eof_reached: bool,
}

impl<S: Symbol, A: Alphabet<S = S>, I: Iterator<Item = S>, const BITS_OF_PRECISION: u32>
    EncoderOutput<'_, S, A, I, BITS_OF_PRECISION>
{
    const WHOLE: usize = 2_usize.pow(BITS_OF_PRECISION);
    const HALF: usize = Self::WHOLE / 2;
    const QUARTER: usize = Self::WHOLE / 4;

    fn set_a_and_b_for_symbol(&mut self, symbol: &S) {
        let total_interval_width = self.encoder.alphabet.total_interval_width();
        let upper_bound = self.encoder.alphabet.interval_upper_bound(symbol);
        let lower_bound = self.encoder.alphabet.interval_lower_bound(symbol);
        self.w = self.b - self.a;
        self.b = self.a + (self.w * upper_bound) / total_interval_width;
        self.a = self.a + (self.w * lower_bound) / total_interval_width;
    }

    fn perform_middle_rescaling(&mut self) {
        while self.a > Self::QUARTER && self.b < (3 * Self::QUARTER) {
            self.s += 1;
            self.a = 2 * (self.a - Self::QUARTER);
            self.b = 2 * (self.b - Self::QUARTER);
        }
    }
}

// Encoder Algorithm
// Adapted from mathematicalmonk's "Finite-precision arithmetic coding - Encoder"
// https://youtu.be/9vhbKiwjJo8?si=qXYFJEJ3o-Jx_ekp
//
// Input Stream: x_1, ..., x_k, EOF
// c: Vector of interval lower bounds
// d: Vector of interval upper bounds
// R: Sum of interval widths for all symbols
//
// <-------------------------------------------------------- Initial
// a = 0, b = whole, s = 0
// for i = 1, ..., k+1      <------------------------------- TopOfSymbolLoop
//     w = b - a
//     b = a + round(w * d_x_i / R)
//     a = a + round(w * c_x_i / R)
//
//     while b < half or a > half:  <----------------------- TopOfRescaleLoop
//         if b < half:
//             emit 0 and s 1's     <----------------------- BLessThanHalf
//             s = 0,
//             a = 2 * a
//             b = 2 * b
//         elif a > half:
//             emit 1 and s 0's     <----------------------- AGreaterThanHalf
//             s = 0
//             a = 2 * (a - half)
//             b = 2 * (b - half)
//     while a > quarter and b < 3 * quarter:
//         s = s + 1
//         a = 2 * (a - quarter)
//         b = 2 * (b - quarter)
//
// <-------------------------------------------------------- AfterSymbolLoop
// s = s + 1
// if a <= quarter:
//     emit 0 and s 1's     <------------------------------- ALessThanEqQuarter
// else:
//     emit 1 and s 0's     <------------------------------- AGreaterThanQuarter
// <-------------------------------------------------------- Final
impl<'e, S: Symbol, A: Alphabet<S = S>, I: Iterator<Item = S>, const BITS_OF_PRECISION: u32>
    Iterator for EncoderOutput<'e, S, A, I, BITS_OF_PRECISION>
{
    type Item = Result<Bit, EncodeError>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.state {
                EncoderOutputState::Initial => {
                    self.a = 0;
                    self.b = Self::WHOLE;
                    self.s = 0;
                    self.state = EncoderOutputState::TopOfSymbolLoop;
                }
                EncoderOutputState::TopOfSymbolLoop => {
                    if self.eof_reached {
                        self.state = EncoderOutputState::AfterSymbolLoop;
                        continue;
                    }
                    match self.input.next() {
                        None => return Some(Err(EncodeError::UnterminatedStream)),
                        Some(symbol) => {
                            if symbol == self.encoder.alphabet.eof() {
                                self.eof_reached = true;
                            }
                            self.set_a_and_b_for_symbol(&symbol);
                            self.state = EncoderOutputState::TopOfRescaleLoop;
                        }
                    }
                }
                EncoderOutputState::TopOfRescaleLoop => {
                    if self.b < Self::HALF {
                        self.state = EncoderOutputState::BLessThanHalf;
                        return Some(Ok(Zero));
                    } else if self.a > Self::HALF {
                        self.state = EncoderOutputState::AGreaterThanHalf;
                        return Some(Ok(One));
                    } else {
                        self.perform_middle_rescaling();
                        self.state = EncoderOutputState::TopOfSymbolLoop;
                    }
                }
                EncoderOutputState::BLessThanHalf => {
                    if self.s > 0 {
                        self.s -= 1;
                        return Some(Ok(One));
                    } else {
                        self.a *= 2;
                        self.b *= 2;
                        self.state = EncoderOutputState::TopOfRescaleLoop;
                    }
                }
                EncoderOutputState::AGreaterThanHalf => {
                    if self.s > 0 {
                        self.s -= 1;
                        return Some(Ok(Zero));
                    } else {
                        self.a = 2 * (self.a - Self::HALF);
                        self.b = 2 * (self.b - Self::HALF);
                        self.state = EncoderOutputState::TopOfRescaleLoop;
                    }
                }
                EncoderOutputState::AfterSymbolLoop => {
                    self.s += 1;
                    if self.a <= Self::QUARTER {
                        self.state = EncoderOutputState::ALessThanEqQuarter;
                        return Some(Ok(Zero));
                    } else {
                        self.state = EncoderOutputState::AGreaterThanQuarter;
                        return Some(Ok(One));
                    }
                }
                EncoderOutputState::ALessThanEqQuarter => {
                    if self.s > 0 {
                        self.s -= 1;
                        return Some(Ok(One));
                    } else {
                        self.state = EncoderOutputState::Final;
                    }
                }
                EncoderOutputState::AGreaterThanQuarter => {
                    if self.s > 0 {
                        self.s -= 1;
                        return Some(Ok(Zero));
                    } else {
                        self.state = EncoderOutputState::Final;
                    }
                }
                EncoderOutputState::Final => return None,
            }
        }
    }
}

impl<S: Symbol, A: Alphabet<S = S>, const BITS_OF_PRECISION: u32> Encoder<S, A, BITS_OF_PRECISION> {
    /// Create an encoder capable of encoding input that comes from the given
    /// alphabet.
    pub fn new(alphabet: A) -> Self {
        Self { alphabet }
    }

    /// Encode a stream of symbols as a stream of bits.
    ///
    /// The input stream must consist of symbols from the encoder's alphabet.
    /// This method will encode a single message from the stream (i.e. the
    /// symbols up until/including the EOF symbol).
    pub fn encode<'e, I>(
        &'e self,
        input: I,
    ) -> EncoderOutput<'e, S, A, I::IntoIter, BITS_OF_PRECISION>
    where
        I: IntoIterator<Item = S>,
    {
        EncoderOutput {
            input: input.into_iter(),
            encoder: self,
            state: EncoderOutputState::Initial,
            a: 0,
            b: 0,
            w: 0,
            s: 0,
            eof_reached: false,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::example::{ExampleAlphabet, ExampleSymbol};
    use ExampleSymbol::*;
    use biterator::Bit::{One, Zero};

    // The below test cases assume 32-bit precision
    const BITS_OF_PRECISION: u32 = 32;

    /// Convenience function for encoding a vector of symbols using the example
    /// alphabet definition, and collecting the output into a single Result.
    fn encode(input: Vec<ExampleSymbol>) -> Result<Vec<Bit>, EncodeError> {
        let alphabet = ExampleAlphabet::new();
        let encoder: Encoder<_, _, BITS_OF_PRECISION> = Encoder::new(alphabet);
        encoder.encode(input).collect()
    }

    #[test]
    fn encode_empty_message() {
        assert_eq!(encode(vec![Eof]), Ok(vec![One, One, One, Zero, One]));
    }

    #[test]
    fn encode_very_small_message() {
        assert_eq!(
            encode(vec![C, Eof]),
            Ok(vec![One, One, One, Zero, Zero, One, Zero]),
        );
    }

    #[test]
    fn encode_small_message() {
        assert_eq!(
            encode(vec![B, A, C, Eof]),
            Ok(vec![Zero, One, Zero, One, One, One, Zero, Zero, One, Zero]),
        )
    }

    #[test]
    fn error_on_unterminated_stream() {
        assert_eq!(encode(vec![A, B, C]), Err(EncodeError::UnterminatedStream))
    }
}
