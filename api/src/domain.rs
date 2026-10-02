//! Rules of the elimination game, independent of storage and transport.

use chrono::{DateTime, Utc};
use uuid::Uuid;

pub const MAX_PARTICIPANTS: usize = 10;
pub const MAX_FILMS: usize = 100;

/// A selection can start when its `film_count - 1` eliminations split evenly
/// between the participants, and there is at least one to play.
pub fn is_playable(film_count: usize, participant_count: usize) -> bool {
    participant_count >= 1 && film_count >= 2 && (film_count - 1).is_multiple_of(participant_count)
}

/// Films to add before a selection can start.
pub fn films_to_add(film_count: usize, participant_count: usize) -> usize {
    if participant_count == 0 {
        return 0;
    }
    let target = film_count.max(2);
    let remainder = (target - 1) % participant_count;
    let playable = if remainder == 0 {
        target
    } else {
        target + participant_count - remainder
    };
    playable - film_count
}

/// Place given to the `index`-th elimination (0-based) of a selection over
/// `film_count` films: the first film out finishes last.
pub fn elimination_place(film_count: i32, index: usize) -> i32 {
    film_count - i32::try_from(index).unwrap_or(i32::MAX)
}

/// Index, in turn order, of the participant who plays the `elimination_index`-th
/// elimination when the participant at `starter_index` opened the selection.
pub fn turn_index(
    starter_index: usize,
    elimination_index: usize,
    participant_count: usize,
) -> usize {
    (starter_index + elimination_index) % participant_count
}

/// Index of the participant who opens the next selection: the one after the
/// previous opener, or the first one when there is no previous selection or
/// its opener has left the list.
pub fn next_starter_index(
    previous_starter_index: Option<usize>,
    participant_count: usize,
) -> usize {
    previous_starter_index.map_or(0, |index| (index + 1) % participant_count)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Watched,
    /// The film could not be watched; the next best film takes its place.
    Abandoned,
}

