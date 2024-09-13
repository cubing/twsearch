use cubing::alg::{ Alg};


use super::{
    super::randomize::{
        randomize_orbit_naïve, OrbitOrientationConstraint, OrbitPermutationConstraint,
    },
    super::scramble_search::{filtered_search, generators_from_vec_str},
    definitions::skewb_fixed_corner_kpuzzle,
};

pub fn scramble_skewb() -> Alg {
    let kpuzzle = skewb_fixed_corner_kpuzzle();
    loop {
        let mut scramble_pattern = kpuzzle.default_pattern();

        let orbit_info = &kpuzzle.data.ordered_orbit_info[0];
        assert_eq!(orbit_info.name.0, "CORNERS1");
        randomize_orbit_naïve(
            &mut scramble_pattern,
            orbit_info,
            OrbitPermutationConstraint::SingleOrbitEvenParity,
            OrbitOrientationConstraint::OrientationsMustSumToZero,
        );
        
        let orbit_info = &kpuzzle.data.ordered_orbit_info[1];
        assert_eq!(orbit_info.name.0, "CORNERS2");
        randomize_orbit_naïve(
            &mut scramble_pattern,
            orbit_info,
            OrbitPermutationConstraint::SingleOrbitEvenParity,
            OrbitOrientationConstraint::OrientationsMustSumToZero,
        );
        
        let orbit_info = &kpuzzle.data.ordered_orbit_info[2];
        assert_eq!(orbit_info.name.0, "CENTERS");
        randomize_orbit_naïve(
            &mut scramble_pattern,
            orbit_info,
            OrbitPermutationConstraint::SingleOrbitEvenParity,
            OrbitOrientationConstraint::OrientationsMustSumToZero,
        );

        let generators = generators_from_vec_str(vec!["U", "L", "R", "B"]); // TODO: cache
        if let Some(scramble) = filtered_search(&scramble_pattern, generators, 7, Some(11), Some(11)) {
            return scramble;
        }
    }
}
