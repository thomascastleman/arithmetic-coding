use crate::alphabet::{Alphabet, Symbol};

#[derive(PartialEq, Debug)]
pub enum ExampleSymbol {
    A,
    B,
    C,
    Eof,
}

impl Symbol for ExampleSymbol {}

pub struct ExampleAlphabet {
    symbols: Vec<ExampleSymbol>,
}

impl ExampleAlphabet {
    pub fn new() -> Self {
        Self {
            symbols: vec![
                ExampleSymbol::A,
                ExampleSymbol::B,
                ExampleSymbol::C,
                ExampleSymbol::Eof,
            ],
        }
    }
}

impl Alphabet for ExampleAlphabet {
    type S = ExampleSymbol;

    fn symbols(&self) -> impl Iterator<Item = &Self::S> {
        self.symbols.iter()
    }

    fn eof(&self) -> ExampleSymbol {
        ExampleSymbol::Eof
    }

    fn interval_width(&self, symbol: &Self::S) -> usize {
        // These correspond to the following probabilities:
        //   A:   0.25
        //   B:   0.50
        //   C:   0.15
        //   EOF: 0.10
        match symbol {
            ExampleSymbol::A => 25,
            ExampleSymbol::B => 50,
            ExampleSymbol::C => 15,
            ExampleSymbol::Eof => 10,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use ExampleSymbol::*;

    #[test]
    fn test_symbols() {
        let alphabet = ExampleAlphabet::new();
        assert_eq!(
            alphabet.symbols().collect::<Vec<_>>(),
            vec![&A, &B, &C, &Eof]
        );
    }

    #[test]
    fn test_eof() {
        let alphabet = ExampleAlphabet::new();
        assert!(alphabet.eof() == Eof);
    }

    #[test]
    fn test_lower_bound() {
        let alphabet = ExampleAlphabet::new();
        assert_eq!(alphabet.interval_lower_bound(&A), 0);
        assert_eq!(alphabet.interval_lower_bound(&B), 25);
        assert_eq!(alphabet.interval_lower_bound(&C), 75);
        assert_eq!(alphabet.interval_lower_bound(&Eof), 90);
    }

    #[test]
    fn test_upper_bound() {
        let alphabet = ExampleAlphabet::new();
        assert_eq!(alphabet.interval_upper_bound(&A), 25);
        assert_eq!(alphabet.interval_upper_bound(&B), 75);
        assert_eq!(alphabet.interval_upper_bound(&C), 90);
        assert_eq!(alphabet.interval_upper_bound(&Eof), 100);
    }
}