impl Outcome {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Watched => "watched",
            Self::Abandoned => "abandoned",
        }
    }

    pub fn from_db(value: &str) -> Option<Self> {
        match value {
            "watched" => Some(Self::Watched),
            "abandoned" => Some(Self::Abandoned),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Eliminating,
    /// One film is left and waits for a verdict; `place` is the place of the
    /// film under judgement (1 for the survivor, then the runners-up).
    AwaitingVerdict {
        place: i32,
    },
    /// A film was watched, or every film was abandoned.
    Completed,
}

/// Phase of a selection over `film_count` films, given how many eliminations
/// were played and the outcomes of the verdicts already given, best place first.
pub fn phase(film_count: i32, elimination_count: usize, verdicts: &[Outcome]) -> Phase {
    let eliminations_needed = usize::try_from(film_count - 1).unwrap_or(0);
    if elimination_count < eliminations_needed {
        return Phase::Eliminating;
    }
    if verdicts.contains(&Outcome::Watched) {
        return Phase::Completed;
    }
    let next_place = i32::try_from(verdicts.len())
        .unwrap_or(i32::MAX)
        .saturating_add(1);
    if next_place > film_count {
        Phase::Completed
    } else {
        Phase::AwaitingVerdict { place: next_place }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Run {
    pub selection_id: Uuid,
    pub started_at: DateTime<Utc>,
    pub place: i32,
    pub film_count: i32,
    pub eliminated_by: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EliminatorCount {
    pub name: String,
    pub count: usize,
}

/// Aggregates over every selection a film took part in.
#[derive(Clone, Debug, PartialEq)]
pub struct FilmStatistics {
    pub selection_count: usize,
    pub best_place: Option<i32>,
    pub worst_place: Option<i32>,
    /// Mean place, when every selection had the same number of films.
    pub average_place: Option<f64>,
    /// Mean relative standing, from 0 (always last) to 1 (always first);
    /// comparable across selections of different sizes.
    pub average_standing: Option<f64>,
    /// Number of selections the film finished second in.
    pub finals_lost: usize,
    /// Who eliminated the film, most frequent first.
    pub eliminated_by: Vec<EliminatorCount>,
}

/// Computes a film's statistics from its runs.
pub fn film_statistics(runs: &[Run]) -> FilmStatistics {
    let places = runs.iter().map(|run| run.place);
    let same_size = runs
        .windows(2)
        .all(|pair| pair[0].film_count == pair[1].film_count);
    let count = runs.len();
    let mean = |sum: f64| {
        #[allow(
            clippy::cast_precision_loss,
            reason = "a film takes part in far fewer than 2^52 selections"
        )]
        let count = count as f64;
        sum / count
    };
    let average_place =
        (count > 0 && same_size).then(|| mean(runs.iter().map(|run| f64::from(run.place)).sum()));
    let average_standing = (count > 0).then(|| {
        mean(
            runs.iter()
                .map(|run| {
                    f64::from(run.film_count - run.place) / f64::from((run.film_count - 1).max(1))
                })
                .sum(),
        )
    });
    let mut eliminated_by: Vec<EliminatorCount> = Vec::new();
    for run in runs {
        match eliminated_by
            .iter_mut()
            .find(|entry| entry.name == run.eliminated_by)
        {
            Some(entry) => entry.count += 1,
            None => eliminated_by.push(EliminatorCount {
                name: run.eliminated_by.clone(),
                count: 1,
            }),
        }
    }
    eliminated_by.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.name.cmp(&b.name)));
    FilmStatistics {
        selection_count: count,
        best_place: places.clone().min(),
        worst_place: places.max(),
        average_place,
        average_standing,
        finals_lost: runs.iter().filter(|run| run.place == 2).count(),
        eliminated_by,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_participants_need_an_odd_film_count() {
        assert!(is_playable(21, 2));
        assert!(!is_playable(20, 2));
    }

    #[test]
    fn three_participants_need_one_film_above_a_multiple_of_three() {
        assert!(is_playable(22, 3));
        assert!(!is_playable(21, 3));
    }

    #[test]
    fn a_solo_list_needs_two_films() {
        assert!(is_playable(2, 1));
        assert!(is_playable(17, 1));
        assert!(!is_playable(1, 1));
    }

    #[test]
    fn a_playable_list_needs_no_more_films() {
        assert_eq!(films_to_add(21, 2), 0);
    }

    #[test]
    fn films_to_add_reach_the_next_playable_count() {
        assert_eq!(films_to_add(20, 2), 1);
        assert_eq!(films_to_add(21, 3), 1);
        assert_eq!(films_to_add(20, 3), 2);
        assert_eq!(films_to_add(1, 2), 2);
    }

    #[test]
    fn first_eliminated_film_finishes_last() {
        assert_eq!(elimination_place(21, 0), 21);
        assert_eq!(elimination_place(21, 19), 2);
    }

    #[test]
    fn turns_rotate_from_the_starter() {
        assert_eq!(turn_index(1, 0, 2), 1);
        assert_eq!(turn_index(1, 1, 2), 0);
        assert_eq!(turn_index(2, 2, 3), 1);
    }

    #[test]
    fn next_starter_follows_the_previous_one() {
        assert_eq!(next_starter_index(None, 2), 0);
        assert_eq!(next_starter_index(Some(0), 2), 1);
        assert_eq!(next_starter_index(Some(1), 2), 0);
    }

    #[test]
    fn phase_is_eliminating_until_one_film_is_left() {
        assert_eq!(phase(21, 19, &[]), Phase::Eliminating);
    }

    #[test]
    fn phase_awaits_the_survivor_verdict() {
        assert_eq!(phase(21, 20, &[]), Phase::AwaitingVerdict { place: 1 });
    }

    #[test]
    fn abandoning_hands_over_to_the_runner_up() {
        assert_eq!(
            phase(21, 20, &[Outcome::Abandoned]),
            Phase::AwaitingVerdict { place: 2 }
        );
    }

    #[test]
    fn watching_completes_the_selection() {
        assert_eq!(
            phase(21, 20, &[Outcome::Abandoned, Outcome::Watched]),
            Phase::Completed
        );
    }

    #[test]
    fn abandoning_everything_completes_the_selection() {
        assert_eq!(phase(3, 2, &[Outcome::Abandoned; 3]), Phase::Completed);
    }

    fn run(place: i32, film_count: i32, eliminated_by: &str) -> Run {
        Run {
            selection_id: Uuid::nil(),
            started_at: DateTime::UNIX_EPOCH,
            place,
            film_count,
            eliminated_by: eliminated_by.to_owned(),
        }
    }

    #[test]
    fn statistics_of_a_film_that_never_played_are_empty() {
        let stats = film_statistics(&[]);
        assert_eq!(stats.selection_count, 0);
        assert_eq!(stats.best_place, None);
        assert_eq!(stats.average_place, None);
        assert_eq!(stats.average_standing, None);
    }

    #[test]
    fn statistics_aggregate_places_and_eliminators() {
        let stats = film_statistics(&[run(9, 21, "Sam"), run(5, 21, "Alex"), run(2, 21, "Alex")]);
        assert_eq!(stats.selection_count, 3);
        assert_eq!(stats.best_place, Some(2));
        assert_eq!(stats.worst_place, Some(9));
        assert_eq!(stats.average_place, Some(16.0 / 3.0));
        assert_eq!(stats.finals_lost, 1);
        assert_eq!(
            stats.eliminated_by,
            vec![
                EliminatorCount {
                    name: "Alex".to_owned(),
                    count: 2
                },
                EliminatorCount {
                    name: "Sam".to_owned(),
                    count: 1
                },
            ]
        );
    }

    #[test]
    fn statistics_drop_the_raw_average_when_sizes_differ() {
        let stats = film_statistics(&[run(2, 21, "Sam"), run(2, 23, "Alex")]);
        assert_eq!(stats.average_place, None);
        let standing = stats.average_standing.expect("standing");
        assert!((standing - f64::midpoint(19.0 / 20.0, 21.0 / 22.0)).abs() < 1e-9);
    }
}
