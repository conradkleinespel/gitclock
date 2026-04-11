pub use crate::spawn_async::{SpawnError, SpawnResult, spawn_async};
use chrono::{DateTime, Utc};
use chrono_tz::Tz;
use std::collections::HashMap;

#[cfg(test)]
use mockall::{automock, predicate::*};

#[cfg_attr(test, automock)]
pub trait GitShell {
    fn spawn_async(
        &self,
        binary: &str,
        args: &[String],
        inherit_stdio: bool,
        env: HashMap<String, String>,
    ) -> Result<SpawnResult, SpawnError>;
}

pub struct RealGitShell;

impl GitShell for RealGitShell {
    fn spawn_async(
        &self,
        binary: &str,
        args: &[String],
        inherit_stdio: bool,
        env: HashMap<String, String>,
    ) -> Result<SpawnResult, SpawnError> {
        spawn_async(binary, args, inherit_stdio, env)
    }
}

pub fn format_date_to_git_date(date: DateTime<Utc>, timezone: &str) -> String {
    let tz: Tz = timezone.parse().unwrap();
    let date_in_tz = date.with_timezone(&tz);
    date_in_tz.format("%s %z").to_string()
}

pub fn git_commit_with_shell(
    shell: &dyn GitShell,
    date: DateTime<Utc>,
    timezone: &str,
    args: &[String],
) -> i32 {
    let git_date = format_date_to_git_date(date, timezone);
    let mut env = HashMap::new();
    env.insert("GIT_AUTHOR_DATE".to_string(), git_date.clone());
    env.insert("GIT_COMMITTER_DATE".to_string(), git_date.clone());

    let mut git_args = vec!["commit".to_string(), "--date".to_string(), git_date];
    git_args.extend_from_slice(args);

    run_command_with_args(shell, "git", &git_args, env)
}

pub fn get_last_commit_date_with_shell(
    shell: &dyn GitShell,
    now: DateTime<Utc>,
) -> anyhow::Result<DateTime<Utc>> {
    match shell.spawn_async(
        "git",
        &[
            "log".to_string(),
            "-1".to_string(),
            "--format=%cI".to_string(),
        ],
        false,
        HashMap::new(),
    ) {
        Ok(result) => {
            let stdout = result.stdout.trim();
            if stdout.is_empty() {
                return Ok(now);
            }
            match DateTime::parse_from_rfc3339(stdout) {
                Ok(dt) => Ok(dt.with_timezone(&Utc)),
                Err(e) => Err(anyhow::anyhow!(
                    "Failed to parse last commit date: {}. Output was: '{}'",
                    e,
                    stdout
                )),
            }
        }
        Err(e) => {
            if e.stderr.contains("does not have any commits yet") {
                // git returns this error when there are no commits starting version 2.22
                Ok(now)
            } else if e.stderr.contains("bad default revision 'HEAD'") {
                // git returns this error when there are no commits before version 2.22
                Ok(now)
            } else {
                Err(anyhow::anyhow!("Git error: {}", e.stderr))
            }
        }
    }
}

pub struct TrackingRemote {
    pub remote: String,
    pub branch: String,
}

pub fn get_tracking_remote_and_branch_with_shell(
    shell: &dyn GitShell,
) -> anyhow::Result<TrackingRemote> {
    match shell.spawn_async(
        "git",
        &[
            "rev-parse".to_string(),
            "--abbrev-ref".to_string(),
            "@{push}".to_string(),
        ],
        false,
        HashMap::new(),
    ) {
        Ok(result) => {
            let parts: Vec<&str> = result.stdout.trim().split('/').collect();
            if parts.len() >= 2 {
                Ok(TrackingRemote {
                    remote: parts[0].to_string(),
                    branch: parts[1..].join("/"),
                })
            } else {
                Err(anyhow::anyhow!(
                    "Unexpected output format: {}",
                    result.stdout
                ))
            }
        }
        Err(e) => Err(anyhow::anyhow!("Git error: {}", e.stderr)),
    }
}

pub struct PastCommit {
    pub commit_hash: String,
}

pub fn get_first_past_commit_hash_with_shell(
    shell: &dyn GitShell,
    now: DateTime<Utc>,
) -> anyhow::Result<PastCommit> {
    let until_arg = format!("--until=\"{}\"", now.to_rfc3339());
    match shell.spawn_async(
        "git",
        &[
            "log".to_string(),
            until_arg,
            "--pretty=format:%H".to_string(),
            "-1".to_string(),
        ],
        false,
        HashMap::new(),
    ) {
        Ok(result) => Ok(PastCommit {
            commit_hash: result.stdout.trim().to_string(),
        }),
        Err(e) => Err(anyhow::anyhow!("Git error: {}", e.stderr)),
    }
}

