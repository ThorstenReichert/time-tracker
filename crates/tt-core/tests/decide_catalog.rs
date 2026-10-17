//! Given State / when Command / then Events-or-error: the catalog commands (R1, R3, R17),
//! plan.md step 2.

use tt_core::testing::{ctx_berlin, given, run, when};
use tt_core::{
    Catalog, Client, ClientId, Command, Counters, DecimalSeparator, DomainError, Entries, Event,
    ItemRef, Name, NameProblem, ProjectId, RoundingDirection, RoundingLevel, RoundingRule,
    RoundingUnit, SettingProblem, Settings, State, TaskId,
};

fn ctx() -> tt_core::Ctx {
    ctx_berlin("2026-10-17T09:00")
}

fn create_client(name: &str) -> Command {
    Command::CreateClient {
        name: name.into(),
        rule: RoundingRule::default(),
    }
}

fn create_project(client: u64, name: &str) -> Command {
    Command::CreateProject {
        client: ClientId::new(client),
        name: name.into(),
    }
}

fn create_task(project: u64, name: &str) -> Command {
    Command::CreateTask {
        project: ProjectId::new(project),
        name: name.into(),
    }
}

/// Acme (1) with Web (1) and its task Review (1); Beta (2) with API (2).
fn acme_and_beta() -> State {
    run(
        State::default(),
        &ctx(),
        [
            create_client("Acme"),
            create_project(1, "Web"),
            create_task(1, "Review"),
            create_client("Beta"),
            create_project(2, "API"),
        ],
    )
}

#[test]
fn create_client_emits_the_full_row_with_the_next_id() {
    let rule = RoundingRule {
        unit: RoundingUnit::M30,
        direction: RoundingDirection::Up,
        level: RoundingLevel::Client,
    };
    let (events, state) = when(
        &given(&[]),
        &ctx(),
        Command::CreateClient {
            name: "  Acme ".into(),
            rule,
        },
    )
    .unwrap();
    assert_eq!(
        events,
        [Event::ClientCreated {
            client: Client {
                id: ClientId::new(1),
                name: Name::parse("Acme").unwrap(),
                rule,
                archived_at: None,
                created_at: ctx().now,
            }
        }]
    );
    assert_eq!(state.catalog().counters().next_client, 2);
}

#[test]
fn ids_increase_per_kind() {
    let state = acme_and_beta();
    let c = state.catalog();
    let clients: Vec<_> = c.clients().map(|c| c.id.get()).collect();
    let projects: Vec<_> = c.projects().map(|p| (p.id.get(), p.client.get())).collect();
    assert_eq!(clients, [1, 2]);
    assert_eq!(projects, [(1, 1), (2, 2)]);
    assert_eq!(
        c.counters(),
        Counters {
            next_client: 3,
            next_project: 3,
            next_task: 2,
            next_entry: 1
        }
    );
}

#[test]
fn client_names_are_unique_ignoring_case() {
    let err = when(&acme_and_beta(), &ctx(), create_client("ACME")).unwrap_err();
    assert_eq!(
        err,
        DomainError::DuplicateName {
            existing: ItemRef::Client(ClientId::new(1))
        }
    );
}

#[test]
fn invalid_names_are_rejected() {
    assert_eq!(
        when(&given(&[]), &ctx(), create_client(" ")).unwrap_err(),
        DomainError::InvalidName(NameProblem::Empty)
    );
}

#[test]
fn update_client_renames_and_changes_the_rule() {
    let rule = RoundingRule {
        unit: RoundingUnit::M6,
        ..RoundingRule::default()
    };
    let (events, state) = when(
        &acme_and_beta(),
        &ctx(),
        Command::UpdateClient {
            id: ClientId::new(1),
            name: "Acme GmbH".into(),
            rule,
        },
    )
    .unwrap();
    assert_eq!(
        events,
        [Event::ClientUpdated {
            id: ClientId::new(1),
            name: Name::parse("Acme GmbH").unwrap(),
            rule
        }]
    );
    let acme = state.catalog().client(ClientId::new(1)).unwrap();
    assert_eq!((acme.name.as_str(), acme.rule), ("Acme GmbH", rule));
}

#[test]
fn a_case_only_rename_of_the_item_itself_is_allowed() {
    let (events, _) = when(
        &acme_and_beta(),
        &ctx(),
        Command::UpdateClient {
            id: ClientId::new(1),
            name: "ACME".into(),
            rule: RoundingRule::default(),
        },
    )
    .unwrap();
    assert_eq!(events.len(), 1);
    let (events, _) = when(
        &acme_and_beta(),
        &ctx(),
        Command::RenameTask {
            id: TaskId::new(1),
            name: "review".into(),
        },
    )
    .unwrap();
    assert_eq!(events.len(), 1);
}

#[test]
fn renaming_to_another_items_name_is_rejected() {
    let err = when(
        &acme_and_beta(),
        &ctx(),
        Command::UpdateClient {
            id: ClientId::new(2),
            name: "acme".into(),
            rule: RoundingRule::default(),
        },
    )
    .unwrap_err();
    assert_eq!(
        err,
        DomainError::DuplicateName {
            existing: ItemRef::Client(ClientId::new(1))
        }
    );
}

