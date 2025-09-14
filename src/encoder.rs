use crate::alphabet::{Alphabet, Symbol};
use biterator::Bit;

struct Encoder<S, A, const BITS_OF_PRECISION: u32>
where
    S: Symbol,
    A: Alphabet<S = S>,
{
    alphabet: A,
}

struct EncoderOutput;

impl Iterator for EncoderOutput {
    type Item = Bit;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

impl<S: Symbol, A: Alphabet<S = S>, const BITS_OF_PRECISION: u32> Encoder<S, A, BITS_OF_PRECISION> {
    const WHOLE: usize = 2_usize.pow(BITS_OF_PRECISION);
    const HALF: usize = Self::WHOLE / 2;
    const QUARTER: usize = Self::WHOLE / 4;

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
    pub fn encode<I>(&self, input: &mut I) -> EncoderOutput
    where
        I: Iterator<Item = S>,
    {
        todo!()
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

    fn assert_encoding_matches(input: Vec<ExampleSymbol>, expected_output: Vec<Bit>) {
        let alphabet = ExampleAlphabet::new();
        let encoder: Encoder<_, _, BITS_OF_PRECISION> = Encoder::new(alphabet);
        let mut input_iter = input.into_iter();

        let output: Vec<_> = encoder.encode(&mut input_iter).collect();

        assert_eq!(output, expected_output)
    }

    #[test]
    fn encode_empty_message() {
        assert_encoding_matches(vec![Eof], vec![One, One, One, Zero, One]);
    }

    #[test]
    fn encode_very_small_message() {
        assert_encoding_matches(vec![C, Eof], vec![One, One, One, Zero, Zero, One, Zero]);
    }

    #[test]
    fn encode_small_message() {
        assert_encoding_matches(
            vec![B, A, C, Eof],
            vec![Zero, One, Zero, One, One, One, Zero, Zero, One, Zero],
        )
    }
}
