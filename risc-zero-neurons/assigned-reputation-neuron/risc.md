# zk Assigned Reputation Neuron — work notes

**Status (2026-08-26):** working proof of concept. The `assigned_reputation` neuron
from `../../neurons/src/assigned_reputation.rs` is ported to the RISC Zero zkVM and
proves real scores for all 672 users from `data/usersDiscord.json`.
Real proof takes ~2 min on the M-series laptop; dev mode is instant.

## What this does

Proves: *"these (public_key, bonus) scores were computed by exactly this neuron
program from some input dataset"* — without revealing the dataset.

- **Public (journal):** `(public_key, bonus)` pairs, sorted by public key.
- **Private (never leaves the guest):** discord ids, usernames, roles, tiers, vote history.
  The discord `id` / `username` fields are stripped in the host and never even enter the zkVM.
- **Known trust gap (accepted for now):** the proof does not attest the input was the
  *real* dataset — users still trust the prover on data authenticity. See next steps.

## Layout

```
core/     neuron-core: shared types + the ported scoring logic (single source of truth,
          compiled into BOTH host and guest; unit-tested natively — includes a test
          mirroring the original neuron_run test)
methods/  risc0 template glue; methods/guest/src/main.rs is the guest:
          env::read(NeuronInput) -> calculate_result() -> env::commit(NeuronOutput)
host/     reads data/usersDiscord.json, builds sorted NeuronInput, proves, decodes
          journal, verifies receipt
data/     usersDiscord.json (672 users; 2 entries have "username": null)
```

Schema adaptation vs the original neuron: instead of two maps
(`users_reputation`, `users_discord_roles`), input is
`Vec<UserRecord { public_key, tier: i32, discord_roles: Vec<String> }>`.
Tier mapping: `-1 -> Unknown, 0 -> Verified, 1 -> Pathfinder, 2 -> Navigator, 3 -> Pilot`
(anything else -> Unknown). Bonus tables are copied verbatim from the original.

## Running

```bash
# fast iteration — fake receipts, DO NOT use in production
RISC0_DEV_MODE=1 cargo run --release -p host

# real proof (~2 min)
cargo run --release -p host

# logic tests (native, no zkVM)
cargo test -p neuron-core --lib
```

Requires the rzup toolchain; `r0vm` **3.0.6** is installed and matches the
`risc0-zkvm = "^3.0.6"` pinned here. (Keep these in sync — a version-mismatched
r0vm fails with cryptic "Connection refused" / "error deserializing ProofRequest".)

## Key decisions

- **Shared `neuron-core` crate** instead of duplicating logic in the guest: what is
  proven is byte-for-byte the code that's unit-tested natively.
- **Host sorts users by public key** before writing to the guest — HashMap iteration
  order is random, sorting makes the journal deterministic (same data -> same journal).
- **Scores are `f64`** for parity with the original neuron. All values are multiples
  of 0.5, so exact. Fine for the PoC; must change for Soroban (below).
- Proving runs fully locally: `default_prover()` spawns the local `r0vm` binary
  (Metal-accelerated). No Bonsai / no network unless `BONSAI_API_*` env vars are set.

## Caveats

- Score values leak a little by inference: with the public rule table, a score
  constrains the possible tier+roles combinations. Inherent to publishing exact scores.
- Dev-mode receipts are fake and verification of them is fake — never ship them.

## Next steps (rough order)

1. **Dataset commitment:** commit `hash(salt || dataset)` to the journal; whoever is
   trusted with the dataset attests to that hash out-of-band. Salt is required —
   per-user records are low-entropy and brute-forceable unsalted.
2. **Integer scores for Soroban:** journal as fixed-point (score × 2 as u32) —
   Soroban contracts can't consume f64.
3. On-chain verification path: risc0 receipt -> Groth16 (`ReceiptKind::Groth16`) and
   check what verifier exists for Soroban; journal format then matters a lot.
4. Port the remaining neurons from `../../neurons/src/` the same way (shared core
   crate pattern); eventually one guest that runs all neurons + aggregation.
5. If per-user scores should ever be private too: commit only a Merkle root of the
   score list, hand each user their inclusion proof.

## Session context / gotchas from the porting session

- The risc0 monorepo at `~/Desktop/STELLAR/risc0` is checked out at tag `v3.0.6`
  to match the installed toolchain. Its `main` branch (v5-dev) cannot build the
  local prover on this machine (needs full Xcode for Metal; also has a build bug in
  `rv32im-sys/build.rs` when the `metal` tool is missing, and no CPU fallback on
  Apple Silicon).
- `git-lfs` was installed via brew for the risc0 repo (circuit archives are LFS files).
- `usersDiscord.json` has `"username": null` for 2 users — hence `Option<String>`.
