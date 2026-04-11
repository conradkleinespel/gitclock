use crate::config::Config;

pub fn run_info_command(config: &Config) -> i32 {
    match config.get_file_path() {
        Ok(path) => {
            println!("Config file: {}", path.display());
            println!("Timezone: {}", config.get_timezone());
            println!(
                "Allow push outside timeslot: {}",
                config.get_allow_push_outside_timeslot()
            );
            if config.data.timeslots.is_empty() {
                println!("Timeslots: None, add a timeslot with `gitclock timeslot --add`");
            } else {
                println!("Timeslots:");
                for timeslot in &config.data.timeslots {
                    println!(
                        "  - Days: {}, Start: {}, End: {}",
                        timeslot.days, timeslot.start, timeslot.end
                    );
                }
            }
            0
        }
        Err(e) => {
            eprintln!("Error getting config path: {}", e);
            1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, ConfigData};

    #[test]
    fn test_info_returns_0() {
        use crate::config::TimeslotConfig;
        let config = Config::create_test_config(ConfigData {
            timeslots: vec![TimeslotConfig {
                days: "1-5".to_string(),
                start: "0900".to_string(),
                end: "1700".to_string(),
            }],
            allow_push_outside_timeslot: Some(true),
            timezone: Some("UTC".to_string()),
        });
        assert_eq!(run_info_command(&config), 0);
    }
}
