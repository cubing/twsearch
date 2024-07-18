use cubing::alg::{Alg, AlgNode, Move};

/// This is a minimal implementation of https://js.cubing.net/cubing/api/classes/alg.Alg.html#experimentalSimplify for collapsing moves between phases.
/// For face turns and face rotations of a cube, pass:
/// - `mod_n`: 4
/// - `mod_offset`: 1
pub fn collapse_adjacent_moves(alg: Alg, mod_n: i32, mod_offset: i32) -> Alg {
    let mut nodes = Vec::<AlgNode>::new();

    let mut maybe_pending_move: Option<Move> = None;
    for node in alg.nodes {
        maybe_pending_move = if let AlgNode::MoveNode(mut new_move) = node {
            if let Some(pending_move) = maybe_pending_move {
                if new_move.quantum.family == pending_move.quantum.family {
                    let new_amount =
                        (new_move.amount + pending_move.amount - mod_offset) % mod_n + mod_offset;
                    if new_amount == 0 {
                        if let Some(popped_node) = nodes.pop() {
                            if let AlgNode::MoveNode(popped_move) = popped_node {
                                Some(popped_move)
                            } else {
                                nodes.push(popped_node);
                                None
                            }
                        } else {
                            None
                        }
                    } else {
                        new_move.amount = new_amount;
                        Some(new_move)
                    }
                } else {
                    nodes.push(pending_move.into());
                    Some(new_move)
                }
            } else {
                Some(new_move)
            }
        } else {
            if let Some(pending_move) = maybe_pending_move {
                nodes.push(pending_move.into());
            }
            nodes.push(node);
            None
        }
    }
    if let Some(pending_move) = maybe_pending_move {
        nodes.push(pending_move.into());
    }
    Alg { nodes }
}

#[test]
fn collapse_test() {
    use cubing::alg::parse_alg;

    assert_eq!(
        collapse_adjacent_moves(
            parse_alg!(
                "R' U' F R2 D U B D U' L' B2 U' F F2 D' B2 D' L2 D' R2 B2 R2 F2 U' B2 D' R' U' F"
            ),
            4,
            -1
        ),
        parse_alg!("R' U' F R2 D U B D U' L' B2 U' F' D' B2 D' L2 D' R2 B2 R2 F2 U' B2 D' R' U' F")
    );

    assert_eq!(
        collapse_adjacent_moves(parse_alg!("R F F' R"), 4, -1),
        parse_alg!("R2")
    );

    assert_eq!(
        collapse_adjacent_moves(parse_alg!("R F F2 F R"), 4, -1),
        parse_alg!("R2")
    );

    assert_eq!(
        collapse_adjacent_moves(parse_alg!("R F F2 . F R"), 4, -1),
        parse_alg!("R F' . F R")
    );

    assert_eq!(
        collapse_adjacent_moves(parse_alg!("R F F2 R"), 5, -2),
        parse_alg!("R F2' R")
    );
}
