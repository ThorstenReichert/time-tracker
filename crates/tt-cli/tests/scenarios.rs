//! End-to-end scenarios through the tt-cli binary on a scratch database (plan.md step 7).
//! Every call is a new process, so every step also runs the startup: lock, backup, migrate,
//! load and check.

use std::path::{Path, PathBuf};
use std::process::Output;

use assert_cmd::Command;
use serde_json::Value;
use tempfile::TempDir;

struct Scenario {
    dir: TempDir,
}

impl Scenario {
    fn new() -> Self {
        Self {
            dir: TempDir::new().unwrap(),
        }
    }

    fn db(&self) -> PathBuf {
        self.dir.path().join("scratch.db")
    }

    /// Runs `tt-cli --db <scratch> --now <now> --tz Europe/Berlin <args>`.
    fn run(&self, now: &str, args: &[&str]) -> Output {
        cli(&self.db(), now, args)
    }

    /// Runs a call that must succeed and returns its JSON.
    fn ok(&self, now: &str, args: &[&str]) -> Value {
        let out = self.run(now, args);
        assert!(
            out.status.success(),
            "{args:?} failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        serde_json::from_slice(&out.stdout).unwrap()
    }
}

fn cli(db: &Path, now: &str, args: &[&str]) -> Output {
    Command::cargo_bin("tt-cli")
        .unwrap()
        .arg("--db")
        .arg(db)
        .args(["--now", now, "--tz", "Europe/Berlin"])
        .args(args)
        .output()
        .unwrap()
}

fn stderr_json(out: &Output) -> Value {
    serde_json::from_slice(&out.stderr).unwrap_or_else(|_| {
        panic!(
            "stderr is not JSON: {}",
            String::from_utf8_lossy(&out.stderr)
        )
    })
}

/// Acme (1): project level, 15 min nearest, project Web (1).
/// Beta (2): client level, 30 min up, projects API (2) and Ops (3).
fn catalog(s: &Scenario) {
    let now = "2026-10-17T08:00";
    s.ok(now, &["client", "add", "Acme"]);
    s.ok(now, &["project", "add", "--client", "1", "Web"]);
    s.ok(
        now,
        &[
            "client",
            "add",
            "Beta",
            "--unit",
            "30",
            "--direction",
            "up",
            "--level",
            "client",
        ],
    );
    s.ok(now, &["project", "add", "--client", "2", "API"]);
    s.ok(now, &["project", "add", "--client", "2", "Ops"]);
}

#[test]
fn the_worked_day_gives_the_planned_numbers() {
    let s = Scenario::new();
    catalog(&s);
    s.ok("2026-10-17T09:00", &["start", "--project", "1"]);
    let switch = s.ok("2026-10-17T10:22", &["start", "--project", "2"]);
    let kinds: Vec<&str> = switch["events"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["type"].as_str().unwrap())
        .collect();
    assert_eq!(kinds, ["EntryStopped", "EntryStarted"]);
    s.ok("2026-10-17T11:07", &["start", "--project", "3"]);
    s.ok("2026-10-17T11:20", &["stop"]);

    let overview = s.ok("2026-10-17T12:00", &["overview", "--date", "2026-10-17"]);
    let clients = overview["clients"].as_array().unwrap();
    let total = |i: usize| {
        let t = &clients[i]["total"];
        (
            t["raw"].as_i64().unwrap(),
            t["rounded"].as_i64().unwrap(),
            t["diff"].as_i64().unwrap(),
        )
    };
    assert_eq!(clients[0]["name"], "Acme");
    assert_eq!(total(0), (4920, 4500, -420));
    assert_eq!(clients[0]["projects"][0]["booked"]["rounded"], 4500);
    assert_eq!(clients[1]["name"], "Beta");
    assert_eq!(total(1), (3480, 3600, 120));
    for project in clients[1]["projects"].as_array().unwrap() {
        assert!(
            project["booked"].is_null(),
            "client level books the client total only"
        );
    }
}

#[test]
fn a_forgotten_stop_is_fixed_with_stop_at() {
    let s = Scenario::new();
    catalog(&s);
    s.ok(
        "2026-10-17T09:00",
        &["start", "--project", "1", "--new-task", "Review"],
    );
    s.ok("2026-10-17T18:00", &["stop", "--at", "2026-10-17T12:30"]);
    let entries = s.ok("2026-10-17T18:05", &["entries", "--date", "2026-10-17"]);
    let entry = &entries[0];
    assert_eq!(entry["combo"]["label"], "Acme / Web / Review");
    assert_eq!(entry["duration"], 3 * 3600 + 30 * 60);
    assert_eq!(entry["running"], false);
}

#[test]
fn an_overlap_is_rejected_with_exit_3_naming_the_entry() {
    let s = Scenario::new();
    catalog(&s);
    s.ok(
        "2026-10-17T18:00",
        &[
            "entry",
            "add",
            "--project",
            "1",
            "--start",
            "2026-10-17T09:00",
            "--end",
            "2026-10-17T10:00",
        ],
    );
    let out = s.run(
        "2026-10-17T18:00",
        &[
            "entry",
            "add",
            "--project",
            "2",
            "--start",
            "2026-10-17T09:30",
            "--end",
            "2026-10-17T11:00",
        ],
    );
    assert_eq!(out.status.code(), Some(3));
    let err = stderr_json(&out);
    assert_eq!(err["error"], "domain");
    assert_eq!(err["message"], "overlaps entry 1");
}

#[test]
fn typed_times_in_the_dst_gap_are_rejected() {
    let s = Scenario::new();
    catalog(&s);
    let out = s.run(
        "2026-03-30T12:00",
        &[
            "entry",
            "add",
            "--project",
            "1",
            "--start",
            "2026-03-29T02:30",
            "--end",
            "2026-03-29T04:00",
        ],
    );
    assert_eq!(out.status.code(), Some(3));
    assert!(
        stderr_json(&out)["message"]
            .as_str()
            .unwrap()
            .contains("doesn't exist")
    );
}

#[test]
fn rule_changes_recompute_past_days() {
    let s = Scenario::new();
    catalog(&s);
    s.ok(
        "2026-10-17T18:00",
        &[
            "entry",
            "add",
            "--project",
            "2",
            "--start",
            "2026-10-17T09:00",
            "--end",
            "2026-10-17T09:58",
        ],
    );
    let rounded = |s: &Scenario| {
        s.ok("2026-10-18T09:00", &["overview", "--date", "2026-10-17"])["clients"][0]["total"]
            ["rounded"]
            .as_i64()
            .unwrap()
    };
    assert_eq!(rounded(&s), 3600);
    s.ok(
        "2026-10-18T09:00",
        &["client", "edit", "2", "--unit", "15", "--direction", "down"],
    );
    assert_eq!(rounded(&s), 2700);
}

#[test]
fn reads_list_recents_and_quick_switch_matches() {
    let s = Scenario::new();
    catalog(&s);
    s.ok("2026-10-17T09:00", &["start", "--project", "1"]);
    s.ok("2026-10-17T10:00", &["start", "--project", "3"]);
    let recents = s.ok("2026-10-17T10:05", &["recents"]);
    let labels: Vec<&str> = recents
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["label"].as_str().unwrap())
        .collect();
    assert_eq!(labels, ["Beta / Ops", "Acme / Web"]);
    let matches = s.ok("2026-10-17T10:05", &["quick-switch", "acweb"]);
    assert_eq!(matches[0]["label"], "Acme / Web");
}

