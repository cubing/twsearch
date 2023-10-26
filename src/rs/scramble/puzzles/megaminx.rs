use cubing::alg::{Alg, AlgNode, Move, Newline};
use rand::{thread_rng, Rng};

const NUM_LINES: usize = 7;
const NUM_RANDOM_MOVE_PAIRS: usize = 5;

pub fn scramble_megaminx() -> Alg {
    let mut rng = thread_rng();
    let mut alg_nodes = Vec::<AlgNode>::new();

    let r_array: [Move; 2] = ["R++".parse().unwrap(), "R--".parse().unwrap()];
    let d_array: [Move; 2] = ["D++".parse().unwrap(), "D--".parse().unwrap()];
    let u_array: [Move; 2] = ["U".parse().unwrap(), "U'".parse().unwrap()];

    for _ in 0..NUM_LINES {
        let mut random_choice: usize = 0;
        for _ in 0..NUM_RANDOM_MOVE_PAIRS {
            for arr in [&r_array, &d_array] {
                random_choice = rng.gen_range(0..=1);
                alg_nodes.push(arr[random_choice].clone().into());
            }
        }
        // Match TNoodle:
        //
        // - `D++` is followed by `U`
        // - `D--` is followed by `U'`
        alg_nodes.push(u_array[random_choice].clone().into());

        alg_nodes.push(Newline::default().into());
    }
    alg_nodes.pop(); // Remove the last newline.

    Alg { nodes: alg_nodes }
}
