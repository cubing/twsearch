// fn abs_gcd_recursive(bigger: i32, smaller: i32) -> i32 {
//     let rem = bigger.rem_euclid(smaller);
//     if rem == 0 {
//         smaller
//     } else {
//         abs_gcd_recursive(smaller, rem)
//     }
// }

// pub fn abs_gcd(a: i32, b: i32) -> i32 {
//     let a = a.abs();
//     let b = b.abs();
//     if a > b {
//         abs_gcd_recursive(a, b)
//     } else {
//         abs_gcd_recursive(b, a)
//     }
// }

// #[cfg(test)]
// mod tests {
//     use crate::commands::gcd::abs_gcd;

//     #[test]
//     fn gcd() -> Result<(), String> {
//         assert_eq!(12, abs_gcd(-24, 84));
//         assert_eq!(3, abs_gcd(3, 12));
//         assert_eq!(1, abs_gcd(1, 1));
//         assert_eq!(1, abs_gcd(1, 24));
//         assert_eq!(1, abs_gcd(1, 24));

//         Ok(())
//     }
// }
