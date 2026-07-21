//! In-process session residency registry.
//!
//! This is intentionally not a second session runtime. It only tracks which
//! opaque Shell-owned actor tokens are resident for each Session ID. Mutations
//! that affect Session content remain on the existing `SessionActor` path
//! injected through [`crate::GrokRuntimeFacade`].

use std::collections::HashMap;

use crate::RuntimeError;

/// Opaque token for a Shell-owned actor. Tower never constructs actors itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ActorToken(u64);

impl ActorToken {
    pub fn as_u64(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResidentSession {
    pub session_id: String,
    pub actor: ActorToken,
}

/// One resident actor token per Session ID.
#[derive(Debug, Default)]
pub struct SessionRegistry {
    residents: HashMap<String, ActorToken>,
    next_token: u64,
}

impl SessionRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.residents.len()
    }

    pub fn is_empty(&self) -> bool {
        self.residents.is_empty()
    }

    pub fn get(&self, session_id: &str) -> Option<ActorToken> {
        self.residents.get(session_id).copied()
    }

    /// Insert a new resident only when the Session is absent.
    /// Returns `(token, created)`.
    pub fn get_or_insert_with<F>(
        &mut self,
        session_id: &str,
        create: F,
    ) -> Result<(ActorToken, bool), RuntimeError>
    where
        F: FnOnce(ActorToken) -> Result<(), RuntimeError>,
    {
        if let Some(existing) = self.residents.get(session_id).copied() {
            return Ok((existing, false));
        }
        self.next_token = self.next_token.checked_add(1).ok_or_else(|| RuntimeError {
            code: "registry_exhausted",
            message: "actor token space exhausted".into(),
        })?;
        let token = ActorToken(self.next_token);
        create(token)?;
        self.residents.insert(session_id.to_owned(), token);
        Ok((token, true))
    }

    pub fn remove(&mut self, session_id: &str) -> Option<ActorToken> {
        self.residents.remove(session_id)
    }

    pub fn residents(&self) -> impl Iterator<Item = ResidentSession> + '_ {
        self.residents
            .iter()
            .map(|(session_id, actor)| ResidentSession {
                session_id: session_id.clone(),
                actor: *actor,
            })
    }
}

#[cfg(test)]
mod one_actor_tests {
    use super::*;
    use std::sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    };

    #[test]
    fn one_actor_per_session_id_serializes_residency() {
        let mut registry = SessionRegistry::new();
        let creations = Arc::new(AtomicUsize::new(0));
        let bump = || {
            let creations = creations.clone();
            move |_token| {
                creations.fetch_add(1, Ordering::SeqCst);
                Ok(())
            }
        };

        let (a, created_a) = registry.get_or_insert_with("session_1", bump()).unwrap();
        let (b, created_b) = registry.get_or_insert_with("session_1", bump()).unwrap();
        let (c, created_c) = registry.get_or_insert_with("session_2", bump()).unwrap();

        assert!(created_a);
        assert!(!created_b);
        assert!(created_c);
        assert_eq!(a, b);
        assert_ne!(a, c);
        assert_eq!(creations.load(Ordering::SeqCst), 2);
        assert_eq!(registry.len(), 2);
    }

    #[test]
    fn one_actor_concurrent_get_or_insert_collapses_to_single_token() {
        let registry = Arc::new(Mutex::new(SessionRegistry::new()));
        let creations = Arc::new(AtomicUsize::new(0));
        let mut handles = Vec::new();
        for _ in 0..16 {
            let registry = registry.clone();
            let creations = creations.clone();
            handles.push(std::thread::spawn(move || {
                let mut guard = registry.lock().unwrap();
                guard
                    .get_or_insert_with("session_race", |_| {
                        creations.fetch_add(1, Ordering::SeqCst);
                        Ok(())
                    })
                    .unwrap()
            }));
        }
        let tokens: Vec<ActorToken> = handles.into_iter().map(|h| h.join().unwrap().0).collect();
        assert_eq!(creations.load(Ordering::SeqCst), 1);
        assert!(tokens.iter().all(|t| *t == tokens[0]));
        assert_eq!(registry.lock().unwrap().len(), 1);
    }
}
