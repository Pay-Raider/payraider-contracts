use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, String};

use analytics::{AnalyticsContract, AnalyticsContractClient};
use governance::{GovernanceContract, GovernanceContractClient, VoteChoice};
use stellar_insights::{StellarInsightsContract, StellarInsightsContractClient};

// ============================================================================
// Helpers
// ============================================================================

fn setup_analytics(env: &Env) -> (AnalyticsContractClient, Address) {
    let contract_id = env.register_contract(None, AnalyticsContract);
    let client = AnalyticsContractClient::new(env, &contract_id);
    let admin = Address::generate(env);
    env.mock_all_auths();
    client.initialize(&admin).unwrap();
    (client, admin)
}

fn setup_stellar_insights(env: &Env) -> (StellarInsightsContractClient, Address) {
    let contract_id = env.register_contract(None, StellarInsightsContract);
    let client = StellarInsightsContractClient::new(env, &contract_id);
    let admin = Address::generate(env);
    env.mock_all_auths();
    client.initialize(&admin);
    (client, admin)
}

fn make_hash(env: &Env, seed: u8) -> BytesN<32> {
    BytesN::from_array(env, &[seed; 32])
}

// ============================================================================
// bench_submit_snapshot
// Measures cost of submitting a single snapshot to AnalyticsContract.
// Each iteration uses a fresh, strictly-increasing epoch.
// ============================================================================

fn bench_submit_snapshot(c: &mut Criterion) {
    let env = Env::default();
    let (client, admin) = setup_analytics(&env);
    let mut epoch = 1u64;

    c.bench_function("analytics::submit_snapshot", |b| {
        b.iter(|| {
            let hash = make_hash(&env, (epoch % 255) as u8);
            client
                .submit_snapshot(black_box(&epoch), black_box(&hash), black_box(&admin))
                .unwrap();
            epoch += 1;
        })
    });
}

// ============================================================================
// bench_get_snapshot
// Measures retrieval cost after pre-populating 100 snapshots.
// ============================================================================

fn bench_get_snapshot(c: &mut Criterion) {
    let env = Env::default();
    let (client, admin) = setup_analytics(&env);

    for epoch in 1u64..=100 {
        let hash = make_hash(&env, (epoch % 255) as u8);
        client.submit_snapshot(&epoch, &hash, &admin).unwrap();
    }

    c.bench_function("analytics::get_snapshot", |b| {
        b.iter(|| client.get_snapshot(black_box(&50u64)))
    });
}

// ============================================================================
// bench_get_latest_snapshot
// Measures retrieval cost of the latest snapshot pointer.
// ============================================================================

fn bench_get_latest_snapshot(c: &mut Criterion) {
    let env = Env::default();
    let (client, admin) = setup_analytics(&env);

    for epoch in 1u64..=50 {
        let hash = make_hash(&env, (epoch % 255) as u8);
        client.submit_snapshot(&epoch, &hash, &admin).unwrap();
    }

    c.bench_function("analytics::get_latest_snapshot", |b| {
        b.iter(|| client.get_latest_snapshot())
    });
}

// ============================================================================
// bench_batch_operations
// Measures cost of submitting N snapshots in sequence.
// Parameterised over batch sizes: 5, 10, 25, 50.
// A fresh Env is created per iteration to keep epochs valid.
// ============================================================================

fn bench_batch_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("analytics::batch_submit");

    for batch_size in [5u64, 10, 25, 50] {
        group.bench_with_input(
            BenchmarkId::from_parameter(batch_size),
            &batch_size,
            |b, &size| {
                b.iter(|| {
                    let env = Env::default();
                    let (client, admin) = setup_analytics(&env);
                    for epoch in 1..=size {
                        let hash = make_hash(&env, (epoch % 255) as u8);
                        client
                            .submit_snapshot(black_box(&epoch), black_box(&hash), black_box(&admin))
                            .unwrap();
                    }
                })
            },
        );
    }

    group.finish();
}

