use crate::config::Config;

pub struct ConfigOptions {
    pub edit: bool,
    pub allow_push_outside_timeslot: Option<bool>,
    pub timezone: Option<String>,
}

pub fn run_config_command(config: &mut Config, options: ConfigOptions) -> i32 {
    if options.edit {
        let path = match config.get_file_path() {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Error getting config path: {}", e);
                return 1;
            }
        };

        let editor = std::env::var("EDITOR").unwrap_or_else(|_| {
            if cfg!(windows) {
                "notepad".to_string()
            } else {
                "vi".to_string()
            }
        });

        return match std::process::Command::new(editor).arg(path).status() {
            Ok(status) if status.success() => 0,
            Ok(status) => {
                eprintln!("Editor exited with error status: {}", status);
                1
            }
            Err(e) => {
                eprintln!("Error starting editor: {}", e);
                1
            }
        };
    }

    if let Some(allow_push) = options.allow_push_outside_timeslot {
        if let Err(e) = config.set_allow_push_outside_timeslot(allow_push) {
            eprintln!("Error setting allow_push_outside_timeslot: {}", e);
            return 1;
        }
        println!("Setting allow_push_outside_timeslot = {}.", allow_push);
    }

    if let Some(timezone) = options.timezone {
        if let Err(e) = config.set_timezone(timezone.clone()) {
            eprintln!("Error setting timezone: {}", e);
            return 1;
        }
        println!("Setting timezone = {}.", timezone);
    }

    println!("Done.");
    println!();
    println!("To view your configuration, open:");
    match config.get_file_path() {
        Ok(path) => println!("  {}", path.display()),
        Err(e) => eprintln!("  Error getting config path: {}", e),
    }

    0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, ConfigData};

    #[test]
    fn sets_timezone_when_it_is_in_options() {
        let mut config = Config::create_test_config(ConfigData::default());
        assert_eq!(config.get_timezone(), "UTC");

        let options = ConfigOptions {
            edit: false,
            allow_push_outside_timeslot: None,
            timezone: Some("Europe/Paris".to_string()),
        };
        run_config_command(&mut config, options);
        assert_eq!(config.get_timezone(), "Europe/Paris");
    }

    #[test]
    fn sets_allow_push_outside_timeslot_when_it_is_in_options() {
        let mut config = Config::create_test_config(ConfigData::default());
        assert_eq!(config.get_allow_push_outside_timeslot(), false);

        let options = ConfigOptions {
            edit: false,
            allow_push_outside_timeslot: Some(true),
            timezone: None,
        };
        run_config_command(&mut config, options);
        assert_eq!(config.get_allow_push_outside_timeslot(), true);

        let options2 = ConfigOptions {
            edit: false,
            allow_push_outside_timeslot: Some(false),
            timezone: None,
        };
        run_config_command(&mut config, options2);
        assert_eq!(config.get_allow_push_outside_timeslot(), false);
    }

    #[test]
    fn does_not_set_anything_when_options_are_empty() {
        let mut config = Config::create_test_config(ConfigData::default());
        let options = ConfigOptions {
            edit: false,
            allow_push_outside_timeslot: None,
            timezone: None,
        };
        run_config_command(&mut config, options);
        assert_eq!(config.get_timezone(), "UTC");
        assert_eq!(config.get_allow_push_outside_timeslot(), false);
    }
}
