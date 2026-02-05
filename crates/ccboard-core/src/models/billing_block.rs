use chrono::{DateTime, Timelike, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a 5-hour billing block as used by Claude Code pricing.
///
/// Claude Code charges based on 5-hour blocks in UTC time:
/// - Block 1: 00:00-04:59 UTC
/// - Block 2: 05:00-09:59 UTC
/// - Block 3: 10:00-14:59 UTC
/// - Block 4: 15:00-19:59 UTC
/// - Block 5: 20:00-23:59 UTC
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BillingBlock {
    /// Date of the block (YYYY-MM-DD)
    pub date: chrono::NaiveDate,
    /// Starting hour of the 5-hour block (0, 5, 10, 15, 20)
    pub block_hour: u8,
}

impl BillingBlock {
    /// Create a BillingBlock from a timestamp
    ///
    /// # Examples
    ///
    /// ```
    /// use chrono::{DateTime, Utc, TimeZone};
    /// use ccboard_core::models::BillingBlock;
    ///
    /// let timestamp = Utc.with_ymd_and_hms(2026, 2, 2, 14, 30, 0).unwrap();
    /// let block = BillingBlock::from_timestamp(&timestamp);
    ///
    /// assert_eq!(block.block_hour, 10); // 14:30 falls in 10:00-14:59 block
    /// ```
    pub fn from_timestamp(timestamp: &DateTime<Utc>) -> Self {
        let date = timestamp.date_naive();
        let hour = timestamp.hour() as u8;

        // Normalize to 5-hour block: block_hour = (hour / 5) * 5
        // 0-4 → 0, 5-9 → 5, 10-14 → 10, 15-19 → 15, 20-23 → 20
        let block_hour = (hour / 5) * 5;

        BillingBlock { date, block_hour }
    }

    /// Get the block label (e.g., "00:00-04:59", "05:00-09:59")
    pub fn label(&self) -> String {
        let end_hour = if self.block_hour == 20 {
            23
        } else {
            self.block_hour + 4
        };
        format!("{:02}:00-{:02}:59", self.block_hour, end_hour)
    }

    /// Get the block number (1-5)
    pub fn block_number(&self) -> u8 {
        (self.block_hour / 5) + 1
    }
}

/// Usage statistics for a billing block
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BillingBlockUsage {
    /// Input tokens
    pub input_tokens: u64,
    /// Output tokens
    pub output_tokens: u64,
    /// Cache creation tokens
    pub cache_creation_tokens: u64,
    /// Cache read tokens
    pub cache_read_tokens: u64,
    /// Total cost in USD
    pub total_cost: f64,
    /// Number of sessions in this block
    pub session_count: usize,
}

impl BillingBlockUsage {
    /// Total tokens (input + output + cache creation + cache read)
    ///
    /// IMPORTANT: This must match ccusage behavior which includes ALL token types.
    /// Previously missed cache_read_tokens, causing discrepancies with ccusage totals.
    pub fn total_tokens(&self) -> u64 {
        self.input_tokens + self.output_tokens + self.cache_creation_tokens + self.cache_read_tokens
    }

    /// Add usage from another block
    pub fn add(&mut self, other: &BillingBlockUsage) {
        self.input_tokens += other.input_tokens;
        self.output_tokens += other.output_tokens;
        self.cache_creation_tokens += other.cache_creation_tokens;
        self.cache_read_tokens += other.cache_read_tokens;
        self.total_cost += other.total_cost;
        self.session_count += other.session_count;
    }
}

/// Manager for billing block tracking
#[derive(Debug, Default)]
pub struct BillingBlockManager {
    /// Map of (date, block_hour) to usage
    blocks: HashMap<BillingBlock, BillingBlockUsage>,
}

impl BillingBlockManager {
    pub fn new() -> Self {
        Self {
            blocks: HashMap::new(),
        }
    }

    /// Add usage to a billing block
    pub fn add_usage(
        &mut self,
        timestamp: &DateTime<Utc>,
        input_tokens: u64,
        output_tokens: u64,
        cache_creation_tokens: u64,
        cache_read_tokens: u64,
        cost: f64,
    ) {
        let block = BillingBlock::from_timestamp(timestamp);
        let usage = self.blocks.entry(block).or_default();

        usage.input_tokens += input_tokens;
        usage.output_tokens += output_tokens;
        usage.cache_creation_tokens += cache_creation_tokens;
        usage.cache_read_tokens += cache_read_tokens;
        usage.total_cost += cost;
        usage.session_count += 1;
    }

    /// Get usage for a specific billing block
    pub fn get_usage(&self, block: &BillingBlock) -> Option<&BillingBlockUsage> {
        self.blocks.get(block)
    }

    /// Get all blocks sorted by date and block_hour
    pub fn get_all_blocks(&self) -> Vec<(BillingBlock, BillingBlockUsage)> {
        let mut blocks: Vec<_> = self
            .blocks
            .iter()
            .map(|(block, usage)| (*block, usage.clone()))
            .collect();

        blocks.sort_by(|a, b| {
            a.0.date
                .cmp(&b.0.date)
                .then_with(|| a.0.block_hour.cmp(&b.0.block_hour))
        });

        blocks
    }

    /// Get blocks for a specific date
    pub fn get_blocks_for_date(
        &self,
        date: chrono::NaiveDate,
    ) -> Vec<(BillingBlock, BillingBlockUsage)> {
        let mut blocks: Vec<_> = self
            .blocks
            .iter()
            .filter(|(block, _)| block.date == date)
            .map(|(block, usage)| (*block, usage.clone()))
            .collect();

        blocks.sort_by_key(|(block, _)| block.block_hour);
        blocks
    }

    /// Get color coding for a block based on cost thresholds
    ///
    /// - Green: < $2.5
    /// - Yellow: < $5.0
    /// - Red: >= $5.0
    pub fn get_color_for_cost(cost: f64) -> &'static str {
        if cost < 2.5 {
            "green"
        } else if cost < 5.0 {
            "yellow"
        } else {
            "red"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_billing_block_normalization() {
        // Block 1: 00:00-04:59
        let ts = Utc.with_ymd_and_hms(2026, 2, 2, 2, 30, 0).unwrap();
        let block = BillingBlock::from_timestamp(&ts);
        assert_eq!(block.block_hour, 0);
        assert_eq!(block.label(), "00:00-04:59");
        assert_eq!(block.block_number(), 1);

        // Block 2: 05:00-09:59
        let ts = Utc.with_ymd_and_hms(2026, 2, 2, 7, 15, 0).unwrap();
        let block = BillingBlock::from_timestamp(&ts);
        assert_eq!(block.block_hour, 5);
        assert_eq!(block.label(), "05:00-09:59");
        assert_eq!(block.block_number(), 2);

        // Block 3: 10:00-14:59
        let ts = Utc.with_ymd_and_hms(2026, 2, 2, 14, 59, 0).unwrap();
        let block = BillingBlock::from_timestamp(&ts);
        assert_eq!(block.block_hour, 10);
        assert_eq!(block.label(), "10:00-14:59");
        assert_eq!(block.block_number(), 3);

        // Block 4: 15:00-19:59
        let ts = Utc.with_ymd_and_hms(2026, 2, 2, 18, 0, 0).unwrap();
        let block = BillingBlock::from_timestamp(&ts);
        assert_eq!(block.block_hour, 15);
        assert_eq!(block.label(), "15:00-19:59");
        assert_eq!(block.block_number(), 4);

        // Block 5: 20:00-23:59
        let ts = Utc.with_ymd_and_hms(2026, 2, 2, 23, 59, 59).unwrap();
        let block = BillingBlock::from_timestamp(&ts);
        assert_eq!(block.block_hour, 20);
        assert_eq!(block.label(), "20:00-23:59");
        assert_eq!(block.block_number(), 5);
    }

    #[test]
    fn test_billing_block_manager() {
        let mut manager = BillingBlockManager::new();

        // Add usage to block 1 (00:00-04:59)
        let ts1 = Utc.with_ymd_and_hms(2026, 2, 2, 2, 0, 0).unwrap();
        manager.add_usage(&ts1, 1000, 500, 100, 50, 0.5);

        // Add more usage to same block
        manager.add_usage(&ts1, 500, 250, 50, 25, 0.25);

        // Add usage to block 2 (05:00-09:59)
        let ts2 = Utc.with_ymd_and_hms(2026, 2, 2, 7, 0, 0).unwrap();
        manager.add_usage(&ts2, 2000, 1000, 200, 100, 1.0);

        // Check block 1 totals
        let block1 = BillingBlock::from_timestamp(&ts1);
        let usage1 = manager.get_usage(&block1).unwrap();
        assert_eq!(usage1.input_tokens, 1500);
        assert_eq!(usage1.output_tokens, 750);
        assert_eq!(usage1.cache_creation_tokens, 150);
        assert_eq!(usage1.cache_read_tokens, 75);
        assert_eq!(usage1.total_cost, 0.75);
        assert_eq!(usage1.session_count, 2);

        // Check block 2 totals
        let block2 = BillingBlock::from_timestamp(&ts2);
        let usage2 = manager.get_usage(&block2).unwrap();
        assert_eq!(usage2.input_tokens, 2000);
        assert_eq!(usage2.total_cost, 1.0);
        assert_eq!(usage2.session_count, 1);

        // Check all blocks sorted
        let all_blocks = manager.get_all_blocks();
        assert_eq!(all_blocks.len(), 2);
        assert_eq!(all_blocks[0].0.block_hour, 0); // Block 1 first
        assert_eq!(all_blocks[1].0.block_hour, 5); // Block 2 second
    }

    #[test]
    fn test_color_coding() {
        assert_eq!(BillingBlockManager::get_color_for_cost(1.0), "green");
        assert_eq!(BillingBlockManager::get_color_for_cost(2.49), "green");
        assert_eq!(BillingBlockManager::get_color_for_cost(2.5), "yellow");
        assert_eq!(BillingBlockManager::get_color_for_cost(4.99), "yellow");
        assert_eq!(BillingBlockManager::get_color_for_cost(5.0), "red");
        assert_eq!(BillingBlockManager::get_color_for_cost(10.0), "red");
    }

    #[test]
    fn test_blocks_for_date() {
        let mut manager = BillingBlockManager::new();

        // Feb 2
        let ts1 = Utc.with_ymd_and_hms(2026, 2, 2, 2, 0, 0).unwrap();
        manager.add_usage(&ts1, 1000, 500, 0, 0, 0.5);

        let ts2 = Utc.with_ymd_and_hms(2026, 2, 2, 14, 0, 0).unwrap();
        manager.add_usage(&ts2, 2000, 1000, 0, 0, 1.0);

        // Feb 3
        let ts3 = Utc.with_ymd_and_hms(2026, 2, 3, 7, 0, 0).unwrap();
        manager.add_usage(&ts3, 500, 250, 0, 0, 0.25);

        // Get blocks for Feb 2
        let date = chrono::NaiveDate::from_ymd_opt(2026, 2, 2).unwrap();
        let blocks = manager.get_blocks_for_date(date);
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].0.block_hour, 0); // Block 1
        assert_eq!(blocks[1].0.block_hour, 10); // Block 3

        // Get blocks for Feb 3
        let date = chrono::NaiveDate::from_ymd_opt(2026, 2, 3).unwrap();
        let blocks = manager.get_blocks_for_date(date);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].0.block_hour, 5); // Block 2
    }
}
