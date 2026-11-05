//! Read models (plan.md step 4): daily overview (R14, R16, R18, R19), day entries (R10),
//! recents (R7), quick-switch matching (R6) and quick task creation (R3).

use jiff::civil;
use tt_core::read::{
    self, ClientBlock, Figure, NameCheck, Overview, ProjectRow, day_bounds, local_date,
};
use tt_core::testing::{at, berlin, ctx_berlin, run};
use tt_core::{
    Catalog, ClientId, Combo, Command, Counters, Ctx, Entries, Entry, EntryId, Instant, Name,
    ProjectId, RoundingDirection, RoundingLevel, RoundingRule, RoundingUnit, Settings, State,
    TaskChoice, TaskId,
};

const MIN: i64 = 60;
const ACME: ClientId = ClientId::new(1);
const BETA: ClientId = ClientId::new(2);
const WEB: ProjectId = ProjectId::new(1);
const API: ProjectId = ProjectId::new(2);
const OPS: ProjectId = ProjectId::new(3);

fn ctx(local: &str) -> Ctx {
    ctx_berlin(local)
}

fn t(local: &str) -> Instant {
    at(&berlin(), local)
}

fn date(s: &str) -> civil::Date {
    s.parse().unwrap()
}

fn rule(unit: RoundingUnit, direction: RoundingDirection, level: RoundingLevel) -> RoundingRule {
    RoundingRule {
        unit,
        direction,
        level,
    }
}

/// Acme: project level, 15 min nearest, project Web (1) with task Review (1).
/// Beta: client level, 30 min up, projects API (2) and Ops (3).
fn catalog() -> State {
    run(
        State::default(),
        &ctx("2026-10-17T08:00"),
        [
            Command::CreateClient {
                name: "Acme".into(),
                abbreviation: None,
                rule: rule(
                    RoundingUnit::M15,
                    RoundingDirection::Nearest,
                    RoundingLevel::Project,
                ),
            },
            Command::CreateProject {
                client: ACME,
                name: "Web".into(),
                abbreviation: None,
            },
            Command::CreateTask {
                project: WEB,
                name: "Review".into(),
                abbreviation: None,
            },
            Command::CreateClient {
                name: "Beta".into(),
                abbreviation: None,
                rule: rule(
                    RoundingUnit::M30,
                    RoundingDirection::Up,
                    RoundingLevel::Client,
                ),
            },
            Command::CreateProject {
                client: BETA,
                name: "API".into(),
                abbreviation: None,
            },
            Command::CreateProject {
                client: BETA,
                name: "Ops".into(),
                abbreviation: None,
            },
        ],
    )
}

fn start_at(state: State, local: &str, project: ProjectId) -> State {
    run(
        state,
        &ctx(local),
        [Command::Start {
            project,
            task: TaskChoice::NoTask,
        }],
    )
}

/// The worked day of plan.md step 4: Web 09:00–10:22, API 10:22–11:07, Ops 11:07–11:20.
fn worked_day() -> State {
    let s = start_at(catalog(), "2026-10-17T09:00", WEB);
    let s = start_at(s, "2026-10-17T10:22", API);
    let s = start_at(s, "2026-10-17T11:07", OPS);
    run(s, &ctx("2026-10-17T11:20"), [Command::Stop { at: None }])
}

fn overview(state: &State, now: &str, day: &str) -> Overview {
    let tz = berlin();
    let (from, to) = day_bounds(date(day), &tz).unwrap();
    let entries = state.entries().started_between(from, to);
    read::daily_overview(state.catalog(), &entries, t(now), &tz, date(day))
}

fn figure(raw: i64, rounded: i64) -> Figure {
    Figure {
        raw,
        rounded,
        diff: rounded - raw,
    }
}

#[test]
fn worked_day_project_and_client_level() {
    let o = overview(&worked_day(), "2026-10-17T12:00", "2026-10-17");
    assert_eq!(o.date, date("2026-10-17"));
    let [acme, beta] = o.clients.as_slice() else {
        panic!("two clients: {o:#?}")
    };

    // Project level: each project rounded; the client total is the sum of the rounded values.
    assert_eq!(
        acme,
        &ClientBlock {
            client: ACME,
            name: Name::parse("Acme").unwrap(),
            rule: rule(
                RoundingUnit::M15,
                RoundingDirection::Nearest,
                RoundingLevel::Project
            ),
            total: figure(4920, 4500),
            projects: vec![ProjectRow {
                project: WEB,
                name: Name::parse("Web").unwrap(),
                raw: 4920,
                booked: Some(figure(4920, 4500)),
                running: false,
            }],
            running: false,
        }
    );
    assert_eq!(acme.total.diff, -420);

    // Client level: one rounded total; the project rows are raw only.
    assert_eq!(beta.total, figure(3480, 3600));
    assert_eq!(beta.total.diff, 120);
    let rows: Vec<_> = beta
        .projects
        .iter()
        .map(|p| (p.name.as_str(), p.raw, p.booked))
        .collect();
    assert_eq!(rows, [("API", 45 * MIN, None), ("Ops", 13 * MIN, None)]);
}

