use neuron_core::{calculate_result, NeuronInput, NeuronOutput};
use risc0_zkvm::guest::env;

fn main() {
    // Read the users (public key, tier, discord roles) from the host.
    let input: NeuronInput = env::read();

    let output: NeuronOutput = calculate_result(&input);

    // Commit the scores to the journal: this is the public, proven output.
    env::commit(&output);
}
