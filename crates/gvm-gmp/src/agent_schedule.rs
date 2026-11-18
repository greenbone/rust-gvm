// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Greenbone AG

//! Structured agent scheduler entries shared by requests and observations.

/// A cron expression and optional maximum randomized scheduling delay.
///
/// Requests always use nested `cron` and `max_delay_seconds` children. The
/// server defaults an omitted delay to zero. Responses preserve omission.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AgentSchedule {
    /// Cron expression; interpretation is owned by Agent Control.
    pub cron: String,
    /// Maximum delay in seconds, restricted to `0..=i32::MAX` on requests.
    pub max_delay_seconds: Option<u32>,
}

impl From<String> for AgentSchedule {
    fn from(cron: String) -> Self {
        Self {
            cron,
            max_delay_seconds: None,
        }
    }
}

impl From<&str> for AgentSchedule {
    fn from(cron: &str) -> Self {
        cron.to_owned().into()
    }
}

pub(crate) fn validate_delay(delay: Option<u32>) -> Result<(), crate::GmpRequestError> {
    if delay.is_some_and(|delay| delay > i32::MAX as u32) {
        return Err(crate::GmpRequestError::invalid_field(
            "max_delay_seconds",
            "must be between 0 and 2147483647",
        ));
    }
    Ok(())
}