#[test]
fn project_level_subtotal_is_the_sum_of_rounded_projects() {
    let s = run(
        catalog(),
        &ctx("2026-10-17T08:00"),
        [Command::CreateProject {
            client: ACME,
            name: "Docs".into(),
            abbreviation: None,
        }],
    );
    let docs = ProjectId::new(4);
    let s = start_at(s, "2026-10-17T09:00", WEB); // 22 min → 15
    let s = start_at(s, "2026-10-17T09:22", docs); // 22 min → 15
    let s = run(s, &ctx("2026-10-17T09:44"), [Command::Stop { at: None }]);
    let o = overview(&s, "2026-10-17T12:00", "2026-10-17");
    let acme = &o.clients[0];
    // Rounding the sum (44 min → 45) would give a different value; R17 project level rounds
    // each project.
    assert_eq!(acme.total, figure(44 * MIN, 30 * MIN));
    let names: Vec<_> = acme.projects.iter().map(|p| p.name.as_str()).collect();
    assert_eq!(names, ["Docs", "Web"], "sorted by name");
}

#[test]
fn an_entry_crossing_midnight_counts_on_its_start_day() {
    let s = start_at(catalog(), "2026-10-17T23:30", WEB);
    let s = run(s, &ctx("2026-10-18T00:30"), [Command::Stop { at: None }]);
    let day1 = overview(&s, "2026-10-18T12:00", "2026-10-17");
    assert_eq!(day1.clients[0].total.raw, 3600);
    let day2 = overview(&s, "2026-10-18T12:00", "2026-10-18");
    assert!(
        day2.clients.is_empty(),
        "groups with no entries don't appear"
    );
}

#[test]
fn a_running_entry_counts_up_to_now_and_is_marked() {
    let s = start_at(catalog(), "2026-10-17T09:00", WEB);
    let o = overview(&s, "2026-10-17T09:40", "2026-10-17");
    let acme = &o.clients[0];
    assert!(acme.running && acme.projects[0].running);
    assert_eq!(acme.total, figure(40 * MIN, 45 * MIN));
}

#[test]
fn rule_changes_apply_retroactively() {
    let s = run(
        worked_day(),
        &ctx("2026-10-18T09:00"),
        [Command::UpdateClient {
            id: BETA,
            name: "Beta".into(),
            abbreviation: None,
            rule: rule(
                RoundingUnit::M15,
                RoundingDirection::Down,
                RoundingLevel::Client,
            ),
        }],
    );
    let beta = &overview(&s, "2026-10-18T09:00", "2026-10-17").clients[1];
    assert_eq!(beta.total, figure(3480, 45 * MIN));

    // Switching Beta to project level books per project instead.
    let s = run(
        s,
        &ctx("2026-10-18T09:00"),
        [Command::UpdateClient {
            id: BETA,
            name: "Beta".into(),
            abbreviation: None,
            rule: rule(
                RoundingUnit::M15,
                RoundingDirection::Down,
                RoundingLevel::Project,
            ),
        }],
    );
    let beta = &overview(&s, "2026-10-18T09:00", "2026-10-17").clients[1];
    let booked: Vec<_> = beta
        .projects
        .iter()
        .map(|p| p.booked.unwrap().rounded)
        .collect();
    assert_eq!(booked, [45 * MIN, 0]);
    assert_eq!(beta.total, figure(3480, 45 * MIN));
}

#[test]
fn stored_entries_stay_raw() {
    let s = worked_day();
    let _ = overview(&s, "2026-10-17T12:00", "2026-10-17");
    let web = s.entries().get(EntryId::new(1)).unwrap();
    assert_eq!(web.duration_until(t("2026-10-17T12:00")), 4920);
}

#[test]
fn day_bounds_follow_dst() {
    let tz = berlin();
    let len = |d: &str| {
        let (from, to) = day_bounds(date(d), &tz).unwrap();
        to.seconds_since(from)
    };
    assert_eq!(len("2026-10-17"), 24 * 3600);
    assert_eq!(len("2026-03-29"), 23 * 3600);
    assert_eq!(len("2026-10-25"), 25 * 3600);
    assert_eq!(
        local_date(t("2026-10-17T23:59:59"), &tz),
        date("2026-10-17")
    );
}

