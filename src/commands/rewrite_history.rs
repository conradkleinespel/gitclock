use crate::commands::commit::get_next_commit_date;
use crate::config::Config;
use crate::git::{
    GitShell, LogEntry, RealGitShell, amend_with_new_date_with_shell, cherry_pick_with_shell,
    get_log_sha_and_dates_with_shell, reset_hard_with_shell,
};
use crate::timeslot::Timeslot;
use chrono::{DateTime, Utc};
use tracing::debug;

pub fn choose_min_date_for_new_commit(
    existing_author_date: DateTime<Utc>,
    existing_commit_date: DateTime<Utc>,
    last_commit_date: Option<DateTime<Utc>>,
) -> DateTime<Utc> {
    let min_date_from_log_entry = if existing_author_date > existing_commit_date {
        existing_author_date
    } else {
        existing_commit_date
    };

    match last_commit_date {
        Some(last) if last > min_date_from_log_entry => last,
        _ => min_date_from_log_entry,
    }
}

fn amend_commit(
    now: DateTime<Utc>,
    shell: &dyn GitShell,
    log_entry: &LogEntry,
    timeslots: &[Timeslot],
    last_commit_date: Option<DateTime<Utc>>,
    timezone: &str,
) -> DateTime<Utc> {
    let min_date = choose_min_date_for_new_commit(
        log_entry.author_date,
        log_entry.commit_date,
        last_commit_date,
    );
    let new_commit_date = get_next_commit_date(now, min_date, timeslots);
    amend_with_new_date_with_shell(shell, new_commit_date, timezone);
    new_commit_date
}

pub fn run_rewrite_history_command(
    now: DateTime<Utc>,
    from_commit: Option<&str>,
    config: &Config,
) -> i32 {
    rewrite_history_with_shell(now, &RealGitShell, from_commit, config)
}

