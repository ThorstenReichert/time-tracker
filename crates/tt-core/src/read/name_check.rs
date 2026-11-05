//! The New project dialog's check while typing (R2c): is the typed name and abbreviation
//! free, does it point at one archived project to unarchive, or is it taken? It is only
//! guidance; `CreateProject` still enforces uniqueness on save (R1, R24, OQ-7).

use serde::{Deserialize, Serialize};

use crate::{Abbreviation, Catalog, ClientId, Name, ProjectId};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NameCheck {
    /// No project of the client is called by the typed name or abbreviation.
    Free,
    /// Exactly one project is hit, it is archived, and it was hit name to name or
    /// abbreviation to abbreviation: offer to unarchive it instead of creating a duplicate.
    Unarchive { project: ProjectId },
    /// Creating is blocked: an active project is hit, two or more projects are, or an archived
    /// one only by a cross match (a name against an abbreviation). Ordered by ID.
    Taken { by: Vec<ProjectId> },
}

/// Checks a typed `name` and `abbreviation` against the projects of `client`, ignoring case
/// and surrounding whitespace like the uniqueness rules. Invalid or blank text hits nothing.
pub fn project_name_check(
    catalog: &Catalog,
    client: ClientId,
    name: &str,
    abbreviation: Option<&str>,
) -> NameCheck {
    let name = Name::parse(name).ok();
    let abbreviation = Abbreviation::parse_optional(abbreviation).ok().flatten();
    let name_key = name.as_ref().map(Name::key);
    let abbreviation_key = abbreviation.as_ref().map(Abbreviation::key);

    let hits: Vec<_> = catalog
        .projects_of(client)
        .filter(|p| {
            [name_key, abbreviation_key]
                .into_iter()
                .flatten()
                .any(|k| p.is_called(k))
        })
        .collect();
    match hits.as_slice() {
        [] => NameCheck::Free,
        [archived]
            if archived.archived_at.is_some()
                && (name_key == Some(archived.name.key())
                    || abbreviation_key.is_some()
                        && abbreviation_key == archived.abbreviation.as_ref().map(|a| a.key())) =>
        {
            NameCheck::Unarchive {
                project: archived.id,
            }
        }
        _ => NameCheck::Taken {
            by: hits.iter().map(|p| p.id).collect(),
        },
    }
}