#[test]
fn day_entries_are_listed_by_start_with_names_and_notes() {
    let s = run(
        worked_day(),
        &ctx("2026-10-17T13:00"),
        [
            Command::AddEntry {
                project: WEB,
                task: TaskChoice::Existing(TaskId::new(1)),
                start: "2026-10-17T08:00".parse().unwrap(),
                end: "2026-10-17T08:30".parse().unwrap(),
                note: Some("standup".into()),
            },
            Command::Start {
                project: API,
                task: TaskChoice::NoTask,
            },
        ],
    );
    let tz = berlin();
    let (from, to) = day_bounds(date("2026-10-17"), &tz).unwrap();
    let entries = s.entries().started_between(from, to);
    let rows = read::day_entries(
        s.catalog(),
        &entries,
        t("2026-10-17T13:10"),
        &tz,
        date("2026-10-17"),
    );
    let summary: Vec<_> = rows
        .iter()
        .map(|r| (r.combo.label.as_str(), r.duration, r.running))
        .collect();
    assert_eq!(
        summary,
        [
            ("Acme / Web / Review", 30 * MIN, false),
            ("Acme / Web", 82 * MIN, false),
            ("Beta / API", 45 * MIN, false),
            ("Beta / Ops", 13 * MIN, false),
            ("Beta / API", 10 * MIN, true),
        ]
    );
    assert_eq!(rows[0].note.as_ref().map(|n| n.as_str()), Some("standup"));
}

#[test]
fn recents_are_distinct_newest_first_and_capped() {
    let s = worked_day();
    let s = start_at(s, "2026-10-17T12:00", WEB);
    let rows = read::recents(s.catalog(), s.entries(), 8);
    let labels: Vec<_> = rows.iter().map(|r| r.label.as_str()).collect();
    assert_eq!(labels, ["Acme / Web", "Beta / Ops", "Beta / API"]);
    assert_eq!(read::recents(s.catalog(), s.entries(), 2).len(), 2);
}

#[test]
fn quick_switch_matches_fuzzily_over_all_active_combos() {
    let s = worked_day();
    let recents = s.entries().recent_combos(8);
    let labels = |q: &str| -> Vec<String> {
        read::quick_switch(s.catalog(), &recents, q)
            .into_iter()
            .map(|r| r.label)
            .collect()
    };
    assert_eq!(
        labels("acweb").first().map(String::as_str),
        Some("Acme / Web")
    );
    assert_eq!(labels("review"), ["Acme / Web / Review"]);
    assert_eq!(labels("beta ops"), ["Beta / Ops"]);
    assert!(labels("zzz").is_empty());
    // A blank query lists the recents, then the other active combos by label.
    assert_eq!(
        labels("  "),
        [
            "Beta / Ops",
            "Beta / API",
            "Acme / Web",
            "Acme / Web / Review"
        ]
    );
}

#[test]
fn quick_switch_lists_untracked_combos_for_a_blank_query() {
    // A fresh catalog has no recents; its combos are still offered, by label.
    let s = worked_day();
    let rows = read::quick_switch(s.catalog(), &[], "");
    let labels: Vec<_> = rows.iter().map(|r| r.label.as_str()).collect();
    assert_eq!(
        labels,
        [
            "Acme / Web",
            "Acme / Web / Review",
            "Beta / API",
            "Beta / Ops"
        ]
    );
}

#[test]
fn quick_switch_lists_only_recents_n_recents_for_a_blank_query() {
    // R7: the recents are the last N distinct combos; older ones move into the A–Z part.
    let s = worked_day();
    let settings = tt_core::Settings {
        recents_n: 1,
        ..s.catalog().settings().clone()
    };
    let s = run(
        s,
        &ctx("2026-10-17T12:00"),
        [Command::UpdateSettings { settings }],
    );
    let recents = s.entries().recent_combos(100);
    let rows = read::quick_switch(s.catalog(), &recents, "");
    let labels: Vec<_> = rows.iter().map(|r| r.label.as_str()).collect();
    assert_eq!(
        labels,
        [
            "Beta / Ops",
            "Acme / Web",
            "Acme / Web / Review",
            "Beta / API"
        ]
    );
}

#[test]
fn quick_switch_caps_a_blank_query() {
    let s = run(
        worked_day(),
        &ctx("2026-10-17T12:00"),
        (0..60).map(|i| Command::CreateTask {
            project: WEB,
            name: format!("Task {i:02}"),
            abbreviation: None,
        }),
    );
    let recents = s.entries().recent_combos(8);
    let rows = read::quick_switch(s.catalog(), &recents, "");
    assert_eq!(rows.len(), read::QUICK_SWITCH_LIMIT);
    assert_eq!(rows[0].label, "Beta / Ops");
}

