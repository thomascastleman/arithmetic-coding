use crate::alphabet::{Alphabet, Symbol};
use biterator::Bit::{self, One, Zero};
use std::iter::{once, repeat_n};

/// Errors that can occur while encoding
#[derive(thiserror::Error, Debug, PartialEq)]
pub enum EncodeError {
    #[error("Stream not terminated by EOF symbol")]
    UnterminatedStream,
}

/// Encoder Algorithm
/// Adapted from mathematicalmonk's ["Finite-precision arithmetic coding - Encoder"][1]
///
/// [1]: https://youtu.be/9vhbKiwjJo8?si=qXYFJEJ3o-Jx_ekp
///
/// ```text
/// Input Stream: x_1, ..., x_k, EOF
/// c: Vector of interval lower bounds
/// d: Vector of interval upper bounds
/// R: Sum of interval widths for all symbols
///
/// <-------------------------------------------------------- Initial
/// a = 0, b = whole, s = 0
/// for i = 1, ..., k+1      <------------------------------- TopOfSymbolLoop
///     w = b - a
///     b = a + round(w * d_x_i / R)
///     a = a + round(w * c_x_i / R)
///
///     while b < half or a > half:  <----------------------- TopOfRescaleLoop
///         if b < half:
///             emit 0 and s 1's     
///             s = 0,
///             a = 2 * a
///             b = 2 * b
///         elif a > half:
///             emit 1 and s 0's     
///             s = 0
///             a = 2 * (a - half)
///             b = 2 * (b - half)
///     while a > quarter and b < 3 * quarter:
///         s = s + 1
///         a = 2 * (a - quarter)
///         b = 2 * (b - quarter)
///
/// <-------------------------------------------------------- AfterSymbolLoop
/// s = s + 1
/// if a <= quarter:
///     emit 0 and s 1's
/// else:
///     emit 1 and s 0's
/// <-------------------------------------------------------- Final
/// ```
#[derive(Copy, Clone, PartialEq)]
enum EncoderState {
    /// The initial state, before encoding has begun
    Initial,
    /// State at the top of the loop iterating over input symbols.
    TopOfSymbolLoop,
    /// State at the top of the loop performing rescaling when the a-b interval
    /// is entirely below or above the midpoint.
    TopOfRescaleLoop,
    /// State after the loop iterating over input symbols has terminated.
    AfterSymbolLoop,
    /// Final state, when encoding has finished.
    Final,
}

use EncoderState::*;

pub struct EncoderOutput<'a, S, A, I, const BITS_OF_PRECISION: u32>
where
    S: Symbol,
    A: Alphabet<S = S>,
    I: Iterator<Item = S>,
{
    input: I,
    alphabet: &'a A,
    state: EncoderState,
    bits_to_emit: Option<Box<dyn Iterator<Item = Bit>>>,
    a: usize,
    b: usize,
    w: usize,
    s: usize,
    eof_reached: bool,
}

