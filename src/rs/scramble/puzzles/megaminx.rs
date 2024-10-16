use cubing::alg::{parse_alg, parse_move, Alg, AlgNode, Move, Newline};
use rand::{thread_rng, Rng};

use crate::scramble::scramble_search::{
    generators_from_vec_str, idfs_with_target_pattern, simple_filtered_search,
};

use super::{
    definitions::{megaminx_centers_kpattern, megaminx_kpuzzle},
    mask_pattern::mask,
};

const NUM_LINES: usize = 7;
const NUM_RANDOM_MOVE_PAIRS: usize = 5;

pub fn scramble_megaminx() -> Alg {
    let kpuzzle = megaminx_kpuzzle();
    let scramble = kpuzzle
        .default_pattern()
        .apply_alg(&parse_alg!(
            "R++ D-- R-- D++ R++ D-- R-- D-- R-- D-- U'
R++ D++ R-- D-- R++ D-- R-- D++ R-- D++ U
R-- D++ R-- D++ R-- D++ R++ D-- R-- D-- U'
R++ D-- R-- D++ R++ D++ R-- D-- R++ D++ U
R-- D++ R-- D++ R++ D-- R++ D++ R++ D++ U
R++ D-- R-- D++ R++ D++ R++ D++ R-- D-- U'
R++ D-- R++ D-- R++ D++ R++ D-- R-- D-- U'"
        ))
        .unwrap();

    let masked = mask(&scramble, &megaminx_centers_kpattern());

    let generators = generators_from_vec_str(vec!["Rv", "Uv"]);
    let mut idfs = idfs_with_target_pattern(
        kpuzzle,
        generators,
        megaminx_centers_kpattern().to_owned(),
        None,
    );

    let mut found = idfs.search(&masked, Default::default());
    dbg!(found.next());

    Alg::default()

    // let mut rng = thread_rng();
    // let mut alg_nodes = Vec::<AlgNode>::new();

    // let r_array: [Move; 2] = [parse_move!("R++"), parse_move!("R--")];
    // let d_array: [Move; 2] = [parse_move!("D++"), parse_move!("D--")];
    // let u_array: [Move; 2] = [parse_move!("U"), parse_move!("U'")];

    // for _ in 0..NUM_LINES {
    //     let mut random_choice: usize = 0;
    //     for _ in 0..NUM_RANDOM_MOVE_PAIRS {
    //         for arr in [&r_array, &d_array] {
    //             random_choice = rng.gen_range(0..=1);
    //             alg_nodes.push(arr[random_choice].clone().into());
    //         }
    //     }
    //     // Match TNoodle:
    //     //
    //     // - `D++` is followed by `U`
    //     // - `D--` is followed by `U'`
    //     alg_nodes.push(u_array[random_choice].clone().into());

    //     alg_nodes.push(Newline::default().into());
    // }
    // alg_nodes.pop(); // Remove the last newline.

    // Alg { nodes: alg_nodes }
}
