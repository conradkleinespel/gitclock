use std::collections::HashMap;
use std::io::Read;
use std::process::{Command, Stdio};

#[derive(Debug)]
pub struct SpawnError {
    pub code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub message: String,
}

impl std::fmt::Display for SpawnError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for SpawnError {}

pub struct SpawnResult {
    pub code: i32,
    pub stdout: String,
    pub stderr: String,
}

pub fn spawn_async(
    binary: &str,
    args: &[String],
    inherit_stdio: bool,
    env: HashMap<String, String>,
) -> Result<SpawnResult, SpawnError> {
    let mut child = Command::new(binary)
        .args(args)
        .env("GITCLOCK", "1")
        .envs(env)
        .stdout(if inherit_stdio {
            Stdio::inherit()
        } else {
            Stdio::piped()
        })
        .stderr(if inherit_stdio {
            Stdio::inherit()
        } else {
            Stdio::piped()
        })
        .spawn()
        .map_err(|e| SpawnError {
            code: None,
            stdout: "".to_string(),
            stderr: "".to_string(),
            message: e.to_string(),
        })?;

    let mut stdout = String::new();
    let mut stderr = String::new();

    if !inherit_stdio {
        if let Some(mut out) = child.stdout.take() {
            out.read_to_string(&mut stdout).ok();
        }
        if let Some(mut err) = child.stderr.take() {
            err.read_to_string(&mut stderr).ok();
        }
    }

    let status = child.wait().map_err(|e| SpawnError {
        code: None,
        stdout: stdout.clone(),
        stderr: stderr.clone(),
        message: e.to_string(),
    })?;

    if status.success() {
        Ok(SpawnResult {
            code: status.code().unwrap_or(0),
            stdout,
            stderr,
        })
    } else {
        Err(SpawnError {
            code: status.code(),
            stdout,
            stderr,
            message: format!("Process exited with code {:?}", status.code()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn throws_spawn_error_when_exit_code_is_not_0() {
        let result = spawn_async("ls", &["invalid".to_string()], false, HashMap::new());
        assert!(result.is_err());
        let err = result.err().unwrap();
        assert!(err.code.unwrap_or(0) > 0);
        assert!(err.stdout.is_empty());
        assert!(!err.stderr.is_empty());
    }

    #[test]
    fn returns_code_and_output_when_subprocess_succeeds() {
        let result = spawn_async("ls", &["/".to_string()], false, HashMap::new());
        assert!(result.is_ok());
        let res = result.unwrap();
        assert_eq!(res.code, 0);
        assert!(!res.stdout.is_empty());
        assert!(res.stderr.is_empty());
    }

    #[test]
    fn send_gitclock_environment_variable() {
        let result = spawn_async("env", &[], false, HashMap::new());
        assert!(result.is_ok());
        let res = result.unwrap();
        assert_eq!(res.code, 0);
        assert!(res.stdout.contains("GITCLOCK=1"));
        assert!(res.stderr.is_empty());
    }

    #[test]
    fn returns_code_without_output_when_subprocess_succeeds_with_inherit_stdio() {
        // In Rust version, inherit_stdio: true means we don't capture output
        let result = spawn_async("ls", &["/".to_string()], true, HashMap::new());
        assert!(result.is_ok());
        let res = result.unwrap();
        assert_eq!(res.code, 0);
        assert!(res.stdout.is_empty());
        assert!(res.stderr.is_empty());
    }
}