// ============================================================================
// bench_snapshot_history_growth
// Measures get_snapshot_history read cost as the map grows.
// ============================================================================

fn bench_snapshot_history_growth(c: &mut Criterion) {
    let mut group = c.benchmark_group("analytics::history_read");

    for size in [10u64, 50, 100] {
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &n| {
            let env = Env::default();
            let (client, admin) = setup_analytics(&env);
            for epoch in 1..=n {
                let hash = make_hash(&env, (epoch % 255) as u8);
                client.submit_snapshot(&epoch, &hash, &admin).unwrap();
            }
            b.iter(|| client.get_snapshot_history())
        });
    }

    group.finish();
}

// ============================================================================
// bench_stellar_insights_submit
// Submit benchmark against the stellar_insights contract variant.
// Result is unwrapped — a panic here indicates a contract regression.
// ============================================================================

fn bench_stellar_insights_submit(c: &mut Criterion) {
    let env = Env::default();
    let (client, admin) = setup_stellar_insights(&env);
    let mut epoch = 1u64;

    c.bench_function("stellar_insights::submit_snapshot", |b| {
        b.iter(|| {
            let hash = make_hash(&env, (epoch % 255) as u8);
            client
                .submit_snapshot(black_box(&epoch), black_box(&hash), black_box(&admin))
                .unwrap();
            epoch += 1;
        })
    });
}

// ============================================================================
// bench_stellar_insights_get
// Measures get_snapshot cost on the stellar_insights contract.
// ============================================================================

fn bench_stellar_insights_get(c: &mut Criterion) {
    let env = Env::default();
    let (client, admin) = setup_stellar_insights(&env);

    for epoch in 1u64..=100 {
        let hash = make_hash(&env, (epoch % 255) as u8);
        client.submit_snapshot(&epoch, &hash, &admin).unwrap();
    }

    c.bench_function("stellar_insights::get_snapshot", |b| {
        b.iter(|| client.get_snapshot(black_box(&50u64)).unwrap())
    });
}

// ============================================================================
// bench_stellar_insights_latest
// Measures latest_snapshot cost on the stellar_insights contract.
// ============================================================================

fn bench_stellar_insights_latest(c: &mut Criterion) {
    let env = Env::default();
    let (client, admin) = setup_stellar_insights(&env);

    for epoch in 1u64..=50 {
        let hash = make_hash(&env, (epoch % 255) as u8);
        client.submit_snapshot(&epoch, &hash, &admin).unwrap();
    }

    c.bench_function("stellar_insights::latest_snapshot", |b| {
        b.iter(|| client.latest_snapshot().unwrap())
    });
}

// ============================================================================
// Registration
// ============================================================================

criterion_group!(
    analytics_benches,
    bench_submit_snapshot,
    bench_get_snapshot,
    bench_get_latest_snapshot,
    bench_batch_operations,
    bench_snapshot_history_growth,
);

criterion_group!(
    stellar_insights_benches,
    bench_stellar_insights_submit,
    bench_stellar_insights_get,
    bench_stellar_insights_latest,
);

// ============================================================================
// Governance helpers
// ============================================================================

fn setup_governance(env: &Env) -> (GovernanceContractClient, Address) {
    let contract_id = env.register_contract(None, GovernanceContract);
    let client = GovernanceContractClient::new(env, &contract_id);
    let admin = Address::generate(env);
    env.mock_all_auths();
    // quorum=1, voting_period=3600 (1 hour)
    client.initialize(&admin, &1u64, &3600u64).unwrap();
    (client, admin)
}

fn make_title(env: &Env, n: u64) -> String {
    // Soroban String::from_str requires a &str literal; use a fixed title and
    // vary the wasm hash seed instead to keep epochs distinct.
    let _ = n;
    String::from_str(env, "Benchmark Proposal")
}

// ============================================================================
// bench_governance_create_proposal
// Measures cost of creating a new governance proposal.
// ============================================================================

