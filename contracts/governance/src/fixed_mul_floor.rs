use soroban_sdk::{Env, I256};

pub(crate) fn fixed_mul_floor(env: &Env, x: &I256, y: &I256, denominator: &I256) -> I256 {
    mul_div_floor(env, x, y, denominator)
}

/// Performs floor(x * y / z)
pub(crate) fn mul_div_floor(env: &Env, x: &I256, y: &I256, z: &I256) -> I256 {
    let zero = I256::from_i32(env, 0);
    let r = x.mul(&y);
    if r < zero || (r > zero && z.clone() < zero) {
        // ceiling is taken by default for a negative result
        let remainder = r.rem_euclid(&z);
        let one = I256::from_i32(env, 1);
        r.div(&z).sub(if remainder > zero { &one } else { &zero })
    } else {
        // floor taken by default for a positive or zero result
        r.div(&z)
    }
}
