//! apply → load equals the evolved State (plan.md step 5), for random sessions from the
//! tt-core generator. tt-app repeats this through its write path in step 6.

use proptest::prelude::*;
use tempfile::TempDir;
use tt_core::testing::{advance, arb_start_ctx, arb_steps, materialize};
use tt_core::{DecimalSeparator, Settings, State, decide, evolve_all};
use tt_store::{MIGRATIONS, Store};

proptest! {
    #![proptest_config(ProptestConfig { cases: 48, ..ProptestConfig::default() })]

    #[test]
    fn loaded_state_equals_the_evolved_state(start in arb_start_ctx(), steps in arb_steps(120)) {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("t.db");
        let (mut store, _) = Store::open(&path, MIGRATIONS).unwrap();
        store.migrate().unwrap();

        let mut ctx = start;
        let mut state = State::default();
        for step in &steps {
            ctx = advance(&ctx, step.advance_secs);
            let cmd = materialize(&step.op, &state, &ctx);
            if let Ok(events) = decide(&state, &ctx, &cmd) {
                store.apply(&events).unwrap();
                state = evolve_all(state, &events);
            }
        }
        drop(store);

        let (store, _) = Store::open(&path, MIGRATIONS).unwrap();
        let loaded = store.load().unwrap();
        loaded.check_invariants().unwrap();
        prop_assert_eq!(&loaded, &state);
        // The same serialised form, as the spec's reload test asks.
        prop_assert_eq!(
            serde_json::to_string(&loaded).unwrap(),
            serde_json::to_string(&state).unwrap()
        );
    }
}

#[test]
fn all_settings_round_trip() {
    let dir = TempDir::new().unwrap();
    let (mut store, _) = Store::open(&dir.path().join("t.db"), MIGRATIONS).unwrap();
    store.migrate().unwrap();
    let settings = Settings {
        hotkey: Some("Ctrl+Alt+K".into()),
        decimal_separator: DecimalSeparator::Comma,
        recents_n: 5,
        backup_dir: Some(r"C:\Users\me\OneDrive\tt-backups".into()),
        backup_keep: 30,
    };
    store
        .apply(&[tt_core::Event::SettingsChanged {
            settings: settings.clone(),
        }])
        .unwrap();
    assert_eq!(store.load().unwrap().catalog().settings(), &settings);

    // Back to the defaults: the optional keys are removed again.
    store
        .apply(&[tt_core::Event::SettingsChanged {
            settings: Settings::default(),
        }])
        .unwrap();
    assert_eq!(
        store.load().unwrap().catalog().settings(),
        &Settings::default()
    );
}

#[test]
fn archive_and_unarchive_round_trip() {
    // R2a (changes/2026-11-05-archive-projects); the property test above covers it at random.
    use tt_core::testing::ctx_berlin;
    use tt_core::{ClientId, Command, ProjectId, RoundingRule};
    let dir = TempDir::new().unwrap();
    let (mut store, _) = Store::open(&dir.path().join("t.db"), MIGRATIONS).unwrap();
    store.migrate().unwrap();
    let ctx = ctx_berlin("2026-10-17T09:00");
    let web = ProjectId::new(1);
    let mut state = State::default();
    for cmd in [
        Command::CreateClient {
            name: "Acme".into(),
            abbreviation: None,
            rule: RoundingRule::default(),
        },
        Command::CreateProject {
            client: ClientId::new(1),
            name: "Web".into(),
            abbreviation: None,
        },
        Command::ArchiveProject { id: web },
    ] {
        let events = decide(&state, &ctx, &cmd).unwrap();
        store.apply(&events).unwrap();
        state = evolve_all(state, &events);
    }
    let loaded = store.load().unwrap();
    assert_eq!(loaded, state);
    assert_eq!(
        loaded.catalog().project(web).unwrap().archived_at,
        Some(ctx.now)
    );

    let events = decide(&state, &ctx, &Command::UnarchiveProject { id: web }).unwrap();
    store.apply(&events).unwrap();
    let state = evolve_all(state, &events);
    let loaded = store.load().unwrap();
    assert_eq!(loaded, state);
    assert_eq!(loaded.catalog().project(web).unwrap().archived_at, None);
}
