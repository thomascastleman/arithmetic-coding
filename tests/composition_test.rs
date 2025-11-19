use arithmetic_coding::alphabet::{Alphabet, Symbol};
use arithmetic_coding::decoder::{Decoder, DecoderEvent};
use arithmetic_coding::encoder::{EncodeError, Encoder};
use biterator::Bit;
use quickcheck::{Arbitrary, Gen};
use quickcheck_macros::quickcheck;
use rand::Rng;
use test_log::test;

// TODO(tcastleman) Tests where size of encoding > precision

/// A symbol type that wraps an integer, so we can easily generate
/// arbitrary-sized alphabets composed of these symbols. EOF is represented
/// by the value zero.
#[derive(Debug, PartialEq, Copy, Clone)]
struct NumSymbol(usize);

impl Symbol for NumSymbol {}

impl NumSymbol {
    fn eof() -> Self {
        NumSymbol(0)
    }
}

#[derive(Debug, Clone)]
struct NumAlphabet {
    symbols: Vec<NumSymbol>,
    interval_widths: Vec<usize>,
}

impl NumAlphabet {
    fn new(interval_widths: Vec<usize>) -> Self {
        assert!(
            !interval_widths.is_empty(),
            "Alphabet must have at least one symbol"
        );
        assert!(
            interval_widths.iter().all(|&width| width > 0),
            "Interval widths must be >0"
        );
        Self {
            symbols: (0..interval_widths.len()).map(NumSymbol).collect(),
            interval_widths,
        }
    }

    /// Generate a stream of random symbols of the indicated length, terminated
    /// by the EOF symbol.
    fn random_symbol_stream(&self, length: usize) -> Vec<NumSymbol> {
        let max = self.symbols.len();

        (0..length)
            .map(|_| NumSymbol(rand::rng().random_range(1..max)))
            .chain(std::iter::once(NumSymbol::eof()))
            .collect()
    }
}

impl Alphabet for NumAlphabet {
    type S = NumSymbol;

    fn symbols(&self) -> impl Iterator<Item = &Self::S> {
        self.symbols.iter()
    }

    fn eof(&self) -> Self::S {
        NumSymbol::eof()
    }

    fn interval_width(&self, symbol: &Self::S) -> usize {
        for (s, &interval) in self.symbols.iter().zip(self.interval_widths.iter()) {
            if symbol == s {
                return interval;
            }
        }

        panic!("Symbol {symbol:?} not in alphabet {self:?}")
    }
}

struct ShrinkingNumAlphabet {
    alphabet: NumAlphabet,
}

impl Iterator for ShrinkingNumAlphabet {
    type Item = NumAlphabet;

    fn next(&mut self) -> Option<Self::Item> {
        // Stop removing symbols when only EOF and one other symbol is left,
        // because this is the smallest alphabet that can support arbitrary
        // length input streams.
        if self.alphabet.symbols.len() <= 2 {
            return None;
        }

        self.alphabet.symbols.pop();
        self.alphabet.interval_widths.pop();

        Some(self.alphabet.clone())
    }
}

const MAX_BITS_OF_PRECISION: u32 = 32;

impl Arbitrary for NumAlphabet {
    fn arbitrary(g: &mut Gen) -> Self {
        let mut interval_widths = Vec::arbitrary(g);

        // Ensure that at least two interval widths are present, to guarantee
        // that there is room for 1) an EOF symbol, and 2) a non-EOF symbol.
        // This is the smallest possible alphabet for which we can generate
        // arbitrary-length random input sequences. An alphabet consisting
        // only of EOF isn't capable of this because EOF immediately terminates
        // the input.
        interval_widths.push(usize::arbitrary(g));
        interval_widths.push(usize::arbitrary(g));

        // Due to the calculations done in the encoder/decoder to determine
        // subintervals, there are constraints on how large R (the sum of all
        // interval widths) can be.
        //
        // Specifically, we must be able to represent 2^precision * R as a usize.
        // i.e. 2^precision * R <= usize::MAX
        //                    R <= usize::MAX / 2^precision
        let max_total_width = usize::MAX / 2usize.pow(MAX_BITS_OF_PRECISION);
        let max_width = max_total_width / interval_widths.len();

        for width in &mut interval_widths {
            // Ensure all widths are greater than 0
            if *width == 0 {
                *width = 1;
            }

            // Ensure the interval widths sum to a suitably small R value (see above)
            *width %= max_width;
        }

        NumAlphabet::new(interval_widths)
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new(ShrinkingNumAlphabet {
            alphabet: self.clone(),
        })
    }
}

const BITS_OF_PRECISION: u32 = 32;

fn encode(alphabet: &NumAlphabet, input: Vec<NumSymbol>) -> Vec<Bit> {
    let encoder_result: Result<Vec<_>, EncodeError> =
        alphabet.encode::<_, BITS_OF_PRECISION>(input).collect();
    encoder_result.expect("Encoding failed")
}

struct Decoded {
    symbols: Vec<NumSymbol>,
    length: Option<usize>,
}

fn decode(alphabet: &NumAlphabet, bits: Vec<Bit>) -> Decoded {
    let decoder_events = alphabet.decode::<_, BITS_OF_PRECISION>(bits);
    let mut symbols = Vec::new();
    let mut length = None;

    for event in decoder_events {
        match event {
            DecoderEvent::DecodedSymbol(symbol) => symbols.push(symbol),
            DecoderEvent::MessageLength(message_length) => length = Some(message_length),
        };
    }

    Decoded { symbols, length }
}

/// Property test verifying that decoding an encoded stream of symbols results
/// in the same stream of symbols.
#[quickcheck]
fn encoder_and_decoder_cancel(alphabet: NumAlphabet, input_length: u8) -> bool {
    let input = alphabet.random_symbol_stream(input_length as usize);
    let expected_output = input.clone();

    let bits = encode(&alphabet, input);
    let decoded = decode(&alphabet, bits);

    decoded.symbols == expected_output
}

/// Property test verifying that the decoder correctly calculates the number of
/// bits from the input that comprise the decoded message.
#[quickcheck]
fn decoder_calculates_length(alphabet: NumAlphabet, input_length: u8) -> bool {
    let input = alphabet.random_symbol_stream(input_length as usize);

    let bits = encode(&alphabet, input);
    let encoding_length = bits.len();

    // TODO(tcastleman) Add random bits after the encoding
    let decoded = decode(&alphabet, bits);

    decoded.length == Some(encoding_length)
}

#[test]
fn minimal_failure() {
    let alphabet = NumAlphabet::new(vec![47549061, 9539461]);
    let input: Vec<_> = std::iter::repeat_n(NumSymbol(1), 13)
        .chain(std::iter::once(NumSymbol::eof()))
        .collect();
    let expected_output = input.clone();

    let bits = encode(&alphabet, input);
    let decoded = decode(&alphabet, bits);

    assert_eq!(decoded.symbols, expected_output);
}