#[test]
fn an_unchanged_update_emits_nothing() {
    let (events, _) = when(
        &acme_and_beta(),
        &ctx(),
        Command::RenameProject {
            id: ProjectId::new(1),
            name: "Web".into(),
        },
    )
    .unwrap();
    assert!(events.is_empty());
}

#[test]
fn project_names_are_unique_per_client_only() {
    let state = acme_and_beta();
    // "API" exists under Beta, so it's free under Acme.
    assert!(when(&state, &ctx(), create_project(1, "API")).is_ok());
    assert_eq!(
        when(&state, &ctx(), create_project(2, "api")).unwrap_err(),
        DomainError::DuplicateName {
            existing: ItemRef::Project(ProjectId::new(2))
        }
    );
}

#[test]
fn task_names_are_unique_per_project_only() {
    let state = acme_and_beta();
    assert!(when(&state, &ctx(), create_task(2, "Review")).is_ok());
    assert_eq!(
        when(&state, &ctx(), create_task(1, " REVIEW ")).unwrap_err(),
        DomainError::DuplicateName {
            existing: ItemRef::Task(TaskId::new(1))
        }
    );
}

#[test]
fn unknown_parents_are_rejected() {
    let state = acme_and_beta();
    assert_eq!(
        when(&state, &ctx(), create_project(9, "X")).unwrap_err(),
        DomainError::UnknownClient(ClientId::new(9))
    );
    assert_eq!(
        when(&state, &ctx(), create_task(9, "X")).unwrap_err(),
        DomainError::UnknownProject(ProjectId::new(9))
    );
    assert_eq!(
        when(
            &state,
            &ctx(),
            Command::RenameTask {
                id: TaskId::new(9),
                name: "X".into()
            }
        )
        .unwrap_err(),
        DomainError::UnknownTask(TaskId::new(9))
    );
}

#[test]
fn settings_are_validated() {
    let state = acme_and_beta();
    let ok = Settings {
        hotkey: Some("Ctrl+Alt+K".into()),
        decimal_separator: DecimalSeparator::Comma,
        ..Settings::default()
    };
    let (events, next) = when(
        &state,
        &ctx(),
        Command::UpdateSettings {
            settings: ok.clone(),
        },
    )
    .unwrap();
    assert_eq!(
        events,
        [Event::SettingsChanged {
            settings: ok.clone()
        }]
    );
    assert_eq!(next.catalog().settings(), &ok);

    let bad = |settings: Settings| {
        when(&state, &ctx(), Command::UpdateSettings { settings }).unwrap_err()
    };
    assert_eq!(
        bad(Settings {
            recents_n: 0,
            ..Settings::default()
        }),
        DomainError::InvalidSetting(SettingProblem::RecentsOutOfRange { max: 50 })
    );
    assert_eq!(
        bad(Settings {
            hotkey: Some(" ".into()),
            ..Settings::default()
        }),
        DomainError::InvalidSetting(SettingProblem::EmptyHotkey)
    );
    assert_eq!(
        bad(Settings {
            backup_keep: 0,
            ..Settings::default()
        }),
        DomainError::InvalidSetting(SettingProblem::BackupKeepOutOfRange { max: 1000 })
    );
}

#[test]
fn a_rejected_command_changes_nothing() {
    let state = acme_and_beta();
    let before = state.clone();
    assert!(when(&state, &ctx(), create_client("acme")).is_err());
    assert_eq!(state, before);
}

#[test]
fn loaded_data_that_breaks_an_invariant_is_detected() {
    let client = |id: u64, name: &str| Client {
        id: ClientId::new(id),
        name: Name::parse(name).unwrap(),
        rule: RoundingRule::default(),
        archived_at: None,
        created_at: ctx().now,
    };
    let counters = Counters {
        next_client: 3,
        ..Counters::default()
    };
    let dup = Catalog::from_parts(
        [client(1, "Acme"), client(2, "ACME")],
        [],
        [],
        Settings::default(),
        counters,
    );
    assert!(
        State::from_parts(dup, Entries::default())
            .check_invariants()
            .is_err()
    );

    // A counter that isn't ahead of the IDs would reuse an ID.
    let behind = Catalog::from_parts(
        [client(1, "Acme")],
        [],
        [],
        Settings::default(),
        Counters::default(),
    );
    assert!(
        State::from_parts(behind, Entries::default())
            .check_invariants()
            .is_err()
    );
}

#[test]
fn commands_and_events_serialise_as_tagged_plain_data() {
    let json = serde_json::to_string(&create_client("Acme")).unwrap();
    assert_eq!(
        json,
        r#"{"type":"CreateClient","name":"Acme","rule":{"unit":15,"direction":"nearest","level":"project"}}"#
    );
    let state = acme_and_beta();
    let round_trip: State = serde_json::from_str(&serde_json::to_string(&state).unwrap()).unwrap();
    assert_eq!(round_trip, state);
}
