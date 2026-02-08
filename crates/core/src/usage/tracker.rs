use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageStats {
    pub tier: String,
    pub used_today: u32,
    pub daily_limit: u32,
    pub used_this_month: u32,
}

impl UsageStats {
    pub fn can_enhance(&self) -> bool {
        self.used_today < self.daily_limit
    }

    pub fn remaining_today(&self) -> u32 {
        self.daily_limit.saturating_sub(self.used_today)
    }
}

pub fn daily_limit_for_tier(tier: &str) -> u32 {
    match tier {
        "pro" => 500,
        "trial" => 10,
        "free" => 5,
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_can_enhance() {
        let stats = UsageStats {
            tier: "pro".to_string(),
            used_today: 10,
            daily_limit: 500,
            used_this_month: 100,
        };
        assert!(stats.can_enhance());
    }

    #[test]
    fn test_cannot_enhance_at_limit() {
        let stats = UsageStats {
            tier: "free".to_string(),
            used_today: 5,
            daily_limit: 5,
            used_this_month: 5,
        };
        assert!(!stats.can_enhance());
    }

    #[test]
    fn test_remaining_today() {
        let stats = UsageStats {
            tier: "pro".to_string(),
            used_today: 100,
            daily_limit: 500,
            used_this_month: 100,
        };
        assert_eq!(stats.remaining_today(), 400);
    }

    #[test]
    fn test_daily_limits() {
        assert_eq!(daily_limit_for_tier("pro"), 500);
        assert_eq!(daily_limit_for_tier("trial"), 10);
        assert_eq!(daily_limit_for_tier("free"), 5);
        assert_eq!(daily_limit_for_tier("unknown"), 0);
    }
}