#[test]
fn quick_switch_prefers_recent_combos_on_equal_scores() {
    let s = worked_day();
    // "Beta" matches both Beta combos equally; Ops was used last.
    let recents = s.entries().recent_combos(8);
    let rows = read::quick_switch(s.catalog(), &recents, "beta");
    let labels: Vec<_> = rows.iter().map(|r| r.label.as_str()).collect();
    assert_eq!(labels, ["Beta / Ops", "Beta / API"]);
}

#[test]
fn archived_items_are_not_offered() {
    // An archived client (no command yet) or task hides its combos, like an archived project.
    let now = t("2026-10-17T08:00");
    let name = |s: &str| Name::parse(s).unwrap();
    let catalog = Catalog::from_parts(
        [
            tt_core::Client {
                id: ACME,
                name: name("Acme"),
                abbreviation: None,
                rule: RoundingRule::default(),
                archived_at: None,
                created_at: now,
            },
            tt_core::Client {
                id: BETA,
                name: name("Beta"),
                abbreviation: None,
                rule: RoundingRule::default(),
                archived_at: Some(now),
                created_at: now,
            },
        ],
        [
            tt_core::Project {
                id: WEB,
                client: ACME,
                name: name("Web"),
                abbreviation: None,
                archived_at: None,
                created_at: now,
            },
            tt_core::Project {
                id: API,
                client: BETA,
                name: name("API"),
                abbreviation: None,
                archived_at: None,
                created_at: now,
            },
        ],
        [tt_core::Task {
            id: TaskId::new(1),
            project: WEB,
            name: name("Review"),
            abbreviation: None,
            archived_at: Some(now),
            created_at: now,
        }],
        Settings::default(),
        Counters {
            next_client: 3,
            next_project: 3,
            next_task: 2,
            next_entry: 1,
        },
    );
    State::from_parts(catalog.clone(), Entries::default())
        .check_invariants()
        .unwrap();
    let all: Vec<_> = read::quick_switch(&catalog, &[], "e")
        .into_iter()
        .map(|r| r.label)
        .collect();
    assert_eq!(all, ["Acme / Web"]);
    let blank: Vec<_> = read::quick_switch(&catalog, &[], "")
        .into_iter()
        .map(|r| r.label)
        .collect();
    assert_eq!(blank, ["Acme / Web"]);
    let on_api = Entries::from_parts([Entry {
        id: EntryId::new(1),
        project: API,
        task: None,
        start: now,
        end: Some(t("2026-10-17T09:00")),
        note: None,
        created_at: now,
        updated_at: now,
    }]);
    assert!(read::recents(&catalog, &on_api, 8).is_empty());
    assert!(read::task_matches(&catalog, WEB, "rev").tasks.is_empty());
    // An archived name still counts, so it isn't offered for creation (OQ-7).
    assert_eq!(
        read::task_matches(&catalog, WEB, "Review").offer_create,
        None
    );
}

#[test]
fn task_matches_offer_create_unless_the_name_exists() {
    let s = catalog();
    let m = read::task_matches(s.catalog(), WEB, "rev");
    let names: Vec<_> = m.tasks.iter().map(|t| t.name.as_str()).collect();
    assert_eq!(names, ["Review"]);
    assert_eq!(m.offer_create, Some(Name::parse("rev").unwrap()));

    assert_eq!(
        read::task_matches(s.catalog(), WEB, " REVIEW ").offer_create,
        None
    );
    let blank = read::task_matches(s.catalog(), WEB, "");
    assert_eq!((blank.tasks.len(), blank.offer_create), (1, None));
    // Tasks are per project.
    assert!(read::task_matches(s.catalog(), API, "rev").tasks.is_empty());
}

#[test]
fn reads_ignore_entries_of_other_days() {
    let s = worked_day();
    let all: Vec<&Entry> = s.entries().started_between(
        Instant::from(jiff::Timestamp::MIN),
        Instant::from(jiff::Timestamp::MAX),
    );
    let tz = berlin();
    let o = read::daily_overview(
        s.catalog(),
        &all,
        t("2026-10-18T12:00"),
        &tz,
        date("2026-10-18"),
    );
    assert!(o.clients.is_empty());
}

// Abbreviations (R24): short labels and matching.

