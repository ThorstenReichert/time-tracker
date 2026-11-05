//! The live check of the New project (R2c) and New client (R2i) dialogs: is the typed name and
//! abbreviation free, does it point at one archived sibling to unarchive, or is it taken? It is
//! only guidance; `CreateProject` and `CreateClient` still enforce uniqueness on save (R1, R24,
//! OQ-7).

use serde::{Deserialize, Serialize};

use crate::{Abbreviation, Catalog, ClientId, Instant, Name, ProjectId};

/// The outcome of a name check. `Id` is the kind of item checked (`ProjectId`, `ClientId`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NameCheck<Id> {
    /// No sibling is called by the typed name or abbreviation.
    Free,
    /// Exactly one sibling is hit, it is archived, and it was hit name to name or
    /// abbreviation to abbreviation: offer to unarchive it instead of creating a duplicate.
    Unarchive { item: Id },
    /// Creating is blocked: an active sibling is hit, two or more siblings are, or an archived
    /// one only by a cross match (a name against an abbreviation). Ordered by ID.
    Taken { by: Vec<Id> },
}

/// Checks a typed `name` and `abbreviation` against the projects of `client` (R2c), ignoring
/// case and surrounding whitespace like the uniqueness rules. Invalid or blank text hits
/// nothing.
pub fn project_name_check(
    catalog: &Catalog,
    client: ClientId,
    name: &str,
    abbreviation: Option<&str>,
) -> NameCheck<ProjectId> {
    check(
        catalog.projects_of(client).map(|p| Sibling {
            id: p.id,
            name: &p.name,
            abbreviation: p.abbreviation.as_ref(),
            archived_at: p.archived_at,
        }),
        name,
        abbreviation,
    )
}

/// Checks a typed `name` and `abbreviation` against all clients (R2i), by the rule of R2c.
pub fn client_name_check(
    catalog: &Catalog,
    name: &str,
    abbreviation: Option<&str>,
) -> NameCheck<ClientId> {
    check(
        catalog.clients().map(|c| Sibling {
            id: c.id,
            name: &c.name,
            abbreviation: c.abbreviation.as_ref(),
            archived_at: c.archived_at,
        }),
        name,
        abbreviation,
    )
}

/// One item of the namespace being checked.
struct Sibling<'a, Id> {
    id: Id,
    name: &'a Name,
    abbreviation: Option<&'a Abbreviation>,
    archived_at: Option<Instant>,
}

impl<Id> Sibling<'_, Id> {
    fn is_called(&self, key: &str) -> bool {
        self.name.key() == key || self.abbreviation.is_some_and(|a| a.key() == key)
    }
}

fn check<'a, Id: Copy>(
    siblings: impl Iterator<Item = Sibling<'a, Id>>,
    name: &str,
    abbreviation: Option<&str>,
) -> NameCheck<Id> {
    let name = Name::parse(name).ok();
    let abbreviation = Abbreviation::parse_optional(abbreviation).ok().flatten();
    let name_key = name.as_ref().map(Name::key);
    let abbreviation_key = abbreviation.as_ref().map(Abbreviation::key);

    let hits: Vec<_> = siblings
        .filter(|s| {
            [name_key, abbreviation_key]
                .into_iter()
                .flatten()
                .any(|k| s.is_called(k))
        })
        .collect();
    match hits.as_slice() {
        [] => NameCheck::Free,
        [archived]
            if archived.archived_at.is_some()
                && (name_key == Some(archived.name.key())
                    || abbreviation_key.is_some()
                        && abbreviation_key == archived.abbreviation.map(Abbreviation::key)) =>
        {
            NameCheck::Unarchive { item: archived.id }
        }
        _ => NameCheck::Taken {
            by: hits.iter().map(|s| s.id).collect(),
        },
    }
}
