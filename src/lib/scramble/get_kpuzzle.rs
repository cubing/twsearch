use cubing::kpuzzle::KPuzzle;

pub trait GetKPuzzle {
    fn get_kpuzzle(&self) -> &KPuzzle;
}
