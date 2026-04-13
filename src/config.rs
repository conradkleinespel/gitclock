use crate::timeslot::Timeslot;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::debug;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct ConfigData {
    pub timeslots: Vec<TimeslotConfig>,
    pub allow_push_outside_timeslot: Option<bool>,
    pub timezone: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TimeslotConfig {
    pub days: String,
    pub start: String,
    pub end: String,
}

pub struct Config {
    pub data: ConfigData,
    config_name: String,
}

impl Config {
    #[cfg(test)]
    pub fn create_test_config(data: ConfigData) -> Self {
        use rand::distributions::{Alphanumeric, DistString};
        let mut rng = rand::thread_rng();
        let suffix = Alphanumeric.sample_string(&mut rng, 10);
        let config_name = format!("test_config_{}", suffix);
        Self { data, config_name }
    }

    pub fn create_from_conf() -> anyhow::Result<Self> {
        Self::create_from_name("gitclock")
    }

    pub fn create_from_name(name: &str) -> anyhow::Result<Self> {
        debug!(config_name = name, "Loading config");
        let data: ConfigData =
            confy::load(name, name).map_err(|e| anyhow::anyhow!("Failed to load config: {}", e))?;
        let filename = confy::get_configuration_file_path(name, Some(name)).unwrap();
        debug!(?data, ?filename, "Config loaded successfully");
        Ok(Self {
            data,
            config_name: name.to_string(),
        })
    }

    pub fn save(&self) -> anyhow::Result<()> {
        debug!(config_name = self.config_name, "Saving config");
        confy::store(
            &self.config_name,
            Some(self.config_name.as_str()),
            &self.data,
        )
        .map_err(|e| anyhow::anyhow!("Failed to save config: {}", e))?;
        debug!("Config saved successfully");
        Ok(())
    }

    pub fn get_timeslots(&self) -> Vec<Timeslot> {
        let timezone = self.get_timezone();
        self.data
            .timeslots
            .iter()
            .filter_map(|t| Timeslot::new(&t.days, &t.start, &t.end, &timezone).ok())
            .collect()
    }

    pub fn add_timeslot(&mut self, days: String, start: String, end: String) -> anyhow::Result<()> {
        let timezone = self.get_timezone();
        let timeslot = Timeslot::new(&days, &start, &end, &timezone)?;
        debug!(?timeslot, "Created new Timeslot");
        self.data
            .timeslots
            .push(TimeslotConfig { days, start, end });
        self.save()
    }

    pub fn get_allow_push_outside_timeslot(&self) -> bool {
        self.data.allow_push_outside_timeslot.unwrap_or(false)
    }

    pub fn set_allow_push_outside_timeslot(&mut self, value: bool) -> anyhow::Result<()> {
        self.data.allow_push_outside_timeslot = Some(value);
        self.save()
    }

    pub fn get_timezone(&self) -> String {
        let tz = self.data.timezone.clone().unwrap_or_else(|| {
            // This is a simplification, but good enough for now
            "UTC".to_string()
        });
        tz
    }

    pub fn set_timezone(&mut self, value: String) -> anyhow::Result<()> {
        self.validate_timezone(value.clone())?;
        self.data.timezone = Some(value);
        self.save()
    }

    pub fn get_file_path(&self) -> anyhow::Result<PathBuf> {
        confy::get_configuration_file_path(&self.config_name, Some(self.config_name.as_str()))
            .map_err(|e| anyhow::anyhow!("Failed to get config path: {}", e))
    }

    pub fn check_config(&self) -> anyhow::Result<()> {
        let timezone = self.get_timezone();
        self.validate_timezone(timezone.clone())?;
        for t in &self.data.timeslots {
            Timeslot::new(&t.days, &t.start, &t.end, &timezone)?;
        }
        Ok(())
    }

    fn validate_timezone(&self, value: String) -> anyhow::Result<()> {
        use chrono_tz::Tz;
        use std::str::FromStr;
        Tz::from_str(&value).map_err(|_| {
            anyhow::anyhow!(
                "Timezone is invalid, expected something like Europe/Paris, but got {}.",
                value
            )
        })?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_timeslots_converts_to_timeslot_instances() {
        let config = Config::create_test_config(ConfigData {
            timeslots: vec![
                TimeslotConfig {
                    days: "1-5".to_string(),
                    start: "0900".to_string(),
                    end: "1700".to_string(),
                },
                TimeslotConfig {
                    days: "6-6".to_string(),
                    start: "0900".to_string(),
                    end: "1200".to_string(),
                },
            ],
            allow_push_outside_timeslot: None,
            timezone: Some("UTC".to_string()),
        });

        let timeslots = config.get_timeslots();
        assert_eq!(timeslots.len(), 2);
    }

    #[test]
    fn get_allow_push_outside_timeslot_returns_boolean() {
        let mut config = Config::create_test_config(ConfigData::default());
        assert_eq!(config.get_allow_push_outside_timeslot(), false);
        config.data.allow_push_outside_timeslot = Some(true);
        assert_eq!(config.get_allow_push_outside_timeslot(), true);
    }

    #[test]
    fn get_timezone_returns_default_or_set_value() {
        let mut config = Config::create_test_config(ConfigData::default());
        assert_eq!(config.get_timezone(), "UTC");
        config.data.timezone = Some("Europe/Paris".to_string());
        assert_eq!(config.get_timezone(), "Europe/Paris");
    }

    #[test]
    fn set_timezone_throws_on_invalid_timezone() {
        let mut config = Config::create_test_config(ConfigData::default());
        assert!(config.set_timezone("invalid".to_string()).is_err());
    }

    #[test]
    fn check_config_is_noop_when_valid() {
        let config = Config::create_test_config(ConfigData {
            timeslots: vec![TimeslotConfig {
                days: "1-5".to_string(),
                start: "0900".to_string(),
                end: "1700".to_string(),
            }],
            allow_push_outside_timeslot: Some(true),
            timezone: Some("Europe/Paris".to_string()),
        });
        assert!(config.check_config().is_ok());
    }

    #[test]
    fn get_file_path_returns_unique_test_path() {
        let config1 = Config::create_test_config(ConfigData::default());
        let config2 = Config::create_test_config(ConfigData::default());

        let path1 = config1
            .get_file_path()
            .expect("Should be able to get config path 1");
        let path2 = config2
            .get_file_path()
            .expect("Should be able to get config path 2");

        assert_ne!(path1, path2, "Each test config should have a unique path");
        assert!(
            path1.to_string_lossy().contains("test_config_"),
            "Path 1 should contain test_config_ prefix"
        );
        assert!(
            path2.to_string_lossy().contains("test_config_"),
            "Path 2 should contain test_config_ prefix"
        );
    }

    #[test]
    fn set_allow_push_outside_timeslot_sets_value() {
        let mut config = Config::create_test_config(ConfigData::default());
        let _ = config.set_allow_push_outside_timeslot(true);
        assert_eq!(config.get_allow_push_outside_timeslot(), true);
    }

    #[test]
    fn add_timeslot_adds_and_validates() {
        let mut config = Config::create_test_config(ConfigData::default());
        let _ = config.add_timeslot("1-5".to_string(), "0900".to_string(), "1700".to_string());
        assert_eq!(config.data.timeslots.len(), 1);
        assert_eq!(config.data.timeslots[0].days, "1-5");

        assert!(
            config
                .add_timeslot(
                    "invalid".to_string(),
                    "0900".to_string(),
                    "1700".to_string()
                )
                .is_err()
        );
        assert_eq!(config.data.timeslots.len(), 1);
    }
}
