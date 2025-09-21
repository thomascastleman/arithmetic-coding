use crate::alphabet::{Alphabet, Symbol};
use biterator::Bit;

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

struct EncoderOutput<'e, S, A, I, const BITS_OF_PRECISION: u32>
where
    S: Symbol,
    A: Alphabet<S = S>,
    I: Iterator<Item = S>,
{
    input: I,
    encoder: &'e Encoder<S, A, BITS_OF_PRECISION>,
}

impl<S: Symbol, A: Alphabet<S = S>, I: Iterator<Item = S>, const BITS_OF_PRECISION: u32>
    EncoderOutput<'_, S, A, I, BITS_OF_PRECISION>
{
    const WHOLE: usize = 2_usize.pow(BITS_OF_PRECISION);
    const HALF: usize = Self::WHOLE / 2;
    const QUARTER: usize = Self::WHOLE / 4;
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
// a = 0, b = whole, s = 0
// for i = 1, ..., k+1
//     w = b - a
//     b = a + round(w * d_x_i / R)
//     a = a + round(w * c_x_i / R)
//
//     while b < half or a > half:
//         if b < half:
//             emit 0 and s 1's
//             s = 0, a = 2*a, b = 2*b
//         elif a > half:
//             emit 1 and s 0's
//             s = 0, a = 2 * (a - half), b = 2 * (b - half)
//     while a > quarter and b < 3 * quarter:
//         s = s + 1
//         a = 2 * (a - quarter)
//         b = 2 * (b - quarter)
// s = s + 1
// if a <= quarter:
//     emit 0 and s 1's
// else:
//     emit 1 and s 0's
impl<'e, S: Symbol, A: Alphabet<S = S>, I: Iterator<Item = S>, const BITS_OF_PRECISION: u32>
    Iterator for EncoderOutput<'e, S, A, I, BITS_OF_PRECISION>
{
    type Item = Result<Bit, EncodeError>;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
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
