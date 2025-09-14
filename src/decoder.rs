use std::marker::PhantomData;

use crate::alphabet::{Alphabet, Symbol};
use biterator::Bit;
struct Decoder<S, A, const BITS_OF_PRECISION: u32>
where
    S: Symbol,
    A: Alphabet<S = S>,
{
    alphabet: A,
}

#[derive(PartialEq, Debug)]
enum DecoderEvent<S: Symbol> {
    /// A symbol was decoded from the input stream.
    DecodedSymbol(S),
    /// Decoding of a single message is complete. The usize indicates how many
    /// bits of the input correspond to the decoded message.
    Done(usize),
}

struct DecoderOutput<S> {
    _marker: PhantomData<S>,
}

impl<S> Iterator for DecoderOutput<S>
where
    S: Symbol,
{
    type Item = DecoderEvent<S>;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

impl<S: Symbol, A: Alphabet<S = S>, const BITS_OF_PRECISION: u32> Decoder<S, A, BITS_OF_PRECISION> {
    const WHOLE: usize = 2_usize.pow(BITS_OF_PRECISION);
    const HALF: usize = Self::WHOLE / 2;
    const QUARTER: usize = Self::WHOLE / 4;

    /// Create a decoder capable of decoding a stream of bits that was encoded
    /// using the given alphabet.
    pub fn new(alphabet: A) -> Self {
        Decoder { alphabet }
    }

    /// Decode a stream of bits as a stream of symbols.
    ///
    /// This method will decode a single message, yielding all the decoded
    /// symbols (including the EOF symbol), and then indicating completion
    /// with the Done event.
    pub fn decode<I>(&self, input: &mut I) -> DecoderOutput<S>
    where
        I: Iterator<Item = Bit>,
    {
        todo!()
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

    fn assert_decoding_matches(input: Vec<Bit>, expected_output: Vec<DecoderEvent<ExampleSymbol>>) {
        let alphabet = ExampleAlphabet::new();
        let decoder: Decoder<_, _, BITS_OF_PRECISION> = Decoder::new(alphabet);
        let mut input_iter = input.into_iter();

        let output: Vec<_> = decoder.decode(&mut input_iter).collect();

        assert_eq!(output, expected_output);
    }

    #[test]
    fn decode_empty_message() {
        assert_decoding_matches(
            vec![One, One, One, Zero, One],
            vec![DecodedSymbol(Eof), Done(5)],
        );
    }

    #[test]
    fn decode_very_small_message() {
        assert_decoding_matches(
            vec![One, One, One, Zero, Zero, One, Zero],
            vec![DecodedSymbol(C), DecodedSymbol(Eof), Done(7)],
        );
    }

    #[test]
    fn decode_small_message() {
        assert_decoding_matches(
            vec![Zero, One, Zero, One, One, One, Zero, Zero, One, Zero],
            vec![
                DecodedSymbol(B),
                DecodedSymbol(A),
                DecodedSymbol(C),
                DecodedSymbol(Eof),
                Done(10),
            ],
        );
    }
}
