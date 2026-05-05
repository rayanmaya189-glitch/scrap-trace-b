//! B-Trace Event Consumer Service
//! 
//! This module implements the event consumer that processes events from NATS JetStream
//! and persists them to PostgreSQL with exactly-once semantics.

use async_nats::jetstream::{self, consumer::{PushConsumer, AckPolicy}, stream::DeliverPolicy};
use sqlx::{PgPool, Row};
use std::sync::Arc;
use std::time::Duration;
use tracing::{info, error, warn, debug};

use crate::nats::{NatsManager, BTraceEvent};
use crate::utils::error::AppResult;
use crate::utils::crypto::validate_hash_chain;

/// Event Consumer for processing B-Trace events
pub struct EventConsumer {
    nats_manager: Arc<NatsManager>,
    db_pool: PgPool,
}

impl EventConsumer {
    /// Create a new event consumer
    pub fn new(nats_manager: Arc<NatsManager>, db_pool: PgPool) -> Self {
        EventConsumer {
            nats_manager,
            db_pool,
        }
    }

    /// Start consuming events from all subjects
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Starting B-Trace event consumer");

        // Create durable consumer for btrace_events stream
        let stream = self.nats_manager.context().get_stream("btrace_events").await?;
        
        let consumer = stream.create_consumer(jetstream::consumer::pull::Config {
            durable_name: Some("btrace_event_processor".to_string()),
            ack_policy: AckPolicy::Explicit,
            ack_wait: Duration::from_secs(30),
            max_deliver: 5,
            filter_subjects: vec!["btrace.events.*".to_string()],
            deliver_policy: DeliverPolicy::All,
            ..Default::default()
        })
        .await?;

        info!("Created durable consumer 'btrace_event_processor'");

        // Process messages in a loop
        loop {
            match consumer.fetch().max_messages(10).timeout(Duration::from_secs(5)).await {
                Ok(mut messages) => {
                    while let Some(msg) = messages.next().await {
                        if let Err(e) = self.process_message(msg).await {
                            error!("Failed to process message: {}", e);
                            // Message will be redelivered due to no ack
                        }
                    }
                }
                Err(async_nats::RequestError::TimedOut) => {
                    // No messages available, continue polling
                    debug!("No messages available, polling again");
                }
                Err(e) => {
                    error!("Error fetching messages: {}", e);
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            }
        }
    }

    /// Process a single NATS message
    async fn process_message(
        &self,
        msg: async_nats::Message,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let subject = msg.subject.clone();
        
        // Parse the event
        let event: BTraceEvent = match serde_json::from_slice(&msg.payload) {
            Ok(e) => e,
            Err(e) => {
                error!("Failed to parse event from {}: {}", subject, e);
                // Don't ack malformed messages - they'll go to DLQ after max_deliver
                return Err(Box::new(e));
            }
        };

        // Get idempotency key from headers or generate one
        let idempotency_key = msg
            .headers
            .and_then(|h| h.get("Idempotency-Key").map(|v| v.to_string()))
            .unwrap_or_else(|| event.idempotency_key());

        // Check for duplicate processing
        let is_duplicate = self.check_idempotency(&idempotency_key).await?;
        if is_duplicate {
            info!("Duplicate event detected (key: {}), skipping", idempotency_key);
            msg.ack().await?;
            return Ok(());
        }

        // Process the event based on type
        let result = self.handle_event(event).await;

        match result {
            Ok(_) => {
                // Record successful processing
                self.record_event_processing(&idempotency_key, &subject).await?;
                
                // Acknowledge the message
                msg.ack().await?;
                info!("Successfully processed event from {}", subject);
            }
            Err(e) => {
                error!("Failed to process event: {}", e);
                // Don't ack - let NATS redeliver
                return Err(e);
            }
        }

        Ok(())
    }

    /// Check if an event has already been processed (idempotency check)
    async fn check_idempotency(&self, key: &str) -> Result<bool, sqlx::Error> {
        let exists = sqlx::query_scalar!(
            "SELECT EXISTS(SELECT 1 FROM event_log WHERE idempotency_key = $1)",
            key
        )
        .fetch_one(&self.db_pool)
        .await?;

        Ok(exists)
    }

