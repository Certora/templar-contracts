use near_sdk::{env, json_types::U64, near};
use crate::models::templar_nondet::*;

/// Configure a method of determining the current time chunk.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[near(serializers = [json, borsh])]
pub enum TimeChunkConfiguration {
    BlockHeight { divisor: U64 },
    EpochHeight { divisor: U64 },
    BlockTimestampMs { divisor: U64 },
}

declare_nondet!(
    TimeChunkConfiguration,
    {
        nondet_choice!(
            TimeChunkConfiguration::BlockHeight { divisor: TemplarNondet::nondet() },
            TimeChunkConfiguration::EpochHeight { divisor: TemplarNondet::nondet() },
            TimeChunkConfiguration::BlockTimestampMs { divisor: TemplarNondet::nondet() }
        )
    }
);

impl TimeChunkConfiguration {
    pub fn now(&self) -> TimeChunk {
        let (time, U64(mut divisor)) = match self {
            Self::BlockHeight { divisor } => (env::block_height(), divisor),
            Self::EpochHeight { divisor } => (env::epoch_height(), divisor),
            Self::BlockTimestampMs { divisor } => (env::block_timestamp_ms(), divisor),
        };
        if divisor == 0 {
            divisor = 1;
        }
        TimeChunk(U64(time / divisor))
    }

    pub fn previous(&self) -> TimeChunk {
        let TimeChunk(U64(time)) = self.now();
        #[allow(clippy::unwrap_used, reason = "Assume now > 0")]
        TimeChunk(U64(time.checked_sub(1).unwrap()))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[near(serializers = [borsh, json])]
pub struct TimeChunk(pub U64);
