use crate::config::Config;
use crate::git::{GitShell, RealGitShell, get_last_commit_date_with_shell, git_commit_with_shell};
use crate::timeslot::Timeslot;
use chrono::{DateTime, Timelike, Utc};
use rand::Rng;

const MIN_TIME_BETWEEN_COMMITS_MINUTES: i64 = 1;
const MAX_TIME_BETWEEN_COMMITS_MINUTES: i64 = 15;

pub fn run_commit_command(now: DateTime<Utc>, args: &[String], config: &Config) -> i32 {
    commit_with_shell(now, &RealGitShell, args, config)
}

pub fn commit_with_shell(
    now: DateTime<Utc>,
    shell: &dyn GitShell,
    args: &[String],
    config: &Config,
) -> i32 {
    let timeslots = config.get_timeslots();
    if timeslots.is_empty() {
        println!("No timeslots found. Please add timeslots.");
        return 1;
    }

    let last_commit_date = match get_last_commit_date_with_shell(shell, now) {
        Ok(date) => date,
        Err(e) => {
            eprintln!("Error: {}", e);
            return 1;
        }
    };
    let min_date_for_next_commit = if last_commit_date > now {
        last_commit_date
    } else {
        now
    };

    let next_commit_date = get_next_commit_date(now, min_date_for_next_commit, &timeslots);

    git_commit_with_shell(shell, next_commit_date, &config.get_timezone(), args)
}

pub fn get_next_commit_date(
    now: DateTime<Utc>,
    min_date: DateTime<Utc>,
    timeslots: &[Timeslot],
) -> DateTime<Utc> {
    let mut next_commit_date: Option<DateTime<Utc>> = None;

    for timeslot in timeslots {
        let this_next_commit_date = timeslot.next_suitable_date(min_date);
        match next_commit_date {
            None => next_commit_date = Some(this_next_commit_date),
            Some(date) => {
                // We want the earliest available timeslot to be used
                if this_next_commit_date < date {
                    next_commit_date = Some(this_next_commit_date)
                }
            }
        }
    }

    let next_commit_date = next_commit_date.expect("Unreachable. There are timeslots, there should be a next commit date. Please report this as a bug.");

    if now == next_commit_date {
        return now;
    }

    let mut rng = rand::thread_rng();
    let minutes_to_add =
        rng.gen_range(MIN_TIME_BETWEEN_COMMITS_MINUTES..=MAX_TIME_BETWEEN_COMMITS_MINUTES);
    let seconds_to_add = if next_commit_date.second() < 60 {
        rng.gen_range(0..=(60 - next_commit_date.second() - 1))
    } else {
        0
    };

    next_commit_date
        + chrono::Duration::minutes(minutes_to_add)
        + chrono::Duration::seconds(seconds_to_add as i64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, ConfigData, TimeslotConfig};
    use crate::git::MockGitShell;
    use crate::spawn_async::SpawnResult;
    use chrono::TimeZone;

    #[test]
    fn get_next_commit_date_returns_time_within_timeslots() {
        let min_date = Utc.with_ymd_and_hms(2023, 7, 4, 9, 0, 0).unwrap();
        let timeslots = vec![Timeslot::new("1-7", "0000", "2359", "UTC").unwrap()];
        let result = get_next_commit_date(min_date, min_date, &timeslots);

        assert!(result >= min_date);
        assert!(result <= min_date + chrono::Duration::minutes(15 + 1));
    }

    #[test]
    fn get_next_commit_date_respects_timeslot_start() {
        let min_date = Utc.with_ymd_and_hms(2023, 7, 4, 9, 0, 0).unwrap();
        // Timeslot starts at 10:00 UTC
        let timeslots = vec![Timeslot::new("1-7", "1000", "1600", "UTC").unwrap()];
        let result = get_next_commit_date(min_date, min_date, &timeslots);

        let min_expected = Utc.with_ymd_and_hms(2023, 7, 4, 10, 0, 0).unwrap();
        let max_expected = min_expected + chrono::Duration::minutes(15 + 1);

        assert!(result >= min_expected);
        assert!(result <= max_expected);
    }

    #[test]
    fn commit_returns_0_when_everything_is_valid() {
        let mut shell = MockGitShell::new();
        let last_commit_date = "2023-07-04T10:00:00Z";
        let now = Utc::now();
        shell
            .expect_spawn_async()
            .with(
                mockall::predicate::eq("git"),
                mockall::predicate::eq(vec![
                    "log".to_string(),
                    "-1".to_string(),
                    "--format=%cI".to_string(),
                ]),
                mockall::predicate::eq(false),
                mockall::predicate::always(),
            )
            .returning(move |_, _, _, _| {
                Ok(SpawnResult {
                    code: 0,
                    stdout: last_commit_date.to_string(),
                    stderr: "".to_string(),
                })
            });

        shell
            .expect_spawn_async()
            .with(
                mockall::predicate::eq("git"),
                mockall::predicate::always(),
                mockall::predicate::eq(true),
                mockall::predicate::always(),
            )
            .returning(|_, _, _, _| {
                Ok(SpawnResult {
                    code: 0,
                    stdout: "".to_string(),
                    stderr: "".to_string(),
                })
            });

        let config = Config::create_test_config(ConfigData {
            timeslots: vec![TimeslotConfig {
                days: "1-7".to_string(),
                start: "0000".to_string(),
                end: "2359".to_string(),
            }],
            timezone: Some("UTC".to_string()),
            ..ConfigData::default()
        });

        let result = commit_with_shell(
            now,
            &shell,
            &["-m".to_string(), "message".to_string()],
            &config,
        );
        assert_eq!(result, 0);
    }

    #[test]
    fn commit_returns_1_when_no_timeslots() {
        let shell = MockGitShell::new();
        let config = Config::create_test_config(ConfigData::default());
        let now = Utc::now();
        let result = commit_with_shell(now, &shell, &[], &config);
        assert_eq!(result, 1);
    }

    #[test]
    fn commit_returns_1_when_git_commit_fails() {
        let mut shell = MockGitShell::new();
        let now = Utc::now();
        shell
            .expect_spawn_async()
            .with(
                mockall::predicate::eq("git"),
                mockall::predicate::eq(vec![
                    "log".to_string(),
                    "-1".to_string(),
                    "--format=%cI".to_string(),
                ]),
                mockall::predicate::eq(false),
                mockall::predicate::always(),
            )
            .returning(|_, _, _, _| {
                Ok(SpawnResult {
                    code: 0,
                    stdout: "2023-07-04T10:00:00Z".to_string(),
                    stderr: "".to_string(),
                })
            });
        shell
            .expect_spawn_async()
            .with(
                mockall::predicate::eq("git"),
                mockall::predicate::always(),
                mockall::predicate::eq(true),
                mockall::predicate::always(),
            )
            .returning(|_, _, _, _| {
                Ok(SpawnResult {
                    code: 1,
                    stdout: "".to_string(),
                    stderr: "".to_string(),
                })
            });

        let config = Config::create_test_config(ConfigData {
            timeslots: vec![TimeslotConfig {
                days: "1-7".to_string(),
                start: "0000".to_string(),
                end: "2359".to_string(),
            }],
            timezone: Some("UTC".to_string()),
            ..ConfigData::default()
        });

        let result = commit_with_shell(now, &shell, &[], &config);
        assert_eq!(result, 1);
    }
}
