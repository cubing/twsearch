use std::fmt::Display;

pub struct PrimePower {
    prime: u64,
    power: u64,
}

impl Display for PrimePower {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}",
            self.prime,
            self.power
                .to_string()
                .replace('0', "⁰")
                .replace('1', "¹")
                .replace('2', "²")
                .replace('3', "³")
                .replace('4', "⁴")
                .replace('5', "⁵")
                .replace('6', "⁶")
                .replace('7', "⁷")
                .replace('8', "⁸")
                .replace('9', "⁹")
        )
    }
}

pub struct Factorization {
    prime_powers: Vec<PrimePower>,
}

impl Display for Factorization {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let primer_powers_list: Vec<String> =
            self.prime_powers.iter().map(|p| p.to_string()).collect();
        write!(f, "{}", primer_powers_list.join(" × "))
    }
}

// TODO: this is a super naïve algorithm, but it should be good enough for most puzzles.
fn factor_number_from(n: u64, from: u64) -> Factorization {
    for i in (from..).take_while(|i| i * i <= n) {
        if n.is_multiple_of(i) {
            let mut recursive_factorization = factor_number_from(n / i, i);
            if recursive_factorization.prime_powers[0].prime == i {
                recursive_factorization.prime_powers[0].power += 1;
            } else {
                recursive_factorization
                    .prime_powers
                    .insert(0, PrimePower { prime: i, power: 1 })
            }
            return recursive_factorization;
        }
    }
    Factorization {
        prime_powers: vec![{ PrimePower { prime: n, power: 1 } }],
    }
}

pub fn factor_number(n: u64) -> Factorization {
    factor_number_from(n, 2)
}
