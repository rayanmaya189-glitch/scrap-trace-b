//! NATS JetStream integration for event-driven architecture

use async_nats::{
    jetstream::{self, Context, Stream},
    Client, ConnectOptions,
};
use serde::{Serialize, Deserialize};
use crate::config::Config;
use crate::error::Result;

/// NATS event types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Event {
    MaterialCreated {
        idempotency_key: String,
        payload: MaterialCreatedPayload,
    },
    HandshakeConfirmed {
        idempotency_key: String,
        payload: HandshakeConfirmedPayload,
    },
    SyncRequested {
        idempotency_key: String,
        payload: SyncRequestedPayload,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialCreatedPayload {
    pub material_id: String,
    pub supplier_id: String,
    pub material_type: String,
    pub batch_weight_kg: String,
    pub material_grade: String,
    pub source_pincode: String,
    pub timestamp: u64,
    pub device_fingerprint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandshakeConfirmedPayload {
    pub handshake_id: String,
    pub material_id: String,
    pub supplier_id: String,
    pub buyer_id: String,
    pub supplier_signature: String,
    pub buyer_signature: String,
    pub payload_hash: String,
    pub hash_prev: String,
    pub hash_current: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncRequestedPayload {
    pub device_id: String,
    pub pending_events: Vec<String>,
}

/// NATS connection wrapper
pub struct NatsConnection {
    client: Client,
    context: Context,
    stream_name: String,
}

impl NatsConnection {
    /// Connect to NATS server and initialize JetStream
    pub async fn connect(config: &Config) -> Result<Self> {
        let client = async_nats::connect(&config.nats_url).await?;
        let context = jetstream::new(client.clone());
        
        Ok(Self {
            client,
            context,
            stream_name: config.nats_stream.clone(),
        })
    }

    /// Create or get the JetStream stream
    pub async fn ensure_stream(&self) -> Result<()> {
        self.context
            .get_or_create_stream(async_nats::jetstream::stream::Config {
                name: self.stream_name.clone(),
                subjects: vec![
                    "btrace.material.*".into(),
                    "btrace.handshake.*".into(),
                    "btrace.sync.*".into(),
                ],
                retention: async_nats::jetstream::stream::Retention::Limits,
                max_messages_per_subject: 10000,
                ..Default::default()
            })
            .await?;
        
        Ok(())
    }

    /// Publish an event to NATS JetStream
    pub async fn publish_event(&self, subject: &str, event: &Event) -> Result<()> {
        let payload = serde_json::to_vec(event)?;
        
        self.context
            .publish(subject.to_string(), payload.into())
            .await?;
        
        Ok(())
    }

    /// Get a stream consumer for processing events
    pub async fn get_consumer(&self, durable_name: &str) -> Result<jetstream::consumer::Consumer<jetstream::consumer::pull::Config>> {
        let stream = self.context.get_stream(&self.stream_name).await?;
        
        let consumer = stream
            .get_or_create_consumer(
                durable_name,
                jetstream::consumer::pull::Config {
                    durable_name: Some(durable_name.to_string()),
                    ack_policy: jetstream::consumer::AckPolicy::Explicit,
                    ack_wait: std::time::Duration::from_secs(30),
                    max_deliver: 5,
                    ..Default::default()
                },
            )
            .await?;
        
        Ok(consumer)
    }

    /// Get the underlying client
    pub fn client(&self) -> &Client {
        &self.client
    }
}