impl<'a, S, A, I, const BITS_OF_PRECISION: u32> EncoderOutput<'a, S, A, I, BITS_OF_PRECISION>
where
    S: Symbol,
    A: Alphabet<S = S>,
    I: Iterator<Item = S>,
{
    const WHOLE: usize = 2_usize.pow(BITS_OF_PRECISION);
    const HALF: usize = Self::WHOLE / 2;
    const QUARTER: usize = Self::WHOLE / 4;

    /// Construct a new EncoderOutput from an input stream of symbols and an alphabet.
    fn new(input: I, alphabet: &'a A) -> Self {
        EncoderOutput {
            input,
            alphabet,
            state: EncoderState::Initial,
            bits_to_emit: None,
            a: 0,
            b: 0,
            w: 0,
            s: 0,
            eof_reached: false,
        }
    }
    /// Determine the next bit in the encoded output. None indicates the end
    /// of the output.
    fn next_bit(&mut self) -> Option<Result<Bit, EncodeError>> {
        loop {
            // If there's a bit to emit next, emit it
            match self.bits_to_emit.as_mut().and_then(|bits| bits.next()) {
                Some(bit) => return Some(Ok(bit)),
                None => self.bits_to_emit = None,
            }

            // No more bits and final state reached: end of output
            if self.state == Final {
                return None;
            }

            // Move to the next state in the state machine
            match self.next_state() {
                Err(e) => return Some(Err(e)),
                Ok(next_state) => self.state = next_state,
            }
        }
    }

    fn next_state(&mut self) -> Result<EncoderState, EncodeError> {
        match self.state {
            Initial => self.execute_initial(),
            TopOfSymbolLoop => self.execute_top_of_symbol_loop(),
            TopOfRescaleLoop => self.execute_top_of_rescale_loop(),
            AfterSymbolLoop => self.execute_after_symbol_loop(),
            Final => Ok(Final),
        }
    }

    fn execute_initial(&mut self) -> Result<EncoderState, EncodeError> {
        self.a = 0;
        self.b = Self::WHOLE;
        self.s = 0;
        Ok(TopOfSymbolLoop)
    }

    fn execute_top_of_symbol_loop(&mut self) -> Result<EncoderState, EncodeError> {
        if self.eof_reached {
            return Ok(AfterSymbolLoop);
        }
        match self.input.next() {
            None => Err(EncodeError::UnterminatedStream),
            Some(symbol) => {
                if symbol == self.alphabet.eof() {
                    self.eof_reached = true;
                }
                self.set_a_and_b_for_symbol(&symbol);
                Ok(TopOfRescaleLoop)
            }
        }
    }

    fn execute_top_of_rescale_loop(&mut self) -> Result<EncoderState, EncodeError> {
        if self.b < Self::HALF {
            self.bits_to_emit = Some(self.zero_and_s_ones());
            self.s = 0;
            self.a *= 2;
            self.b *= 2;
            Ok(TopOfRescaleLoop)
        } else if self.a > Self::HALF {
            self.bits_to_emit = Some(self.one_and_s_zeros());
            self.s = 0;
            self.a = 2 * (self.a - Self::HALF);
            self.b = 2 * (self.b - Self::HALF);
            Ok(TopOfRescaleLoop)
        } else {
            self.perform_middle_rescaling();
            Ok(TopOfSymbolLoop)
        }
    }

    fn execute_after_symbol_loop(&mut self) -> Result<EncoderState, EncodeError> {
        self.s += 1;
        if self.a <= Self::QUARTER {
            self.bits_to_emit = Some(self.zero_and_s_ones());
        } else {
            self.bits_to_emit = Some(self.one_and_s_zeros());
        }

        Ok(Final)
    }

    fn set_a_and_b_for_symbol(&mut self, symbol: &S) {
        let total_interval_width = self.alphabet.total_interval_width();
        let upper_bound = self.alphabet.interval_upper_bound(symbol);
        let lower_bound = self.alphabet.interval_lower_bound(symbol);
        self.w = self.b - self.a;
        self.b = self.a + (self.w * upper_bound) / total_interval_width;
        self.a += (self.w * lower_bound) / total_interval_width;
    }

    fn one_and_s_zeros(&self) -> Box<dyn Iterator<Item = Bit>> {
        Box::new(once(One).chain(repeat_n(Zero, self.s)))
    }

    fn zero_and_s_ones(&self) -> Box<dyn Iterator<Item = Bit>> {
        Box::new(once(Zero).chain(repeat_n(One, self.s)))
    }

    fn perform_middle_rescaling(&mut self) {
        while self.a > Self::QUARTER && self.b < (3 * Self::QUARTER) {
            self.s += 1;
            self.a = 2 * (self.a - Self::QUARTER);
            self.b = 2 * (self.b - Self::QUARTER);
        }
    }
}

impl<'a, S: Symbol, A: Alphabet<S = S>, I: Iterator<Item = S>, const BITS_OF_PRECISION: u32>
    Iterator for EncoderOutput<'a, S, A, I, BITS_OF_PRECISION>
{
    type Item = Result<Bit, EncodeError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_bit()
    }
}

trait Encoder<S, A>
where
    S: Symbol,
    A: Alphabet<S = S>,
{
    /// Encode a stream of symbols as a stream of bits.
    ///
    /// The input stream must consist of symbols from the alphabet.
    /// This method will encode a single message from the stream (i.e. the
    /// symbols up until/including the EOF symbol).
    fn encode<I, const BITS_OF_PRECISION: u32>(
        &self,
        input: I,
    ) -> EncoderOutput<'_, S, A, I::IntoIter, BITS_OF_PRECISION>
    where
        I: IntoIterator<Item = S>;
}

impl<S, A> Encoder<S, A> for A
where
    S: Symbol,
    A: Alphabet<S = S>,
{
    fn encode<IntoI, const BITS_OF_PRECISION: u32>(
        &self,
        input: IntoI,
    ) -> EncoderOutput<'_, S, A, IntoI::IntoIter, BITS_OF_PRECISION>
    where
        IntoI: IntoIterator<Item = S>,
    {
        EncoderOutput::new(input.into_iter(), self)
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
        alphabet.encode::<_, BITS_OF_PRECISION>(input).collect()
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
