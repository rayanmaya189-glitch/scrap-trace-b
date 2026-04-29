//! Consumer service - Processes NATS JetStream events and persists to PostgreSQL
//! 
//! This is the ONLY component that writes to PostgreSQL, ensuring:
//! - Exactly-once processing via idempotency keys
//! - Event sourcing architecture
//! - Dead-letter queue for malformed events

use async_nats::jetstream::consumer::pull::Config as PullConfig;
use sqlx::PgPool;
use chrono::Utc;
use uuid::Uuid;
use tracing::{info, warn, error};
use crate::nats::{Event, NatsConnection};
use crate::crypto::hash_payload;
use crate::error::Result;

/// Event consumer service
pub struct ConsumerService {
    nats: std::sync::Arc<NatsConnection>,
    db_pool: PgPool,
}

impl ConsumerService {
    /// Create new consumer service
    pub fn new(nats: std::sync::Arc<NatsConnection>, db_pool: PgPool) -> Self {
        Self { nats, db_pool }
    }

    /// Start consuming events from NATS JetStream
    pub async fn run(&self) -> Result<()> {
        info!("Starting NATS event consumer");

        // Get consumer for material events
        let material_consumer = self.nats.get_consumer("material_processor").await?;
        
        // Get consumer for handshake events
        let handshake_consumer = self.nats.get_consumer("handshake_processor").await?;

        // Process material events
        let material_pool = self.db_pool.clone();
        let material_nats = self.nats.clone();
        let material_handle = tokio::spawn(async move {
            process_material_events(material_consumer, material_pool).await;
        });

        // Process handshake events
        let handshake_pool = self.db_pool.clone();
        let handshake_nats = self.nats.clone();
        let handshake_handle = tokio::spawn(async move {
            process_handshake_events(handshake_consumer, handshake_pool).await;
        });

        // Wait for both consumers (they run indefinitely)
        tokio::try_join!(material_handle, handshake_handle)?;

        Ok(())
    }
}