fn bench_governance_create_proposal(c: &mut Criterion) {
    let env = Env::default();
    let (client, admin) = setup_governance(&env);
    let target = Address::generate(&env);
    let mut seed = 1u8;

    c.bench_function("governance::create_proposal", |b| {
        b.iter(|| {
            let hash = BytesN::from_array(&env, &[seed; 32]);
            let title = make_title(&env, seed as u64);
            client
                .create_proposal(
                    black_box(&admin),
                    black_box(&title),
                    black_box(&target),
                    black_box(&hash),
                )
                .unwrap();
            seed = seed.wrapping_add(1);
        })
    });
}

// ============================================================================
// bench_governance_vote
// Measures cost of casting a vote on an active proposal.
// A fresh env + proposal is created per iteration so the voter is always new.
// ============================================================================

fn bench_governance_vote(c: &mut Criterion) {
    c.bench_function("governance::vote", |b| {
        b.iter(|| {
            let env = Env::default();
            let (client, admin) = setup_governance(&env);
            let target = Address::generate(&env);
            let hash = BytesN::from_array(&env, &[1u8; 32]);
            let title = make_title(&env, 1);
            let proposal_id = client
                .create_proposal(&admin, &title, &target, &hash)
                .unwrap();

            let voter = Address::generate(&env);
            client
                .vote(black_box(&voter), black_box(&proposal_id), black_box(&VoteChoice::For))
                .unwrap();
        })
    });
}

// ============================================================================
// bench_governance_multi_vote
// Measures tally update cost as voter count grows.
// Parameterised over voter counts: 5, 10, 25, 50.
// ============================================================================

fn bench_governance_multi_vote(c: &mut Criterion) {
    let mut group = c.benchmark_group("governance::multi_vote");

    for voter_count in [5u32, 10, 25, 50] {
        group.bench_with_input(
            BenchmarkId::from_parameter(voter_count),
            &voter_count,
            |b, &n| {
                b.iter(|| {
                    let env = Env::default();
                    let (client, admin) = setup_governance(&env);
                    let target = Address::generate(&env);
                    let hash = BytesN::from_array(&env, &[1u8; 32]);
                    let title = make_title(&env, 1);
                    let proposal_id = client
                        .create_proposal(&admin, &title, &target, &hash)
                        .unwrap();

                    for _ in 0..n {
                        let voter = Address::generate(&env);
                        client
                            .vote(black_box(&voter), black_box(&proposal_id), black_box(&VoteChoice::For))
                            .unwrap();
                    }
                })
            },
        );
    }

    group.finish();
}

// ============================================================================
// bench_governance_get_proposal
// Measures read cost of fetching a proposal by id.
// ============================================================================

fn bench_governance_get_proposal(c: &mut Criterion) {
    let env = Env::default();
    let (client, admin) = setup_governance(&env);
    let target = Address::generate(&env);
    let hash = BytesN::from_array(&env, &[1u8; 32]);
    let title = make_title(&env, 1);
    let proposal_id = client
        .create_proposal(&admin, &title, &target, &hash)
        .unwrap();

    c.bench_function("governance::get_proposal", |b| {
        b.iter(|| client.get_proposal(black_box(&proposal_id)))
    });
}

// ============================================================================
// Registration
// ============================================================================

criterion_group!(
    analytics_benches,
    bench_submit_snapshot,
    bench_get_snapshot,
    bench_get_latest_snapshot,
    bench_batch_operations,
    bench_snapshot_history_growth,
);

criterion_group!(
    stellar_insights_benches,
    bench_stellar_insights_submit,
    bench_stellar_insights_get,
    bench_stellar_insights_latest,
);

criterion_group!(
    governance_benches,
    bench_governance_create_proposal,
    bench_governance_vote,
    bench_governance_multi_vote,
    bench_governance_get_proposal,
);

criterion_main!(analytics_benches, stellar_insights_benches, governance_benches);
