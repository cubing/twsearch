use cubing::alg::{Alg, AlgNode, Move, Newline};
use rand::{thread_rng, Rng};

const NUM_LINES: usize = 7;
const NUM_GROUPS: usize = 5;

pub fn scramble_megaminx() -> Alg {
    let mut rng = thread_rng();
    let mut alg_nodes = Vec::<AlgNode>::new();

    let r_array: [Move; 2] = [
        "R_PLUSPLUS_".parse().unwrap(),
        "R_PLUSPLUS_'".parse().unwrap(),
    ];
    let d_array: [Move; 2] = [
        "D_PLUSPLUS_".parse().unwrap(),
        "D_PLUSPLUS_'".parse().unwrap(),
    ];
    let u_array: [Move; 2] = ["U".parse().unwrap(), "U'".parse().unwrap()];

    for _ in 0..NUM_LINES {
        let mut random_choice: usize = 0;
        for _ in 0..NUM_GROUPS {
            random_choice = rng.gen_range(0..=1);
            alg_nodes.push(AlgNode::MoveNode(r_array[random_choice].clone()));
            random_choice = rng.gen_range(0..=1);
            alg_nodes.push(AlgNode::MoveNode(d_array[random_choice].clone()));
        }
        alg_nodes.push(AlgNode::MoveNode(u_array[random_choice].clone()));
        alg_nodes.push(AlgNode::NewlineNode(Newline::default()));
    }
    alg_nodes.pop();

    Alg { nodes: alg_nodes }
}
