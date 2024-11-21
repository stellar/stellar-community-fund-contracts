use governance::types::Vote;
use neurons::Submission;
use serde_json::{Map, Value};
use soroban_sdk::{
    map, vec, Env, Map as SorobanMap, String as SorobanString, Vec as SorobanVec, I256,
};
use std::fs;

fn vote_from_str(s: &str) -> Vote {
    match s {
        "Abstain" => Vote::Abstain,
        "Yes" => Vote::Yes,
        "No" => Vote::No,
        _ => panic!("invalid vote"),
    }
}
fn i256_from_json_value(env: &Env, value: &Value) -> I256 {
    I256::from_i128(
        &env,
        value.as_str().unwrap().to_string().parse::<i128>().unwrap(),
    )
}
pub fn submissions(env: &Env) -> SorobanVec<(SorobanString, SorobanString)> {
    let submissions_raw = fs::read_to_string("../neurons/data/submissions.json").unwrap();
    let submissions: Vec<Submission> = serde_json::from_str(submissions_raw.as_str()).unwrap();
    let mut submissions_soroban: SorobanVec<(SorobanString, SorobanString)> = vec![&env];
    submissions.iter().for_each(|s| {
        submissions_soroban.push_back(match s.category {
            neurons::SubmissionCategory::Applications => (
                SorobanString::from_str(&env, &s.name),
                SorobanString::from_str(&env, "Applications"),
            ),
            neurons::SubmissionCategory::FinancialProtocols => (
                SorobanString::from_str(&env, &s.name),
                SorobanString::from_str(&env, "FinancialProtocols"),
            ),
            neurons::SubmissionCategory::InfrastructureAndServices => (
                SorobanString::from_str(&env, &s.name),
                SorobanString::from_str(&env, "InfrastructureAndServices"),
            ),
            neurons::SubmissionCategory::DeveloperTooling => (
                SorobanString::from_str(&env, &s.name),
                SorobanString::from_str(&env, "DeveloperTooling"),
            ),
        });
    });
    submissions_soroban
}
pub fn normalized_votes(env: &Env) -> SorobanMap<SorobanString, SorobanMap<SorobanString, Vote>> {
    let normalized_votes_raw =
        fs::read_to_string("../neurons/result/normalized_votes.json").unwrap();
    let normalized_votes_serde: Map<String, Value> =
        serde_json::from_str(normalized_votes_raw.as_str()).unwrap();
    let mut normalized_votes_soroban: SorobanMap<SorobanString, SorobanMap<SorobanString, Vote>> =
        map![&env];
    normalized_votes_serde
        .iter()
        .for_each(|(sumbmission_id, votes)| {
            let mut mapped_votes: SorobanMap<SorobanString, Vote> = map![&env];
            votes
                .as_object()
                .unwrap()
                .iter()
                .for_each(|(public_key, vote_str)| {
                    let vote = vote_from_str(vote_str.as_str().unwrap());
                    mapped_votes.set(SorobanString::from_str(&env, public_key), vote);
                });
            normalized_votes_soroban
                .set(SorobanString::from_str(&env, sumbmission_id), mapped_votes);
        });
    normalized_votes_soroban
}
pub fn trust(env: &Env) -> SorobanMap<SorobanString, I256> {
    let trust_raw = fs::read_to_string("../neurons/result/trust_graph_neuron.json").unwrap();
    let trust_serde: Map<String, Value> = serde_json::from_str(trust_raw.as_str()).unwrap();
    let mut trust_soroban: SorobanMap<SorobanString, I256> = map![&env];
    trust_serde.iter().for_each(|(public_key, value)| {
        trust_soroban.set(
            SorobanString::from_str(&env, public_key),
            i256_from_json_value(&env, value),
        );
    });
    trust_soroban
}
pub fn reputation(env: &Env) -> SorobanMap<SorobanString, I256> {
    let reputation_raw =
        fs::read_to_string("../neurons/result/assigned_reputation_neuron.json").unwrap();
    let reputation_serde: Map<String, Value> =
        serde_json::from_str(reputation_raw.as_str()).unwrap();
    let mut reputation_soroban: SorobanMap<SorobanString, I256> = map![&env];
    reputation_serde.iter().for_each(|(public_key, value)| {
        reputation_soroban.set(
            SorobanString::from_str(&env, public_key),
            i256_from_json_value(&env, value),
        );
    });

    reputation_soroban
}
pub fn voting_history(env: &Env) -> SorobanMap<SorobanString, I256> {
    let voting_history_raw =
        fs::read_to_string("../neurons/result/prior_voting_history_neuron.json").unwrap();
    let voting_history_serde: Map<String, Value> =
        serde_json::from_str(voting_history_raw.as_str()).unwrap();
    let mut voting_history_soroban: SorobanMap<SorobanString, I256> = map![&env];
    voting_history_serde.iter().for_each(|(public_key, value)| {
        voting_history_soroban.set(
            SorobanString::from_str(&env, public_key),
            i256_from_json_value(&env, value),
        );
    });
    voting_history_soroban
}
