//! Instance identity scaffold for `20-tower-core/v1-03`.

use std::{fmt, str::FromStr, sync::Arc};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TowerInstanceId(String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TowerInstanceIdError;

impl TowerInstanceId {
    pub const DEFAULT: &'static str = "default";
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromStr for TowerInstanceId {
    type Err = TowerInstanceIdError;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let valid = !value.is_empty()
            && value.len() <= 64
            && value
                .bytes()
                .next()
                .is_some_and(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
            && value.bytes().all(|b| {
                b.is_ascii_lowercase() || b.is_ascii_digit() || matches!(b, b'.' | b'_' | b'-')
            });
        valid
            .then(|| Self(value.to_owned()))
            .ok_or(TowerInstanceIdError)
    }
}

impl fmt::Display for TowerInstanceId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// Cloneable identity-bearing handle scaffold. Runtime commands are added by
/// `20-tower-core` epics; this type deliberately cannot report fake readiness.
#[derive(Debug, Clone)]
pub struct TowerHandle {
    instance_id: Arc<TowerInstanceId>,
}

impl TowerHandle {
    pub fn scaffold(instance_id: TowerInstanceId) -> Self {
        Self {
            instance_id: Arc::new(instance_id),
        }
    }
    pub fn instance_id(&self) -> &TowerInstanceId {
        &self.instance_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn tower_instance_id_enforces_wire_format() {
        assert_eq!(
            "default".parse::<TowerInstanceId>().unwrap().as_str(),
            "default"
        );
        for invalid in ["", "UPPER", "-leading", "contains space", "é"] {
            assert!(invalid.parse::<TowerInstanceId>().is_err(), "{invalid}");
        }
    }
}
