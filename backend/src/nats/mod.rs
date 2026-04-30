//! B-Trace NATS JetStream Integration
//! 
//! This module provides the event bus infrastructure for B-Trace protocol.
//! All write operations are published as events to NATS JetStream for
//! exactly-once processing and audit trail.

use async_nats::{Client, ConnectOptions, jetstream::{self, Context, StreamConfig, Retention, StorageType}};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tracing::{info, error, warn};
use uuid::Uuid;

/// NATS Manager for handling JetStream connections and streams
#[derive(Clone)]
pub struct NatsManager {
    client: Client,
    context: Context,
}

impl NatsManager {
    /// Create a new NATS manager with JetStream context
    pub async fn new(url: &str) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let client = async_nats::connect_with_options(
            url,
            ConnectOptions::default()
                .retry_on_initial_connect()
                .ping_interval(Duration::from_secs(5))
                .max_reconnects(10),
        )
        .await?;

        let context = jetstream::new(client.clone());

        Ok(NatsManager { client, context })
    }

    /// Get the NATS client
    pub fn client(&self) -> &Client {
        &self.client
    }

    /// Get the JetStream context
    pub fn context(&self) -> &Context {
        &self.context
    }

    /// Initialize required streams for B-Trace protocol
    pub async fn init_streams(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Main events stream for all B-Trace events
        self.context.get_or_create_stream(StreamConfig {
            name: "btrace_events".to_string(),
            description: Some("B-Trace Protocol Events Stream".to_string()),
            subjects: vec![
                "btrace.events.material.*".to_string(),
                "btrace.events.handshake.*".to_string(),
                "btrace.events.score.*".to_string(),
                "btrace.events.supplier.*".to_string(),
                "btrace.events.compliance.*".to_string(),
            ],
            retention: Retention::Limits,
            max_consumers: -1,
            max_messages_per_subject: 100_000,
            max_bytes: 1024 * 1024 * 1024, // 1GB
            discard: async_nats::jetstream::stream::DiscardPolicy::Old,
            storage: StorageType::File,
            num_replicas: 1,
            duplicate_window: Duration::from_secs(300), // 5 min dedup window
            ..Default::default()
        })
        .await?;

        info!("Initialized btrace_events stream");

        // Dead letter queue stream
        self.context.get_or_create_stream(StreamConfig {
            name: "btrace_dlq".to_string(),
            description: Some("B-Trace Dead Letter Queue".to_string()),
            subjects: vec!["btrace.dlq.*".to_string()],
            retention: Retention::Limits,
            max_consumers: -1,
            storage: StorageType::File,
            num_replicas: 1,
            ..Default::default()
        })
        .await?;

        info!("Initialized btrace_dlq stream");

        Ok(())
    }

    /// Publish an event to NATS JetStream
    pub async fn publish_event(
        &self,
        subject: &str,
        payload: &[u8],
        idempotency_key: Option<&str>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut headers = async_nats::HeaderMap::new();
        
        if let Some(key) = idempotency_key {
            headers.insert("Idempotency-Key", key.parse()?);
        }

        self.client
            .publish_with_headers(subject.to_string(), headers, payload.into())
            .await?;

        Ok(())
    }
}

// ============================================================================
// Event Types
// ============================================================================

/// B-Trace Protocol Events
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event_type", rename_all = "snake_case")]
pub enum BTraceEvent {
    // Material Events
    MaterialCreated(MaterialCreatedEvent),
    MaterialUpdated(MaterialUpdatedEvent),
    MaterialStatusChanged(MaterialStatusChangedEvent),
    
    // Handshake Events
    HandshakeInitiated(HandshakeInitiatedEvent),
    HandshakeConfirmed(HandshakeConfirmedEvent),
    HandshakeDisputed(HandshakeDisputedEvent),
    
    // Scoring Events
    ScoreCalculated(ScoreCalculatedEvent),
    ScoreUpdated(ScoreUpdatedEvent),
    
    // Supplier Events
    SupplierRegistered(SupplierRegisteredEvent),
    SupplierVerified(SupplierVerifiedEvent),
    SupplierProfileUpdated(SupplierProfileUpdatedEvent),
    
    // Compliance Events
    ComplianceReportGenerated(ComplianceReportGeneratedEvent),
    ConsentGranted(ConsentGrantedEvent),
    ConsentRevoked(ConsentRevokedEvent),
}

