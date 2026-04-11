use crate::config::Config;
use crate::git::{GitShell, RealGitShell, get_push_object_date_with_shell};
use chrono::{DateTime, Utc};
use std::env;
use std::io::{self, Read};

pub fn run_pre_push_hook_command(now: DateTime<Utc>, config: &Config) -> i32 {
    let mut stdin_input = String::new();
    io::stdin().read_to_string(&mut stdin_input).ok();
    pre_push_hook_with_shell(now, &RealGitShell, config, &stdin_input)
}

pub fn pre_push_hook_with_shell(
    now: DateTime<Utc>,
    shell: &dyn GitShell,
    config: &Config,
    stdin_input: &str,
) -> i32 {
    println!("Running gitclock pre-push-hook...");

    let timeslots = config.get_timeslots();
    if timeslots.is_empty() {
        eprintln!("Error: No timeslots found. Please add timeslots.");
        return 1;
    }

    let is_within_timeslot = timeslots.iter().any(|t| t.is_date_within(now));
    let gitclock_env = env::var("GITCLOCK").unwrap_or_default();

    if gitclock_env != "1" && !config.get_allow_push_outside_timeslot() && !is_within_timeslot {
        eprintln!("Error: Cannot push outside timeslot. This could cause CI to trigger.");
        return 1;
    }

    if stdin_input.trim().is_empty() {
        return 0;
    }

    let input_parts: Vec<&str> = stdin_input.split_whitespace().collect();
    if input_parts.len() < 4 {
        return 0;
    }

    let local_object_name = input_parts[1];
    match get_push_object_date_with_shell(shell, local_object_name) {
        Ok(local_object_date) => {
            if gitclock_env != "1" && local_object_date > now {
                eprintln!("Error: Trying to push commits that are in the future. Aborting.");
                return 1;
            }
        }
        Err(e) => {
            eprintln!("Error getting object date: {}", e);
            return 1;
        }
    }

    println!("Pre-push hook finished successfully.");
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
    fn fails_when_no_timeslots_at_all() {
        let config = Config::create_test_config(ConfigData::default());
        let shell = MockGitShell::new();
        let now = Utc::now();
        assert_eq!(pre_push_hook_with_shell(now, &shell, &config, ""), 1);
    }

    #[test]
    #[serial]
    fn fails_when_no_timeslots_match_now() {
        let now = Utc::now();
        let weekday = now.weekday().number_from_monday();
        let other_weekday = if weekday == 7 { 1 } else { weekday + 1 };

        let config = Config::create_test_config(ConfigData {
            timeslots: vec![TimeslotConfig {
                days: format!("{}-{}", other_weekday, other_weekday),
                start: "0900".to_string(),
                end: "1700".to_string(),
            }],
            timezone: Some("Africa/Nairobi".to_string()),
            allow_push_outside_timeslot: Some(false),
        });

        unsafe {
            env::remove_var("GITCLOCK");
        }
        let shell = MockGitShell::new();
        assert_eq!(pre_push_hook_with_shell(now, &shell, &config, ""), 1);
    }

    #[test]
    #[serial]
    fn succeeds_when_no_timeslots_match_but_allowed_to_push_outside() {
        let now = Utc::now();
        let config = Config::create_test_config(ConfigData {
            timeslots: vec![TimeslotConfig {
                days: "6-7".to_string(), // hopefully not now
                start: "0900".to_string(),
                end: "1700".to_string(),
            }],
            timezone: Some("Europe/Paris".to_string()),
            allow_push_outside_timeslot: Some(true),
        });

        let mut shell = MockGitShell::new();
        let stdin = format!(
            "{} {} refs/heads/master {}",
            "0".repeat(40),
            "1".repeat(40),
            "2".repeat(40)
        );
        let commit_date = now.to_rfc3339();
        shell.expect_spawn_async().returning(move |_, _, _, _| {
            Ok(SpawnResult {
                code: 0,
                stdout: commit_date.clone(),
                stderr: "".to_string(),
            })
        });

        assert_eq!(pre_push_hook_with_shell(now, &shell, &config, &stdin), 0);
    }

    #[test]
    #[serial]
    fn succeeds_when_no_commits_to_push() {
        let now = Utc::now();
        let config = Config::create_test_config(ConfigData {
            timeslots: vec![TimeslotConfig {
                days: "1-7".to_string(),
                start: "0000".to_string(),
                end: "2359".to_string(),
            }],
            timezone: Some("Europe/Paris".to_string()),
            allow_push_outside_timeslot: Some(true),
        });

        let shell = MockGitShell::new();
        assert_eq!(pre_push_hook_with_shell(now, &shell, &config, ""), 0);
    }

    #[test]
    #[serial]
    fn succeeds_when_commit_date_is_past() {
        let now = Utc::now();
        let config = Config::create_test_config(ConfigData {
            timeslots: vec![TimeslotConfig {
                days: "1-7".to_string(),
                start: "0000".to_string(),
                end: "2359".to_string(),
            }],
            timezone: Some("Africa/Nairobi".to_string()),
            allow_push_outside_timeslot: Some(false),
        });

        let mut shell = MockGitShell::new();
        let stdin = format!(
            "{} {} refs/heads/master {}",
            "0".repeat(40),
            "1".repeat(40),
            "2".repeat(40)
        );
        let past_date = (now - chrono::Duration::hours(1)).to_rfc3339();
        shell.expect_spawn_async().returning(move |_, _, _, _| {
            Ok(SpawnResult {
                code: 0,
                stdout: past_date.clone(),
                stderr: "".to_string(),
            })
        });

        assert_eq!(pre_push_hook_with_shell(now, &shell, &config, &stdin), 0);
    }

    #[test]
    #[serial]
    fn fails_when_commit_date_is_future() {
        let now = Utc::now();
        let config = Config::create_test_config(ConfigData {
            timeslots: vec![TimeslotConfig {
                days: "1-7".to_string(),
                start: "0000".to_string(),
                end: "2359".to_string(),
            }],
            timezone: Some("Africa/Nairobi".to_string()),
            allow_push_outside_timeslot: Some(false),
        });

        let mut shell = MockGitShell::new();
        let stdin = format!(
            "{} {} refs/heads/master {}",
            "0".repeat(40),
            "1".repeat(40),
            "2".repeat(40)
        );
        let future_date = (now + chrono::Duration::hours(1)).to_rfc3339();
        shell.expect_spawn_async().returning(move |_, _, _, _| {
            Ok(SpawnResult {
                code: 0,
                stdout: future_date.clone(),
                stderr: "".to_string(),
            })
        });

        unsafe {
            env::remove_var("GITCLOCK");
        }
        assert_eq!(pre_push_hook_with_shell(now, &shell, &config, &stdin), 1);
    }

    #[test]
    #[serial]
    fn fails_when_git_show_fails() {
        let now = Utc::now();
        let config = Config::create_test_config(ConfigData {
            timeslots: vec![TimeslotConfig {
                days: "1-7".to_string(),
                start: "0000".to_string(),
                end: "2359".to_string(),
            }],
            timezone: Some("Africa/Nairobi".to_string()),
            allow_push_outside_timeslot: Some(false),
        });

        let mut shell = MockGitShell::new();
        let stdin = format!(
            "{} {} refs/heads/master {}",
            "0".repeat(40),
            "1".repeat(40),
            "2".repeat(40)
        );
        shell.expect_spawn_async().returning(|_, _, _, _| {
            Err(crate::spawn_async::SpawnError {
                code: Some(128),
                stdout: "".to_string(),
                stderr: "fatal: ambiguous argument".to_string(),
                message: "".to_string(),
            })
        });

        assert_eq!(pre_push_hook_with_shell(now, &shell, &config, &stdin), 1);
    }
}
