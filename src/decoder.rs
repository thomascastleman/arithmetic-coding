use crate::{Alphabet, Symbol};
use biterator::Bit;
struct Decoder<S, A>
where
    S: Symbol,
    A: Alphabet<S = S>,
{
    alphabet: A,
}

enum DecoderEvent<S: Symbol> {
    /// A symbol was decoded from the input stream.
    DecodedSymbol(S),
    /// Decoding of a single message is complete. The usize indicates how many
    /// bits of the input correspond to the decoded message.
    Done(usize),
}

impl<S: Symbol, A: Alphabet<S = S>> Decoder<S, A> {
    /// Create a decoder capable of decoding a stream of bits that was encoded
    /// using the given alphabet.
    pub fn new(alphabet: A) -> Self {
        Decoder { alphabet }
    }

    /// Decode a stream of bits as a stream of symbols.
    ///
    /// This method will decode a single "message", yielding all the decoded
    /// symbols (including the EOF symbol), and then indicating completion
    /// with the Done event.
    pub fn decode<I, O>(&self, input: &mut I) -> O
    where
        I: Iterator<Item = Bit>,
        O: Iterator<Item = DecoderEvent<S>>,
    {
        todo!()
    }
}
