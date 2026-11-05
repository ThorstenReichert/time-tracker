//! The Archive view (R2e, R2j): archived clients and archived projects grouped by client, each
//! project with its active period.

use std::cmp::Reverse;

use jiff::{civil, tz::TimeZone};
use serde::{Deserialize, Serialize};

use crate::read::local_date;
use crate::{Catalog, ClientId, Entries, Project, ProjectId};

/// A client of the Archive view with its projects, in the R2e order. An archived client lists
/// all its projects; an active one only its archived projects (R2j).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArchivedClient {
    pub client: ClientId,
    /// The client itself is archived.
    pub archived: bool,
    pub projects: Vec<ArchivedProject>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArchivedProject {
    pub project: ProjectId,
    /// The project itself is archived. `false` only under an archived client.
    pub archived: bool,
    /// `None` if the project has no entries.
    pub period: Option<ActivePeriod>,
}

/// The local dates of the start of a project's first and last entry. Entries count on the
/// date they start (R18 step 1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActivePeriod {
    pub first: civil::Date,
    pub last: civil::Date,
}

/// The archive by client (R2e, R2j). Clients by name, then ID, and only those that are
/// archived or have an archived project. An archived client brings all its projects, an active
/// one its archived projects. Within a client: projects without entries first, then the first day
/// newest first, then the last day newest first, then the name, then the ID. Names compare
/// case-insensitively. Reaches the entries only through `Entries::start_span` (OQ-9).
pub fn archived_projects(
    catalog: &Catalog,
    entries: &Entries,
    tz: &TimeZone,
) -> Vec<ArchivedClient> {
    let mut clients: Vec<_> = catalog
        .clients()
        .filter_map(|client| {
            let archived = client.archived_at.is_some();
            let mut projects: Vec<(&Project, Option<ActivePeriod>)> = catalog
                .projects_of(client.id)
                .filter(|p| archived || p.archived_at.is_some())
                .map(|p| (p, period(entries, tz, p.id)))
                .collect();
            if !archived && projects.is_empty() {
                return None;
            }
            // `false < true`: projects without entries come first.
            projects.sort_by_cached_key(|(p, period)| {
                (
                    period.is_some(),
                    period.map(|d| Reverse(d.first)),
                    period.map(|d| Reverse(d.last)),
                    p.name.key().to_owned(),
                    p.id,
                )
            });
            Some((
                client,
                ArchivedClient {
                    client: client.id,
                    archived,
                    projects: projects
                        .into_iter()
                        .map(|(p, period)| ArchivedProject {
                            project: p.id,
                            archived: p.archived_at.is_some(),
                            period,
                        })
                        .collect(),
                },
            ))
        })
        .collect();
    clients.sort_by(|(a, _), (b, _)| (a.name.key(), a.id).cmp(&(b.name.key(), b.id)));
    clients.into_iter().map(|(_, archived)| archived).collect()
}

fn period(entries: &Entries, tz: &TimeZone, project: ProjectId) -> Option<ActivePeriod> {
    entries
        .start_span(project)
        .map(|(first, last)| ActivePeriod {
            first: local_date(first, tz),
            last: local_date(last, tz),
        })
}
