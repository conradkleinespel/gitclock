use chrono::{DateTime, Datelike, Timelike, Utc};
use chrono_tz::Tz;
use regex::Regex;
use std::str::FromStr;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Timeslot {
    pub day_range: (u32, u32),
    pub start_time: (u32, u32),
    pub end_time: (u32, u32),
    pub timezone: String,
}

impl Timeslot {
    pub fn new(
        day_range_str: &str,
        start_time_str: &str,
        end_time_str: &str,
        timezone: &str,
    ) -> anyhow::Result<Self> {
        let (start_day, end_day) = extract_day(day_range_str)?;
        let (start_hour, start_min) = extract_hours_and_minutes(start_time_str)?;
        let (end_hour, end_min) = extract_hours_and_minutes(end_time_str)?;

        let _: Tz = Tz::from_str(timezone)
            .map_err(|_| anyhow::anyhow!("Timezone must be string, eg +0200 or Europe/Paris"))?;

        let res = Self {
            day_range: (start_day, end_day),
            start_time: (start_hour, start_min),
            end_time: (end_hour, end_min),
            timezone: timezone.to_string(),
        };
        Ok(res)
    }

    pub fn is_date_within(&self, date: DateTime<Utc>) -> bool {
        let tz: Tz = self.timezone.parse().unwrap();
        let date_in_tz = date.with_timezone(&tz);

        let day = date_in_tz.weekday().number_from_monday();
        let hour = date_in_tz.hour();
        let minute = date_in_tz.minute();

        if day < self.day_range.0 || day > self.day_range.1 {
            return false;
        }
        if hour < self.start_time.0 || (hour == self.start_time.0 && minute < self.start_time.1) {
            return false;
        }
        if hour > self.end_time.0 || (hour == self.end_time.0 && minute > self.end_time.1) {
            return false;
        }
        true
    }

    pub fn next_suitable_date(&self, mut min_date: DateTime<Utc>) -> DateTime<Utc> {
        while !self.is_date_within(min_date) {
            min_date += chrono::Duration::minutes(1);
        }
        min_date
    }
}

fn extract_hours_and_minutes(time_str: &str) -> anyhow::Result<(u32, u32)> {
    if !Regex::new(r"^[0-2][0-9][0-5][0-9]$")?.is_match(time_str) {
        return Err(anyhow::anyhow!(
            "Invalid time format, it should be like 0900 or 1530"
        ));
    }
    let hour: u32 = time_str[0..2].parse()?;
    let minutes: u32 = time_str[2..4].parse()?;

    if hour > 23 {
        return Err(anyhow::anyhow!(
            "Invalid time format, it should be like 0900 or 1530"
        ));
    }

    Ok((hour, minutes))
}

fn extract_day(day_range_str: &str) -> anyhow::Result<(u32, u32)> {
    if !Regex::new(r"^[1-7]-[1-7]$")?.is_match(day_range_str) {
        return Err(anyhow::anyhow!(
            "Invalid day range format. It should be like 1-7"
        ));
    }
    let (start_day, end_day) = (day_range_str[0..1].parse()?, day_range_str[2..3].parse()?);
    if start_day > end_day {
        return Err(anyhow::anyhow!(
            "Invalid day range, first day should be lower or equal to second day"
        ));
    }
    Ok((start_day, end_day))
}

