use crate::alphabet::{Alphabet, Symbol};
use biterator::Bit::{self, One, Zero};
use log::debug;

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
#[derive(PartialEq, Debug)]
enum DecoderState {
    Initial,
    TopOfSymbolLoop,
    Rescaling,
    CalculateLength,
    Final,
}

use DecoderState::*;

#[derive(PartialEq, Debug)]
pub enum DecoderEvent<S: Symbol> {
    /// A symbol was decoded from the input stream.
    DecodedSymbol(S),
    /// Decoding of a single message is complete. The usize indicates how many
    /// bits of the input correspond to the decoded message.
    MessageLength(usize),
}

pub struct DecoderOutput<'a, S, A, I, const BITS_OF_PRECISION: u32>
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
    type Item = DecoderEvent<S>;

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

    /// Construct a new DecoderOutput from a stream of bits and an alphabet
    fn new(input: I, alphabet: &'a A) -> Self {
        debug!(
            "Decoding with {BITS_OF_PRECISION} bits (whole={} half={} quarter={})",
            Self::WHOLE,
            Self::HALF,
            Self::QUARTER
        );
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

    /// Continue the decoding process until the next event is emitted. None
    /// indicates no more events are available.
    fn next_event(&mut self) -> Option<DecoderEvent<S>> {
        loop {
            if let Some(event) = self.event_to_emit.take() {
                debug!("Emitting event: {event:?}");
                return Some(event);
            }

            if self.state == Final {
                return None;
            }

            self.state = self.execute();
        }
    }

    /// Execute the decoder state machine from its current state, producing the
    /// next state.
    fn execute(&mut self) -> DecoderState {
        debug!("Executing {:?} state", self.state);
        debug!("[pre]  a={:<12} b={:<12} z={:<12}", self.a, self.b, self.z);
        let next = match self.state {
            Initial => self.execute_initial(),
            Rescaling => self.execute_rescaling(),
            TopOfSymbolLoop => self.execute_top_of_symbol_loop(),
            CalculateLength => self.execute_calculate_length(),
            Final => Final,
        };
        debug!("[post] a={:<12} b={:<12} z={:<12}", self.a, self.b, self.z);
        next
    }

    /// Execute from the Initial state, which initializes state variables.
    ///
    /// Returns the next state.
    fn execute_initial(&mut self) -> DecoderState {
        self.a = 0;
        self.b = Self::WHOLE;
        self.initialize_z();
        TopOfSymbolLoop
    }

    /// Set z to its initial value by reading bits from the input and shifting
    /// them into their appropriate positions.
    fn initialize_z(&mut self) {
        self.z = 0;
        for i in 1..=BITS_OF_PRECISION {
            match self.input.next() {
                None => {
                    debug!(
                        "Initialized z with {} bits from input (z={})",
                        i - 1,
                        self.z
                    );
                    break;
                }
                Some(Zero) => continue,
                Some(One) => self.z += 2usize.pow(BITS_OF_PRECISION - i),
            }
        }
    }

    /// Execute from the TopOfSymbolLoop state, searching for the symbol
    /// identified by the subinterval containing the current value of z.
    ///
    /// Returns the next state.
    fn execute_top_of_symbol_loop(&mut self) -> DecoderState {
        for symbol in self.alphabet.symbols() {
            let (sub_a, sub_b) = self.subinterval_for_symbol(symbol);

            if (sub_a..sub_b).contains(&self.z) {
                self.event_to_emit = Some(DecoderEvent::DecodedSymbol(*symbol));
                self.a = sub_a;
                self.b = sub_b;

                if *symbol == self.alphabet.eof() {
                    return CalculateLength;
                } else {
                    return Rescaling;
                }
            }
        }

        // As z is within [a, b), some subinterval must contain it
        unreachable!(
            "No subinterval of [a, b) contained z (z={:<12} a={:<12} b={:<12})",
            self.z, self.a, self.b
        );
    }

    /// Determine the lower and upper bounds for the subinterval corresponding
    /// to the given symbol.
    fn subinterval_for_symbol(&self, symbol: &S) -> (usize, usize) {
        let total_interval_width = self.alphabet.total_interval_width();
        let upper_bound = self.alphabet.interval_upper_bound(symbol);
        let lower_bound = self.alphabet.interval_lower_bound(symbol);

        let w = self.b - self.a;
        let sub_b = self.a + (w * upper_bound) / total_interval_width;
        let sub_a = self.a + (w * lower_bound) / total_interval_width;

        (sub_a, sub_b)
    }

    /// Execute from the Rescaling state, performing rescaling operations as
    /// necessary to prevent a and b from nearing too close to each other.
    ///
    /// Returns the next state.
    fn execute_rescaling(&mut self) -> DecoderState {
        self.side_rescaling();
        self.middle_rescaling();
        TopOfSymbolLoop
    }

    /// Perform "side rescaling" by identifying scenarios in which the a-b range
    /// lies entirely in the lower or upper half of the total region (from 0-WHOLE).
    fn side_rescaling(&mut self) {
        while self.b < Self::HALF || self.a > Self::HALF {
            if self.b < Self::HALF {
                debug!("Interval fully contained in 0 half");
                self.a *= 2;
                self.b *= 2;
                self.z *= 2;
            } else if self.a > Self::HALF {
                debug!("Interval fully contained in 1 half");
                self.a = 2 * (self.a - Self::HALF);
                self.b = 2 * (self.b - Self::HALF);
                self.z = 2 * (self.z - Self::HALF);
            }

            self.add_next_bit_to_z();
        }
    }

    /// Perform "middle rescaling" by identifying scenarios in which a and b are
    /// straddling the midpoint of the 0-WHOLE region and have grown close enough
    /// together.
    fn middle_rescaling(&mut self) {
        while self.a > Self::QUARTER && self.b < 3 * Self::QUARTER {
            debug!(
                "Middle rescaling a={:<12} b={:<12} z={:<12}",
                self.a, self.b, self.z
            );
            self.a = 2 * (self.a - Self::QUARTER);
            self.b = 2 * (self.b - Self::QUARTER);
            self.z = 2 * (self.z - Self::QUARTER);
            self.add_next_bit_to_z();
        }
    }

    /// Take the next bit from the input stream, and add it as the least
    /// significant bit of z.
    fn add_next_bit_to_z(&mut self) {
        self.z_rescale_counter += 1;
        if let Some(One) = self.input.next() {
            self.z += 1;
        }

        debug!("Next bit: {}", self.z & 1);
    }

    /// Determine the number of bits that were used to encode the message that
    /// was just decoded.
    ///
    /// We do this by determining the number of bits of z that are necessary
    /// for unambiguously indicating the [a, b) interval, and add this to the
    /// number of bits of z that we've already discarded via rescaling.
    fn execute_calculate_length(&mut self) -> DecoderState {
        let prefix_size = self.minimal_z_prefix_size() as usize;
        debug!("Minimal prefix of z: {prefix_size} bits");

        let encoded_message_length = prefix_size + self.z_rescale_counter;
        self.event_to_emit = Some(DecoderEvent::MessageLength(encoded_message_length));
        Final
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

        // As z is in [a, b), the prefix containing all of z's bits is necessarily
        // also contained in this interval
        unreachable!(
            "No prefix of z is within [a, b) (z={:<12} a={:<12} b={:<12})",
            self.z, self.a, self.b
        );
    }
}

