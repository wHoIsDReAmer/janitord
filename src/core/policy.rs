use anyhow::{Result, bail};
use jiff::{Span, Timestamp, ZonedDateTime};

#[derive(Debug, Clone, Default)]
pub struct Policy {
    // delete conditions
    old_threshold: Option<Span>,
    size_threshold: Option<u64>,

    // daemon condition
    free_space: Option<FreeSpace>,
}

#[derive(Debug, Clone, Copy)]
struct FreeSpace {
    start_below: f64,
    stop_at: f64,
}

#[derive(Debug, Default)]
pub struct PolicyBuilder(Policy);

impl Policy {
    pub fn builder() -> PolicyBuilder {
        PolicyBuilder::default()
    }

    pub fn matches(&self, size: u64, modified: Timestamp, now: &ZonedDateTime) -> Result<bool> {
        if self.is_empty() {
            return Ok(false);
        }

        Ok(self.is_large(size) && self.is_old(modified, now)?)
    }

    fn is_empty(&self) -> bool {
        self.old_threshold.is_none() && self.size_threshold.is_none()
    }

    fn is_large(&self, size: u64) -> bool {
        self.size_threshold.is_none_or(|min| size >= min)
    }

    fn is_old(&self, modified: Timestamp, now: &ZonedDateTime) -> Result<bool> {
        let Some(span) = self.old_threshold else {
            return Ok(true);
        };

        let cutoff = now.checked_sub(span)?.timestamp();
        Ok(modified <= cutoff)
    }
}

impl PolicyBuilder {
    pub fn old_threshold(mut self, span: Span) -> Self {
        self.0.old_threshold = Some(span);
        self
    }

    pub fn size_threshold(mut self, bytes: u64) -> Self {
        self.0.size_threshold = Some(bytes);
        self
    }

    pub fn free_space(mut self, start_below: f64, stop_at: f64) -> Self {
        self.0.free_space = Some(FreeSpace {
            start_below,
            stop_at,
        });
        self
    }

    pub fn build(self) -> Result<Policy> {
        let policy = self.0;

        if let Some(span) = policy.old_threshold
            && span.is_negative()
        {
            bail!("old_threshold must not be negative: {span}");
        }

        if let Some(FreeSpace {
            start_below,
            stop_at,
        }) = policy.free_space
        {
            if !(0.0..=1.0).contains(&start_below) || !(0.0..=1.0).contains(&stop_at) {
                bail!("free space ratios must be between 0 and 1");
            }
            if stop_at <= start_below {
                bail!("stop_at ({stop_at}) must be greater than start_below ({start_below})");
            }
        }

        Ok(policy)
    }
}

#[cfg(test)]
mod tests {
    use jiff::{ToSpan, tz::TimeZone};

    use super::*;

    fn now() -> ZonedDateTime {
        Timestamp::from_second(1_800_000_000)
            .unwrap()
            .to_zoned_datetime(TimeZone::UTC)
    }

    fn days_ago(days: i64) -> Timestamp {
        now().checked_sub(days.days()).unwrap().timestamp()
    }

    #[test]
    fn empty_policy_matches_nothing() -> Result<()> {
        let policy = Policy::builder().build()?;
        assert!(!policy.matches(u64::MAX, days_ago(10_000), &now())?);
        Ok(())
    }

    #[test]
    fn every_set_policy_must_hold() -> Result<()> {
        let policy = Policy::builder()
            .old_threshold(30.days())
            .size_threshold(100)
            .build()?;
        let now = now();
        assert!(policy.matches(100, days_ago(30), &now)?);
        assert!(!policy.matches(100, days_ago(29), &now)?);
        assert!(!policy.matches(99, days_ago(365), &now)?);
        Ok(())
    }

    #[test]
    fn negative_old_threshold_is_rejected() {
        assert!(
            Policy::builder()
                .old_threshold((-30).days())
                .build()
                .is_err()
        );
    }

    #[test]
    fn invalid_free_space_is_rejected() {
        for (start_below, stop_at) in [(0.20, 0.10), (0.10, 0.10), (0.10, 1.5), (0.10, f64::NAN)] {
            assert!(
                Policy::builder()
                    .free_space(start_below, stop_at)
                    .build()
                    .is_err(),
                "free_space({start_below}, {stop_at}) should be rejected"
            );
        }
    }
}
