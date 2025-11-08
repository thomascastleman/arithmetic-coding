use crate::alphabet::{Alphabet, Symbol};
use biterator::Bit::{self, One, Zero};

/// Decoder Algorithm
/// Adapted from mathematicalmonk's ["Finite-precision arithmetic coding - Decoder"][1]
///
/// [1]: https://youtu.be/RFPwltOz1IU?si=1mhwNwfUjXGRrV-Y
///
/// ```text
/// Input Bitstream: B_1, ..., B_m, B_m+1, ..., B_M
/// c: Vector of interval lower bounds
/// d: Vector of interval upper bounds
/// R: Sum of interval widths for all symbols
///
/// <-------------------------------------------------------- Initial
/// a = 0, b = whole, z = 0, i = 1
/// while i <= precision and i <= M:
///     if B_i == 1:
///         z = z + 2 ^ (precision - i)
///     i = i + 1
///
/// while True:
///     for j = 0, 1, ..., n: <------------------------------ TopOfSymbolLoop
///         w = b - a
///         b_0 = a + round(w * d_j / R)    
///         a_0 = a + round(w * c_j / R)    
///         if a_0 <= z < b_0:
///             emit j, a = a_0, b = b_0
///             if j == EOF:
///                 quit
///             else:
///                 break
///
///     while b < half or a > half: <------------------------ Rescaling
///         if b < half:
///             a = 2 * a
///             b = 2 * b
///             z = 2 * z
///         elif a > half:
///             a = 2 * (a - half)
///             b = 2 * (b - half)
///             z = 2 * (z - half)
///         if i <= M and B_i == 1:
///             z = z + 1
///         i = i + 1
///
///     while a > quarter and b < 3 * quarter:
///         a = 2 * (a - quarter)
///         b = 2 * (b - quarter)
///         z = 2 * (z - quarter)
///         if i <= M and B_i == 1:
///             z = z + 1
///         i = i + 1
/// <-------------------------------------------------------- CalculateLength
/// <-------------------------------------------------------- Final
/// ```
#[derive(PartialEq)]
enum DecoderState {
    Initial,
    TopOfSymbolLoop,
    Rescaling,
    CalculateLength,
    Final,
}

use DecoderState::*;

#[derive(PartialEq, Debug)]
enum DecoderEvent<S: Symbol> {
    /// A symbol was decoded from the input stream.
    DecodedSymbol(S),
    /// Decoding of a single message is complete. The usize indicates how many
    /// bits of the input correspond to the decoded message.
    Done(usize),
}

/// Errors that can occur while decoding
#[derive(thiserror::Error, Debug, PartialEq)]
pub enum DecodeError {
    // TODO(tcastleman) This error condition is not valid, because every bit
    // stream will decode to *some* stream of symbols. There is no way to detect
    // that we ran out of bits while decoding because the decoder input is just
    // a number and any number is a valid encoding of some stream.
    //
    // Remove this variant and probably the entire enum.
    #[error("Unexpected end of bit stream before EOF decoded")]
    UnterminatedStream,
}

struct DecoderOutput<'a, S, A, I, const BITS_OF_PRECISION: u32>
where
    S: Symbol,
    A: Alphabet<S = S>,
    I: Iterator<Item = Bit>,
{
    input: I,
    alphabet: &'a A,
    state: DecoderState,
    event_to_emit: Option<DecoderEvent<S>>,
    a: usize,
    b: usize,
    z: usize,
    z_rescale_counter: usize,
}

impl<S, A, I, const BITS_OF_PRECISION: u32> Iterator
    for DecoderOutput<'_, S, A, I, BITS_OF_PRECISION>
where
    S: Symbol,
    A: Alphabet<S = S>,
    I: Iterator<Item = Bit>,
{
    type Item = Result<DecoderEvent<S>, DecodeError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_event()
    }
}

