mod alphabet;
mod decoder;
mod encoder;
mod example;

const BITS_OF_PRECISION: u32 = 32;
const WHOLE: usize = 2_usize.pow(BITS_OF_PRECISION);
const HALF: usize = WHOLE / 2;
const QUARTER: usize = WHOLE / 4;
