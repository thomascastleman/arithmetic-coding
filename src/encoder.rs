use crate::alphabet::{Alphabet, Symbol};
use biterator::Bit;

struct Encoder<S, A>
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

impl<S: Symbol, A: Alphabet<S = S>> Encoder<S, A> {
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

    #[test]
    fn test_small_message() {
        let alphabet = ExampleAlphabet::new();
        let encoder = Encoder::new(alphabet);
        let mut input = vec![B, A, C, Eof].into_iter();
        let output: Vec<_> = encoder.encode(&mut input).collect();

        // TODO Manually encode, put the bits here
        assert_eq!(output, vec![])
    }
}
