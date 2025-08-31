use crate::{Alphabet, Symbol};
use biterator::Bit;

struct Encoder<S, A>
where
    S: Symbol,
    A: Alphabet<S = S>,
{
    alphabet: A,
}

impl<S: Symbol, A: Alphabet<S = S>> Encoder<S, A> {
    /// Create an encoder capable of encoding input that comes from the given
    /// alphabet.
    pub fn new(alphabet: A) -> Self {
        Self { alphabet }
    }

    /// Encode a stream of symbols as a stream of bits.
    pub fn encode<I, O>(&self, input: &mut I) -> O
    where
        I: Iterator<Item = S>,
        O: Iterator<Item = Bit>,
    {
        todo!()
    }
}
