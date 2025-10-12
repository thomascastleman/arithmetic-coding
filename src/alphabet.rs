pub trait Symbol: PartialEq + Copy {}

pub trait Alphabet {
    type S: Symbol;

    /// An iterator over the symbols in the alphabet.
    fn symbols(&self) -> impl Iterator<Item = &Self::S>;

    /// The "end of file" symbol, which terminates the input stream.
    fn eof(&self) -> Self::S;

    /// The probability that this symbol is the next symbol in the input stream,
    /// represented as an integer such that dividing it by the sum of all integer
    /// probabilities in the symbol set yields the probability.
    ///
    /// This is r_i.
    fn interval_width(&self, symbol: &Self::S) -> usize;

    /// The sum of all interval widths.
    /// This is R.
    fn total_interval_width(&self) -> usize {
        let mut sum = 0;
        for symbol in self.symbols() {
            sum += self.interval_width(symbol);
        }
        sum
    }

    /// This is c_j.
    fn interval_lower_bound(&self, symbol: &Self::S) -> usize {
        let mut sum = 0;
        for s in self.symbols() {
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
