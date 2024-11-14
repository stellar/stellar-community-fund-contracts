use neurons::Submission;
use serde_json::{Map, Value};
use std::fs;

fn normalized_votes() {
    println!("-- normalized votes neuron results --");
    let normalized_votes_json = fs::read_to_string("result/normalized_votes.json").unwrap();
    let normalized_votes: Map<String, Value> =
        serde_json::from_str(normalized_votes_json.as_str()).unwrap();
    let normalized_votes_mapped: Vec<(String, Vec<(String, String)>)> = normalized_votes
        .iter()
        .map(|(sumbmission_id, votes)| {
            let mapped_votes: Vec<(String, String)> = match votes.as_object() {
                Some(votes) => votes
                    .iter()
                    .map(|(public_key, vote_str)| {
                        let vv = match vote_str.as_str().unwrap() {
                            "Abstain" => "Vote::Abstain",
                            "Yes" => "Vote::Yes",
                            "No" => "Vote::No",
                            _ => "error",
                        };
                        (public_key.to_string(), vv.to_string())
                    })
                    .collect(),
                None => vec![],
            };
            (sumbmission_id.to_string(), mapped_votes)
        })
        .collect();
    for (submission_id, votes) in normalized_votes_mapped {
        let votes_map_string: Vec<String> = votes
            .iter()
            .map(|(public_key, vote)| {
                format!(
                    "(String::from_str(&env, {:#?}), {})",
                    public_key.to_string(),
                    vote
                )
            })
            .collect();
        println!(
            "(String::from_str(&env, {:?}), map![&env, {:#?}])",
            submission_id, votes_map_string
        );
    }
}
fn submissions() {
    println!("-- submissions --");
    let submissions_raw = fs::read_to_string("data/submissions.json").unwrap();
    let submissions: Vec<Submission> = serde_json::from_str(submissions_raw.as_str()).unwrap();
    let mapped: Vec<(String, String)> = submissions
        .iter()
        .map(|s| match s.category {
            neurons::SubmissionCategory::Applications => {
                (s.name.clone(), "Applications".to_string())
            }
            neurons::SubmissionCategory::FinancialProtocols => {
                (s.name.clone(), "FinancialProtocols".to_string())
            }
            neurons::SubmissionCategory::InfrastructureAndServices => {
                (s.name.clone(), "InfrastructureAndServices".to_string())
            }
            neurons::SubmissionCategory::DeveloperTooling => {
                (s.name.clone(), "DeveloperTooling".to_string())
            }
        })
        .collect();
    for (name, category) in mapped {
        println!(
            "(String::from_str(&env,{:?}), String::from_str(&env,{:?})),",
            name, category
        );
    }
}
fn trust() {
    println!("-- trust graph neuron results --");
    let trust_graph_neuron_raw = fs::read_to_string("result/trust_graph_neuron.json").unwrap();
    let trust_graph_neuron: Map<String, Value> =
        serde_json::from_str(trust_graph_neuron_raw.as_str()).unwrap();
    let trust_graph_mapped: Vec<(String, i128)> = trust_graph_neuron
        .iter()
        .map(|(public_key, value)| {
            let n = value.as_str().unwrap().to_string();
            let int = n.parse::<i128>().unwrap();
            (public_key.to_string(), int)
        })
        .collect();
    for (public_key, value) in trust_graph_mapped {
        println!(
            "(String::from_str(&env,{:?}), I256::from_i128(&env,{})),",
            public_key, value
        );
    }
}
fn reputation() {
    println!("-- assigned reputation neuron results --");
    let assigned_reputation_neuron_raw =
        fs::read_to_string("result/assigned_reputation_neuron.json").unwrap();
    let assigned_reputation_neuron: Map<String, Value> =
        serde_json::from_str(assigned_reputation_neuron_raw.as_str()).unwrap();
    let assigned_reputation_neuron_mapped: Vec<(String, i128)> = assigned_reputation_neuron
        .iter()
        .map(|(public_key, value)| {
            let n = value.as_str().unwrap().to_string();
            let int = n.parse::<i128>().unwrap();
            (public_key.to_string(), int)
        })
        .collect();
    for (public_key, value) in assigned_reputation_neuron_mapped {
        println!(
            "(String::from_str(&env,{:?}), I256::from_i128(&env,{})),",
            public_key, value
        );
    }
}
fn voting_history() {
    println!("-- prior voting history neuron results --");
    let prior_voting_history_neuron_raw =
        fs::read_to_string("result/prior_voting_history_neuron.json").unwrap();
    let prior_voting_history_neuron: Map<String, Value> =
        serde_json::from_str(prior_voting_history_neuron_raw.as_str()).unwrap();
    let prior_voting_history_neuron_mapped: Vec<(String, i128)> = prior_voting_history_neuron
        .iter()
        .map(|(public_key, value)| {
            let n = value.as_str().unwrap().to_string();
            let int = n.parse::<i128>().unwrap();
            (public_key.to_string(), int)
        })
        .collect();
    for (public_key, value) in prior_voting_history_neuron_mapped {
        println!(
            "(String::from_str(&env,{:?}), I256::from_i128(&env,{})),",
            public_key, value
        );
    }
}