pub fn get_push_object_date_with_shell(
    shell: &dyn GitShell,
    object_name: &str,
) -> anyhow::Result<DateTime<Utc>> {
    let result = shell
        .spawn_async(
            "git",
            &[
                "show".to_string(),
                "-s".to_string(),
                "--format=%cI".to_string(),
                object_name.to_string(),
            ],
            false,
            HashMap::new(),
        )
        .map_err(|e| anyhow::anyhow!("Git error: {}", e.stderr))?;
    let datetime = DateTime::parse_from_rfc3339(result.stdout.trim())?.with_timezone(&Utc);
    Ok(datetime)
}

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub sha: String,
    pub author_date: DateTime<Utc>,
    pub commit_date: DateTime<Utc>,
}

pub fn get_log_sha_and_dates_with_shell(shell: &dyn GitShell) -> anyhow::Result<Vec<LogEntry>> {
    let result = shell
        .spawn_async(
            "git",
            &[
                "log".to_string(),
                "--pretty=format:%H %aI %cI".to_string(),
                "--reverse".to_string(),
            ],
            false,
            HashMap::new(),
        )
        .map_err(|e| anyhow::anyhow!("Git error: {}", e.stderr))?;

    let mut entries = Vec::new();
    for line in result.stdout.trim().lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() == 3 {
            entries.push(LogEntry {
                sha: parts[0].to_string(),
                author_date: DateTime::parse_from_rfc3339(parts[1])?.with_timezone(&Utc),
                commit_date: DateTime::parse_from_rfc3339(parts[2])?.with_timezone(&Utc),
            });
        }
    }
    Ok(entries)
}

pub fn cherry_pick_with_shell(shell: &dyn GitShell, sha: &str) -> i32 {
    run_command_with_args(
        shell,
        "git",
        &["cherry-pick".to_string(), sha.to_string()],
        HashMap::new(),
    )
}

pub fn reset_hard_with_shell(shell: &dyn GitShell, sha: &str) -> i32 {
    run_command_with_args(
        shell,
        "git",
        &["reset".to_string(), "--hard".to_string(), sha.to_string()],
        HashMap::new(),
    )
}

pub fn amend_with_new_date_with_shell(
    shell: &dyn GitShell,
    new_date: DateTime<Utc>,
    timezone: &str,
) -> i32 {
    let git_date = format_date_to_git_date(new_date, timezone);
    let mut env = HashMap::new();
    env.insert("GIT_AUTHOR_DATE".to_string(), git_date.clone());
    env.insert("GIT_COMMITTER_DATE".to_string(), git_date.clone());

    run_command_with_args(
        shell,
        "git",
        &[
            "commit".to_string(),
            "--amend".to_string(),
            "--no-edit".to_string(),
            "--date".to_string(),
            git_date,
        ],
        env,
    )
}

pub fn git_push_with_shell(shell: &dyn GitShell, args: &[String]) -> i32 {
    let mut git_args = vec!["push".to_string()];
    git_args.extend_from_slice(args);
    run_command_with_args(shell, "git", &git_args, HashMap::new())
}

pub fn git_rebase_with_shell(shell: &dyn GitShell, args: &[String]) -> i32 {
    let mut git_args = vec!["rebase".to_string()];
    git_args.extend_from_slice(args);
    run_command_with_args(shell, "git", &git_args, HashMap::new())
}