impl std::fmt::Display for Timeslot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let days = [
            "Monday",
            "Tuesday",
            "Wednesday",
            "Thursday",
            "Friday",
            "Saturday",
            "Sunday",
        ];

        let formatted_start_time = format!("{:02}:{:02}", self.start_time.0, self.start_time.1);
        let formatted_end_time = format!("{:02}:{:02}", self.end_time.0, self.end_time.1);

        let mut day_range_statement = days[self.day_range.0 as usize - 1].to_string();
        if self.day_range.0 != self.day_range.1 {
            day_range_statement += &format!(" to {}", days[self.day_range.1 as usize - 1]);
        }

        write!(
            f,
            "{}, between {} and {} in timezone {}",
            day_range_statement, formatted_start_time, formatted_end_time, self.timezone
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn throws_error_on_invalid_day_range() {
        assert!(Timeslot::new("8-9", "0900", "1500", "UTC").is_err());
        assert!(Timeslot::new("2-1", "0900", "1500", "UTC").is_err());
    }

    #[test]
    fn throws_error_on_invalid_start_time() {
        assert!(Timeslot::new("1-5", "2500", "1500", "UTC").is_err());
    }

    #[test]
    fn throws_error_on_invalid_end_time() {
        assert!(Timeslot::new("1-5", "0900", "2500", "UTC").is_err());
    }

    #[test]
    fn is_date_within_returns_the_right_value() {
        let timeslot = Timeslot::new("1-5", "0915", "1500", "Asia/Kolkata").unwrap();

        let test_cases = vec![
            (20, 16, 0, "Asia/Kolkata", false),    // Saturday
            (17, 16, 0, "Asia/Kolkata", false),    // Wednesday, late hour
            (17, 15, 1, "Asia/Kolkata", false),    // Wednesday, late minute
            (17, 8, 0, "Asia/Kolkata", false),     // Wednesday, early hour
            (17, 9, 10, "Asia/Kolkata", false),    // Wednesday, early minute
            (17, 9, 45, "Asia/Kolkata", true),     // Wednesday, within schedule
            (17, 14, 45, "Africa/Nairobi", false), // Wednesday, within schedule but wrong timezone so outside schedule
        ];

        for (day, hour, minute, computer_timezone, expected) in test_cases {
            let tz: Tz = computer_timezone.parse().unwrap();
            let date = tz
                .with_ymd_and_hms(2024, 4, day, hour, minute, 0)
                .unwrap()
                .with_timezone(&Utc);
            assert_eq!(
                timeslot.is_date_within(date),
                expected,
                "Failed for day={}, hour={}, minute={}, tz={}",
                day,
                hour,
                minute,
                computer_timezone
            );
        }
    }

    #[test]
    fn next_suitable_date_returns_next_day_if_time_exceeds_end_time() {
        let test_cases = vec![("Africa/Nairobi", 9, 0, 18), ("Asia/Kolkata", 9, 0, 18)];

        for (timezone, expected_hour, expected_min, expected_day) in test_cases {
            let timeslot = Timeslot::new("1-5", "0900", "1500", timezone).unwrap();
            let wednesday_at_four_pm = DateTime::parse_from_rfc3339("2024-04-17T16:00:00Z")
                .unwrap()
                .with_timezone(&Utc);
            let result = timeslot.next_suitable_date(wednesday_at_four_pm);
            let result_in_tz = result.with_timezone(&timezone.parse::<Tz>().unwrap());
            assert_eq!(
                result_in_tz.day(),
                expected_day,
                "Failed day for timezone {}",
                timezone
            );
            assert_eq!(
                result_in_tz.hour(),
                expected_hour,
                "Failed hour for timezone {}",
                timezone
            );
            assert_eq!(
                result_in_tz.minute(),
                expected_min,
                "Failed minute for timezone {}",
                timezone
            );
        }
    }

    #[test]
    fn next_suitable_date_returns_first_day_of_next_week_if_date_is_in_weekend() {
        let test_cases = vec![("Africa/Nairobi", 9, 0, 22), ("Asia/Kolkata", 9, 0, 22)];

        for (timezone, expected_hour, expected_min, expected_day) in test_cases {
            let timeslot = Timeslot::new("1-5", "0900", "1500", timezone).unwrap();
            let sunday = DateTime::parse_from_rfc3339("2024-04-21T00:00:00Z")
                .unwrap()
                .with_timezone(&Utc);
            let result = timeslot.next_suitable_date(sunday);
            let result_in_tz = result.with_timezone(&timezone.parse::<Tz>().unwrap());
            assert_eq!(
                result_in_tz.day(),
                expected_day,
                "Failed day for timezone {}",
                timezone
            );
            assert_eq!(
                result_in_tz.hour(),
                expected_hour,
                "Failed hour for timezone {}",
                timezone
            );
            assert_eq!(
                result_in_tz.minute(),
                expected_min,
                "Failed minute for timezone {}",
                timezone
            );
        }
    }

    #[test]
    fn next_suitable_date_returns_date_itself_if_it_is_within() {
        let test_cases = vec![("Africa/Nairobi", 9, 0, 22), ("Asia/Kolkata", 9, 0, 22)];

        for (timezone, expected_hour, expected_min, expected_day) in test_cases {
            let timeslot = Timeslot::new("1-5", "0900", "1500", timezone).unwrap();
            let sunday = DateTime::parse_from_rfc3339("2024-04-21T10:45:00Z")
                .unwrap()
                .with_timezone(&Utc);
            let result = timeslot.next_suitable_date(sunday);
            let result_in_tz = result.with_timezone(&timezone.parse::<Tz>().unwrap());
            assert_eq!(
                result_in_tz.day(),
                expected_day,
                "Failed day for timezone {}",
                timezone
            );
            assert_eq!(
                result_in_tz.hour(),
                expected_hour,
                "Failed hour for timezone {}",
                timezone
            );
            assert_eq!(
                result_in_tz.minute(),
                expected_min,
                "Failed minute for timezone {}",
                timezone
            );
        }
    }

    #[test]
    fn test_extract_day() {
        assert_eq!(extract_day("1-5").unwrap(), (1, 5));
        assert_eq!(extract_day("1-1").unwrap(), (1, 1));
        assert_eq!(extract_day("7-7").unwrap(), (7, 7));
        assert!(extract_day("0-5").is_err());
        assert!(extract_day("1-8").is_err());
        assert!(extract_day("5-1").is_err());
        assert!(extract_day("1:5").is_err());
        assert!(extract_day("15").is_err());
        assert!(extract_day("").is_err());
    }

    #[test]
    fn test_extract_hours_and_minutes() {
        assert_eq!(extract_hours_and_minutes("0900").unwrap(), (9, 0));
        assert_eq!(extract_hours_and_minutes("1530").unwrap(), (15, 30));
        assert_eq!(extract_hours_and_minutes("0000").unwrap(), (0, 0));
        assert_eq!(extract_hours_and_minutes("2359").unwrap(), (23, 59));
        assert!(extract_hours_and_minutes("2400").is_err());
        assert!(extract_hours_and_minutes("0960").is_err());
        assert!(extract_hours_and_minutes("900").is_err());
        assert!(extract_hours_and_minutes("09:00").is_err());
        assert!(extract_hours_and_minutes("abcd").is_err());
        assert!(extract_hours_and_minutes("").is_err());
    }

    #[test]
    fn to_string_returns_correct_string() {
        let timeslot = Timeslot::new("1-5", "0900", "1500", "Africa/Nairobi").unwrap();
        assert_eq!(
            timeslot.to_string(),
            "Monday to Friday, between 09:00 and 15:00 in timezone Africa/Nairobi"
        );

        let timeslot2 = Timeslot::new("6-6", "1000", "1400", "Africa/Nairobi").unwrap();
        assert_eq!(
            timeslot2.to_string(),
            "Saturday, between 10:00 and 14:00 in timezone Africa/Nairobi"
        );
    }
}