/// NWI (client 3) with the long project ASCP (4), its task Review (2, "Rev") and a decoy
/// project (5) whose name contains the letters of "ascp" scattered.
fn with_long_names() -> State {
    run(
        catalog(),
        &ctx("2026-10-17T08:00"),
        [
            Command::CreateClient {
                name: "Northwind Instruments".into(),
                abbreviation: Some("NWI".into()),
                rule: RoundingRule::default(),
            },
            Command::CreateProject {
                client: ClientId::new(3),
                name: "adaptive signal calibration pipeline".into(),
                abbreviation: Some("ASCP".into()),
            },
            Command::CreateTask {
                project: ProjectId::new(4),
                name: "Review".into(),
                abbreviation: Some("Rev".into()),
            },
            Command::CreateProject {
                client: ClientId::new(3),
                name: "Assembly of sensor components and parts".into(),
                abbreviation: None,
            },
        ],
    )
}

#[test]
fn combo_rows_carry_full_and_short_labels() {
    let s = with_long_names();
    let row = |project: u64, task: Option<u64>| {
        read::combo_row(
            s.catalog(),
            Combo {
                project: ProjectId::new(project),
                task: task.map(TaskId::new),
            },
        )
        .unwrap()
    };
    let ascp = row(4, Some(2));
    assert_eq!(
        ascp.label,
        "Northwind Instruments / adaptive signal calibration pipeline / Review"
    );
    assert_eq!(ascp.short_label, "NWI / ASCP / Rev");
    assert_eq!(
        row(5, None).short_label,
        "NWI / Assembly of sensor components and parts"
    );
}

#[test]
fn quick_switch_matches_abbreviations_and_mixed_queries() {
    let s = with_long_names();
    let labels = |q: &str| -> Vec<String> {
        read::quick_switch(s.catalog(), &[], q)
            .into_iter()
            .map(|r| r.short_label)
            .collect()
    };
    // An exact abbreviation ranks first, ahead of a fuzzy decoy (AC-7).
    let ascp = labels("ascp");
    assert!(
        ascp.iter().any(|l| l.contains("Assembly")),
        "the decoy matches too: {ascp:?}"
    );
    assert!(ascp[0].starts_with("NWI / ASCP"), "{ascp:?}");
    // The task's own combo ranks first; the project alone may match fuzzily too.
    assert_eq!(
        labels("ascp rev").first().map(String::as_str),
        Some("NWI / ASCP / Rev")
    );
    // Names and abbreviations mix in one query.
    assert_eq!(labels("nwi calibration review"), ["NWI / ASCP / Rev"]);
    assert_eq!(
        labels("Northwind ascp").first().map(String::as_str),
        Some("NWI / ASCP")
    );
}

#[test]
fn task_matches_cover_abbreviations_and_offer_no_create_for_one() {
    let s = with_long_names();
    let m = read::task_matches(s.catalog(), ProjectId::new(4), "REV");
    assert_eq!(m.tasks.len(), 1);
    assert_eq!(
        m.tasks[0].abbreviation.as_ref().map(|a| a.as_str()),
        Some("Rev")
    );
    assert_eq!(m.offer_create, None, "REV is the task's abbreviation");
    let m = read::task_matches(s.catalog(), ProjectId::new(4), "re");
    assert_eq!(m.offer_create, Some(Name::parse("re").unwrap()));
}

#[test]
fn an_exact_abbreviation_beats_a_name_containing_the_same_word() {
    // A sibling named "ASCP rollout" scores as well as the abbreviation ASCP; the exact
    // abbreviation still ranks first (R24), even when the decoy is the most recent combo.
    let s = run(
        with_long_names(),
        &ctx("2026-10-17T08:00"),
        [Command::CreateProject {
            client: ClientId::new(3),
            name: "ASCP rollout".into(),
            abbreviation: None,
        }],
    );
    let decoy = Combo {
        project: ProjectId::new(6),
        task: None,
    };
    for recents in [vec![], vec![decoy]] {
        let first = read::quick_switch(s.catalog(), &recents, "ascp")
            .into_iter()
            .next()
            .map(|r| r.short_label);
        assert_eq!(first.as_deref(), Some("NWI / ASCP"), "recents {recents:?}");
    }
}

#[test]
fn task_matches_rank_an_exact_abbreviation_first() {
    let s = run(
        with_long_names(),
        &ctx("2026-10-17T08:00"),
        [
            Command::CreateTask {
                project: ProjectId::new(4),
                name: "Bug triage".into(),
                abbreviation: Some("BT".into()),
            },
            Command::CreateTask {
                project: ProjectId::new(4),
                name: "BTW notes".into(),
                abbreviation: None,
            },
        ],
    );
    let names: Vec<String> = read::task_matches(s.catalog(), ProjectId::new(4), "bt")
        .tasks
        .into_iter()
        .map(|t| t.name.to_string())
        .collect();
    assert_eq!(names, ["Bug triage", "BTW notes"]);
}

// R2c, R7: changes/2026-11-05-archive-projects.

