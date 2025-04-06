use cubing::alg::{Alg, AlgNode};

use crate::_internal::puzzle_traits::puzzle_traits::SemiGroupActionPuzzle;

pub fn apply_flat_alg<TPuzzle: SemiGroupActionPuzzle>(
    tpuzzle: &TPuzzle,
    pattern: &TPuzzle::Pattern,
    alg: &Alg,
) -> Option<TPuzzle::Pattern> {
    // TODO: avoid the initial clone somehow?
    let mut pattern = pattern.clone();
    for r#move in alg.nodes.iter() {
        match r#move {
            AlgNode::MoveNode(r#move) => {
                let transformation = tpuzzle.puzzle_transformation_from_move(r#move).ok()?;
                pattern = tpuzzle.pattern_apply_transformation(&pattern, &transformation)?;
            }
            AlgNode::PauseNode(_) => {}
            AlgNode::LineCommentNode(_) => {}
            AlgNode::NewlineNode(_) => {}
            _ => todo!("Saw a nested alg node where a flat alg was expected."),
        }
    }
    Some(pattern)
}