impl BTraceEvent {
    /// Get the subject for this event type
    pub fn subject(&self) -> String {
        match self {
            BTraceEvent::MaterialCreated(_) => "btrace.events.material.created".to_string(),
            BTraceEvent::MaterialUpdated(_) => "btrace.events.material.updated".to_string(),
            BTraceEvent::MaterialStatusChanged(_) => "btrace.events.material.status_changed".to_string(),
            BTraceEvent::HandshakeInitiated(_) => "btrace.events.handshake.initiated".to_string(),
            BTraceEvent::HandshakeConfirmed(_) => "btrace.events.handshake.confirmed".to_string(),
            BTraceEvent::HandshakeDisputed(_) => "btrace.events.handshake.disputed".to_string(),
            BTraceEvent::ScoreCalculated(_) => "btrace.events.score.calculated".to_string(),
            BTraceEvent::ScoreUpdated(_) => "btrace.events.score.updated".to_string(),
            BTraceEvent::SupplierRegistered(_) => "btrace.events.supplier.registered".to_string(),
            BTraceEvent::SupplierVerified(_) => "btrace.events.supplier.verified".to_string(),
            BTraceEvent::SupplierProfileUpdated(_) => "btrace.events.supplier.profile_updated".to_string(),
            BTraceEvent::ComplianceReportGenerated(_) => "btrace.events.compliance.report_generated".to_string(),
            BTraceEvent::ConsentGranted(_) => "btrace.events.consent.granted".to_string(),
            BTraceEvent::ConsentRevoked(_) => "btrace.events.consent.revoked".to_string(),
        }
    }

    /// Generate idempotency key for this event
    pub fn idempotency_key(&self) -> String {
        use sha2::{Digest, Sha256};
        let payload = serde_json::to_vec(self).unwrap_or_default();
        let mut hasher = Sha256::new();
        hasher.update(&payload);
        hex::encode(hasher.finalize())
    }

    /// Serialize event to JSON bytes
    pub fn to_json_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(self)
    }
}

// ============================================================================
// Material Events
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialCreatedEvent {
    pub material_id: Uuid,
    pub supplier_id: Uuid,
    pub material_type: String,
    pub batch_weight_kg: rust_decimal::Decimal,
    pub material_grade: String,
    pub source_pincode: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub device_fingerprint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialUpdatedEvent {
    pub material_id: Uuid,
    pub supplier_id: Uuid,
    pub updated_fields: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialStatusChangedEvent {
    pub material_id: Uuid,
    pub old_status: String,
    pub new_status: String,
    pub changed_by: Uuid,
    pub reason: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

// ============================================================================
// Handshake Events
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandshakeInitiatedEvent {
    pub handshake_id: Uuid,
    pub material_id: Uuid,
    pub supplier_id: Uuid,
    pub buyer_id: Uuid,
    pub payload_hash: String,
    pub hash_prev: String,
    pub version_vector: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandshakeConfirmedEvent {
    pub handshake_id: Uuid,
    pub material_id: Uuid,
    pub supplier_id: Uuid,
    pub buyer_id: Uuid,
    pub supplier_sig: String,
    pub buyer_sig: String,
    pub payload_hash: String,
    pub hash_prev: String,
    pub hash_current: String,
    pub version_vector: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandshakeDisputedEvent {
    pub handshake_id: Uuid,
    pub material_id: Uuid,
    pub disputed_by: Uuid,
    pub reason: String,
    pub evidence: Option<serde_json::Value>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

// ============================================================================
// Scoring Events
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreCalculatedEvent {
    pub supplier_id: Uuid,
    pub ics_score: i32,
    pub risk_grade: String,
    pub default_probability_90d: f64,
    pub default_probability_180d: f64,
    pub stability_index: f64,
    pub recommended_limit_inr: f64,
    pub final_rate_percent: f64,
    pub methodology_version: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreUpdatedEvent {
    pub supplier_id: Uuid,
    pub old_score: i32,
    pub new_score: i32,
    pub reason: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

// ============================================================================
// Supplier Events
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupplierRegisteredEvent {
    pub supplier_id: Uuid,
    pub phone: String,
    pub role: String,
    pub pincode: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupplierVerifiedEvent {
    pub supplier_id: Uuid,
    pub verified_by: Uuid,
    pub verification_type: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupplierProfileUpdatedEvent {
    pub supplier_id: Uuid,
    pub updated_fields: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

// ============================================================================
// Compliance Events
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceReportGeneratedEvent {
    pub report_id: Uuid,
    pub supplier_id: Uuid,
    pub report_type: String, // CBAM, EPR, GST
    pub format: String,      // PDF, CSV, JSON
    pub generated_by: Uuid,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsentGrantedEvent {
    pub consent_id: Uuid,
    pub supplier_id: Uuid,
    pub purpose: String,
    pub granted_to: Uuid,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsentRevokedEvent {
    pub consent_id: Uuid,
    pub supplier_id: Uuid,
    pub purpose: String,
    pub revoked_at: chrono::DateTime<chrono::Utc>,
}