/// Acme (1): Web (1, "W", active), Old web (2, "OW", archived), Legacy (3, "LG", archived).
/// Beta (2): Portal (4, archived).
fn archive_catalog() -> State {
    let project =
        |client: ClientId, name: &str, abbreviation: Option<&str>| Command::CreateProject {
            client,
            name: name.into(),
            abbreviation: abbreviation.map(Into::into),
        };
    run(
        State::default(),
        &ctx("2026-10-17T08:00"),
        [
            Command::CreateClient {
                name: "Acme".into(),
                abbreviation: None,
                rule: RoundingRule::default(),
            },
            Command::CreateClient {
                name: "Beta".into(),
                abbreviation: None,
                rule: RoundingRule::default(),
            },
            project(ACME, "Web", Some("W")),
            project(ACME, "Old web", Some("OW")),
            project(ACME, "Legacy", Some("LG")),
            project(BETA, "Portal", None),
            Command::ArchiveProject {
                id: ProjectId::new(2),
            },
            Command::ArchiveProject {
                id: ProjectId::new(3),
            },
            Command::ArchiveProject {
                id: ProjectId::new(4),
            },
        ],
    )
}

#[test]
fn name_check_is_free_for_a_new_name() {
    let s = archive_catalog();
    let check = |name: &str, abbreviation: Option<&str>| {
        read::project_name_check(s.catalog(), ACME, name, abbreviation)
    };
    assert_eq!(check("Mobile", None), NameCheck::Free);
    assert_eq!(check("Mobile", Some("MOB")), NameCheck::Free);
    assert_eq!(check("Mobile", Some("  ")), NameCheck::Free);
    assert_eq!(check("", None), NameCheck::Free);
    // Another client's archived project doesn't count.
    assert_eq!(check("Portal", None), NameCheck::Free);
}

#[test]
fn name_check_offers_to_unarchive_on_a_direct_hit() {
    let s = archive_catalog();
    let check = |name: &str, abbreviation: Option<&str>| {
        read::project_name_check(s.catalog(), ACME, name, abbreviation)
    };
    let old_web = NameCheck::Unarchive {
        project: ProjectId::new(2),
    };
    // Name to name, ignoring case and surrounding whitespace.
    assert_eq!(check("old web", None), old_web);
    assert_eq!(check("  OLD WEB ", Some("")), old_web);
    // Abbreviation to abbreviation, with a new name.
    assert_eq!(check("Webshop", Some("ow")), old_web);
    // Both point at the same project, also with its name as the abbreviation.
    assert_eq!(check("Old web", Some("OW")), old_web);
    assert_eq!(check("Old web", Some("old web")), old_web);
    // Another client: Beta's archived Portal.
    assert_eq!(
        read::project_name_check(s.catalog(), BETA, "portal", None),
        NameCheck::Unarchive {
            project: ProjectId::new(4)
        }
    );
}

#[test]
fn name_check_blocks_active_cross_and_multiple_hits() {
    let s = archive_catalog();
    let check = |name: &str, abbreviation: Option<&str>| {
        read::project_name_check(s.catalog(), ACME, name, abbreviation)
    };
    let taken = |ids: &[u64]| NameCheck::Taken {
        by: ids.iter().map(|id| ProjectId::new(*id)).collect(),
    };
    // An active project, by name or abbreviation, in either field.
    assert_eq!(check("web", None), taken(&[1]));
    assert_eq!(check("Mobile", Some("w")), taken(&[1]));
    assert_eq!(check("W", None), taken(&[1]));
    // A cross match with an archived project: its abbreviation typed as the name, or its
    // name typed as the abbreviation.
    assert_eq!(check("ow", None), taken(&[2]));
    assert_eq!(check("Mobile", Some("Legacy")), taken(&[3]));
    // Two archived projects, or an archived and an active one.
    assert_eq!(check("Old web", Some("LG")), taken(&[2, 3]));
    assert_eq!(check("Old web", Some("W")), taken(&[1, 2]));
}

