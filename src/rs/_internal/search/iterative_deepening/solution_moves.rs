use cubing::alg::{Alg, AlgNode, Move};

pub struct SolutionPreviousMoves<'a> {
    latest_move: &'a Move,
    previous_moves: &'a SolutionMoves<'a>,
}

impl<'a> From<&'a SolutionPreviousMoves<'a>> for SolutionMoves<'a> {
    fn from(value: &'a SolutionPreviousMoves<'a>) -> Self {
        SolutionMoves(Some(value))
    }
}

#[derive(Clone, Default)]
pub struct SolutionMoves<'a>(Option<&'a SolutionPreviousMoves<'a>>);

impl<'a> From<&SolutionMoves<'a>> for Alg {
    fn from(value: &SolutionMoves<'a>) -> Self {
        let nodes = value.snapshot_alg_nodes();
        Alg { nodes }
    }
}

impl SolutionMoves<'_> {
    pub fn push<'a>(&'a self, r#move: &'a Move) -> SolutionPreviousMoves<'a> {
        SolutionPreviousMoves {
            latest_move: r#move,
            previous_moves: self,
        }
    }

    pub fn snapshot_alg_nodes(&self) -> Vec<AlgNode> {
        match self.0 {
            Some(solution_previous_moves) => {
                let mut nodes = solution_previous_moves.previous_moves.snapshot_alg_nodes();
                nodes.push(cubing::alg::AlgNode::MoveNode(
                    solution_previous_moves.latest_move.clone(),
                ));
                nodes
            }
            None => vec![],
        }
    }

    pub fn reverse_move_iter(&self) -> SolutionMovesReverseIterator {
        SolutionMovesReverseIterator {
            solution_moves: self,
        }
    }
}

pub struct SolutionMovesReverseIterator<'a> {
    solution_moves: &'a SolutionMoves<'a>,
}

impl<'a> Iterator for SolutionMovesReverseIterator<'a> {
    type Item = &'a Move;

    fn next(&mut self) -> Option<Self::Item> {
        let solution_previous_moves = self.solution_moves.0?;
        self.solution_moves = solution_previous_moves.previous_moves;
        Some(solution_previous_moves.latest_move)
    }
}

pub(crate) fn alg_from_moves(moves: &[Move]) -> Alg {
    let nodes = moves.iter().map(|m| AlgNode::MoveNode(m.clone())).collect();
    Alg { nodes }
}

pub(crate) fn alg_to_moves(alg: &Alg) -> Option<Vec<Move>> {
    let mut moves: Vec<Move> = vec![];
    for alg_node in &alg.nodes {
        let AlgNode::MoveNode(r#move) = alg_node else {
            return None;
        };
        moves.push(r#move.clone());
    }
    Some(moves)
}
