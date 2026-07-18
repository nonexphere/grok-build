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

/// Two Tower instances are isolated when their instance IDs differ and they
/// do not share registry handles.
#[derive(Debug, Default)]
pub struct InstanceDirectory {
    handles: std::collections::HashMap<TowerInstanceId, TowerHandle>,
}

impl InstanceDirectory {
    pub fn insert(&mut self, handle: TowerHandle) -> Result<(), TowerInstanceIdError> {
        let id = handle.instance_id().clone();
        if self.handles.contains_key(&id) {
            return Err(TowerInstanceIdError);
        }
        self.handles.insert(id, handle);
        Ok(())
    }

    pub fn get(&self, id: &TowerInstanceId) -> Option<&TowerHandle> {
        self.handles.get(id)
    }

    pub fn len(&self) -> usize {
        self.handles.len()
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

    #[test]
    fn multi_instance_directory_isolates_handles() {
        let a: TowerInstanceId = "default".parse().unwrap();
        let b: TowerInstanceId = "branch-a".parse().unwrap();
        let mut dir = InstanceDirectory::default();
        dir.insert(TowerHandle::scaffold(a.clone())).unwrap();
        dir.insert(TowerHandle::scaffold(b.clone())).unwrap();
        assert!(dir.insert(TowerHandle::scaffold(a.clone())).is_err());
        assert_eq!(dir.len(), 2);
        assert_ne!(
            dir.get(&a).unwrap().instance_id().as_str(),
            dir.get(&b).unwrap().instance_id().as_str()
        );
    }
}

#[cfg(test)]
mod two_instances_tests {
    use super::*;
    use crate::SessionRegistry;

    #[test]
    fn two_instances_have_disjoint_registries() {
        let a: TowerInstanceId = "default".parse().unwrap();
        let b: TowerInstanceId = "worktree-1".parse().unwrap();
        let mut dir = InstanceDirectory::default();
        dir.insert(TowerHandle::scaffold(a.clone())).unwrap();
        dir.insert(TowerHandle::scaffold(b.clone())).unwrap();
        let mut reg_a = SessionRegistry::new();
        let mut reg_b = SessionRegistry::new();
        let (ta, _) = reg_a.get_or_insert_with("s1", |_| Ok(())).unwrap();
        let (tb, _) = reg_b.get_or_insert_with("s1", |_| Ok(())).unwrap();
        // Same session id string may exist in both instances without shared token identity.
        assert_eq!(ta.as_u64(), 1);
        assert_eq!(tb.as_u64(), 1);
        assert_ne!(dir.get(&a).unwrap().instance_id(), dir.get(&b).unwrap().instance_id());
        // No cross-registry steal: removing from a does not affect b.
        reg_a.remove("s1");
        assert!(reg_a.get("s1").is_none());
        assert!(reg_b.get("s1").is_some());
    }

    #[test]
    fn instance_contention_duplicate_id_rejected() {
        let id: TowerInstanceId = "default".parse().unwrap();
        let mut dir = InstanceDirectory::default();
        dir.insert(TowerHandle::scaffold(id.clone())).unwrap();
        assert!(dir.insert(TowerHandle::scaffold(id)).is_err());
    }
}
