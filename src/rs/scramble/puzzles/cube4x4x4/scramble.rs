use std::sync::Mutex;

use cubing::alg::Alg;
use lazy_static::lazy_static;

use crate::scramble::puzzles::cube4x4x4::four_phase::Scramble4x4x4FourPhase;

// TODO: switch to `LazyLock` once that's stable: https://doc.rust-lang.org/nightly/std/cell/struct.LazyCell.html
lazy_static! {
    static ref SCRAMBLE4X4X4_FOUR_PHASE: Mutex<Scramble4x4x4FourPhase> =
        Mutex::new(Scramble4x4x4FourPhase::default());
}

pub fn scramble_4x4x4() -> Alg {
    SCRAMBLE4X4X4_FOUR_PHASE.lock().unwrap().scramble_4x4x4()
}
