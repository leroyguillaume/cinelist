//! Assembles lists and selections from their rows, in a handful of batched queries.

use std::collections::HashMap;

use sqlx::PgConnection;
use uuid::Uuid;

use crate::{
    db::{
        self,
        films::FilmRow,
        lists::ListRow,
        participants::ParticipantRow,
        selections::{EliminationRow, SelectionRow, VerdictRow},
    },
    domain::{self, Outcome, Phase},
};

pub struct ListSnapshot {
    pub list: ListRow,
    /// In turn order.
    pub participants: Vec<ParticipantRow>,
    /// Oldest first.
    pub films: Vec<FilmRow>,
    pub active_selection: Option<SelectionSnapshot>,
    latest_starter_id: Option<Uuid>,
}

impl ListSnapshot {
    /// Participant who opens the next selection.
    pub fn next_starter(&self) -> Option<&ParticipantRow> {
        let previous = self.latest_starter_id.and_then(|id| {
            self.participants
                .iter()
                .position(|participant| participant.id == id)
        });
        let count = self.participants.len();
        (count > 0).then(|| &self.participants[domain::next_starter_index(previous, count)])
    }

    pub fn film_count(&self) -> i32 {
        i32::try_from(self.films.len()).unwrap_or(i32::MAX)
    }
}

pub struct SelectionSnapshot {
    pub selection: SelectionRow,
    /// Latest first.
    pub eliminations: Vec<EliminationRow>,
    /// Best place first.
    pub verdicts: Vec<VerdictRow>,
}

impl SelectionSnapshot {
    pub fn phase(&self) -> Phase {
        let outcomes: Vec<Outcome> = self
            .verdicts
            .iter()
            .filter_map(|verdict| Outcome::from_db(&verdict.outcome))
            .collect();
        domain::phase(
            self.selection.film_count,
            self.eliminations.len(),
            &outcomes,
        )
    }

    /// Participant whose turn it is, while films are being eliminated.
    pub fn current_participant<'a>(
        &self,
        participants: &'a [ParticipantRow],
    ) -> Option<&'a ParticipantRow> {
        if self.phase() != Phase::Eliminating || participants.is_empty() {
            return None;
        }
        let starter = self
            .selection
            .starter_id
            .and_then(|id| {
                participants
                    .iter()
                    .position(|participant| participant.id == id)
            })
            .unwrap_or(0);
        participants.get(domain::turn_index(
            starter,
            self.eliminations.len(),
            participants.len(),
        ))
    }

    /// Films of the list that are still in the running.
    pub fn remaining_films<'a>(&self, films: &'a [FilmRow]) -> Vec<&'a FilmRow> {
        films
            .iter()
            .filter(|film| {
                !self
                    .eliminations
                    .iter()
                    .any(|elimination| elimination.film_id == Some(film.id))
            })
            .collect()
    }

    /// Film waiting for a verdict: the survivor at place 1, then the runners-up
    /// in the order they were eliminated last.
    pub fn candidate<'a>(&self, films: &'a [FilmRow]) -> Option<Candidate<'a>> {
        let Phase::AwaitingVerdict { place } = self.phase() else {
            return None;
        };
        if place == 1 {
            let survivor = *self.remaining_films(films).first()?;
            return Some(Candidate {
                film_id: survivor.id,
                title: &survivor.title,
                place,
            });
        }
        let elimination = self
            .eliminations
            .iter()
            .find(|elimination| elimination.place == place)?;
        let film = films
            .iter()
            .find(|film| Some(film.id) == elimination.film_id)?;
        Some(Candidate {
            film_id: film.id,
            title: &film.title,
            place,
        })
    }
}

pub struct Candidate<'a> {
    pub film_id: Uuid,
    pub title: &'a str,
    pub place: i32,
}

/// Loads everything shown about the given lists, keeping their order.
pub async fn load_lists(
    conn: &mut PgConnection,
    lists: Vec<ListRow>,
) -> sqlx::Result<Vec<ListSnapshot>> {
    let ids: Vec<Uuid> = lists.iter().map(|list| list.id).collect();
    let mut participants = group_by(
        db::participants::for_lists(&mut *conn, &ids).await?,
        |row| row.list_id,
    );
    let mut films = group_by(db::films::for_lists(&mut *conn, &ids).await?, |row| {
        row.list_id
    });
    let latest: HashMap<Uuid, Option<Uuid>> = db::selections::latest_for_lists(&mut *conn, &ids)
        .await?
        .into_iter()
        .map(|row| (row.list_id, row.starter_id))
        .collect();
    let active_rows = db::selections::active_for_lists(&mut *conn, &ids).await?;
    let mut active: HashMap<Uuid, SelectionSnapshot> = load_selections(conn, active_rows)
        .await?
        .into_iter()
        .map(|snapshot| (snapshot.selection.list_id, snapshot))
        .collect();
    Ok(lists
        .into_iter()
        .map(|list| ListSnapshot {
            participants: participants.remove(&list.id).unwrap_or_default(),
            films: films.remove(&list.id).unwrap_or_default(),
            active_selection: active.remove(&list.id),
            latest_starter_id: latest.get(&list.id).copied().flatten(),
            list,
        })
        .collect())
}

pub async fn load_list(conn: &mut PgConnection, list: ListRow) -> sqlx::Result<ListSnapshot> {
    let mut snapshots = load_lists(conn, vec![list]).await?;
    Ok(snapshots.remove(0))
}

/// Loads the eliminations and verdicts of the given selections, keeping their order.
pub async fn load_selections(
    conn: &mut PgConnection,
    selections: Vec<SelectionRow>,
) -> sqlx::Result<Vec<SelectionSnapshot>> {
    let ids: Vec<Uuid> = selections.iter().map(|selection| selection.id).collect();
    let mut eliminations = group_by(
        db::selections::eliminations_for(&mut *conn, &ids).await?,
        |row| row.selection_id,
    );
    let mut verdicts = group_by(
        db::selections::verdicts_for(&mut *conn, &ids).await?,
        |row| row.selection_id,
    );
    Ok(selections
        .into_iter()
        .map(|selection| SelectionSnapshot {
            eliminations: eliminations.remove(&selection.id).unwrap_or_default(),
            verdicts: verdicts.remove(&selection.id).unwrap_or_default(),
            selection,
        })
        .collect())
}

fn group_by<T>(rows: Vec<T>, key: impl Fn(&T) -> Uuid) -> HashMap<Uuid, Vec<T>> {
    let mut groups: HashMap<Uuid, Vec<T>> = HashMap::new();
    for row in rows {
        groups.entry(key(&row)).or_default().push(row);
    }
    groups
}