impl<'a, S, A, I, const BITS_OF_PRECISION: u32> DecoderOutput<'a, S, A, I, BITS_OF_PRECISION>
where
    S: Symbol,
    A: Alphabet<S = S>,
    I: Iterator<Item = Bit>,
{
    const WHOLE: usize = 2_usize.pow(BITS_OF_PRECISION);
    const HALF: usize = Self::WHOLE / 2;
    const QUARTER: usize = Self::WHOLE / 4;

    fn new(input: I, alphabet: &'a A) -> Self {
        DecoderOutput {
            input,
            alphabet,
            state: Initial,
            event_to_emit: None,
            a: 0,
            b: 0,
            z: 0,
            z_rescale_counter: 0,
        }
    }

    fn next_event(&mut self) -> Option<Result<DecoderEvent<S>, DecodeError>> {
        loop {
            if let Some(event) = self.event_to_emit.take() {
                return Some(Ok(event));
            }

            if self.state == Final {
                return None;
            }

            match self.execute() {
                Err(e) => return Some(Err(e)),
                Ok(next_state) => self.state = next_state,
            };
        }
    }

    fn execute(&mut self) -> Result<DecoderState, DecodeError> {
        match self.state {
            Initial => self.execute_initial(),
            Rescaling => self.execute_rescaling(),
            TopOfSymbolLoop => self.execute_top_of_symbol_loop(),
            CalculateLength => self.execute_calculate_length(),
            Final => Ok(Final),
        }
    }

    fn execute_initial(&mut self) -> Result<DecoderState, DecodeError> {
        self.a = 0;
        self.b = Self::WHOLE;
        self.initialize_z();
        Ok(TopOfSymbolLoop)
    }

    fn initialize_z(&mut self) {
        self.z = 0;
        for i in 1..BITS_OF_PRECISION {
            match self.input.next() {
                None => break,
                Some(Zero) => continue,
                Some(One) => self.z += 2usize.pow(BITS_OF_PRECISION - i),
            }
        }
    }

    fn execute_top_of_symbol_loop(&mut self) -> Result<DecoderState, DecodeError> {
        for symbol in self.alphabet.symbols() {
            let (sub_a, sub_b) = self.subinterval_for_symbol(symbol);

            if (sub_a..sub_b).contains(&self.z) {
                self.event_to_emit = Some(DecoderEvent::DecodedSymbol(*symbol));
                self.a = sub_a;
                self.b = sub_b;

                if *symbol == self.alphabet.eof() {
                    return Ok(CalculateLength);
                } else {
                    return Ok(Rescaling);
                }
            }
        }

        unreachable!("z must be within [a, b), so some subinterval contains it")
    }

    fn subinterval_for_symbol(&self, symbol: &S) -> (usize, usize) {
        let total_interval_width = self.alphabet.total_interval_width();
        let upper_bound = self.alphabet.interval_upper_bound(symbol);
        let lower_bound = self.alphabet.interval_lower_bound(symbol);
        let w = self.b - self.a;
        let sub_b = self.a + (w * upper_bound) / total_interval_width;
        let sub_a = self.a + (w * lower_bound) / total_interval_width;
        (sub_a, sub_b)
    }

    fn execute_rescaling(&mut self) -> Result<DecoderState, DecodeError> {
        self.side_rescaling();
        self.middle_rescaling();
        Ok(TopOfSymbolLoop)
    }

    fn side_rescaling(&mut self) {
        while self.b < Self::HALF || self.a > Self::HALF {
            if self.b < Self::HALF {
                self.a *= 2;
                self.b *= 2;
                self.z *= 2;
            } else if self.a > Self::HALF {
                self.a = 2 * (self.a - Self::HALF);
                self.b = 2 * (self.b - Self::HALF);
                self.z = 2 * (self.z - Self::HALF);
            }

            self.add_next_bit_to_z();
        }
    }

    fn middle_rescaling(&mut self) {
        while self.a > Self::QUARTER && self.b < 3 * Self::QUARTER {
            self.a = 2 * (self.a - Self::QUARTER);
            self.b = 2 * (self.b - Self::QUARTER);
            self.z = 2 * (self.z - Self::QUARTER);
            self.add_next_bit_to_z();
        }
    }

    fn add_next_bit_to_z(&mut self) {
        self.z_rescale_counter += 1;
        if let Some(One) = self.input.next() {
            self.z += 1;
        }
    }

    fn execute_calculate_length(&mut self) -> Result<DecoderState, DecodeError> {
        // Determine the number of bits that were used to encode the message that
        // was just decoded. We do this by determining the number of bits of z
        // that are necessary for unambiguously indicating the [a, b) interval,
        // and add this to the number of bits of z that we've already discarded
        // via rescaling.
        let encoded_message_length = self.minimal_z_prefix_size() as usize + self.z_rescale_counter;
        self.event_to_emit = Some(DecoderEvent::Done(encoded_message_length));
        Ok(Final)
    }

    /// Find the size of the smallest prefix of z that describes an interval
    /// contained in [a, b).
    fn minimal_z_prefix_size(&self) -> u32 {
        for bit_position in (0..BITS_OF_PRECISION).rev() {
            // Generate a mask for the N most significant bits of z
            let prefix_size = BITS_OF_PRECISION - bit_position;
            let prefix_mask: usize = !((1 << bit_position) - 1);

            let lower_bound = self.z & prefix_mask;
            let upper_bound = (self.z & prefix_mask) | !prefix_mask;

            if lower_bound >= self.a && upper_bound < self.b {
                return prefix_size;
            }
        }

        unreachable!(
            "z must be within [a, b), so at minimum the prefix containing all of z is within [a, b)"
        );
    }
}

