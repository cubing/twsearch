use cubing::alg::{Alg, AlgNode, Move};

enum CombinedMoves {
    Cancelled(),
    Collapsed(Move),
    Separate(Move, Move),
}

fn combine_adjacent_moves(
    move1: Move,
    mut move2: Move,
    mod_n: i32,
    mod_offset: i32,
) -> CombinedMoves {
    if move2.quantum.family != move1.quantum.family {
        return CombinedMoves::Separate(move1, move2);
    }

    let new_amount = (move2.amount + move1.amount - mod_offset) % mod_n + mod_offset;
    if new_amount == 0 {
        return CombinedMoves::Cancelled();
    }
    move2.amount = new_amount;
    CombinedMoves::Collapsed(move2)
}

fn pop_final_move(nodes: &mut Vec<AlgNode>) -> Option<Move> {
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
}

/// This is a minimal implementation of https://js.cubing.net/cubing/api/classes/alg.Alg.html#experimentalSimplify for collapsing moves between phases.
/// For face turns and face rotations of a cube, pass:
/// - `mod_n`: 4
/// - `mod_offset`: 1
pub fn collapse_adjacent_moves(alg: Alg, mod_n: i32, mod_offset: i32) -> Alg {
    let mut nodes = Vec::<AlgNode>::new();

    let mut maybe_pending_move: Option<Move> = None;
    for new_node in alg.nodes {
        maybe_pending_move = if let AlgNode::MoveNode(new_move) = new_node {
            if let Some(pending_move) = maybe_pending_move {
                match combine_adjacent_moves(pending_move, new_move, mod_n, mod_offset) {
                    CombinedMoves::Cancelled() => pop_final_move(&mut nodes),
                    CombinedMoves::Collapsed(r#move) => Some(r#move),
                    CombinedMoves::Separate(move1, move2) => {
                        nodes.push(move1.into());
                        Some(move2)
                    }
                }
            } else {
                Some(new_move)
            }
        } else {
            if let Some(pending_move) = maybe_pending_move {
                nodes.push(pending_move.into());
            }
            nodes.push(new_node);
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
