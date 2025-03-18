use cubing::alg::{parse_move, Alg, AlgNode, Move};
use rand::{thread_rng, Rng};

pub fn scramble_clock() -> Alg {
    let mut rng = thread_rng();
    let mut alg_nodes = Vec::<AlgNode>::new();

    // TODO: implement `parse_quantum_move!(…)`?
    let back_moves = vec![
        parse_move!("U_PLUS_").quantum.to_owned(),
        parse_move!("R_PLUS_").quantum.to_owned(),
        parse_move!("D_PLUS_").quantum.to_owned(),
        parse_move!("L_PLUS_").quantum.to_owned(),
        parse_move!("ALL_PLUS_").quantum.to_owned(),
    ];

    let front_moves = [
        back_moves.clone(),
        vec![
            parse_move!("UR_PLUS_").quantum.to_owned(),
            parse_move!("DR_PLUS_").quantum.to_owned(),
            parse_move!("DL_PLUS_").quantum.to_owned(),
            parse_move!("UL_PLUS_").quantum.to_owned(),
        ],
    ]
    .concat();

    for front_move in front_moves {
        alg_nodes.push(
            Move {
                quantum: front_move,
                amount: rng.gen_range(-5..7),
            }
            .into(),
        );
    }
    alg_nodes.push(parse_move!("y2").clone().into());
    for back_move in back_moves {
        alg_nodes.push(
            Move {
                quantum: back_move,
                amount: rng.gen_range(-5..7),
            }
            .into(),
        );
    }

    Alg { nodes: alg_nodes }
}
