use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::cmp::Ordering;

/// Version Vector for CRDT Conflict Resolution
/// Tracks logical clocks per device to determine causality
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VersionVector {
    pub clocks: HashMap<String, u64>, // DeviceID -> Clock
}

impl VersionVector {
    pub fn new() -> Self {
        Self {
            clocks: HashMap::new(),
        }
    }

    pub fn increment(&mut self, device_id: &str) {
        let entry = self.clocks.entry(device_id.to_string()).or_insert(0);
        *entry += 1;
    }

    pub fn merge(&mut self, other: &VersionVector) {
        for (device_id, clock) in &other.clocks {
            let entry = self.clocks.entry(device_id.clone()).or_insert(0);
            *entry = (*entry).max(*clock);
        }
    }

    /// Compare two version vectors
    /// Returns:
    /// - Ordering::Less if self < other (self is older)
    /// - Ordering::Greater if self > other (self is newer)
    /// - Ordering::Equal if identical
    /// - None if concurrent (conflict detected)
    pub fn compare(&self, other: &VersionVector) -> Option<Ordering> {
        let mut self_greater = false;
        let mut other_greater = false;

        // Check all keys in self
        for (key, val) in &self.clocks {
            let other_val = other.clocks.get(key).unwrap_or(&0);
            match val.cmp(other_val) {
                Ordering::Greater => self_greater = true,
                Ordering::Less => other_greater = true,
                Ordering::Equal => {}
            }
        }

        // Check keys only in other
        for (key, val) in &other.clocks {
            if !self.clocks.contains_key(key) {
                other_greater = true;
            }
        }

        match (self_greater, other_greater) {
            (true, false) => Some(Ordering::Greater),
            (false, true) => Some(Ordering::Less),
            (false, false) => Some(Ordering::Equal),
            (true, true) => None, // Concurrent update -> Conflict
        }
    }
}

/// Conflict Resolution Strategy Result
#[derive(Debug, Clone)]
pub enum ConflictResolution<T> {
    NoConflict(T),          // Dominant version found
    Conflict(T, T),         // Concurrent versions (needs manual resolution or specific rule)
    MergeSuccess(T),        // Successfully merged
}

/// Resolver for Handshake Conflicts
pub struct HandshakeResolver;

impl HandshakeResolver {
    /// Resolve conflict based on "Last Writer Wins" with device priority tie-breaker
    /// In B-Trace: If concurrent, prefer the one with more signatures or earlier timestamp if equal
    pub fn resolve<T: Clone>(
        v1: &VersionVector,
        v2: &VersionVector,
        payload1: &T,
        payload2: &T,
    ) -> ConflictResolution<T> {
        match v1.compare(v2) {
            Some(Ordering::Greater) => ConflictResolution::NoConflict(payload1.clone()),
            Some(Ordering::Less) => ConflictResolution::NoConflict(payload2.clone()),
            Some(Ordering::Equal) => ConflictResolution::NoConflict(payload1.clone()),
            None => {
                // Concurrent: In a real scenario, we might merge fields or flag for manual review.
                // For MVP, we flag as conflict but return the one with the lexicographically larger ID if payloads have IDs
                // Here we simply return Conflict to be handled by the consumer
                ConflictResolution::Conflict(payload1.clone(), payload2.clone())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_vector_increment() {
        let mut vv = VersionVector::new();
        vv.increment("device_A");
        assert_eq!(vv.clocks.get("device_A"), Some(&1));
        vv.increment("device_A");
        assert_eq!(vv.clocks.get("device_A"), Some(&2));
    }

    #[test]
    fn test_version_vector_merge() {
        let mut vv1 = VersionVector::new();
        vv1.increment("device_A"); // 1
        
        let mut vv2 = VersionVector::new();
        vv2.increment("device_B"); // 1
        vv2.increment("device_B"); // 2

        vv1.merge(&vv2);
        assert_eq!(vv1.clocks.get("device_A"), Some(&1));
        assert_eq!(vv1.clocks.get("device_B"), Some(&2));
    }

    #[test]
    fn test_concurrent_detection() {
        let mut vv1 = VersionVector::new();
        vv1.increment("A");

        let mut vv2 = VersionVector::new();
        vv2.increment("B");

        // A has 1, B has 0 vs A has 0, B has 1 -> Concurrent
        assert_eq!(vv1.compare(&vv2), None);
    }
}
