use bellman::Circuit;
use bellman::ConstraintSystem;
use bellman::SynthesisError;
use bellman::groth16::VerifyingKey;
use bellman::groth16::{
    Parameters, Proof, create_random_proof, generate_random_parameters, prepare_verifying_key,
    verify_proof,
};
use bls12_381::{Bls12, Scalar};
use std::io::Cursor;
struct RankMulCircuit {
    rank: Option<Scalar>,
    voting_power: Option<Scalar>,
}

impl Circuit<Scalar> for RankMulCircuit {
    fn synthesize<CS: ConstraintSystem<Scalar>>(self, cs: &mut CS) -> Result<(), SynthesisError> {
        let rank_var = cs.alloc(
            || "rank",
            || self.rank.ok_or(SynthesisError::AssignmentMissing),
        )?;
        let vp_var = cs.alloc_input(
            || "voting_power",
            || self.voting_power.ok_or(SynthesisError::AssignmentMissing),
        )?;
        let inv_ten = Scalar::from(10u64);
        cs.enforce(
            || "rank × 0.1 = vp",
            |lc| lc + rank_var,
            |lc| lc + (inv_ten, CS::one()),
            |lc| lc + vp_var,
        );
        Ok(())
    }
}

fn main() {
    let mut rng = rand::thread_rng();

    let params: Parameters<Bls12> = generate_random_parameters(
        RankMulCircuit {
            rank: None,
            voting_power: None,
        },
        &mut rng,
    )
    .unwrap();
    let pvk = prepare_verifying_key(&params.vk);

    let rank = Scalar::from(2u64);
    let inv_ten = Scalar::from(10u64);
    let voting_power = rank * inv_ten;
    println!("{:?}", voting_power);
    let proof: Proof<Bls12> = create_random_proof(
        RankMulCircuit {
            rank: Some(rank),
            voting_power: Some(voting_power),
        },
        &params,
        &mut rng,
    )
    .unwrap();

    assert!(verify_proof(&pvk, &proof, &[voting_power]).is_ok());
    println!("voting_power {:?}", voting_power);
    println!("params.vk.alpha_g1 {:?}", params.vk.alpha_g1);
    println!("params.vk.beta_g1 {:?}", params.vk.beta_g1);
    println!("proof {:?}", proof);
}