pub fn rewrite_history_with_shell(
    now: DateTime<Utc>,
    shell: &dyn GitShell,
    from_commit: Option<&str>,
    config: &Config,
) -> i32 {
    let timeslots = config.get_timeslots();
    debug!(?timeslots, "Rewriting history with timeslots");
    if timeslots.is_empty() {
        println!("No timeslots found. Please add timeslots.");
        return 1;
    }

    println!("Rewriting commit dates.");

    let log_entries = match get_log_sha_and_dates_with_shell(shell) {
        Ok(entries) => {
            debug!(count = entries.len(), "Retrieved log entries");
            entries
        }
        Err(e) => {
            eprintln!("Error getting log: {}", e);
            return 1;
        }
    };

    if log_entries.is_empty() {
        println!("No log entries to rewrite");
        return 0;
    }

    let (start_index, initial_last_commit_date) = match from_commit {
        Some(target) => {
            match log_entries
                .iter()
                .position(|entry| entry.sha == target || entry.sha.starts_with(target))
            {
                Some(i) => {
                    let parent_date = if i > 0 {
                        Some(log_entries[i - 1].commit_date)
                    } else {
                        None
                    };
                    (i, parent_date)
                }
                None => {
                    println!("Commit {} not found in repository history", target);
                    return 1;
                }
            }
        }
        None => (0, None),
    };

    let entries_to_rewrite = &log_entries[start_index..];
    println!("Found {} log entries to rewrite", entries_to_rewrite.len());

    // Amend the first commit to bootstrap the process
    println!(
        "Rewriting commit {}",
        entries_to_rewrite[0].sha.to_string().as_str()
    );
    reset_hard_with_shell(shell, &entries_to_rewrite[0].sha);
    let mut last_commit_date = Some(amend_commit(
        now,
        shell,
        &entries_to_rewrite[0],
        &timeslots,
        initial_last_commit_date,
        &config.get_timezone(),
    ));

    for log_entry in entries_to_rewrite.iter().skip(1) {
        println!("Rewriting commit {}", log_entry.sha.to_string().as_str());
        cherry_pick_with_shell(shell, &log_entry.sha);
        last_commit_date = Some(amend_commit(
            now,
            shell,
            log_entry,
            &timeslots,
            last_commit_date,
            &config.get_timezone(),
        ));
    }

    println!("Finished rewriting commit dates");

    0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, ConfigData, TimeslotConfig};
    use crate::git::{MockGitShell, SpawnResult};
    use chrono::TimeZone;

    #[test]
    fn test_choose_min_date_for_new_commit() {
        let date1 = Utc.with_ymd_and_hms(2023, 7, 4, 10, 0, 0).unwrap();
        let date2 = Utc.with_ymd_and_hms(2023, 7, 4, 11, 0, 0).unwrap();
        let last = Utc.with_ymd_and_hms(2023, 7, 4, 12, 0, 0).unwrap();

        assert_eq!(choose_min_date_for_new_commit(date1, date2, None), date2);
        assert_eq!(choose_min_date_for_new_commit(date2, date1, None), date2);
        assert_eq!(
            choose_min_date_for_new_commit(date1, date2, Some(last)),
            last
        );
    }

    #[test]
    fn test_rewrite_history_returns_1_if_no_timeslots() {
        let config = Config::create_test_config(ConfigData::default());
        let now = Utc::now();
        assert_eq!(run_rewrite_history_command(now, None, &config), 1);
    }

    #[test]
    fn test_rewrite_history_success() {
        let mut shell = MockGitShell::new();
        let now = Utc::now();
        let log_output = format!(
            "{} 2023-07-04T10:00:00Z 2023-07-04T10:00:00Z\n{} 2023-07-04T11:00:00Z 2023-07-04T11:00:00Z",
            "a".repeat(40),
            "b".repeat(40)
        );

        shell
            .expect_spawn_async()
            .with(
                mockall::predicate::eq("git"),
                mockall::predicate::eq(vec![
                    "log".to_string(),
                    "--pretty=format:%H %aI %cI".to_string(),
                    "--reverse".to_string(),
                ]),
                mockall::predicate::always(),
                mockall::predicate::always(),
            )
            .returning(move |_, _, _, _| {
                Ok(SpawnResult {
                    code: 0,
                    stdout: log_output.clone(),
                    stderr: "".to_string(),
                })
            });

        shell
            .expect_spawn_async()
            .with(
                mockall::predicate::eq("git"),
                mockall::predicate::eq(vec![
                    "reset".to_string(),
                    "--hard".to_string(),
                    "a".repeat(40),
                ]),
                mockall::predicate::always(),
                mockall::predicate::always(),
            )
            .returning(|_, _, _, _| {
                Ok(SpawnResult {
                    code: 0,
                    stdout: "".to_string(),
                    stderr: "".to_string(),
                })
            });

        // amend_commit for the first commit
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

        shell
            .expect_spawn_async()
            .with(
                mockall::predicate::eq("git"),
                mockall::predicate::eq(vec!["cherry-pick".to_string(), "b".repeat(40)]),
                mockall::predicate::always(),
                mockall::predicate::always(),
            )
            .returning(|_, _, _, _| {
                Ok(SpawnResult {
                    code: 0,
                    stdout: "".to_string(),
                    stderr: "".to_string(),
                })
            });

        // amend_commit for the second commit
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

        assert_eq!(rewrite_history_with_shell(now, &shell, None, &config), 0);
    }

    #[test]
    fn test_rewrite_history_from_specific_commit_success() {
        let mut shell = MockGitShell::new();
        let now = Utc.with_ymd_and_hms(2023, 7, 4, 18, 0, 0).unwrap();
        let log_output = format!(
            "{} 2023-07-04T10:00:00Z 2023-07-04T10:00:00Z\n{} 2023-07-04T11:00:00Z 2023-07-04T11:00:00Z\n{} 2023-07-04T12:00:00Z 2023-07-04T12:00:00Z",
            "a".repeat(40),
            "b".repeat(40),
            "c".repeat(40)
        );

        shell
            .expect_spawn_async()
            .with(
                mockall::predicate::eq("git"),
                mockall::predicate::eq(vec![
                    "log".to_string(),
                    "--pretty=format:%H %aI %cI".to_string(),
                    "--reverse".to_string(),
                ]),
                mockall::predicate::always(),
                mockall::predicate::always(),
            )
            .returning(move |_, _, _, _| {
                Ok(SpawnResult {
                    code: 0,
                    stdout: log_output.clone(),
                    stderr: "".to_string(),
                })
            });

        // Reset directly to commit B (not A)
        shell
            .expect_spawn_async()
            .with(
                mockall::predicate::eq("git"),
                mockall::predicate::eq(vec![
                    "reset".to_string(),
                    "--hard".to_string(),
                    "b".repeat(40),
                ]),
                mockall::predicate::always(),
                mockall::predicate::always(),
            )
            .returning(|_, _, _, _| {
                Ok(SpawnResult {
                    code: 0,
                    stdout: "".to_string(),
                    stderr: "".to_string(),
                })
            });

        // amend_commit for commit B
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

        // cherry-pick commit C
        shell
            .expect_spawn_async()
            .with(
                mockall::predicate::eq("git"),
                mockall::predicate::eq(vec!["cherry-pick".to_string(), "c".repeat(40)]),
                mockall::predicate::always(),
                mockall::predicate::always(),
            )
            .returning(|_, _, _, _| {
                Ok(SpawnResult {
                    code: 0,
                    stdout: "".to_string(),
                    stderr: "".to_string(),
                })
            });

        // amend_commit for commit C
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

        assert_eq!(
            rewrite_history_with_shell(now, &shell, Some(&"b".repeat(40)), &config),
            0
        );
    }

    #[test]
    fn test_rewrite_history_from_short_hash_prefix() {
        let mut shell = MockGitShell::new();
        let now = Utc.with_ymd_and_hms(2023, 7, 4, 18, 0, 0).unwrap();
        let log_output = format!(
            "{} 2023-07-04T10:00:00Z 2023-07-04T10:00:00Z\n{} 2023-07-04T11:00:00Z 2023-07-04T11:00:00Z",
            "a".repeat(40),
            "b".repeat(40)
        );

        shell
            .expect_spawn_async()
            .with(
                mockall::predicate::eq("git"),
                mockall::predicate::eq(vec![
                    "log".to_string(),
                    "--pretty=format:%H %aI %cI".to_string(),
                    "--reverse".to_string(),
                ]),
                mockall::predicate::always(),
                mockall::predicate::always(),
            )
            .returning(move |_, _, _, _| {
                Ok(SpawnResult {
                    code: 0,
                    stdout: log_output.clone(),
                    stderr: "".to_string(),
                })
            });

        shell
            .expect_spawn_async()
            .with(
                mockall::predicate::eq("git"),
                mockall::predicate::eq(vec![
                    "reset".to_string(),
                    "--hard".to_string(),
                    "b".repeat(40),
                ]),
                mockall::predicate::always(),
                mockall::predicate::always(),
            )
            .returning(|_, _, _, _| {
                Ok(SpawnResult {
                    code: 0,
                    stdout: "".to_string(),
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

        // Call using short prefix (7 chars)
        assert_eq!(
            rewrite_history_with_shell(now, &shell, Some(&"b".repeat(7)), &config),
            0
        );
    }

    #[test]
    fn test_rewrite_history_from_head_commit() {
        let mut shell = MockGitShell::new();
        let now = Utc.with_ymd_and_hms(2023, 7, 4, 18, 0, 0).unwrap();
        let log_output = format!(
            "{} 2023-07-04T10:00:00Z 2023-07-04T10:00:00Z\n{} 2023-07-04T11:00:00Z 2023-07-04T11:00:00Z",
            "a".repeat(40),
            "b".repeat(40)
        );

        shell
            .expect_spawn_async()
            .with(
                mockall::predicate::eq("git"),
                mockall::predicate::eq(vec![
                    "log".to_string(),
                    "--pretty=format:%H %aI %cI".to_string(),
                    "--reverse".to_string(),
                ]),
                mockall::predicate::always(),
                mockall::predicate::always(),
            )
            .returning(move |_, _, _, _| {
                Ok(SpawnResult {
                    code: 0,
                    stdout: log_output.clone(),
                    stderr: "".to_string(),
                })
            });

        shell
            .expect_spawn_async()
            .with(
                mockall::predicate::eq("git"),
                mockall::predicate::eq(vec![
                    "reset".to_string(),
                    "--hard".to_string(),
                    "b".repeat(40),
                ]),
                mockall::predicate::always(),
                mockall::predicate::always(),
            )
            .returning(|_, _, _, _| {
                Ok(SpawnResult {
                    code: 0,
                    stdout: "".to_string(),
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

        // Target HEAD commit (b)
        assert_eq!(
            rewrite_history_with_shell(now, &shell, Some(&"b".repeat(40)), &config),
            0
        );
    }

    #[test]
    fn test_rewrite_history_from_unknown_commit_fails() {
        let mut shell = MockGitShell::new();
        let now = Utc.with_ymd_and_hms(2023, 7, 4, 18, 0, 0).unwrap();
        let log_output = format!(
            "{} 2023-07-04T10:00:00Z 2023-07-04T10:00:00Z\n{} 2023-07-04T11:00:00Z 2023-07-04T11:00:00Z",
            "a".repeat(40),
            "b".repeat(40)
        );

        shell
            .expect_spawn_async()
            .with(
                mockall::predicate::eq("git"),
                mockall::predicate::eq(vec![
                    "log".to_string(),
                    "--pretty=format:%H %aI %cI".to_string(),
                    "--reverse".to_string(),
                ]),
                mockall::predicate::always(),
                mockall::predicate::always(),
            )
            .returning(move |_, _, _, _| {
                Ok(SpawnResult {
                    code: 0,
                    stdout: log_output.clone(),
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

        // Unknown commit should return 1 and not execute reset or cherry-pick
        assert_eq!(
            rewrite_history_with_shell(now, &shell, Some("deadbeef"), &config),
            1
        );
    }

    #[test]
    fn test_rewrite_history_from_commit_respects_parent_date() {
        let mut shell = MockGitShell::new();
        let now = Utc.with_ymd_and_hms(2023, 7, 4, 18, 0, 0).unwrap();
        // Commit A is at 15:00, Commit B author/commit date is earlier at 10:00
        let log_output = format!(
            "{} 2023-07-04T15:00:00Z 2023-07-04T15:00:00Z\n{} 2023-07-04T10:00:00Z 2023-07-04T10:00:00Z",
            "a".repeat(40),
            "b".repeat(40)
        );

        shell
            .expect_spawn_async()
            .with(
                mockall::predicate::eq("git"),
                mockall::predicate::eq(vec![
                    "log".to_string(),
                    "--pretty=format:%H %aI %cI".to_string(),
                    "--reverse".to_string(),
                ]),
                mockall::predicate::always(),
                mockall::predicate::always(),
            )
            .returning(move |_, _, _, _| {
                Ok(SpawnResult {
                    code: 0,
                    stdout: log_output.clone(),
                    stderr: "".to_string(),
                })
            });

        shell
            .expect_spawn_async()
            .with(
                mockall::predicate::eq("git"),
                mockall::predicate::eq(vec![
                    "reset".to_string(),
                    "--hard".to_string(),
                    "b".repeat(40),
                ]),
                mockall::predicate::always(),
                mockall::predicate::always(),
            )
            .returning(|_, _, _, _| {
                Ok(SpawnResult {
                    code: 0,
                    stdout: "".to_string(),
                    stderr: "".to_string(),
                })
            });

        // amend_commit for commit B: check that the formatted git date is set after parent commit's date
        // rather than B's own original date
        let minimum_expected_rewritten_timestamp = Utc
            .with_ymd_and_hms(2023, 7, 4, 15, 0, 0)
            .unwrap()
            .timestamp();
        shell
            .expect_spawn_async()
            .with(
                mockall::predicate::eq("git"),
                mockall::predicate::function(move |args: &[String]| {
                    if args.len() == 5
                        && args[0] == "commit"
                        && args[1] == "--amend"
                        && args[2] == "--no-edit"
                        && args[3] == "--date"
                    {
                        // Parse timestamp (seconds) from "TIMESTAMP +0000"
                        if let Some(ts_str) = args[4].split_whitespace().next() {
                            if let Ok(ts) = ts_str.parse::<i64>() {
                                return ts >= minimum_expected_rewritten_timestamp;
                            }
                        }
                    }
                    false
                }),
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

        assert_eq!(
            rewrite_history_with_shell(now, &shell, Some(&"b".repeat(40)), &config),
            0
        );
    }
}
