use crate::alphabet::{Alphabet, Symbol};
use biterator::Bit;

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
/// a = 0, b = whole, z = 0, i = 1
/// while i <= precision and i <= M:
///     if B_i == 1:
///         z = z + 2 ^ (precision - i)
///     i = i + 1
///
/// while True:
///     for j = 0, 1, ..., n:
///         w = b - a
///         b_0 = a + round(w * d_j / R)    
///         a_0 = a + round(w * c_j / R)    
///         if a_0 <= z < b_0:
///             emit j, a = a_0, b = b_0
///             if j == EOF:
///                 quit
///
///     while b < half or a > half:
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
/// ```
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
        DecoderOutput { input, alphabet }
    }

    fn next_event(&self) -> Option<Result<DecoderEvent<S>, DecodeError>> {
        todo!()
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
}
