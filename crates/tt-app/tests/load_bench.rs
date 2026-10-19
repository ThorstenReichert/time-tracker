//! The OQ-9 trigger benchmark (spec §2 "Testing approach"): about 100k entries (roughly 10×
//! the expected volume). Startup load plus invariant check stays under 500 ms, and process
//! memory with the State loaded stays under 50 MB. Also times one switch, because every
//! command clones the State into a candidate.
//!
//! Run: `cargo test -p tt-app --release -- --ignored load_bench --nocapture`

mod common;

use std::time::Instant as Clock;

use common::{Scratch, ctx};
use tt_app::{App, AppConfig};
use tt_core::{
    Client, ClientId, Command, Entry, EntryId, Event, Instant, Name, Note, Project, ProjectId,
    RoundingRule, Task, TaskChoice, TaskId,
};
use tt_store::{MIGRATIONS, Store};

const ENTRIES: u64 = 100_000;
const LOAD_LIMIT_MS: u128 = 500;
const MEMORY_LIMIT_MB: usize = 50;

fn at(seconds: i64) -> Instant {
    Instant::from_unix_seconds(seconds).unwrap()
}

/// 5 clients × 4 projects × 3 tasks, and 100k entries of 20 minutes every 30 minutes from
/// 2021 on (about six years of full days).
fn generated_events() -> Vec<Event> {
    let created = at(1_600_000_000);
    let name = |s: String| Name::parse(&s).unwrap();
    let mut events = Vec::new();
    for c in 1..=5u64 {
        events.push(Event::ClientCreated {
            client: Client {
                id: ClientId::new(c),
                name: name(format!("Client {c}")),
                rule: RoundingRule::default(),
                archived_at: None,
                created_at: created,
            },
        });
        for p in 1..=4u64 {
            let pid = (c - 1) * 4 + p;
            events.push(Event::ProjectCreated {
                project: Project {
                    id: ProjectId::new(pid),
                    client: ClientId::new(c),
                    name: name(format!("Project {p}")),
                    archived_at: None,
                    created_at: created,
                },
            });
            for t in 1..=3u64 {
                events.push(Event::TaskCreated {
                    task: Task {
                        id: TaskId::new((pid - 1) * 3 + t),
                        project: ProjectId::new(pid),
                        name: name(format!("Task {t}")),
                        archived_at: None,
                        created_at: created,
                    },
                });
            }
        }
    }
    let first = 1_609_459_200; // 2021-01-01T00:00:00Z
    for i in 0..ENTRIES {
        let pid = i % 20 + 1;
        let start = first + i as i64 * 1800;
        events.push(Event::EntryAdded {
            entry: Entry {
                id: EntryId::new(i + 1),
                project: ProjectId::new(pid),
                task: (i % 2 == 0).then(|| TaskId::new((pid - 1) * 3 + i % 3 + 1)),
                start: at(start),
                end: Some(at(start + 1200)),
                note: if i % 10 == 0 {
                    Note::parse("a typical short note about the work").unwrap()
                } else {
                    None
                },
                created_at: at(start),
                updated_at: at(start + 1200),
            },
        });
    }
    events
}

/// Fails rather than reading 0, which would pass the memory limit unchecked.
fn physical_mb() -> usize {
    memory_stats::memory_stats()
        .expect("process memory can be measured")
        .physical_mem
        / (1024 * 1024)
}

#[test]
#[ignore = "benchmark; run with --release -- --ignored"]
fn load_bench() {
    let scratch = Scratch::new();
    {
        let (mut store, _) = Store::open(&scratch.db(), MIGRATIONS).unwrap();
        store.migrate().unwrap();
        let built = Clock::now();
        store.apply(&generated_events()).unwrap();
        println!("generated {ENTRIES} entries in {:?}", built.elapsed());
    }
    let size_mb = std::fs::metadata(scratch.db()).unwrap().len() / (1024 * 1024);
    let before_mb = physical_mb();

    // Load + invariant check, as at startup.
    let (store, _) = Store::open(&scratch.db(), MIGRATIONS).unwrap();
    let loading = Clock::now();
    let state = store.load().unwrap();
    state.check_invariants().unwrap();
    let load_ms = loading.elapsed().as_millis();
    let loaded_mb = physical_mb();
    drop((state, store));

    // The full startup (lock, backup, migrations, load, check) and one switch.
    let now = ctx("2027-01-01T12:00");
    let opening = Clock::now();
    let app = App::open(AppConfig::new(scratch.db()), &now).unwrap();
    let open_ms = opening.elapsed().as_millis();
    assert_eq!(app.status().broken, None);
    let switching = Clock::now();
    app.execute(
        &now,
        Command::Start {
            project: ProjectId::new(1),
            task: TaskChoice::NoTask,
        },
    )
    .unwrap();
    let switch_ms = switching.elapsed().as_millis();

    println!("database file:          {size_mb} MB");
    println!("load + check:           {load_ms} ms (limit {LOAD_LIMIT_MS} ms)");
    println!(
        "process memory loaded:  {loaded_mb} MB (before load {before_mb} MB, limit {MEMORY_LIMIT_MB} MB)"
    );
    println!("full startup incl. backup: {open_ms} ms");
    println!("one execute(Start):     {switch_ms} ms");

    if cfg!(debug_assertions) {
        println!("debug build: limits not checked; run with --release");
        return;
    }
    assert!(load_ms < LOAD_LIMIT_MS, "load + check took {load_ms} ms");
    assert!(loaded_mb < MEMORY_LIMIT_MB, "process memory {loaded_mb} MB");
}
