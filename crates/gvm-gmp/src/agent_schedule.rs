// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Greenbone AG

//! Structured agent scheduler entries shared by requests and observations.

/// A cron expression and optional maximum randomized scheduling delay.
///
/// Requests always use nested `cron` and `max_delay_seconds` children. The
/// server defaults an omitted delay to zero. Responses preserve omission.
/// With the `serde` feature, deserialization also accepts a legacy cron string,
/// treating its delay as omitted. Serialization always emits the structured
/// object with `cron` and `max_delay_seconds` fields.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct AgentSchedule {
    /// Cron expression; interpretation is owned by Agent Control.
    pub cron: String,
    /// Maximum delay in seconds, restricted to `0..=i32::MAX` on requests.
    pub max_delay_seconds: Option<u32>,
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for AgentSchedule {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        #[serde(untagged)]
        enum Representation {
            Legacy(String),
            Structured {
                cron: String,
                max_delay_seconds: Option<u32>,
            },
        }

        Ok(
            match <Representation as serde::Deserialize>::deserialize(deserializer)? {
                Representation::Legacy(cron) => cron.into(),
                Representation::Structured {
                    cron,
                    max_delay_seconds,
                } => Self {
                    cron,
                    max_delay_seconds,
                },
            },
        )
    }
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