#[test]
fn archived_combos_do_not_use_up_the_recents() {
    // R7: two older active combos, then three newer ones on a project that gets archived.
    let c = ctx("2026-10-17T18:00");
    let add = |project: ProjectId, start: &str, end: &str| Command::AddEntry {
        project,
        task: TaskChoice::NoTask,
        start: start.parse().unwrap(),
        end: end.parse().unwrap(),
        note: None,
    };
    let s = run(
        catalog(),
        &c,
        [
            add(OPS, "2026-10-17T08:00", "2026-10-17T09:00"),
            add(API, "2026-10-17T09:00", "2026-10-17T10:00"),
            Command::AddEntry {
                project: WEB,
                task: TaskChoice::Existing(TaskId::new(1)),
                start: "2026-10-17T10:00".parse().unwrap(),
                end: "2026-10-17T11:00".parse().unwrap(),
                note: None,
            },
            add(WEB, "2026-10-17T11:00", "2026-10-17T12:00"),
            Command::ArchiveProject { id: WEB },
        ],
    );
    let labels = |n| -> Vec<String> {
        read::recents(s.catalog(), s.entries(), n)
            .into_iter()
            .map(|r| r.label)
            .collect()
    };
    assert_eq!(labels(2), ["Beta / API", "Beta / Ops"]);
    assert_eq!(
        read::recent_combos(s.catalog(), s.entries(), 8),
        [
            Combo {
                project: API,
                task: None
            },
            Combo {
                project: OPS,
                task: None
            },
        ]
    );
    // The quick-switch's blank list and its matches skip the archived project too.
    let recents = read::recent_combos(s.catalog(), s.entries(), read::QUICK_SWITCH_RECENCY);
    let blank: Vec<_> = read::quick_switch(s.catalog(), &recents, "")
        .into_iter()
        .map(|r| r.label)
        .collect();
    assert_eq!(blank, ["Beta / API", "Beta / Ops"]);
    assert!(read::quick_switch(s.catalog(), &recents, "web").is_empty());
}

#[test]
fn the_task_input_offers_nothing_for_an_archived_project() {
    // R2a, R2b: Create "<text>" would be rejected, so it isn't offered (stage 5 review).
    let s = run(
        catalog(),
        &ctx("2026-10-17T09:00"),
        [Command::ArchiveProject { id: WEB }],
    );
    let matches = read::task_matches(s.catalog(), WEB, "Docs");
    assert!(matches.tasks.is_empty());
    assert_eq!(matches.offer_create, None);
    // An active project still gets the offer.
    assert!(
        read::task_matches(s.catalog(), API, "Docs")
            .offer_create
            .is_some()
    );
}

// R2e, R2f: changes/2026-11-05-archive-view.

fn add_entry(project: ProjectId, start: &str, end: &str) -> Command {
    Command::AddEntry {
        project,
        task: TaskChoice::NoTask,
        start: start.parse().unwrap(),
        end: end.parse().unwrap(),
        note: None,
    }
}

#[test]
fn start_span_follows_adds_edits_moves_and_deletes() {
    let now = ctx("2026-10-20T18:00");
    let s = run(
        catalog(),
        &now,
        [
            add_entry(WEB, "2026-10-15T09:00", "2026-10-15T10:00"),
            add_entry(WEB, "2026-10-12T09:00", "2026-10-12T10:00"),
            add_entry(API, "2026-10-19T09:00", "2026-10-19T10:00"),
        ],
    );
    let span = |s: &State, p| s.entries().start_span(p);
    assert_eq!(
        span(&s, WEB),
        Some((t("2026-10-12T09:00"), t("2026-10-15T09:00")))
    );
    // Another project's entries don't count, and a project without entries has no span.
    assert_eq!(
        span(&s, API),
        Some((t("2026-10-19T09:00"), t("2026-10-19T09:00")))
    );
    assert_eq!(span(&s, OPS), None);

    // Editing the start moves the span.
    let edit = |id, project, start: &str, end: &str| Command::EditEntry {
        id: EntryId::new(id),
        project,
        task: TaskChoice::NoTask,
        start: start.parse().unwrap(),
        end: Some(end.parse().unwrap()),
        note: None,
    };
    let s = run(
        s,
        &now,
        [edit(2, WEB, "2026-10-13T09:00", "2026-10-13T10:00")],
    );
    assert_eq!(
        span(&s, WEB),
        Some((t("2026-10-13T09:00"), t("2026-10-15T09:00")))
    );

    // Moving the last Web entry to Ops takes it out of Web's span and into Ops'.
    let s = run(
        s,
        &now,
        [edit(1, OPS, "2026-10-15T09:00", "2026-10-15T10:00")],
    );
    assert_eq!(
        span(&s, WEB),
        Some((t("2026-10-13T09:00"), t("2026-10-13T09:00")))
    );
    assert_eq!(
        span(&s, OPS),
        Some((t("2026-10-15T09:00"), t("2026-10-15T09:00")))
    );

    // Deleting a project's only entry leaves it without a span.
    let s = run(
        s,
        &now,
        [Command::DeleteEntry {
            id: EntryId::new(2),
        }],
    );
    assert_eq!(span(&s, WEB), None);
}

#[test]
fn start_span_counts_the_running_entry() {
    let s = start_at(catalog(), "2026-10-17T09:00", WEB);
    let s = start_at(s, "2026-10-17T10:00", API);
    assert_eq!(
        s.entries().start_span(API),
        Some((t("2026-10-17T10:00"), t("2026-10-17T10:00")))
    );
}