fn run_command_with_args(
    shell: &dyn GitShell,
    binary: &str,
    args: &[String],
    env: HashMap<String, String>,
) -> i32 {
    match shell.spawn_async(binary, args, true, env) {
        Ok(res) => res.code,
        Err(e) => e.code.unwrap_or(1),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_format_date_to_git_date() {
        let date = Utc.with_ymd_and_hms(2023, 7, 4, 8, 0, 0).unwrap(); // 10:00+02:00 is 08:00Z
        let formatted = format_date_to_git_date(date, "Africa/Nairobi"); // UTC+3
        // 2023-07-04 08:00:00 UTC is 2023-07-04 11:00:00 +0300
        // Timestamp should be 1688457600
        assert_eq!(formatted, "1688457600 +0300");
    }

    #[test]
    fn test_log_entry_data_is_stored_correctly() {
        let author_date = Utc.with_ymd_and_hms(2023, 7, 1, 10, 0, 0).unwrap();
        let commit_date = Utc.with_ymd_and_hms(2023, 7, 2, 10, 0, 0).unwrap();
        let entry = LogEntry {
            sha: "abcd".to_string(),
            author_date,
            commit_date,
        };

        assert_eq!(entry.sha, "abcd");
        assert_eq!(entry.author_date, author_date);
        assert_eq!(entry.commit_date, commit_date);
    }

    #[test]
    fn test_get_push_object_date_returns_date() {
        let mut mock = MockGitShell::new();
        let date_string = "2023-07-04T10:00:00.000+02:00";
        mock.expect_spawn_async().returning(move |_, _, _, _| {
            Ok(SpawnResult {
                code: 0,
                stdout: format!("{}\n", date_string),
                stderr: "".to_string(),
            })
        });

        let result = get_push_object_date_with_shell(&mock, "HEAD").unwrap();
        let expected = DateTime::parse_from_rfc3339(date_string)
            .unwrap()
            .with_timezone(&Utc);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_get_push_object_date_throws_error() {
        let mut mock = MockGitShell::new();
        mock.expect_spawn_async().returning(|_, _, _, _| {
            Err(SpawnError {
                code: Some(1),
                stdout: "".to_string(),
                stderr: "some git error".to_string(),
                message: "error".to_string(),
            })
        });

        let result = get_push_object_date_with_shell(&mock, "HEAD");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Git error: some git error");
    }

    #[test]
    fn test_get_last_commit_date_returns_date() {
        let mut mock = MockGitShell::new();
        let now = Utc::now();
        let date_string = "2023-07-04T10:17:00.000+02:00";
        mock.expect_spawn_async().returning(move |_, _, _, _| {
            Ok(SpawnResult {
                code: 0,
                stdout: date_string.to_string(),
                stderr: "".to_string(),
            })
        });

        let result = get_last_commit_date_with_shell(&mock, now).unwrap();
        let expected = DateTime::parse_from_rfc3339(date_string)
            .unwrap()
            .with_timezone(&Utc);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_get_last_commit_date_returns_now_on_no_commits() {
        let mut mock = MockGitShell::new();
        let now = Utc.with_ymd_and_hms(2023, 7, 4, 10, 17, 0).unwrap();
        mock.expect_spawn_async().returning(|_, _, _, _| {
            Err(SpawnError {
                code: Some(128),
                stdout: "".to_string(),
                stderr: "fatal: your current branch 'main' does not have any commits yet"
                    .to_string(),
                message: "error".to_string(),
            })
        });

        let result = get_last_commit_date_with_shell(&mock, now).unwrap();
        assert_eq!(result, now);
    }

    #[test]
    fn test_get_last_commit_date_returns_error_on_other_git_error() {
        let mut mock = MockGitShell::new();
        let now = Utc::now();
        mock.expect_spawn_async().returning(|_, _, _, _| {
            Err(SpawnError {
                code: Some(1),
                stdout: "".to_string(),
                stderr: "some other git error".to_string(),
                message: "error".to_string(),
            })
        });

        let result = get_last_commit_date_with_shell(&mock, now);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Git error: some other git error"
        );
    }

    #[test]
    fn test_get_tracking_remote_and_branch_returns_error() {
        let mut mock = MockGitShell::new();
        mock.expect_spawn_async().returning(|_, _, _, _| {
            Err(SpawnError {
                code: Some(1),
                stdout: "".to_string(),
                stderr: "some git error".to_string(),
                message: "error".to_string(),
            })
        });

        assert_eq!(
            get_tracking_remote_and_branch_with_shell(&mock)
                .err()
                .unwrap()
                .to_string(),
            "Git error: some git error".to_string()
        );
    }

    #[test]
    fn test_get_tracking_remote_and_branch_returns_remote_and_branch() {
        let mut mock = MockGitShell::new();
        mock.expect_spawn_async().returning(|_, _, _, _| {
            Ok(SpawnResult {
                code: 0,
                stdout: "origin/master".to_string(),
                stderr: "".to_string(),
            })
        });

        let result = get_tracking_remote_and_branch_with_shell(&mock).unwrap();
        assert_eq!(result.remote, "origin".to_string());
        assert_eq!(result.branch, "master".to_string());
    }

    #[test]
    fn test_get_first_past_commit_hash_returns_error() {
        let mut mock = MockGitShell::new();
        let now = Utc::now();
        mock.expect_spawn_async().returning(|_, _, _, _| {
            Err(SpawnError {
                code: Some(1),
                stdout: "".to_string(),
                stderr: "some git error".to_string(),
                message: "error".to_string(),
            })
        });

        assert_eq!(
            get_first_past_commit_hash_with_shell(&mock, now)
                .err()
                .unwrap()
                .to_string(),
            "Git error: some git error".to_string()
        );
    }

    #[test]
    fn test_get_first_past_commit_hash_returns_hash() {
        let mut mock = MockGitShell::new();
        let now = Utc::now();
        let hash = "a".repeat(40);
        let hash_clone = hash.clone();
        mock.expect_spawn_async().returning(move |_, _, _, _| {
            Ok(SpawnResult {
                code: 0,
                stdout: format!("{}\n", hash_clone),
                stderr: "".to_string(),
            })
        });

        let result = get_first_past_commit_hash_with_shell(&mock, now).unwrap();
        assert_eq!(result.commit_hash, hash);
    }

    #[test]
    fn test_get_log_sha_and_dates_returns_entries() {
        let mut mock = MockGitShell::new();
        let hash_a = "a".repeat(40);
        let hash_b = "b".repeat(40);
        let output = format!(
            "{} 2023-07-01T10:00:00.000Z 2023-07-02T10:00:00.000Z\n{} 2023-07-03T10:00:00.000Z 2023-07-04T10:00:00.000Z",
            hash_a, hash_b
        );
        mock.expect_spawn_async().returning(move |_, _, _, _| {
            Ok(SpawnResult {
                code: 0,
                stdout: output.clone(),
                stderr: "".to_string(),
            })
        });

        use chrono::Datelike;
        let entries = get_log_sha_and_dates_with_shell(&mock).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].sha, hash_a);
        assert_eq!(entries[1].sha, hash_b);
        assert_eq!(entries[0].author_date.day(), 1);
        assert_eq!(entries[1].author_date.day(), 3);
    }

    #[test]
    fn test_cherry_pick_calls_git() {
        let mut mock = MockGitShell::new();
        let hash = "a".repeat(40);
        let hash_clone = hash.clone();
        mock.expect_spawn_async()
            .with(
                eq("git"),
                eq(vec!["cherry-pick".to_string(), hash_clone]),
                eq(true),
                eq(HashMap::new()),
            )
            .returning(|_, _, _, _| {
                Ok(SpawnResult {
                    code: 0,
                    stdout: "".to_string(),
                    stderr: "".to_string(),
                })
            });

        let result = cherry_pick_with_shell(&mock, &hash);
        assert_eq!(result, 0);
    }

    #[test]
    fn test_reset_hard_calls_git() {
        let mut mock = MockGitShell::new();
        let hash = "a".repeat(40);
        let hash_clone = hash.clone();
        mock.expect_spawn_async()
            .with(
                eq("git"),
                eq(vec!["reset".to_string(), "--hard".to_string(), hash_clone]),
                eq(true),
                eq(HashMap::new()),
            )
            .returning(|_, _, _, _| {
                Ok(SpawnResult {
                    code: 0,
                    stdout: "".to_string(),
                    stderr: "".to_string(),
                })
            });

        let result = reset_hard_with_shell(&mock, &hash);
        assert_eq!(result, 0);
    }

    #[test]
    fn test_amend_with_new_date_calls_git() {
        let mut mock = MockGitShell::new();
        let date = Utc.with_ymd_and_hms(2023, 8, 4, 10, 0, 0).unwrap();
        // 1691143200 is 2023-08-04 10:00:00Z
        // TZ is Africa/Nairobi -> +0300
        // 1691143200 +0300
        mock.expect_spawn_async().returning(|_, _, _, _| {
            Ok(SpawnResult {
                code: 0,
                stdout: "".to_string(),
                stderr: "".to_string(),
            })
        });

        let result = amend_with_new_date_with_shell(&mock, date, "Africa/Nairobi");
        assert_eq!(result, 0);
    }

    #[test]
    fn test_git_commit_returns_code() {
        let mut mock = MockGitShell::new();
        mock.expect_spawn_async().returning(|_, _, _, _| {
            Ok(SpawnResult {
                code: 42,
                stdout: "".to_string(),
                stderr: "".to_string(),
            })
        });

        let result = git_commit_with_shell(
            &mock,
            Utc::now(),
            "UTC",
            &["-m".to_string(), "msg".to_string()],
        );
        assert_eq!(result, 42);
    }

    #[test]
    fn test_git_push_calls_git() {
        let mut mock = MockGitShell::new();
        mock.expect_spawn_async()
            .with(
                eq("git"),
                eq(vec!["push".to_string(), "--force".to_string()]),
                eq(true),
                eq(HashMap::new()),
            )
            .returning(|_, _, _, _| {
                Ok(SpawnResult {
                    code: 0,
                    stdout: "".to_string(),
                    stderr: "".to_string(),
                })
            });

        let result = git_push_with_shell(&mock, &["--force".to_string()]);
        assert_eq!(result, 0);
    }

    #[test]
    fn test_git_rebase_calls_git() {
        let mut mock = MockGitShell::new();
        mock.expect_spawn_async()
            .with(
                eq("git"),
                eq(vec!["rebase".to_string(), "master".to_string()]),
                eq(true),
                eq(HashMap::new()),
            )
            .returning(|_, _, _, _| {
                Ok(SpawnResult {
                    code: 0,
                    stdout: "".to_string(),
                    stderr: "".to_string(),
                })
            });

        let result = git_rebase_with_shell(&mock, &["master".to_string()]);
        assert_eq!(result, 0);
    }
}
