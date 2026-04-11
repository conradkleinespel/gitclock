use crate::config::Config;

pub struct TimeslotOptions {
    pub add: bool,
    pub list: bool,
    pub days: Option<String>,
    pub start: Option<String>,
    pub end: Option<String>,
}

pub fn run_timeslot_command(options: TimeslotOptions, config: &mut Config) -> i32 {
    if options.add && options.list {
        eprintln!("Error: Options --add and --list are incompatible.");
        return 1;
    }

    if options.add {
        let (days, start, end) = match (options.days, options.start, options.end) {
            (Some(d), Some(s), Some(e)) => (d, s, e),
            _ => {
                eprintln!("Error: Need --days, --start and --end.");
                return 1;
            }
        };
        return timeslot_add(days, start, end, config);
    }
    if options.list {
        return timeslot_list(config);
    }

    eprintln!("Error: Need one of --add or --list.");
    1
}

fn timeslot_add(days: String, start: String, end: String, config: &mut Config) -> i32 {
    if let Err(err) = config.add_timeslot(days, start, end) {
        eprintln!("Error: {}", err);
        return 1;
    }

    println!("Timeslot added.");
    println!();
    println!("To remove a timeslot, edit:");
    match config.get_file_path() {
        Ok(path) => println!("  {}", path.display()),
        Err(e) => eprintln!("  Error getting config path: {}", e),
    }
    0
}

fn timeslot_list(config: &Config) -> i32 {
    let timeslots = config.get_timeslots();
    if timeslots.is_empty() {
        println!("No timeslots.");
        return 0;
    }

    println!("Current timeslots:");
    for t in timeslots {
        println!("{}", t);
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, ConfigData, TimeslotConfig};

    #[test]
    fn timeslot_add_list_incompatible() {
        let mut config = Config::create_test_config(ConfigData::default());
        let options = TimeslotOptions {
            add: true,
            list: true,
            days: None,
            start: None,
            end: None,
        };
        assert_eq!(run_timeslot_command(options, &mut config), 1);
    }

    #[test]
    fn timeslot_add_missing_args() {
        let mut config = Config::create_test_config(ConfigData::default());
        let options = TimeslotOptions {
            add: true,
            list: false,
            days: Some("1-5".to_string()),
            start: None,
            end: None,
        };
        assert_eq!(run_timeslot_command(options, &mut config), 1);
    }

    #[test]
    fn timeslot_add_success() {
        let mut config = Config::create_test_config(ConfigData::default());
        let options = TimeslotOptions {
            add: true,
            list: false,
            days: Some("1-5".to_string()),
            start: Some("0900".to_string()),
            end: Some("1700".to_string()),
        };
        // It might fail on save() if confy has no home dir in CI, but add_timeslot should still work in-memory
        let _ = run_timeslot_command(options, &mut config);
        assert_eq!(config.data.timeslots.len(), 1);
    }

    #[test]
    fn timeslot_list_works() {
        let mut config = Config::create_test_config(ConfigData {
            timeslots: vec![TimeslotConfig {
                days: "1-5".to_string(),
                start: "0900".to_string(),
                end: "1700".to_string(),
            }],
            ..ConfigData::default()
        });
        let options = TimeslotOptions {
            add: false,
            list: true,
            days: None,
            start: None,
            end: None,
        };
        assert_eq!(run_timeslot_command(options, &mut config), 0);
    }

    #[test]
    fn timeslot_list_no_timeslots() {
        let mut config = Config::create_test_config(ConfigData::default());
        let options = TimeslotOptions {
            add: false,
            list: true,
            days: None,
            start: None,
            end: None,
        };
        assert_eq!(run_timeslot_command(options, &mut config), 0);
    }

    #[test]
    fn timeslot_need_one_of_add_or_list() {
        let mut config = Config::create_test_config(ConfigData::default());
        let options = TimeslotOptions {
            add: false,
            list: false,
            days: None,
            start: None,
            end: None,
        };
        assert_eq!(run_timeslot_command(options, &mut config), 1);
    }
}
