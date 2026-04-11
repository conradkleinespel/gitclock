use crate::config::Config;
use chrono::{DateTime, Utc};
use std::env;

pub fn run_pre_rebase_hook_command(now: DateTime<Utc>, config: &Config) -> i32 {
    println!("Running gitclock pre-rebase-hook...");

    let timeslots = config.get_timeslots();
    if timeslots.is_empty() {
        eprintln!("No timeslots found. Please add timeslots.");
        return 1;
    }

    let is_within_timeslot = timeslots.iter().any(|t| t.is_date_within(now));
    let gitclock_env = env::var("GITCLOCK").unwrap_or_default();

    if gitclock_env != "1" && !is_within_timeslot {
        eprintln!("Cannot rebase outside timeslot. Use `gitclock rebase`.");
        return 1;
    }

    println!("Pre-rebase hook finished successfully.");
    0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, ConfigData, TimeslotConfig};
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
        assert_eq!(run_pre_rebase_hook_command(now, &config), 1);
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
        assert_eq!(run_pre_rebase_hook_command(now, &config), 0);
    }

    #[test]
    #[serial]
    fn fails_when_no_timeslots_even_with_gitclock_env() {
        let now = Utc::now();
        let config = Config::create_test_config(ConfigData::default());
        unsafe {
            env::set_var("GITCLOCK", "1");
        }
        assert_eq!(run_pre_rebase_hook_command(now, &config), 1);
        unsafe {
            env::remove_var("GITCLOCK");
        }
    }

    #[test]
    #[serial]
    fn succeeds_with_gitclock_env_even_if_outside_timeslot() {
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
        assert_eq!(run_pre_rebase_hook_command(now, &config), 0);
        unsafe {
            env::remove_var("GITCLOCK");
        }
    }
}
