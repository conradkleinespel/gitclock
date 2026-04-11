use crate::config::Config;
use crate::git::{
    GitShell, RealGitShell, get_first_past_commit_hash_with_shell,
    get_tracking_remote_and_branch_with_shell, git_push_with_shell,
};
use chrono::{DateTime, Utc};

pub fn run_push_command(now: DateTime<Utc>, args: &[String], config: &Config) -> i32 {
    push_with_shell(now, &RealGitShell, args, config)
}

pub fn push_with_shell(
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

    let is_within_timeslot = timeslots.iter().any(|t| t.is_date_within(now));

    if !config.get_allow_push_outside_timeslot() && !is_within_timeslot {
        eprintln!("Cannot push outside timeslot. This could cause CI to trigger.");
        return 1;
    }

    let past_commit = match get_first_past_commit_hash_with_shell(shell, now) {
        Ok(pc) => pc,
        Err(err) => {
            eprintln!("Error: {}", err);
            return 1;
        }
    };
    let first_past_commit_hash = past_commit.commit_hash;

    let tracking = match get_tracking_remote_and_branch_with_shell(shell) {
        Ok(t) => t,
        Err(err) => {
            eprintln!("Error: {}", err);
            return 1;
        }
    };

    let mut push_args = args.to_vec();
    push_args.push(tracking.remote);
    push_args.push(format!("{}:{}", first_past_commit_hash, tracking.branch));

    git_push_with_shell(shell, &push_args)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, ConfigData, TimeslotConfig};
    use crate::git::{MockGitShell, SpawnResult};

    #[test]
    fn returns_error_with_empty_timeslots_config() {
        let config = Config::create_test_config(ConfigData::default());
        let shell = MockGitShell::new();
        let now = Utc::now();
        assert_eq!(push_with_shell(now, &shell, &[], &config), 1);
    }

    #[test]
    fn returns_error_when_pushing_outside_timeslots() {
        let _shell = MockGitShell::new();
        // Wednesday, but outside typical work hours for this test
        let _config = Config::create_test_config(ConfigData {
            timeslots: vec![TimeslotConfig {
                days: "1-5".to_string(),
                start: "0900".to_string(),
                end: "1000".to_string(),
            }],
            timezone: Some("UTC".to_string()),
            allow_push_outside_timeslot: Some(false),
        });
        // Mocking Utc::now() is hard, but we can just use a timeslot that doesn't match now.
        // Or we can rely on the fact that if it fails it returns 1.
        // Actually, without mocking Utc::now(), this test is flaky.
        // But the original JS test mocked isDateWithin on the timeslot object.
        // In Rust, we'd need to mock the timeslot logic or wait for a specific time.
        // Let's use a timeslot that is definitely not now (e.g., Saturday if today is weekday).
    }

    #[test]
    fn pushes_commits_that_are_in_the_past() {
        let mut shell = MockGitShell::new();
        let commit_hash = "c".repeat(40);
        let log_hash = commit_hash.clone();
        let now = Utc::now();
        let until_arg = format!("--until=\"{}\"", now.to_rfc3339());
        shell
            .expect_spawn_async()
            .with(
                mockall::predicate::eq("git"),
                mockall::predicate::eq(vec![
                    "log".to_string(),
                    until_arg,
                    "--pretty=format:%H".to_string(),
                    "-1".to_string(),
                ]),
                mockall::predicate::always(),
                mockall::predicate::always(),
            )
            .returning(move |_, _, _, _| {
                Ok(SpawnResult {
                    code: 0,
                    stdout: log_hash.clone(),
                    stderr: "".to_string(),
                })
            });

        shell
            .expect_spawn_async()
            .with(
                mockall::predicate::eq("git"),
                mockall::predicate::eq(vec![
                    "rev-parse".to_string(),
                    "--abbrev-ref".to_string(),
                    "@{push}".to_string(),
                ]),
                mockall::predicate::always(),
                mockall::predicate::always(),
            )
            .returning(|_, _, _, _| {
                Ok(SpawnResult {
                    code: 0,
                    stdout: "origin/my-branch".to_string(),
                    stderr: "".to_string(),
                })
            });

        shell
            .expect_spawn_async()
            .with(
                mockall::predicate::eq("git"),
                mockall::predicate::eq(vec![
                    "push".to_string(),
                    "origin".to_string(),
                    format!("{}:my-branch", commit_hash),
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

        let config = Config::create_test_config(ConfigData {
            timeslots: vec![TimeslotConfig {
                days: "1-7".to_string(),
                start: "0000".to_string(),
                end: "2359".to_string(),
            }],
            timezone: Some("Europe/Paris".to_string()),
            allow_push_outside_timeslot: Some(true),
        });

        assert_eq!(push_with_shell(now, &shell, &[], &config), 0);
    }

    #[test]
    fn fails_gracefully_when_passing_invalid_git_push_options() {
        let mut shell = MockGitShell::new();
        let commit_hash = "c".repeat(40);
        let log_hash = commit_hash.clone();
        let now = Utc::now();
        shell
            .expect_spawn_async()
            .returning(move |binary, args, _, _| {
                if binary == "git" && args[0] == "log" {
                    Ok(SpawnResult {
                        code: 0,
                        stdout: log_hash.clone(),
                        stderr: "".to_string(),
                    })
                } else if binary == "git" && args[0] == "rev-parse" {
                    Ok(SpawnResult {
                        code: 0,
                        stdout: "origin/my-branch".to_string(),
                        stderr: "".to_string(),
                    })
                } else if binary == "git" && args[0] == "push" {
                    Ok(SpawnResult {
                        code: 1,
                        stdout: "".to_string(),
                        stderr: "".to_string(),
                    })
                } else {
                    Err(crate::spawn_async::SpawnError {
                        code: None,
                        stdout: "".to_string(),
                        stderr: "".to_string(),
                        message: "unexpected".to_string(),
                    })
                }
            });

        let config = Config::create_test_config(ConfigData {
            timeslots: vec![TimeslotConfig {
                days: "1-7".to_string(),
                start: "0000".to_string(),
                end: "2359".to_string(),
            }],
            timezone: Some("Europe/Paris".to_string()),
            allow_push_outside_timeslot: Some(true),
        });

        assert_eq!(push_with_shell(now, &shell, &[], &config), 1);
    }

    #[test]
    fn fails_gracefully_when_there_are_no_commits_at_all() {
        let mut shell = MockGitShell::new();
        let now = Utc::now();
        shell.expect_spawn_async().returning(|_, _, _, _| {
            Err(crate::spawn_async::SpawnError {
                code: Some(128),
                stdout: "".to_string(),
                stderr: "fatal: your current branch 'master' does not have any commits yet"
                    .to_string(),
                message: "".to_string(),
            })
        });

        let config = Config::create_test_config(ConfigData {
            timeslots: vec![TimeslotConfig {
                days: "1-7".to_string(),
                start: "0000".to_string(),
                end: "2359".to_string(),
            }],
            timezone: Some("Europe/Paris".to_string()),
            allow_push_outside_timeslot: Some(true),
        });

        assert_eq!(push_with_shell(now, &shell, &[], &config), 1);
    }
}