    /// Record successful event processing
    async fn record_event_processing(
        &self,
        idempotency_key: &str,
        subject: &str,
    ) -> Result<(), sqlx::Error> {
        use sha2::{Digest, Sha256};
        
        // We don't have the original payload here, so we hash the key
        let mut hasher = Sha256::new();
        hasher.update(idempotency_key.as_bytes());
        let payload_hash = hex::encode(hasher.finalize());

        sqlx::query!(
            "INSERT INTO event_log (idempotency_key, subject, payload_hash, processed_at) 
             VALUES ($1, $2, $3, NOW())",
            idempotency_key,
            subject,
            payload_hash
        )
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    /// Handle an event based on its type
    async fn handle_event(&self, event: BTraceEvent) -> AppResult<()> {
        match event {
            BTraceEvent::MaterialCreated(e) => self.handle_material_created(e).await?,
            BTraceEvent::MaterialUpdated(e) => self.handle_material_updated(e).await?,
            BTraceEvent::MaterialStatusChanged(e) => self.handle_material_status_changed(e).await?,
            BTraceEvent::HandshakeInitiated(e) => self.handle_handshake_initiated(e).await?,
            BTraceEvent::HandshakeConfirmed(e) => self.handle_handshake_confirmed(e).await?,
            BTraceEvent::HandshakeDisputed(e) => self.handle_handshake_disputed(e).await?,
            BTraceEvent::ScoreCalculated(e) => self.handle_score_calculated(e).await?,
            BTraceEvent::ScoreUpdated(e) => self.handle_score_updated(e).await?,
            BTraceEvent::SupplierRegistered(e) => self.handle_supplier_registered(e).await?,
            BTraceEvent::SupplierVerified(e) => self.handle_supplier_verified(e).await?,
            BTraceEvent::SupplierProfileUpdated(e) => self.handle_supplier_profile_updated(e).await?,
            BTraceEvent::ComplianceReportGenerated(e) => self.handle_compliance_report_generated(e).await?,
            BTraceEvent::ConsentGranted(e) => self.handle_consent_granted(e).await?,
            BTraceEvent::ConsentRevoked(e) => self.handle_consent_revoked(e).await?,
        }

        Ok(())
    }

    // ========================================================================
    // Event Handlers
    // ========================================================================

    async fn handle_material_created(
        &self,
        event: crate::nats::MaterialCreatedEvent,
    ) -> AppResult<()> {
        info!("Processing MaterialCreated event for material {}", event.material_id);

        // Material is already created by the API handler before publishing
        // Here we just log and potentially trigger downstream actions
        
        sqlx::query!(
            "INSERT INTO audit_log (actor_id, action, resource_type, resource_id, details, created_at)
             VALUES ($1, $2, $3, $4, $5, $6)",
            event.supplier_id,
            "material.created",
            "material_passport",
            event.material_id,
            serde_json::json!({
                "material_type": event.material_type,
                "batch_weight_kg": event.batch_weight_kg.to_string(),
                "grade": event.material_grade,
                "source_pincode": event.source_pincode
            }),
            event.timestamp
        )
        .execute(&self.db_pool)
        .await
        .map_err(|e| crate::utils::error::AppError::DatabaseError(e))?;

        Ok(())
    }

    async fn handle_material_updated(
        &self,
        event: crate::nats::MaterialUpdatedEvent,
    ) -> AppResult<()> {
        info!("Processing MaterialUpdated event for material {}", event.material_id);

        sqlx::query!(
            "INSERT INTO audit_log (actor_id, action, resource_type, resource_id, details, created_at)
             VALUES ($1, $2, $3, $4, $5, $6)",
            event.supplier_id,
            "material.updated",
            "material_passport",
            event.material_id,
            &event.updated_fields,
            event.timestamp
        )
        .execute(&self.db_pool)
        .await
        .map_err(|e| crate::utils::error::AppError::DatabaseError(e))?;

        Ok(())
    }

    async fn handle_material_status_changed(
        &self,
        event: crate::nats::MaterialStatusChangedEvent,
    ) -> AppResult<()> {
        info!(
            "Processing MaterialStatusChanged event for material {}: {} -> {}",
            event.material_id, event.old_status, event.new_status
        );

        sqlx::query!(
            "INSERT INTO audit_log (actor_id, action, resource_type, resource_id, details, created_at)
             VALUES ($1, $2, $3, $4, $5, $6)",
            event.changed_by,
            "material.status_changed",
            "material_passport",
            event.material_id,
            serde_json::json!({
                "old_status": event.old_status,
                "new_status": event.new_status,
                "reason": event.reason
            }),
            event.timestamp
        )
        .execute(&self.db_pool)
        .await
        .map_err(|e| crate::utils::error::AppError::DatabaseError(e))?;

        Ok(())
    }

    async fn handle_handshake_initiated(
        &self,
        event: crate::nats::HandshakeInitiatedEvent,
    ) -> AppResult<()> {
        info!("Processing HandshakeInitiated event for handshake {}", event.handshake_id);

        sqlx::query!(
            "INSERT INTO audit_log (actor_id, action, resource_type, resource_id, details, created_at)
             VALUES ($1, $2, $3, $4, $5, $6)",
            event.supplier_id,
            "handshake.initiated",
            "digital_handshake",
            event.handshake_id,
            serde_json::json!({
                "material_id": event.material_id,
                "buyer_id": event.buyer_id,
                "payload_hash": event.payload_hash
            }),
            event.timestamp
        )
        .execute(&self.db_pool)
        .await
        .map_err(|e| crate::utils::error::AppError::DatabaseError(e))?;

        Ok(())
    }

    async fn handle_handshake_confirmed(
        &self,
        event: crate::nats::HandshakeConfirmedEvent,
    ) -> AppResult<()> {
        info!("Processing HandshakeConfirmed event for handshake {}", event.handshake_id);


        // Validate hash chain integrity - check if hash_prev matches previous handshake's hash_current
        let previous_hash_current: Option<String> = sqlx::query_scalar!(
            "SELECT hash_current FROM digital_handshake WHERE material_id = $1 ORDER BY timestamp_utc DESC LIMIT 1",
            event.material_id
        )
        .fetch_optional(&self.db_pool)
        .await
        .map_err(|e| crate::utils::error::AppError::DatabaseError(e))?;

        if let Some(prev_hash) = previous_hash_current {
            // For existing materials with previous handshakes, validate the chain
            if !validate_hash_chain(&event.hash_prev, &prev_hash) {
                return Err(crate::utils::error::AppError::BadRequest(
                    format!("Hash chain validation failed: hash_prev does not match previous handshake's hash_current for material {}", event.material_id)
                ));
            }
            info!("Hash chain validated successfully for material {}", event.material_id);
        } else {
            // First handshake for this material - hash_prev should be zeros or genesis hash
            info!("First handshake for material {}, skipping chain validation", event.material_id);
        }

        // Insert the handshake record into the database
        sqlx::query!(
            "INSERT INTO digital_handshake 
             (id, material_id, supplier_sig, buyer_sig, payload_hash, hash_prev, hash_current, version_vector, sync_status, timestamp_utc)
             VALUES ($1, $2, decode($3, 'hex'), decode($4, 'hex'), $5, $6, $7, $8, 'SYNCED', $9)",
            event.handshake_id,
            event.material_id,
            event.supplier_sig,
            event.buyer_sig,
            event.payload_hash,
            event.hash_prev,
            event.hash_current,
            &event.version_vector,
            event.timestamp
        )
        .execute(&self.db_pool)
        .await
        .map_err(|e| crate::utils::error::AppError::DatabaseError(e))?;

        // Update material status
        sqlx::query!(
            "UPDATE material_passport SET status = 'CONFIRMED', updated_at = $1 WHERE id = $2",
            event.timestamp,
            event.material_id
        )
        .execute(&self.db_pool)
        .await
        .map_err(|e| crate::utils::error::AppError::DatabaseError(e))?;

        // Log audit trail
        sqlx::query!(
            "INSERT INTO audit_log (actor_id, action, resource_type, resource_id, details, created_at)
             VALUES ($1, $2, $3, $4, $5, $6)",
            event.buyer_id,
            "handshake.confirmed",
            "digital_handshake",
            event.handshake_id,
            serde_json::json!({
                "material_id": event.material_id,
                "supplier_id": event.supplier_id
            }),
            event.timestamp
        )
        .execute(&self.db_pool)
        .await
        .map_err(|e| crate::utils::error::AppError::DatabaseError(e))?;

        Ok(())
    }

    async fn handle_handshake_disputed(
        &self,
        event: crate::nats::HandshakeDisputedEvent,
    ) -> AppResult<()> {
        info!("Processing HandshakeDisputed event for handshake {}", event.handshake_id);

        // Update handshake status to DISPUTED
        sqlx::query!(
            "UPDATE digital_handshake SET sync_status = 'DISPUTED' WHERE id = $1",
            event.handshake_id
        )
        .execute(&self.db_pool)
        .await
        .map_err(|e| crate::utils::error::AppError::DatabaseError(e))?;

        // Update material status
        sqlx::query!(
            "UPDATE material_passport SET status = 'DISPUTED' WHERE id = $1",
            event.material_id
        )
        .execute(&self.db_pool)
        .await
        .map_err(|e| crate::utils::error::AppError::DatabaseError(e))?;

        // Log audit trail
        sqlx::query!(
            "INSERT INTO audit_log (actor_id, action, resource_type, resource_id, details, created_at)
             VALUES ($1, $2, $3, $4, $5, $6)",
            event.disputed_by,
            "handshake.disputed",
            "digital_handshake",
            event.handshake_id,
            serde_json::json!({
                "material_id": event.material_id,
                "reason": event.reason,
                "evidence": event.evidence
            }),
            event.timestamp
        )
        .execute(&self.db_pool)
        .await
        .map_err(|e| crate::utils::error::AppError::DatabaseError(e))?;

        Ok(())
    }

    async fn handle_score_calculated(
        &self,
        event: crate::nats::ScoreCalculatedEvent,
    ) -> AppResult<()> {
        info!("Processing ScoreCalculated event for supplier {}", event.supplier_id);

        // Insert or update scoring output
        sqlx::query!(
            "INSERT INTO scoring_output 
             (supplier_id, ics_score, risk_grade, default_probability_90d, default_probability_180d, 
              stability_index, recommended_limit_inr, pricing_spread_percent, base_rate_percent, 
              final_rate_percent, collateral_required, methodology_version, calculated_at, expires_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
             ON CONFLICT (supplier_id) DO UPDATE SET
                ics_score = EXCLUDED.ics_score,
                risk_grade = EXCLUDED.risk_grade,
                default_probability_90d = EXCLUDED.default_probability_90d,
                default_probability_180d = EXCLUDED.default_probability_180d,
                stability_index = EXCLUDED.stability_index,
                recommended_limit_inr = EXCLUDED.recommended_limit_inr,
                pricing_spread_percent = EXCLUDED.pricing_spread_percent,
                base_rate_percent = EXCLUDED.base_rate_percent,
                final_rate_percent = EXCLUDED.final_rate_percent,
                collateral_required = EXCLUDED.collateral_required,
                methodology_version = EXCLUDED.methodology_version,
                calculated_at = EXCLUDED.calculated_at,
                expires_at = EXCLUDED.expires_at",
            event.supplier_id,
            event.ics_score,
            event.risk_grade,
            rust_decimal::Decimal::from_f64_retain(event.default_probability_90d).unwrap_or(rust_decimal::Decimal::ZERO),
            rust_decimal::Decimal::from_f64_retain(event.default_probability_180d).unwrap_or(rust_decimal::Decimal::ZERO),
            rust_decimal::Decimal::from_f64_retain(event.stability_index).unwrap_or(rust_decimal::Decimal::ZERO),
            rust_decimal::Decimal::from_f64_retain(event.recommended_limit_inr).unwrap_or(rust_decimal::Decimal::ZERO),
            rust_decimal::Decimal::from_f64_retain(0.0).unwrap_or(rust_decimal::Decimal::ZERO), // pricing_spread calculated elsewhere
            rust_decimal::Decimal::from_f64_retain(event.final_rate_percent).unwrap_or(rust_decimal::Decimal::ZERO),
            rust_decimal::Decimal::from_f64_retain(event.final_rate_percent).unwrap_or(rust_decimal::Decimal::ZERO),
            event.collateral_required,
            event.methodology_version,
            event.timestamp,
            Some(event.timestamp + chrono::Duration::days(90))
        )
        .execute(&self.db_pool)
        .await
        .map_err(|e| crate::utils::error::AppError::DatabaseError(e))?;

        // Log audit trail
        sqlx::query!(
            "INSERT INTO audit_log (action, resource_type, resource_id, details, created_at)
             VALUES ($1, $2, $3, $4, $5)",
            "score.calculated",
            "scoring_output",
            event.supplier_id,
            serde_json::json!({
                "ics_score": event.ics_score,
                "risk_grade": event.risk_grade,
                "stability_index": event.stability_index
            }),
            event.timestamp
        )
        .execute(&self.db_pool)
        .await
        .map_err(|e| crate::utils::error::AppError::DatabaseError(e))?;

        Ok(())
    }

    async fn handle_score_updated(
        &self,
        event: crate::nats::ScoreUpdatedEvent,
    ) -> AppResult<()> {
        info!(
            "Processing ScoreUpdated event for supplier {}: {} -> {}",
            event.supplier_id, event.old_score, event.new_score
        );

        // Log to audit trail
        sqlx::query!(
            "INSERT INTO audit_log (action, resource_type, resource_id, details, created_at)
             VALUES ($1, $2, $3, $4, $5)",
            "score.updated",
            "scoring_output",
            event.supplier_id,
            serde_json::json!({
                "old_score": event.old_score,
                "new_score": event.new_score,
                "reason": event.reason
            }),
            event.timestamp
        )
        .execute(&self.db_pool)
        .await
        .map_err(|e| crate::utils::error::AppError::DatabaseError(e))?;

        Ok(())
    }

    async fn handle_supplier_registered(
        &self,
        event: crate::nats::SupplierRegisteredEvent,
    ) -> AppResult<()> {
        info!("Processing SupplierRegistered event for supplier {}", event.supplier_id);

        sqlx::query!(
            "INSERT INTO audit_log (actor_id, action, resource_type, resource_id, details, created_at)
             VALUES ($1, $2, $3, $4, $5, $6)",
            event.supplier_id,
            "supplier.registered",
            "supplier_profile",
            event.supplier_id,
            serde_json::json!({
                "phone": event.phone,
                "role": event.role,
                "pincode": event.pincode
            }),
            event.timestamp
        )
        .execute(&self.db_pool)
        .await
        .map_err(|e| crate::utils::error::AppError::DatabaseError(e))?;

        Ok(())
    }

    async fn handle_supplier_verified(
        &self,
        event: crate::nats::SupplierVerifiedEvent,
    ) -> AppResult<()> {
        info!("Processing SupplierVerified event for supplier {}", event.supplier_id);

        // Update supplier verification status
        sqlx::query!(
            "UPDATE supplier_profile SET is_verified = TRUE, updated_at = $1 WHERE id = $2",
            event.timestamp,
            event.supplier_id
        )
        .execute(&self.db_pool)
        .await
        .map_err(|e| crate::utils::error::AppError::DatabaseError(e))?;

        // Log audit trail
        sqlx::query!(
            "INSERT INTO audit_log (actor_id, action, resource_type, resource_id, details, created_at)
             VALUES ($1, $2, $3, $4, $5, $6)",
            event.verified_by,
            "supplier.verified",
            "supplier_profile",
            event.supplier_id,
            serde_json::json!({
                "verification_type": event.verification_type
            }),
            event.timestamp
        )
        .execute(&self.db_pool)
        .await
        .map_err(|e| crate::utils::error::AppError::DatabaseError(e))?;

        Ok(())
    }

    async fn handle_supplier_profile_updated(
        &self,
        event: crate::nats::SupplierProfileUpdatedEvent,
    ) -> AppResult<()> {
        info!("Processing SupplierProfileUpdated event for supplier {}", event.supplier_id);

        sqlx::query!(
            "INSERT INTO audit_log (actor_id, action, resource_type, resource_id, details, created_at)
             VALUES ($1, $2, $3, $4, $5, $6)",
            event.supplier_id,
            "supplier.profile_updated",
            "supplier_profile",
            event.supplier_id,
            &event.updated_fields,
            event.timestamp
        )
        .execute(&self.db_pool)
        .await
        .map_err(|e| crate::utils::error::AppError::DatabaseError(e))?;

        Ok(())
    }

    async fn handle_compliance_report_generated(
        &self,
        event: crate::nats::ComplianceReportGeneratedEvent,
    ) -> AppResult<()> {
        info!(
            "Processing ComplianceReportGenerated event: {} for supplier {}",
            event.report_type, event.supplier_id
        );

        sqlx::query!(
            "INSERT INTO audit_log (actor_id, action, resource_type, details, created_at)
             VALUES ($1, $2, $3, $4, $5)",
            event.generated_by,
            "compliance.report_generated",
            "compliance_report",
            serde_json::json!({
                "report_id": event.report_id,
                "report_type": event.report_type,
                "format": event.format,
                "supplier_id": event.supplier_id
            }),
            event.timestamp
        )
        .execute(&self.db_pool)
        .await
        .map_err(|e| crate::utils::error::AppError::DatabaseError(e))?;

        Ok(())
    }

    async fn handle_consent_granted(
        &self,
        event: crate::nats::ConsentGrantedEvent,
    ) -> AppResult<()> {
        info!(
            "Processing ConsentGranted event: {} for supplier {}",
            event.purpose, event.supplier_id
        );

        // Insert consent record
        sqlx::query!(
            "INSERT INTO consent_log (id, supplier_id, purpose, granted, revoked_at, created_at)
             VALUES ($1, $2, $3, TRUE, NULL, $4)",
            event.consent_id,
            event.supplier_id,
            event.purpose,
            event.timestamp
        )
        .execute(&self.db_pool)
        .await
        .map_err(|e| crate::utils::error::AppError::DatabaseError(e))?;

        // Log audit trail
        sqlx::query!(
            "INSERT INTO audit_log (actor_id, action, resource_type, resource_id, details, created_at)
             VALUES ($1, $2, $3, $4, $5, $6)",
            event.supplier_id,
            "consent.granted",
            "consent_log",
            event.consent_id,
            serde_json::json!({
                "purpose": event.purpose,
                "granted_to": event.granted_to,
                "expires_at": event.expires_at
            }),
            event.timestamp
        )
        .execute(&self.db_pool)
        .await
        .map_err(|e| crate::utils::error::AppError::DatabaseError(e))?;

        Ok(())
    }

    async fn handle_consent_revoked(
        &self,
        event: crate::nats::ConsentRevokedEvent,
    ) -> AppResult<()> {
        info!(
            "Processing ConsentRevoked event: {} for supplier {}",
            event.purpose, event.supplier_id
        );

        // Update consent record
        sqlx::query!(
            "UPDATE consent_log SET revoked_at = $1 WHERE supplier_id = $2 AND purpose = $3",
            event.revoked_at,
            event.supplier_id,
            event.purpose
        )
        .execute(&self.db_pool)
        .await
        .map_err(|e| crate::utils::error::AppError::DatabaseError(e))?;

        // Log audit trail
        sqlx::query!(
            "INSERT INTO audit_log (actor_id, action, resource_type, details, created_at)
             VALUES ($1, $2, $3, $4, $5)",
            event.supplier_id,
            "consent.revoked",
            "consent_log",
            serde_json::json!({
                "purpose": event.purpose
            }),
            event.revoked_at
        )
        .execute(&self.db_pool)
        .await
        .map_err(|e| crate::utils::error::AppError::DatabaseError(e))?;

        Ok(())
    }
}
