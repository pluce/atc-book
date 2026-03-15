use chrono::{Datelike, NaiveDate};

/// Reference epoch: AIRAC cycle 2601 starts on 2026-01-22
const EPOCH: (i32, u32, u32) = (2026, 1, 22);
const CYCLE_DAYS: i64 = 28;

#[derive(Debug, Clone, PartialEq)]
pub struct AiracCycle {
    pub code: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
}

impl AiracCycle {
    /// Calculate the current AIRAC cycle for a given date.
    pub fn for_date(date: NaiveDate) -> Self {
        let epoch = NaiveDate::from_ymd_opt(EPOCH.0, EPOCH.1, EPOCH.2).unwrap();
        let diff = date.signed_duration_since(epoch).num_days();

        let (start_date, cycle_number_in_year) = if diff >= 0 {
            let cycles_since_epoch = diff / CYCLE_DAYS;
            let start = epoch + chrono::Duration::days(cycles_since_epoch * CYCLE_DAYS);
            // Count which cycle number within the year
            let year_start = NaiveDate::from_ymd_opt(start.year(), 1, 1).unwrap();
            let day_of_year = start.signed_duration_since(year_start).num_days();
            // Find the first AIRAC cycle start in this year
            let epoch_to_year_start = year_start.signed_duration_since(epoch).num_days();
            let first_cycle_offset = if epoch_to_year_start <= 0 {
                (-epoch_to_year_start) % CYCLE_DAYS
            } else {
                (CYCLE_DAYS - (epoch_to_year_start % CYCLE_DAYS)) % CYCLE_DAYS
            };
            let first_cycle_day = first_cycle_offset;
            let cycle_in_year = ((day_of_year - first_cycle_day) / CYCLE_DAYS) + 1;
            (start, cycle_in_year as u32)
        } else {
            // Before epoch — go backwards
            let cycles_before = (-diff - 1) / CYCLE_DAYS + 1;
            let start = epoch - chrono::Duration::days(cycles_before * CYCLE_DAYS);
            let year_start = NaiveDate::from_ymd_opt(start.year(), 1, 1).unwrap();
            let day_of_year = start.signed_duration_since(year_start).num_days();
            let epoch_to_year_start = year_start.signed_duration_since(epoch).num_days();
            let first_cycle_offset = if epoch_to_year_start <= 0 {
                (-epoch_to_year_start) % CYCLE_DAYS
            } else {
                (CYCLE_DAYS - (epoch_to_year_start % CYCLE_DAYS)) % CYCLE_DAYS
            };
            let cycle_in_year = ((day_of_year - first_cycle_offset) / CYCLE_DAYS) + 1;
            (start, cycle_in_year as u32)
        };

        let end_date = start_date + chrono::Duration::days(CYCLE_DAYS - 1);
        let year_short = (start_date.year() % 100) as u32;
        let code = format!("{:02}{:02}", year_short, cycle_number_in_year);

        Self {
            code,
            start_date,
            end_date,
        }
    }

    /// Get current AIRAC cycle
    pub fn current() -> Self {
        Self::for_date(chrono::Local::now().date_naive())
    }

    /// Is this cycle currently in effect?
    pub fn is_active(&self) -> bool {
        let today = chrono::Local::now().date_naive();
        today >= self.start_date && today <= self.end_date
    }

    /// Format for SIA URL: `eAIP_22_JAN_2026`
    pub fn sia_cycle_name(&self) -> String {
        let month = match self.start_date.month() {
            1 => "JAN",
            2 => "FEB",
            3 => "MAR",
            4 => "APR",
            5 => "MAY",
            6 => "JUN",
            7 => "JUL",
            8 => "AUG",
            9 => "SEP",
            10 => "OCT",
            11 => "NOV",
            12 => "DEC",
            _ => unreachable!(),
        };
        format!(
            "eAIP_{:02}_{}_{}",
            self.start_date.day(),
            month,
            self.start_date.year()
        )
    }

    /// Format for SIA URL: `AIRAC-2026-01-22`
    pub fn sia_airac_date(&self) -> String {
        format!("AIRAC-{}", self.start_date.format("%Y-%m-%d"))
    }

    /// Format for UK NATS URL: `2026-01-22-AIRAC`
    pub fn nats_airac_part(&self) -> String {
        format!("{}-AIRAC", self.start_date.format("%Y-%m-%d"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_epoch_cycle() {
        let date = NaiveDate::from_ymd_opt(2026, 1, 22).unwrap();
        let cycle = AiracCycle::for_date(date);
        assert_eq!(cycle.code, "2601");
        assert_eq!(cycle.start_date, date);
    }

    #[test]
    fn test_cycle_2602() {
        let date = NaiveDate::from_ymd_opt(2026, 2, 19).unwrap();
        let cycle = AiracCycle::for_date(date);
        assert_eq!(cycle.code, "2602");
        assert_eq!(cycle.start_date, date);
    }

    #[test]
    fn test_mid_cycle() {
        let date = NaiveDate::from_ymd_opt(2026, 2, 1).unwrap();
        let cycle = AiracCycle::for_date(date);
        assert_eq!(cycle.code, "2601");
    }

    #[test]
    fn test_sia_format() {
        let date = NaiveDate::from_ymd_opt(2026, 1, 22).unwrap();
        let cycle = AiracCycle::for_date(date);
        assert_eq!(cycle.sia_cycle_name(), "eAIP_22_JAN_2026");
        assert_eq!(cycle.sia_airac_date(), "AIRAC-2026-01-22");
    }

    #[test]
    fn test_nats_format() {
        let date = NaiveDate::from_ymd_opt(2026, 1, 22).unwrap();
        let cycle = AiracCycle::for_date(date);
        assert_eq!(cycle.nats_airac_part(), "2026-01-22-AIRAC");
    }

    #[test]
    fn test_march_5_2026() {
        let date = NaiveDate::from_ymd_opt(2026, 3, 5).unwrap();
        let cycle = AiracCycle::for_date(date);
        println!("March 5 cycle: {:?}", cycle);
        println!("sia_cycle_name: {}", cycle.sia_cycle_name());
        println!("sia_airac_date: {}", cycle.sia_airac_date());
        assert_eq!(cycle.code, "2602");
        assert_eq!(
            cycle.start_date,
            NaiveDate::from_ymd_opt(2026, 2, 19).unwrap()
        );
        assert_eq!(cycle.sia_cycle_name(), "eAIP_19_FEB_2026");
        assert_eq!(cycle.sia_airac_date(), "AIRAC-2026-02-19");
    }
}
