//! Exact rational numbers.

use std::fmt;
use std::ops::{Add, Div, Mul};
use std::str::FromStr;

use crate::issue::TempoChangeError;

/// A rational number.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rat {
    num: i64,
    den: i64,
}

impl Rat {
    /// `num / den`, reduced.
    ///
    /// # Panics
    ///
    /// If `den` is zero, or the reduced value doesn't fit in `i64`.
    pub fn new(num: i64, den: i64) -> Rat {
        Rat::reduced(i128::from(num), i128::from(den))
    }

    fn wide(self) -> (i128, i128) {
        (i128::from(self.num), i128::from(self.den))
    }

    fn reduced(num: i128, den: i128) -> Rat {
        let g = gcd(num, den) * den.signum();
        let narrow = |n: i128| i64::try_from(n / g).expect("rational arithmetic stays within i64");
        Rat {
            num: narrow(num),
            den: narrow(den),
        }
    }

    /// The numerator.
    pub fn num(self) -> i64 {
        self.num
    }

    /// The denominator.
    pub fn den(self) -> i64 {
        self.den
    }
}

fn gcd(a: i128, b: i128) -> i128 {
    let (mut a, mut b) = (a.abs(), b.abs());
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a
}

impl Add for Rat {
    type Output = Rat;

    /// # Panics
    ///
    /// If the reduced sum doesn't fit in `i64`.
    fn add(self, other: Rat) -> Rat {
        let (a, b) = (self.wide(), other.wide());
        Rat::reduced(a.0 * b.1 + b.0 * a.1, a.1 * b.1)
    }
}

impl Mul<i64> for Rat {
    type Output = Rat;

    /// # Panics
    ///
    /// If the reduced product doesn't fit in `i64`.
    fn mul(self, k: i64) -> Rat {
        let (num, den) = self.wide();
        Rat::reduced(num * i128::from(k), den)
    }
}

impl Div<i64> for Rat {
    type Output = Rat;

    /// # Panics
    ///
    /// If `k` is zero, or the reduced quotient doesn't fit in `i64`.
    #[expect(
        clippy::suspicious_arithmetic_impl,
        reason = "dividing by k multiplies the denominator"
    )]
    fn div(self, k: i64) -> Rat {
        let (num, den) = self.wide();
        Rat::reduced(num, den * i128::from(k))
    }
}

impl FromStr for Rat {
    type Err = TempoChangeError;

    /// `p/q` or `p`, in digits.
    fn from_str(s: &str) -> Result<Rat, TempoChangeError> {
        let bad = || TempoChangeError::BadFraction {
            text: s.to_string(),
        };
        let digits = |t: &str| {
            let all_digits = !t.is_empty() && t.bytes().all(|b| b.is_ascii_digit());
            all_digits.then(|| t.parse::<i64>().ok()).flatten()
        };
        let (num, den) = match s.split_once('/') {
            Some((num, den)) => (digits(num).ok_or_else(bad)?, digits(den).ok_or_else(bad)?),
            None => (digits(s).ok_or_else(bad)?, 1),
        };
        if den == 0 {
            return Err(TempoChangeError::BadRational { num, den });
        }
        Ok(Rat::new(num, den))
    }
}

impl fmt::Display for Rat {
    /// `p/q`, or `p` when the denominator is 1.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.den {
            1 => write!(f, "{}", self.num),
            den => write!(f, "{}/{den}", self.num),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reduces() {
        assert_eq!(Rat::new(2, 4), Rat::new(1, 2));
    }

    #[test]
    fn adds_exactly() {
        assert_eq!(Rat::new(1, 3) + Rat::new(1, 6), Rat::new(1, 2));
    }

    #[test]
    fn parses_fractions() {
        let rows = [
            ("3/2", Ok(Rat::new(3, 2))),
            ("1", Ok(Rat::new(1, 1))),
            ("1/0", Err("bad rational 1/0")),
            ("-1", Err("bad fraction '-1'")),
            ("1/", Err("bad fraction '1/'")),
            ("x", Err("bad fraction 'x'")),
            (
                "99999999999999999999",
                Err("bad fraction '99999999999999999999'"),
            ),
        ];
        for (s, parsed) in rows {
            let got = s.parse::<Rat>().map_err(|e| e.to_string());
            assert_eq!(got, parsed.map_err(str::to_string), "{s}");
        }
    }

    #[test]
    fn prints_fractions() {
        for (rat, text) in [(Rat::new(3, 2), "3/2"), (Rat::new(2, 1), "2")] {
            assert_eq!(rat.to_string(), text);
        }
    }
}
