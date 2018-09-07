use std::collections::HashMap;
use std::fmt;
use std::fmt::Display;
use super::mtklogo::ColorMode;

extern crate ansi_term;

use self::ansi_term::Colour::Blue;
use self::ansi_term::Colour::Purple;

#[derive(Debug, Clone)]
struct Factor {
    pub factor: usize,
    pub power: usize,
    val: usize,
}

impl Factor {
    fn power_zero(factor: usize) -> Factor {
        Factor { factor, power: 0, val: 1 }
    }
    #[inline(always)]
    fn value(&self) -> usize {
        self.val
    }
    #[inline(always)]
    fn higher_mut(&mut self) {
        self.power += 1;
        self.val *= self.factor;
    }
    fn divisors(&self) -> Vec<Factor> {
        let mut subfactors = Vec::with_capacity(self.power);
        let mut val: usize = 1;
        for power in 1..(self.power + 1) {
            val = val * self.factor;
            subfactors.push(Factor { factor: self.factor, power, val });
        }
        subfactors
    }
    /// decomposes in prime factors.
    fn decompose(n: usize) -> Vec<Factor> {
        let mut remainder = n;
        // For sure, n is divisible by 1, but we'll omit that solution.
        // let mut factors = vec!(Factor::power_one(1));
        let mut factors = Vec::new();
        // let's start attempting a division by 2.
        let mut f = Factor::power_zero(2);
        while f.factor <= remainder {
            if remainder % f.factor == 0 {
                remainder /= f.factor;
                f.higher_mut();
            } else {
                // will try a next number.
                let next_factor = f.factor + 1;
                // was the previous factor acknowledged?
                if f.power > 0 {
                    // factor is confirmed.
                    factors.push(f)
                }
                f = Factor::power_zero(next_factor);
            }
        }
        // was the last factor acknowledged?
        if f.power > 0 {
            // factor is confirmed.
            factors.push(f)
        }
        factors
    }
}

struct Factors<'a> {
    factors: &'a Vec<Factor>
}

impl<'a> Display for Factors<'a> {
    fn fmt(&self, format: &mut fmt::Formatter) -> fmt::Result {
        for (i, f) in self.factors.iter().enumerate() {
            Display::fmt(f, format)?;
            if i < self.factors.len() - 1 {
                write!(format, " * ")?;
            }
        }
        Ok(())
    }
}

impl Display for Factor {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self.power {
            0 => fmt.write_str("1"),
            1 => fmt.write_fmt(format_args!("{:?}", self.factor)),
            _ => fmt.write_fmt(format_args!("{:?}^{:?}", self.factor, self.power))
        }
    }
}


pub fn run_guess(size: usize) {
    fn explore(available_factors: &Vec<Factor>, a_factors: Vec<Factor>, n: usize) {
        let sz = available_factors.len();
        // Takes a factor in the available factor bag.
        let mut next_available = available_factors.clone();
        for _ in 0..sz {
            let current_factors = next_available.remove(0);
            for pow in current_factors.divisors() {
                // Puts it in the possible factors for "a".
                let mut next_a = a_factors.clone();
                next_a.push(pow);
                // The first member of the solution is the product of all a 's.
                let product = |mut prod: usize, x: &Factor| {
                    prod *= x.value();
                    prod
                };
                let www = next_a.iter().fold(1 as usize, product);
                // This is the second member of the solution.
                let hhh = n / www;
                let dimension = format!("{} x {}", www, hhh);
                let explanation = format!("{} = ({}) * {}", n, Factors { factors: &next_a }, hhh);
                println!("It could be {} ... {}", Blue.paint(dimension), Purple.paint(explanation));
                // continues
                explore(&next_available, next_a, n);
            }
        }
    }

    // group color modes by bytes per pixels.
    let mut table: HashMap<u32, Vec<ColorMode>> = HashMap::new();
    // Cowboy style, just for the pleasure to do it one line !
    ColorMode::enumerate().iter().for_each(
        |mode| table.entry(mode.bytes_per_pixel()).or_insert_with(|| Vec::new()).push(mode.clone()));
    // Guess dimensions for each bytes per pixel.
    for (bpp, modes) in table.iter() {
        let sz = size / (*bpp as usize);
        let factors = Factor::decompose(sz);
        println!("if {} bytes per pixel (modes: {:?}), {} bytes is {} pixels and has following divisors: {}.",
                 *bpp, modes, size, sz, Factors { factors: &factors });
        // explores possible arrangements
        explore(&factors, Vec::new(), sz);
    }
}
