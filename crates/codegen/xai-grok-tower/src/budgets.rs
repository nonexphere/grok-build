//! Measured safety budgets — not arbitrary product Session caps.

use crate::RuntimeError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResourceBudgets {
    pub max_resident_sessions: usize,
    pub max_active_turns: usize,
    pub max_pending_loads: usize,
    pub max_event_queue: usize,
}

impl Default for ResourceBudgets {
    fn default() -> Self {
        Self {
            max_resident_sessions: 256,
            max_active_turns: 256,
            max_pending_loads: 64,
            max_event_queue: 1_024,
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ResourceUsage {
    pub resident_sessions: usize,
    pub active_turns: usize,
    pub pending_loads: usize,
    pub queued_events: usize,
    pub peak_resident_sessions: usize,
    pub peak_active_turns: usize,
}

impl ResourceUsage {
    pub fn record_resident(&mut self, delta: isize) {
        if delta >= 0 {
            self.resident_sessions += delta as usize;
        } else {
            self.resident_sessions = self.resident_sessions.saturating_sub((-delta) as usize);
        }
        self.peak_resident_sessions = self.peak_resident_sessions.max(self.resident_sessions);
    }

    pub fn record_turn(&mut self, delta: isize) {
        if delta >= 0 {
            self.active_turns += delta as usize;
        } else {
            self.active_turns = self.active_turns.saturating_sub((-delta) as usize);
        }
        self.peak_active_turns = self.peak_active_turns.max(self.active_turns);
    }
}

pub fn admit_resident(budgets: &ResourceBudgets, usage: &ResourceUsage) -> Result<(), RuntimeError> {
    if usage.resident_sessions >= budgets.max_resident_sessions {
        return Err(RuntimeError {
            code: "resource_budget_admission",
            message: format!(
                "resident session budget exhausted ({}/{})",
                usage.resident_sessions, budgets.max_resident_sessions
            ),
        });
    }
    Ok(())
}

pub fn admit_turn(budgets: &ResourceBudgets, usage: &ResourceUsage) -> Result<(), RuntimeError> {
    if usage.active_turns >= budgets.max_active_turns {
        return Err(RuntimeError {
            code: "resource_budget_admission",
            message: format!(
                "active turn budget exhausted ({}/{})",
                usage.active_turns, budgets.max_active_turns
            ),
        });
    }
    Ok(())
}

#[cfg(test)]
mod resource_budget_tests {
    use super::*;

    #[test]
    fn resource_budget_admission_fails_explicitly_without_product_cap_on_dormant() {
        let budgets = ResourceBudgets {
            max_resident_sessions: 2,
            max_active_turns: 1,
            max_pending_loads: 1,
            max_event_queue: 8,
        };
        let mut usage = ResourceUsage::default();
        usage.record_resident(1);
        admit_resident(&budgets, &usage).unwrap();
        usage.record_resident(1);
        let err = admit_resident(&budgets, &usage).unwrap_err();
        assert_eq!(err.code, "resource_budget_admission");
        // Dormant listing is unbounded by this admission helper — only residents count.
        assert_eq!(usage.peak_resident_sessions, 2);
    }

    #[test]
    fn telemetry_peaks_retain_high_water_marks() {
        let mut usage = ResourceUsage::default();
        usage.record_resident(3);
        usage.record_resident(-2);
        usage.record_turn(5);
        usage.record_turn(-4);
        assert_eq!(usage.resident_sessions, 1);
        assert_eq!(usage.active_turns, 1);
        assert_eq!(usage.peak_resident_sessions, 3);
        assert_eq!(usage.peak_active_turns, 5);
    }
}