#[test]
fn every_call_backs_up_next_to_the_scratch_database() {
    let s = Scenario::new();
    catalog(&s);
    let backups: Vec<_> = std::fs::read_dir(s.dir.path().join("backups"))
        .unwrap()
        .collect();
    assert_eq!(
        backups.len(),
        1,
        "same --now second, so one file was replaced"
    );
}

#[test]
fn db_is_required() {
    let out = Command::cargo_bin("tt-cli")
        .unwrap()
        .args(["state"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
}

#[test]
fn the_live_database_is_refused_before_touching_it() {
    let home = TempDir::new().unwrap();
    let live = home
        .path()
        .join("ThorSoft.TimeTracker")
        .join("timetracker.db");
    let elsewhere = home.path().join("elsewhere");
    std::fs::create_dir(&elsewhere).unwrap();
    for spelled in [
        live.clone(),
        PathBuf::from(live.to_string_lossy().to_uppercase()),
        PathBuf::from(format!(r"\\?\{}", live.display())),
        PathBuf::from(r"..\ThorSoft.TimeTracker\.\timetracker.db"),
    ] {
        assert_refused(home.path(), &elsewhere, &spelled);
    }
    assert!(!live.exists(), "nothing was created");
    assert!(!live.parent().unwrap().exists());
}

#[test]
fn the_live_database_is_refused_through_a_junction() {
    // A junction needs no admin rights; the path text no longer matches, the folder does.
    let home = TempDir::new().unwrap();
    let folder = home.path().join("ThorSoft.TimeTracker");
    std::fs::create_dir(&folder).unwrap();
    let link = home.path().join("link");
    let made = std::process::Command::new("cmd")
        .args(["/C", "mklink", "/J"])
        .arg(&link)
        .arg(&folder)
        .output()
        .unwrap();
    assert!(made.status.success(), "{made:?}");
    let spelled = link.join("timetracker.db");

    // The live database doesn't exist yet: the folders are compared.
    assert_refused(home.path(), home.path(), &spelled);
    assert!(
        !folder.join("timetracker.db").exists(),
        "nothing was created"
    );

    // It exists: the files are compared, and it stays untouched.
    let live = folder.join("timetracker.db");
    std::fs::write(&live, b"").unwrap();
    assert_refused(home.path(), home.path(), &spelled);
    assert_eq!(std::fs::metadata(&live).unwrap().len(), 0);
    assert_eq!(
        std::fs::read_dir(&folder).unwrap().count(),
        1,
        "no -wal, no backups"
    );
}

fn assert_refused(local_app_data: &Path, cwd: &Path, db: &Path) {
    let out = Command::cargo_bin("tt-cli")
        .unwrap()
        .env("LOCALAPPDATA", local_app_data)
        .current_dir(cwd)
        .arg("--db")
        .arg(db)
        .args([
            "--now",
            "2026-10-17T09:00",
            "--tz",
            "Europe/Berlin",
            "state",
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1), "{db:?}");
    assert!(
        stderr_json(&out)["message"]
            .as_str()
            .unwrap()
            .contains("live database"),
        "{db:?}"
    );
}

#[test]
fn abbreviations_are_set_matched_and_removed() {
    let s = Scenario::new();
    catalog(&s);
    let now = "2026-10-17T09:00";
    s.ok(
        now,
        &[
            "project",
            "add",
            "--client",
            "1",
            "adaptive signal calibration pipeline",
            "--abbr",
            "ASCP",
        ],
    );
    s.ok(
        now,
        &["task", "add", "--project", "4", "Review", "--abbr", "Rev"],
    );
    s.ok(now, &["client", "edit", "1", "--abbr", "AC"]);
    let matches = s.ok(now, &["quick-switch", "ascp rev"]);
    assert_eq!(matches[0]["short_label"], "AC / ASCP / Rev");
    assert_eq!(
        matches[0]["label"],
        "Acme / adaptive signal calibration pipeline / Review"
    );

    // A name may not clash with a sibling's abbreviation (exit 3).
    let out = s.run(now, &["project", "edit", "1", "--name", "ascp"]);
    assert_eq!(out.status.code(), Some(3));
    let err = stderr_json(&out);
    assert!(err.to_string().contains("DuplicateName"), "{err}");

    // Renaming keeps the abbreviation; an empty --abbr removes it.
    s.ok(
        now,
        &["project", "edit", "4", "--name", "Adaptive calibration"],
    );
    let state = s.ok(now, &["state"]);
    assert_eq!(state["catalog"]["projects"]["4"]["abbreviation"], "ASCP");
    s.ok(now, &["project", "edit", "4", "--abbr", ""]);
    let state = s.ok(now, &["state"]);
    assert_eq!(
        state["catalog"]["projects"]["4"]["abbreviation"],
        Value::Null
    );
}

#[test]
fn an_archived_project_is_not_bookable_until_unarchived() {
    // R2a, R2b (changes/2026-11-05-archive-projects).
    let s = Scenario::new();
    catalog(&s);
    s.ok("2026-10-17T09:00", &["start", "--project", "1"]);
    s.ok("2026-10-17T10:00", &["start", "--project", "3"]);
    // Ops (3) runs, so it can't be archived; Web (1) can.
    let out = s.run("2026-10-17T10:05", &["project", "archive", "3"]);
    assert_eq!(out.status.code(), Some(3));
    assert_eq!(
        stderr_json(&out)["message"],
        "the timer is running on project 3; stop it first"
    );
    s.ok("2026-10-17T10:05", &["project", "archive", "1"]);

    let out = s.run("2026-10-17T10:10", &["start", "--project", "1"]);
    assert_eq!(out.status.code(), Some(3));
    assert_eq!(stderr_json(&out)["message"], "project 1 is archived");
    let recents = s.ok("2026-10-17T10:10", &["recents"]);
    assert_eq!(recents.as_array().unwrap().len(), 1);
    assert_eq!(recents[0]["label"], "Beta / Ops");
    let matches = s.ok("2026-10-17T10:10", &["quick-switch", "web"]);
    assert!(matches.as_array().unwrap().is_empty());

    s.ok("2026-10-17T10:15", &["project", "unarchive", "1"]);
    s.ok("2026-10-17T10:20", &["start", "--project", "1"]);
}

#[test]
fn the_archive_lists_archived_projects_by_period() {
    // R2e (changes/2026-11-05-archive-view).
    let s = Scenario::new();
    catalog(&s);
    let now = "2026-10-20T18:00";
    let add = |project: &str, start: &str, end: &str| {
        s.ok(
            now,
            &[
                "entry",
                "add",
                "--project",
                project,
                "--start",
                start,
                "--end",
                end,
            ],
        );
    };
    add("2", "2026-10-12T09:00", "2026-10-12T10:00");
    add("3", "2026-10-15T09:00", "2026-10-15T10:00");
    add("3", "2026-10-16T23:30", "2026-10-17T00:30");
    s.ok(now, &["project", "archive", "2"]);
    s.ok(now, &["project", "archive", "3"]);

    // Beta's Ops started later than API, so it comes first; Acme has nothing archived.
    let archive = s.ok(now, &["archive"]);
    let clients = archive.as_array().unwrap();
    assert_eq!(clients.len(), 1);
    assert_eq!(clients[0]["client"], 2);
    let projects = &clients[0]["projects"];
    assert_eq!(projects[0]["project"], 3);
    assert_eq!(projects[0]["period"]["first"], "2026-10-15");
    assert_eq!(projects[0]["period"]["last"], "2026-10-16");
    assert_eq!(projects[1]["project"], 2);
    assert_eq!(projects[1]["period"]["first"], "2026-10-12");

    s.ok(now, &["project", "unarchive", "3"]);
    let archive = s.ok(now, &["archive"]);
    let projects = archive[0]["projects"].as_array().unwrap();
    assert_eq!(projects.len(), 1);
    assert_eq!(projects[0]["project"], 2);
}
