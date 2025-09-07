use std::marker::PhantomData;

use crate::alphabet::{Alphabet, Symbol};
use biterator::Bit;
struct Decoder<S, A>
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

impl<S: Symbol, A: Alphabet<S = S>> Decoder<S, A> {
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

    #[test]
    fn test_small_message() {
        let alphabet = ExampleAlphabet::new();
        let decoder = Decoder::new(alphabet);
        let bits = vec![Zero, One, Zero, One, One, One, Zero, Zero, One, Zero];
        let input_size = bits.len();
        let mut input = bits.into_iter();

        let output: Vec<_> = decoder.decode(&mut input).collect();

        assert_eq!(
            output,
            vec![
                DecodedSymbol(B),
                DecodedSymbol(A),
                DecodedSymbol(C),
                DecodedSymbol(Eof),
                Done(input_size),
            ]
        );
    }
}
