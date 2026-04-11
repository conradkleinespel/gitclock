use crate::config::Config;
use crate::git::{GitShell, RealGitShell, get_last_commit_date_with_shell};
use chrono::{DateTime, Utc};
use std::env;

pub fn run_pre_commit_hook_command(now: DateTime<Utc>, config: &Config) -> i32 {
    pre_commit_hook_with_shell(now, &RealGitShell, config)
}

pub fn pre_commit_hook_with_shell(
    now: DateTime<Utc>,
    shell: &dyn GitShell,
    config: &Config,
) -> i32 {
    println!("Running gitclock pre-commit-hook...");

    let timeslots = config.get_timeslots();
    if timeslots.is_empty() {
        eprintln!("No timeslots found. Please add timeslots.");
        return 1;
    }

    let is_within_timeslot = timeslots.iter().any(|t| t.is_date_within(now));
    let gitclock_env = env::var("GITCLOCK").unwrap_or_default();

    if gitclock_env != "1" && !is_within_timeslot {
        eprintln!("Cannot commit outside timeslot. Use gitclock to create your commit.");
        return 1;
    }

    let last_commit_date = match get_last_commit_date_with_shell(shell, now) {
        Ok(date) => date,
        Err(e) => {
            eprintln!("Error checking last commit date: {}", e);
            return 1;
        }
    };

    if gitclock_env != "1" && env::var("GIT_COMMITTER_DATE").is_err() && last_commit_date > now {
        eprintln!(
            "Cannot commit with current date, because last commit is in the future. Use `gitclock commit`."
        );
        return 1;
    }

    println!("Pre-commit hook finished successfully.");
    0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, ConfigData, TimeslotConfig};
    use crate::git::{MockGitShell, SpawnResult};
    use chrono::Datelike;
    use serial_test::serial;

    #[test]
    #[serial]
    fn fails_when_outside_timeslot_and_no_gitclock_env() {
        let now = Utc::now();
        let weekday = now.weekday().number_from_monday();
        let other_weekday = if weekday == 7 { 1 } else { weekday + 1 };

        let config = Config::create_test_config(ConfigData {
            timeslots: vec![TimeslotConfig {
                days: format!("{}-{}", other_weekday, other_weekday),
                start: "0900".to_string(),
                end: "1200".to_string(),
            }],
            timezone: Some("UTC".to_string()),
            ..ConfigData::default()
        });

        unsafe {
            env::remove_var("GITCLOCK");
        }
        let mut shell = MockGitShell::new();
        shell.expect_spawn_async().returning(|_, _, _, _| {
            Ok(SpawnResult {
                code: 0,
                stdout: Utc::now().to_rfc3339(),
                stderr: "".to_string(),
            })
        });
        assert_eq!(pre_commit_hook_with_shell(now, &shell, &config), 1);
    }

    #[test]
    #[serial]
    fn fails_when_last_commit_is_in_future_and_no_gitclock_env() {
        let now = Utc::now();
        let config = Config::create_test_config(ConfigData {
            timeslots: vec![TimeslotConfig {
                days: "1-7".to_string(),
                start: "0000".to_string(),
                end: "2359".to_string(),
            }],
            timezone: Some("UTC".to_string()),
            ..ConfigData::default()
        });

        unsafe {
            env::remove_var("GITCLOCK");
            env::remove_var("GIT_COMMITTER_DATE");
        }

        let mut shell = MockGitShell::new();
        let future_date = (now + chrono::Duration::days(1)).to_rfc3339();
        shell.expect_spawn_async().returning(move |_, _, _, _| {
            Ok(SpawnResult {
                code: 0,
                stdout: future_date.clone(),
                stderr: "".to_string(),
            })
        });

        assert_eq!(pre_commit_hook_with_shell(now, &shell, &config), 1);
    }

    #[test]
    #[serial]
    fn succeeds_when_within_timeslot_and_no_gitclock_env() {
        let now = Utc::now();
        let config = Config::create_test_config(ConfigData {
            timeslots: vec![TimeslotConfig {
                days: "1-7".to_string(),
                start: "0000".to_string(),
                end: "2359".to_string(),
            }],
            timezone: Some("UTC".to_string()),
            ..ConfigData::default()
        });

        unsafe {
            env::remove_var("GITCLOCK");
        }
        let mut shell = MockGitShell::new();
        shell.expect_spawn_async().returning(move |_, _, _, _| {
            Ok(SpawnResult {
                code: 0,
                stdout: (now - chrono::Duration::days(1)).to_rfc3339(),
                stderr: "".to_string(),
            })
        });

        assert_eq!(pre_commit_hook_with_shell(now, &shell, &config), 0);
    }

    #[test]
    #[serial]
    fn fails_when_no_timeslots_even_with_gitclock_env() {
        let now = Utc::now();
        let config = Config::create_test_config(ConfigData::default());
        unsafe {
            env::set_var("GITCLOCK", "1");
        }
        let mut shell = MockGitShell::new();
        shell.expect_spawn_async().returning(move |_, _, _, _| {
            Ok(SpawnResult {
                code: 0,
                stdout: now.to_rfc3339(),
                stderr: "".to_string(),
            })
        });
        assert_eq!(pre_commit_hook_with_shell(now, &shell, &config), 1);
        unsafe {
            env::remove_var("GITCLOCK");
        }
    }

    #[test]
    #[serial]
    fn succeeds_with_gitclock_env_even_if_outside_timeslot_or_future_commit() {
        let now = Utc::now();
        let weekday = now.weekday().number_from_monday();
        let other_weekday = if weekday == 7 { 1 } else { weekday + 1 };

        let config = Config::create_test_config(ConfigData {
            timeslots: vec![TimeslotConfig {
                days: format!("{}-{}", other_weekday, other_weekday),
                start: "0900".to_string(),
                end: "1200".to_string(),
            }],
            timezone: Some("UTC".to_string()),
            ..ConfigData::default()
        });

        unsafe {
            env::set_var("GITCLOCK", "1");
        }
        let mut shell = MockGitShell::new();
        shell.expect_spawn_async().returning(move |_, _, _, _| {
            Ok(SpawnResult {
                code: 0,
                stdout: now.to_rfc3339(),
                stderr: "".to_string(),
            })
        });

        assert_eq!(pre_commit_hook_with_shell(now, &shell, &config), 0);
        unsafe {
            env::remove_var("GITCLOCK");
        }
    }
}