pub trait Decoder<S, A>
where
    S: Symbol,
    A: Alphabet<S = S>,
{
    /// Decode a stream of bits as a stream of symbols.
    ///
    /// This method will decode a single message, yielding all the decoded
    /// symbols (including the EOF symbol), and then indicating completion
    /// with the MessageLength event.
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
    use test_log::test;

    // The below test cases assume 32-bit precision
    const BITS_OF_PRECISION: u32 = 32;

    fn decode(input: Vec<Bit>) -> Vec<DecoderEvent<ExampleSymbol>> {
        let alphabet = ExampleAlphabet::new();
        alphabet.decode::<_, BITS_OF_PRECISION>(input).collect()
    }

    #[test]
    fn decode_empty_message() {
        assert_eq!(
            decode(vec![One, One, One, Zero, One]),
            vec![DecodedSymbol(Eof), MessageLength(5)],
        );
    }

    #[test]
    fn decode_very_small_message() {
        assert_eq!(
            decode(vec![One, One, One, Zero, Zero, One, Zero]),
            vec![DecodedSymbol(C), DecodedSymbol(Eof), MessageLength(7)],
        );
    }

    #[test]
    fn decode_small_message() {
        assert_eq!(
            decode(vec![Zero, One, Zero, One, One, One, Zero, Zero, One, Zero]),
            vec![
                DecodedSymbol(B),
                DecodedSymbol(A),
                DecodedSymbol(C),
                DecodedSymbol(Eof),
                MessageLength(10),
            ],
        );
    }

    #[test]
    fn decode_message_with_middle_rescaling() {
        assert_eq!(
            decode(vec![One, Zero, Zero, One, One, One]),
            vec![
                DecodedSymbol(B),
                DecodedSymbol(B),
                DecodedSymbol(Eof),
                MessageLength(6),
            ],
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
            vec![DecodedSymbol(C), DecodedSymbol(Eof), MessageLength(7)],
        )
    }
}
