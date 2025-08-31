mod decoder;
mod encoder;

const BITS_OF_PRECISION: u32 = 32;
const WHOLE: usize = 2_usize.pow(BITS_OF_PRECISION);
const HALF: usize = WHOLE / 2;
const QUARTER: usize = WHOLE / 4;

trait Symbol: PartialEq {}

trait Alphabet {
    type S: Symbol;

    /// An iterator over the symbols in the alphabet.
    fn symbols(&self) -> impl Iterator<Item = Self::S>;

    /// Determine whether or not this symbol is the "end of file" symbol, which
    /// terminates the input stream.
    fn is_eof(&self, symbol: &Self::S) -> bool;

    /// The probability that this symbol is the next symbol in the input stream,
    /// represented as an integer such that dividing it by the sum of all integer
    /// probabilities in the symbol set yields the probability.
    ///
    /// This is r_i.
    fn interval_width(&self, symbol: &Self::S) -> usize;

    /// This is c_j.
    fn interval_lower_bound(&self, symbol: &Self::S) -> usize {
        let mut sum = 0;
        for ref s in self.symbols() {
            if s == symbol {
                break;
            }

            sum += self.interval_width(s);
        }
        sum
    }

    /// This is d_j.
    fn interval_upper_bound(&self, symbol: &Self::S) -> usize {
        self.interval_lower_bound(symbol) + self.interval_width(symbol)
    }
}