trait Decoder<S, A>
where
    S: Symbol,
    A: Alphabet<S = S>,
{
    /// Decode a stream of bits as a stream of symbols.
    ///
    /// This method will decode a single message, yielding all the decoded
    /// symbols (including the EOF symbol), and then indicating completion
    /// with the Done event.
    fn decode<IntoI, const BITS_OF_PRECISION: u32>(
        &self,
        input: IntoI,
    ) -> DecoderOutput<'_, S, A, IntoI::IntoIter, BITS_OF_PRECISION>
    where
        IntoI: IntoIterator<Item = Bit>;
}

impl<S, A> Decoder<S, A> for A
where
    S: Symbol,
    A: Alphabet<S = S>,
{
    fn decode<IntoI, const BITS_OF_PRECISION: u32>(
        &self,
        input: IntoI,
    ) -> DecoderOutput<'_, S, A, IntoI::IntoIter, BITS_OF_PRECISION>
    where
        IntoI: IntoIterator<Item = Bit>,
    {
        DecoderOutput::new(input.into_iter(), self)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::example::{ExampleAlphabet, ExampleSymbol};
    use DecoderEvent::*;
    use ExampleSymbol::*;
    use biterator::Bit::{One, Zero};

    // The below test cases assume 32-bit precision
    const BITS_OF_PRECISION: u32 = 32;

    // TODO(tcastleman) Tests where size of encoding < precision

    fn decode(input: Vec<Bit>) -> Result<Vec<DecoderEvent<ExampleSymbol>>, DecodeError> {
        let alphabet = ExampleAlphabet::new();
        alphabet.decode::<_, BITS_OF_PRECISION>(input).collect()
    }

    #[test]
    fn decode_empty_message() {
        assert_eq!(
            decode(vec![One, One, One, Zero, One]),
            Ok(vec![DecodedSymbol(Eof), Done(5)]),
        );
    }

    #[test]
    fn decode_very_small_message() {
        assert_eq!(
            decode(vec![One, One, One, Zero, Zero, One, Zero]),
            Ok(vec![DecodedSymbol(C), DecodedSymbol(Eof), Done(7)]),
        );
    }

    #[test]
    fn decode_small_message() {
        assert_eq!(
            decode(vec![Zero, One, Zero, One, One, One, Zero, Zero, One, Zero]),
            Ok(vec![
                DecodedSymbol(B),
                DecodedSymbol(A),
                DecodedSymbol(C),
                DecodedSymbol(Eof),
                Done(10),
            ]),
        );
    }

    #[test]
    fn decode_message_with_middle_rescaling() {
        assert_eq!(
            decode(vec![One, Zero, Zero, One, One, One]),
            Ok(vec![
                DecodedSymbol(B),
                DecodedSymbol(B),
                DecodedSymbol(Eof),
                Done(6),
            ]),
        );
    }

    #[test]
    #[rustfmt::skip]
    fn decodes_single_message() {
        // Even if the input stream contains multiple messages (terminated by
        // EOF), a call to decode decodes only the first one.
        assert_eq!(
            decode(vec![
                // First message: C, Eof
                One, One, One, Zero, Zero, One, Zero, 
                // Second message: B, A, C, Eof
                Zero, One, Zero, One, One, One, Zero, Zero, One, Zero
            ]),
            Ok(vec![DecodedSymbol(C), DecodedSymbol(Eof), Done(7)]),
        )
    }
}