/// Clients "Zeta" (1), "acme" (2), "Gamma" (3, nothing archived). Projects, in ID order:
/// acme: Old (1), Mid (2), Twin (3), Same start long (4), Unused b (5), unused A (6),
/// Active (7, not archived); Zeta: Portal (8); Gamma: Kept (9, not archived).
fn archive_view_state() -> State {
    let client = |name: &str| Command::CreateClient {
        name: name.into(),
        abbreviation: None,
        rule: RoundingRule::default(),
    };
    let project = |client: u64, name: &str| Command::CreateProject {
        client: ClientId::new(client),
        name: name.into(),
        abbreviation: None,
    };
    let p = ProjectId::new;
    let mut cmds = vec![client("Zeta"), client("acme"), client("Gamma")];
    for name in [
        "Old",
        "Mid",
        "Twin",
        "Same start long",
        "Unused b",
        "unused A",
        "Active",
    ] {
        cmds.push(project(2, name));
    }
    cmds.push(project(1, "Portal"));
    cmds.push(project(3, "Kept"));
    cmds.extend([
        add_entry(p(1), "2026-03-02T09:00", "2026-03-02T10:00"),
        add_entry(p(1), "2026-05-10T09:00", "2026-05-10T10:00"),
        add_entry(p(2), "2026-06-01T09:00", "2026-06-01T10:00"),
        add_entry(p(2), "2026-06-20T09:00", "2026-06-20T10:00"),
        add_entry(p(3), "2026-06-01T11:00", "2026-06-01T12:00"),
        add_entry(p(3), "2026-06-20T11:00", "2026-06-20T12:00"),
        add_entry(p(4), "2026-06-01T13:00", "2026-06-01T14:00"),
        add_entry(p(4), "2026-07-01T13:00", "2026-07-01T14:00"),
        add_entry(p(7), "2026-08-01T09:00", "2026-08-01T10:00"),
        add_entry(p(8), "2026-09-01T09:00", "2026-09-01T10:00"),
        add_entry(p(9), "2026-09-02T09:00", "2026-09-02T10:00"),
    ]);
    cmds.extend([1, 2, 3, 4, 5, 6, 8].map(|id| Command::ArchiveProject { id: p(id) }));
    run(State::default(), &ctx("2026-10-20T18:00"), cmds)
}

fn period(first: &str, last: &str) -> Option<read::ActivePeriod> {
    Some(read::ActivePeriod {
        first: date(first),
        last: date(last),
    })
}

#[test]
fn archive_view_groups_and_orders_archived_projects() {
    let s = archive_view_state();
    let view = read::archived_projects(s.catalog(), s.entries(), &berlin());
    let rows = |c: &read::ArchivedClient| -> Vec<(u64, Option<read::ActivePeriod>)> {
        c.projects
            .iter()
            .map(|p| (p.project.get(), p.period))
            .collect()
    };
    // Clients by name, ignoring case; Gamma has nothing archived and is left out.
    let clients: Vec<_> = view.iter().map(|c| c.client.get()).collect();
    assert_eq!(clients, [2, 1]);
    // No entries first (by name, ignoring case), then the first day newest first, then the
    // last day newest first, then the name. Active is left out.
    assert_eq!(
        rows(&view[0]),
        [
            (6, None),
            (5, None),
            (4, period("2026-06-01", "2026-07-01")),
            (2, period("2026-06-01", "2026-06-20")),
            (3, period("2026-06-01", "2026-06-20")),
            (1, period("2026-03-02", "2026-05-10")),
        ]
    );
    assert_eq!(rows(&view[1]), [(8, period("2026-09-01", "2026-09-01"))]);
}

#[test]
fn archive_view_is_empty_without_archived_projects() {
    let s = catalog();
    assert!(read::archived_projects(s.catalog(), s.entries(), &berlin()).is_empty());
}

#[test]
fn archive_view_days_are_local_start_dates() {
    // 00:30 in Berlin is the previous day in UTC; an entry past midnight counts on its
    // start day (R18 step 1).
    let s = run(
        catalog(),
        &ctx("2026-10-20T18:00"),
        [
            add_entry(WEB, "2026-10-10T00:30", "2026-10-10T01:00"),
            add_entry(WEB, "2026-10-17T23:30", "2026-10-18T00:30"),
            Command::ArchiveProject { id: WEB },
        ],
    );
    let view = read::archived_projects(s.catalog(), s.entries(), &berlin());
    assert_eq!(view.len(), 1);
    assert_eq!(view[0].client, ACME);
    assert_eq!(
        view[0].projects[0].period,
        period("2026-10-10", "2026-10-17")
    );
}
