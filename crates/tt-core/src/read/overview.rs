//! The daily billing overview (R14) with the rounding algorithm of R18. This and the billing
//! error are the only places rounding is applied (R16). Always computed from the current
//! entries and rules (R19).

use std::collections::BTreeMap;

use jiff::{civil, tz::TimeZone};
use serde::{Deserialize, Serialize};

use super::local_date;
use crate::rounding::round;
use crate::{
    Catalog, Client, ClientId, Entry, Instant, Name, ProjectId, RoundingLevel, RoundingRule,
};

/// Raw and rounded seconds of a value to book, and the difference (`rounded − raw`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Figure {
    pub raw: i64,
    pub rounded: i64,
    pub diff: i64,
}

impl Figure {
    fn of(raw: i64, rounded: i64) -> Self {
        Self {
            raw,
            rounded,
            diff: rounded - raw,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectRow {
    pub project: ProjectId,
    pub name: Name,
    pub raw: i64,
    /// The value to book: per project for project-level clients; `None` for client-level
    /// clients, whose project rows are raw and informational only.
    pub booked: Option<Figure>,
    pub running: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClientBlock {
    pub client: ClientId,
    pub name: Name,
    pub rule: RoundingRule,
    /// Client level: the one rounded client total (the value to book). Project level: the
    /// subtotal of the rounded project values, with the raw total.
    pub total: Figure,
    pub projects: Vec<ProjectRow>,
    pub running: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Overview {
    pub date: civil::Date,
    pub clients: Vec<ClientBlock>,
}

/// The overview for `date`. `entries` are the entries that start on that day (callers pass
/// `Entries::started_between(day_bounds(date))`); others are ignored. A running entry counts
/// up to `now` and is marked (R14).
pub fn daily_overview(
    catalog: &Catalog,
    entries: &[&Entry],
    now: Instant,
    tz: &TimeZone,
    date: civil::Date,
) -> Overview {
    // R18 steps 1–2: entries by local start date, summed in seconds per project.
    let mut per_project: BTreeMap<ProjectId, (i64, bool)> = BTreeMap::new();
    for entry in entries.iter().filter(|e| local_date(e.start, tz) == date) {
        let sum = per_project.entry(entry.project).or_default();
        sum.0 += entry.duration_until(now);
        sum.1 |= entry.is_running();
    }

    let mut per_client: BTreeMap<ClientId, Vec<(ProjectId, i64, bool)>> = BTreeMap::new();
    for (project, (raw, running)) in per_project {
        if let Some(p) = catalog.project(project) {
            per_client
                .entry(p.client)
                .or_default()
                .push((project, raw, running));
        }
    }

    let mut clients: Vec<ClientBlock> = per_client
        .into_iter()
        .filter_map(|(id, projects)| Some(client_block(catalog, catalog.client(id)?, projects)))
        .collect();
    clients.sort_by(|a, b| (a.name.key(), a.client).cmp(&(b.name.key(), b.client)));
    Overview { date, clients }
}

/// R18 step 3 for one client: round per project or once for the client, by its level.
fn client_block(
    catalog: &Catalog,
    client: &Client,
    projects: Vec<(ProjectId, i64, bool)>,
) -> ClientBlock {
    let rule = client.rule;
    let mut rows: Vec<ProjectRow> = projects
        .into_iter()
        .filter_map(|(id, raw, running)| {
            let name = catalog.project(id)?.name.clone();
            let booked = match rule.level {
                RoundingLevel::Project => Some(Figure::of(raw, round(raw, &rule))),
                RoundingLevel::Client => None,
            };
            Some(ProjectRow {
                project: id,
                name,
                raw,
                booked,
                running,
            })
        })
        .collect();
    rows.sort_by(|a, b| (a.name.key(), a.project).cmp(&(b.name.key(), b.project)));

    let raw: i64 = rows.iter().map(|r| r.raw).sum();
    let rounded = match rule.level {
        RoundingLevel::Project => rows
            .iter()
            .filter_map(|r| r.booked)
            .map(|f| f.rounded)
            .sum(),
        RoundingLevel::Client => round(raw, &rule),
    };
    ClientBlock {
        client: client.id,
        name: client.name.clone(),
        rule,
        total: Figure::of(raw, rounded),
        running: rows.iter().any(|r| r.running),
        projects: rows,
    }
}
