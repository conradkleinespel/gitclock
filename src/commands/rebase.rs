use crate::config::Config;
use crate::git::{GitShell, RealGitShell, git_rebase_with_shell};
use chrono::{DateTime, Utc};

pub fn run_rebase_command(_now: DateTime<Utc>, args: &[String], config: &Config) -> i32 {
    rebase_with_shell(&RealGitShell, args, config)
}

pub fn rebase_with_shell(shell: &dyn GitShell, args: &[String], config: &Config) -> i32 {
    let timeslots = config.get_timeslots();
    if timeslots.is_empty() {
        println!("No timeslots found. Please add timeslots.");
        return 1;
    }

    let mut rebase_args = vec!["--committer-date-is-author-date".to_string()];
    rebase_args.extend_from_slice(args);

    git_rebase_with_shell(shell, &rebase_args)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, ConfigData, TimeslotConfig};
    use crate::git::MockGitShell;
    use crate::spawn_async::SpawnResult;

    #[test]
    fn returns_error_with_empty_timeslots_config() {
        let config = Config::create_test_config(ConfigData::default());
        let shell = MockGitShell::new();
        assert_eq!(rebase_with_shell(&shell, &[], &config), 1);
    }

    #[test]
    fn runs_rebase_when_config_is_valid() {
        let mut shell = MockGitShell::new();
        shell
            .expect_spawn_async()
            .with(
                mockall::predicate::eq("git"),
                mockall::predicate::eq(vec![
                    "rebase".to_string(),
                    "--committer-date-is-author-date".to_string(),
                    "--some".to_string(),
                    "option".to_string(),
                ]),
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
            timezone: Some("Africa/Nairobi".to_string()),
            ..ConfigData::default()
        });

        assert_eq!(
            rebase_with_shell(
                &shell,
                &["--some".to_string(), "option".to_string()],
                &config
            ),
            0
        );
    }
}