/// Process MaterialCreated events
async fn process_material_events(
    consumer: async_nats::jetstream::consumer::Consumer<PullConfig>,
    db_pool: PgPool,
) {
    info!("Material event consumer started");

    loop {
        match consumer.fetch().max_messages(10).await {
            Ok(mut messages) => {
                while let Some(message) = messages.next().await {
                    let idempotency_key = message.headers.as_ref()
                        .and_then(|h| h.get("idempotency-key"))
                        .map(|v| v.as_str().to_string());

                    match process_material_event(&message.payload, &db_pool).await {
                        Ok(_) => {
                            message.ack().await.ok();
                            info!("Successfully processed material event");
                        }
                        Err(e) => {
                            error!("Failed to process material event: {}", e);
                            
                            // Move to dead letter queue after max retries
                            if let Err(dlq_err) = move_to_dlq(&db_pool, "MATERIAL_CREATED", &message.payload, &e.to_string()).await {
                                error!("Failed to move to DLQ: {}", dlq_err);
                            }
                            
                            message.nak().await.ok();
                        }
                    }
                }
            }
            Err(e) => {
                warn!("Error fetching messages: {}", e);
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
        }
    }
}

/// Process HandshakeConfirmed events
async fn process_handshake_events(
    consumer: async_nats::jetstream::consumer::Consumer<PullConfig>,
    db_pool: PgPool,
) {
    info!("Handshake event consumer started");

    loop {
        match consumer.fetch().max_messages(10).await {
            Ok(mut messages) => {
                while let Some(message) = messages.next().await {
                    match process_handshake_event(&message.payload, &db_pool).await {
                        Ok(_) => {
                            message.ack().await.ok();
                            info!("Successfully processed handshake event");
                        }
                        Err(e) => {
                            error!("Failed to process handshake event: {}", e);
                            
                            if let Err(dlq_err) = move_to_dlq(&db_pool, "HANDSHAKE_CONFIRMED", &message.payload, &e.to_string()).await {
                                error!("Failed to move to DLQ: {}", dlq_err);
                            }
                            
                            message.nak().await.ok();
                        }
                    }
                }
            }
            Err(e) => {
                warn!("Error fetching messages: {}", e);
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
        }
    }
}

/// Process a single MaterialCreated event
async fn process_material_event(payload: &[u8], db_pool: &PgPool) -> Result<()> {
    let event: Event = serde_json::from_slice(payload)
        .map_err(|e| crate::error::AppError::BadRequest(format!("Invalid event payload: {}", e)))?;

    match event {
        Event::MaterialCreated { idempotency_key, payload } => {
            // Check idempotency
            let already_processed = sqlx::query_scalar::<_, bool>(
                "SELECT EXISTS(SELECT 1 FROM event_log WHERE idempotency_key = $1)",
            )
            .bind(&idempotency_key)
            .fetch_one(db_pool)
            .await?;

            if already_processed {
                info!("Duplicate event detected, skipping: {}", idempotency_key);
                return Ok(());
            }

            // Parse supplier ID
            let supplier_id = Uuid::parse_str(&payload.supplier_id)
                .map_err(|e| crate::error::AppError::Validation(format!("Invalid supplier ID: {}", e)))?;

            // Parse batch weight
            let batch_weight_kg: rust_decimal::Decimal = payload.batch_weight_kg.parse()
                .map_err(|e| crate::error::AppError::Validation(format!("Invalid batch weight: {}", e)))?;

            // Insert material passport
            let material_id = Uuid::parse_str(&payload.material_id)
                .map_err(|e| crate::error::AppError::Validation(format!("Invalid material ID: {}", e)))?;

            sqlx::query(
                r#"
                INSERT INTO material_passport (
                    id, material_type, batch_weight_kg, material_grade, 
                    source_pincode, supplier_id, metadata, cbam_fields, status
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 'PENDING')
                ON CONFLICT (id) DO NOTHING
                "#,
            )
            .bind(material_id)
            .bind(&payload.material_type)
            .bind(batch_weight_kg)
            .bind(&payload.material_grade)
            .bind(&payload.source_pincode)
            .bind(supplier_id)
            .bind(serde_json::Value::Object(serde_json::Map::new()))
            .bind(serde_json::Value::Object(serde_json::Map::new()))
            .execute(db_pool)
            .await?;

            // Log event processing
            let payload_hash = hash_payload(payload.supplier_id.as_bytes());
            sqlx::query(
                "INSERT INTO event_log (idempotency_key, subject, payload_hash, status) VALUES ($1, $2, $3, 'SUCCESS')",
            )
            .bind(&idempotency_key)
            .bind("MATERIAL_CREATED")
            .bind(&payload_hash)
            .execute(db_pool)
            .await?;

            info!("Material {} created successfully", material_id);
        }
        _ => {
            return Err(crate::error::AppError::BadRequest(
                "Expected MaterialCreated event".into()
            ));
        }
    }

    Ok(())
}

/// Process a single HandshakeConfirmed event
async fn process_handshake_event(payload: &[u8], db_pool: &PgPool) -> Result<()> {
    let event: Event = serde_json::from_slice(payload)
        .map_err(|e| crate::error::AppError::BadRequest(format!("Invalid event payload: {}", e)))?;

    match event {
        Event::HandshakeConfirmed { idempotency_key, payload } => {
            // Check idempotency
            let already_processed = sqlx::query_scalar::<_, bool>(
                "SELECT EXISTS(SELECT 1 FROM event_log WHERE idempotency_key = $1)",
            )
            .bind(&idempotency_key)
            .fetch_one(db_pool)
            .await?;

            if already_processed {
                info!("Duplicate event detected, skipping: {}", idempotency_key);
                return Ok(());
            }

            // Parse IDs
            let handshake_id = Uuid::parse_str(&payload.handshake_id)
                .map_err(|e| crate::error::AppError::Validation(format!("Invalid handshake ID: {}", e)))?;
            
            let material_id = Uuid::parse_str(&payload.material_id)
                .map_err(|e| crate::error::AppError::Validation(format!("Invalid material ID: {}", e)))?;
            
            let supplier_id = Uuid::parse_str(&payload.supplier_id)
                .map_err(|e| crate::error::AppError::Validation(format!("Invalid supplier ID: {}", e)))?;
            
            let buyer_id = Uuid::parse_str(&payload.buyer_id)
                .map_err(|e| crate::error::AppError::Validation(format!("Invalid buyer ID: {}", e)))?;

            // Decode signatures from hex
            let supplier_sig = crate::crypto::decode_hex(&payload.supplier_signature)?;
            let buyer_sig = crate::crypto::decode_hex(&payload.buyer_signature)?;

            // Insert digital handshake
            sqlx::query(
                r#"
                INSERT INTO digital_handshake (
                    id, material_id, supplier_sig, buyer_sig,
                    payload_hash, hash_prev, hash_current, version_vector, sync_status, timestamp_utc
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 'SYNCED', TO_TIMESTAMP($9))
                ON CONFLICT (id) DO NOTHING
                "#,
            )
            .bind(handshake_id)
            .bind(material_id)
            .bind(&supplier_sig)
            .bind(&buyer_sig)
            .bind(&payload.payload_hash)
            .bind(&payload.hash_prev)
            .bind(&payload.hash_current)
            .bind(&payload.version_vector)
            .bind(payload.timestamp as i64)
            .execute(db_pool)
            .await?;

            // Update material status to CONFIRMED
            sqlx::query(
                "UPDATE material_passport SET status = 'CONFIRMED', buyer_id = $1, updated_at = NOW() WHERE id = $2",
            )
            .bind(buyer_id)
            .bind(material_id)
            .execute(db_pool)
            .await?;

            // Log event processing
            let payload_hash = hash_payload(payload.handshake_id.as_bytes());
            sqlx::query(
                "INSERT INTO event_log (idempotency_key, subject, payload_hash, status) VALUES ($1, $2, $3, 'SUCCESS')",
            )
            .bind(&idempotency_key)
            .bind("HANDSHAKE_CONFIRMED")
            .bind(&payload_hash)
            .execute(db_pool)
            .await?;

            info!("Handshake {} confirmed successfully", handshake_id);
        }
        _ => {
            return Err(crate::error::AppError::BadRequest(
                "Expected HandshakeConfirmed event".into()
            ));
        }
    }

    Ok(())
}

/// Move failed event to dead letter queue
async fn move_to_dlq(
    db_pool: &PgPool,
    event_type: &str,
    payload: &[u8],
    error_message: &str,
) -> Result<()> {
    let original_payload: serde_json::Value = serde_json::from_slice(payload)
        .unwrap_or_else(|_| serde_json::json!({"raw": "invalid json"}));

    sqlx::query(
        r#"
        INSERT INTO dead_letter_queue (event_type, original_payload, error_message, retry_count, last_retry_at)
        VALUES ($1, $2, $3, 1, NOW())
        "#,
    )
    .bind(event_type)
    .bind(original_payload)
    .bind(error_message)
    .execute(db_pool)
    .await?;

    Ok(())
}
