use data_generator::{normalized_votes, reputation, submissions, trust, voting_history};
use governance::types::Vote;
use offchain::manual_tally;
use soroban_sdk::{Env, Map as SorobanMap, String as SorobanString, Vec as SorobanVec, I256};
mod data_generator;
mod offchain;

fn main() {
    let env = Env::default();
    env.cost_estimate().budget().reset_unlimited();

    let submissions: SorobanVec<(SorobanString, SorobanString)> = submissions(&env);
    let normalized_votes: SorobanMap<SorobanString, SorobanMap<SorobanString, Vote>> =
        normalized_votes(&env);
    let trust_graph_neuron_result: SorobanMap<SorobanString, I256> = trust(&env);
    let assigned_reputation_neuron_result: SorobanMap<SorobanString, I256> = reputation(&env);
    let prior_voting_history_neuron_result: SorobanMap<SorobanString, I256> = voting_history(&env);

    manual_tally(
        &env,
        submissions,
        normalized_votes,
        trust_graph_neuron_result,
        assigned_reputation_neuron_result,
        prior_voting_history_neuron_result,
    );
}
