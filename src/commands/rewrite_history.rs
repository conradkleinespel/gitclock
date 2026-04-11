use crate::commands::commit::get_next_commit_date;
use crate::config::Config;
use crate::git::{
    GitShell, LogEntry, RealGitShell, amend_with_new_date_with_shell, cherry_pick_with_shell,
    get_log_sha_and_dates_with_shell, reset_hard_with_shell,
};
use crate::timeslot::Timeslot;
use chrono::{DateTime, Utc};

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

pub fn run_rewrite_history_command(now: DateTime<Utc>, config: &Config) -> i32 {
    rewrite_history_with_shell(now, &RealGitShell, config)
}

pub fn rewrite_history_with_shell(
    now: DateTime<Utc>,
    shell: &dyn GitShell,
    config: &Config,
) -> i32 {
    let timeslots = config.get_timeslots();
    if timeslots.is_empty() {
        println!("No timeslots found. Please add timeslots.");
        return 1;
    }

    println!("Rewriting commit dates.");

    let log_entries = match get_log_sha_and_dates_with_shell(shell) {
        Ok(entries) => entries,
        Err(e) => {
            eprintln!("Error getting log: {}", e);
            return 1;
        }
    };

    if log_entries.is_empty() {
        return 0;
    }

    // Amend the first commit to bootstrap the process
    reset_hard_with_shell(shell, &log_entries[0].sha);
    let mut last_commit_date = Some(amend_commit(
        now,
        shell,
        &log_entries[0],
        &timeslots,
        None,
        &config.get_timezone(),
    ));

    for log_entry in log_entries.iter().skip(1) {
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
        assert_eq!(run_rewrite_history_command(now, &config), 1);
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

        assert_eq!(rewrite_history_with_shell(now, &shell, &config), 0);
    }
}
