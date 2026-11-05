//! Given State / when Command / then Events-or-error: the catalog commands (R1, R3, R17),
//! plan.md step 2.

use tt_core::testing::{ctx_berlin, given, run, when};
use tt_core::{
    Abbreviation, Catalog, Client, ClientId, Command, Counters, DecimalSeparator, DomainError,
    Entries, Event, ItemRef, Name, NameProblem, ProjectId, RoundingDirection, RoundingLevel,
    RoundingRule, RoundingUnit, SettingProblem, Settings, State, TaskId,
};

fn ctx() -> tt_core::Ctx {
    ctx_berlin("2026-10-17T09:00")
}

fn create_client(name: &str) -> Command {
    Command::CreateClient {
        name: name.into(),
        abbreviation: None,
        rule: RoundingRule::default(),
    }
}

fn create_project(client: u64, name: &str) -> Command {
    Command::CreateProject {
        client: ClientId::new(client),
        name: name.into(),
        abbreviation: None,
    }
}

fn create_task(project: u64, name: &str) -> Command {
    Command::CreateTask {
        project: ProjectId::new(project),
        name: name.into(),
        abbreviation: None,
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
            abbreviation: None,
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
                abbreviation: None,
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
            abbreviation: None,
            rule,
        },
    )
    .unwrap();
    assert_eq!(
        events,
        [Event::ClientUpdated {
            id: ClientId::new(1),
            name: Name::parse("Acme GmbH").unwrap(),
            abbreviation: None,
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
            abbreviation: None,
            rule: RoundingRule::default(),
        },
    )
    .unwrap();
    assert_eq!(events.len(), 1);
    let (events, _) = when(
        &acme_and_beta(),
        &ctx(),
        Command::UpdateTask {
            id: TaskId::new(1),
            name: "review".into(),
            abbreviation: None,
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
            abbreviation: None,
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
        Command::UpdateProject {
            id: ProjectId::new(1),
            name: "Web".into(),
            abbreviation: None,
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
            Command::UpdateTask {
                id: TaskId::new(9),
                name: "X".into(),
                abbreviation: None,
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
    // R22: a relative folder would depend on the working directory.
    for relative in ["backups", r"OneDrive\TT", r"\TT", "C:TT", r"%OneDrive%\TT"] {
        assert_eq!(
            bad(Settings {
                backup_dir: Some(relative.into()),
                ..Settings::default()
            }),
            DomainError::InvalidSetting(SettingProblem::RelativeBackupDir),
            "{relative}"
        );
    }
    for full in [r"C:\Users\Me\OneDrive\TT", r"\\server\share\TT"] {
        let settings = Settings {
            backup_dir: Some(full.into()),
            ..Settings::default()
        };
        assert!(
            when(&state, &ctx(), Command::UpdateSettings { settings }).is_ok(),
            "{full}"
        );
    }
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
        abbreviation: None,
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
        r#"{"type":"CreateClient","name":"Acme","abbreviation":null,"rule":{"unit":15,"direction":"nearest","level":"project"}}"#
    );
    let state = acme_and_beta();
    let round_trip: State = serde_json::from_str(&serde_json::to_string(&state).unwrap()).unwrap();
    assert_eq!(round_trip, state);
}

// Abbreviations (R24) and the shared namespace of names and abbreviations (R1).

fn abbr(text: &str) -> Option<String> {
    Some(text.into())
}

fn update_project(id: u64, name: &str, abbreviation: Option<String>) -> Command {
    Command::UpdateProject {
        id: ProjectId::new(id),
        name: name.into(),
        abbreviation,
    }
}

/// Acme (1) with Web (1, "W") and its task Review (1, "Rev"), and Mobile (3).
fn with_abbreviations() -> State {
    run(
        acme_and_beta(),
        &ctx(),
        [
            update_project(1, "Web", abbr("W")),
            Command::UpdateTask {
                id: TaskId::new(1),
                name: "Review".into(),
                abbreviation: abbr("Rev"),
            },
            create_project(1, "Mobile"),
        ],
    )
}

const LONG: &str = "Northwind Instruments: adaptive signal calibration pipeline";

#[test]
fn an_abbreviation_is_set_on_create_and_changed_or_removed_on_update() {
    let (events, state) = when(
        &acme_and_beta(),
        &ctx(),
        Command::CreateProject {
            client: ClientId::new(1),
            name: LONG.into(),
            abbreviation: abbr(" ASCP "),
        },
    )
    .unwrap();
    let Event::ProjectCreated { project } = &events[0] else {
        panic!("{events:?}");
    };
    assert_eq!(
        project.abbreviation,
        Some(Abbreviation::parse("ASCP").unwrap())
    );
    assert_eq!(project.short_name(), "ASCP");

    let (events, state) = when(&state, &ctx(), update_project(3, LONG, abbr("ascp2"))).unwrap();
    assert_eq!(
        events,
        [Event::ProjectUpdated {
            id: ProjectId::new(3),
            name: Name::parse(LONG).unwrap(),
            abbreviation: Some(Abbreviation::parse("ascp2").unwrap()),
        }]
    );
    // Blank text removes it.
    let (events, state) = when(&state, &ctx(), update_project(3, LONG, abbr("  "))).unwrap();
    assert!(matches!(
        &events[..],
        [Event::ProjectUpdated {
            abbreviation: None,
            ..
        }]
    ));
    let project = state.catalog().project(ProjectId::new(3)).unwrap();
    assert_eq!(project.short_name(), LONG);
}

#[test]
fn an_update_without_a_change_emits_nothing() {
    let state = with_abbreviations();
    let (events, _) = when(&state, &ctx(), update_project(1, "Web", abbr("W"))).unwrap();
    assert!(events.is_empty());
    let (events, _) = when(&state, &ctx(), update_project(1, "Web", abbr("w"))).unwrap();
    assert_eq!(events.len(), 1, "a case change is a change");
}

#[test]
fn an_abbreviation_may_not_clash_with_a_sibling_name_or_abbreviation() {
    let state = with_abbreviations();
    let web = ItemRef::Project(ProjectId::new(1));
    // New abbreviation vs a sibling's name and abbreviation, ignoring case.
    for clash in ["web", "w"] {
        assert_eq!(
            when(&state, &ctx(), update_project(3, "Mobile", abbr(clash))).unwrap_err(),
            DomainError::DuplicateAbbreviation { existing: web },
            "{clash}"
        );
    }
    // New name vs a sibling's abbreviation.
    assert_eq!(
        when(&state, &ctx(), update_project(3, "W", None)).unwrap_err(),
        DomainError::DuplicateName { existing: web }
    );
    assert_eq!(
        when(&state, &ctx(), create_project(1, "w")).unwrap_err(),
        DomainError::DuplicateName { existing: web }
    );
    // Tasks and clients follow the same rule.
    assert_eq!(
        when(&state, &ctx(), create_task(1, "REV")).unwrap_err(),
        DomainError::DuplicateName {
            existing: ItemRef::Task(TaskId::new(1))
        }
    );
    assert_eq!(
        when(
            &state,
            &ctx(),
            Command::CreateClient {
                name: "Gamma".into(),
                abbreviation: abbr("beta"),
                rule: RoundingRule::default(),
            }
        )
        .unwrap_err(),
        DomainError::DuplicateAbbreviation {
            existing: ItemRef::Client(ClientId::new(2))
        }
    );
}

#[test]
fn the_namespace_is_per_parent_and_an_item_may_reuse_its_own_name() {
    let state = with_abbreviations();
    // Another client's project may use the same abbreviation.
    assert!(when(&state, &ctx(), update_project(2, "API", abbr("W"))).is_ok());
    // An item's abbreviation may equal its own name.
    assert!(when(&state, &ctx(), update_project(1, "Web", abbr("WEB"))).is_ok());
    // A task created on the fly (R3) can't take a sibling's abbreviation as its name.
    assert_eq!(
        when(
            &state,
            &ctx(),
            Command::Start {
                project: ProjectId::new(1),
                task: tt_core::TaskChoice::New("rev".into()),
            }
        )
        .unwrap_err(),
        DomainError::DuplicateName {
            existing: ItemRef::Task(TaskId::new(1))
        }
    );
}

#[test]
fn invalid_abbreviations_are_rejected() {
    let state = acme_and_beta();
    assert_eq!(
        when(
            &state,
            &ctx(),
            update_project(1, "Web", abbr(&"x".repeat(21)))
        )
        .unwrap_err(),
        DomainError::InvalidAbbreviation(NameProblem::TooLong {
            max: Abbreviation::MAX_CHARS
        })
    );
    assert_eq!(
        when(&state, &ctx(), update_project(1, "Web", abbr("a\nb"))).unwrap_err(),
        DomainError::InvalidAbbreviation(NameProblem::ControlCharacter)
    );
}

#[test]
fn loaded_data_with_a_name_clashing_with_an_abbreviation_is_detected() {
    let client = |id: u64, name: &str, abbreviation: Option<&str>| Client {
        id: ClientId::new(id),
        name: Name::parse(name).unwrap(),
        abbreviation: abbreviation.map(|a| Abbreviation::parse(a).unwrap()),
        rule: RoundingRule::default(),
        archived_at: None,
        created_at: ctx().now,
    };
    let counters = Counters {
        next_client: 3,
        ..Counters::default()
    };
    let valid = |clients: [Client; 2]| {
        State::from_parts(
            Catalog::from_parts(clients, [], [], Settings::default(), counters),
            Entries::default(),
        )
        .check_invariants()
        .is_ok()
    };
    assert!(valid([
        client(1, "Acme", Some("ACME")),
        client(2, "Beta", Some("B"))
    ]));
    assert!(valid([
        client(1, "Acme", Some("B")),
        client(2, "Beta", None)
    ]));
    assert!(!valid([
        client(1, "Acme", Some("beta")),
        client(2, "Beta", None)
    ]));
    assert!(!valid([
        client(1, "Acme", Some("X")),
        client(2, "Beta", Some("x"))
    ]));
}

#[test]
fn archived_siblings_count_for_names_and_abbreviations() {
    // R1, R24, OQ-7: an archived item keeps its name and abbreviation.
    let project =
        |id: u64, name: &str, abbreviation: Option<&str>, archived: bool| tt_core::Project {
            id: ProjectId::new(id),
            client: ClientId::new(1),
            name: Name::parse(name).unwrap(),
            abbreviation: abbreviation.map(|a| Abbreviation::parse(a).unwrap()),
            archived_at: archived.then(|| ctx().now),
            created_at: ctx().now,
        };
    let client = Client {
        id: ClientId::new(1),
        name: Name::parse("Acme").unwrap(),
        abbreviation: None,
        rule: RoundingRule::default(),
        archived_at: None,
        created_at: ctx().now,
    };
    let counters = Counters {
        next_client: 2,
        next_project: 3,
        ..Counters::default()
    };
    let state = State::from_parts(
        Catalog::from_parts(
            [client],
            [
                project(1, "Old web", Some("OW"), true),
                project(2, "Mobile", None, false),
            ],
            [],
            Settings::default(),
            counters,
        ),
        Entries::default(),
    );
    state.check_invariants().unwrap();
    let archived = ItemRef::Project(ProjectId::new(1));
    assert_eq!(
        when(&state, &ctx(), create_project(1, "ow")).unwrap_err(),
        DomainError::DuplicateName { existing: archived }
    );
    assert_eq!(
        when(&state, &ctx(), update_project(2, "Mobile", abbr("old web"))).unwrap_err(),
        DomainError::DuplicateAbbreviation { existing: archived }
    );
}

// R2a: archive and unarchive a project (changes/2026-11-05-archive-projects).

const WEB: ProjectId = ProjectId::new(1);

#[test]
fn archive_and_unarchive_a_project() {
    let (events, state) = when(
        &acme_and_beta(),
        &ctx(),
        Command::ArchiveProject { id: WEB },
    )
    .unwrap();
    assert_eq!(
        events,
        [Event::ProjectArchived {
            id: WEB,
            at: ctx().now
        }]
    );
    assert_eq!(
        state.catalog().project(WEB).unwrap().archived_at,
        Some(ctx().now)
    );
    assert!(!state.catalog().is_project_active(WEB));
    assert!(!state.catalog().is_task_active(TaskId::new(1)));

    // Archiving again changes nothing.
    assert_eq!(
        when(&state, &ctx(), Command::ArchiveProject { id: WEB })
            .unwrap()
            .0,
        []
    );

    let (events, state) = when(&state, &ctx(), Command::UnarchiveProject { id: WEB }).unwrap();
    assert_eq!(events, [Event::ProjectUnarchived { id: WEB }]);
    assert!(state.catalog().is_project_active(WEB));
    assert!(state.catalog().is_task_active(TaskId::new(1)));

    // Unarchiving an active project changes nothing.
    assert_eq!(
        when(&state, &ctx(), Command::UnarchiveProject { id: WEB })
            .unwrap()
            .0,
        []
    );
}

#[test]
fn archive_and_unarchive_an_unknown_project_are_rejected() {
    let missing = ProjectId::new(99);
    for cmd in [
        Command::ArchiveProject { id: missing },
        Command::UnarchiveProject { id: missing },
    ] {
        assert_eq!(
            when(&acme_and_beta(), &ctx(), cmd).unwrap_err(),
            DomainError::UnknownProject(missing)
        );
    }
}

#[test]
fn the_running_project_can_not_be_archived() {
    let running = run(
        acme_and_beta(),
        &ctx(),
        [Command::Start {
            project: WEB,
            task: tt_core::TaskChoice::NoTask,
        }],
    );
    assert_eq!(
        when(&running, &ctx(), Command::ArchiveProject { id: WEB }).unwrap_err(),
        DomainError::ProjectRunning { project: WEB }
    );
    // Another project can be archived while the timer runs.
    assert!(
        when(
            &running,
            &ctx(),
            Command::ArchiveProject {
                id: ProjectId::new(2)
            }
        )
        .is_ok()
    );
}

#[test]
fn an_archived_project_gets_no_new_tasks_but_can_be_renamed() {
    let state = run(
        acme_and_beta(),
        &ctx(),
        [Command::ArchiveProject { id: WEB }],
    );
    assert_eq!(
        when(&state, &ctx(), create_task(1, "Docs")).unwrap_err(),
        DomainError::Archived {
            item: ItemRef::Project(WEB)
        }
    );
    let rename = Command::UpdateProject {
        id: WEB,
        name: "Old web".into(),
        abbreviation: None,
    };
    assert!(when(&state, &ctx(), rename).is_ok());
}

#[test]
fn an_archived_project_keeps_its_name_and_unarchives_as_it_was() {
    // OQ-7: the name stays taken while archived, so unarchiving can't clash.
    let state = run(
        acme_and_beta(),
        &ctx(),
        [Command::ArchiveProject { id: WEB }],
    );
    assert_eq!(
        when(&state, &ctx(), create_project(1, "web")).unwrap_err(),
        DomainError::DuplicateName {
            existing: ItemRef::Project(WEB)
        }
    );
    let (_, after) = when(&state, &ctx(), Command::UnarchiveProject { id: WEB }).unwrap();
    assert_eq!(
        after.catalog().project(WEB),
        acme_and_beta().catalog().project(WEB)
    );
}
