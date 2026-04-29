-- B-Trace Database Schema
-- PostgreSQL 15+ with pgcrypto extension

-- Enable required extensions
CREATE EXTENSION IF NOT EXISTS "pgcrypto";
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- ============================================================================
-- SUPPLIER PROFILE TABLE
-- ============================================================================
CREATE TABLE IF NOT EXISTS supplier_profile (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    phone VARCHAR(15) NOT NULL UNIQUE,
    pincode VARCHAR(6) NOT NULL,
    business_type VARCHAR(50) NOT NULL,
    gst_number VARCHAR(15),
    is_verified BOOLEAN DEFAULT FALSE,
    device_public_key BYTEA,
    consent_preferences JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_supplier_phone ON supplier_profile(phone);
CREATE INDEX idx_supplier_pincode ON supplier_profile(pincode);
CREATE INDEX idx_supplier_verified ON supplier_profile(is_verified);

-- ============================================================================
-- MATERIAL PASSPORT TABLE
-- ============================================================================
CREATE TABLE IF NOT EXISTS material_passport (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    material_type VARCHAR(50) NOT NULL,
    batch_weight_kg DECIMAL(10,2) NOT NULL CHECK (batch_weight_kg > 0),
    material_grade VARCHAR(20) NOT NULL,
    source_pincode VARCHAR(6) NOT NULL,
    supplier_id UUID NOT NULL REFERENCES supplier_profile(id) ON DELETE CASCADE,
    buyer_id UUID REFERENCES supplier_profile(id) ON DELETE SET NULL,
    metadata JSONB DEFAULT '{}'::jsonb,
    cbam_fields JSONB DEFAULT '{}'::jsonb,
    status VARCHAR(20) DEFAULT 'PENDING' CHECK (status IN ('PENDING', 'CONFIRMED', 'IN_TRANSIT', 'DELIVERED', 'DISPUTED')),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_material_supplier ON material_passport(supplier_id);
CREATE INDEX idx_material_buyer ON material_passport(buyer_id);
CREATE INDEX idx_material_status ON material_passport(status);
CREATE INDEX idx_material_type ON material_passport(material_type);
CREATE INDEX idx_material_created_at ON material_passport(created_at);

-- ============================================================================
-- DIGITAL HANDSHAKE TABLE
-- ============================================================================
CREATE TABLE IF NOT EXISTS digital_handshake (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    material_id UUID NOT NULL REFERENCES material_passport(id) ON DELETE CASCADE,
    supplier_sig BYTEA NOT NULL,
    buyer_sig BYTEA NOT NULL,
    payload_hash VARCHAR(64) NOT NULL,
    hash_prev VARCHAR(64) NOT NULL,
    hash_current VARCHAR(64) NOT NULL,
    version_vector JSONB NOT NULL,
    sync_status VARCHAR(20) DEFAULT 'LOCAL' CHECK (sync_status IN ('LOCAL', 'SYNCING', 'SYNCED', 'CONFLICT', 'DISPUTED')),
    timestamp_utc TIMESTAMPTZ DEFAULT NOW(),
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_handshake_material ON digital_handshake(material_id);
CREATE INDEX idx_handshake_sync_status ON digital_handshake(sync_status);
CREATE INDEX idx_handshake_timestamp ON digital_handshake(timestamp_utc);

-- ============================================================================
-- EVENT LOG TABLE (Idempotency Tracking)
-- ============================================================================
CREATE TABLE IF NOT EXISTS event_log (
    idempotency_key VARCHAR(64) PRIMARY KEY,
    subject VARCHAR(100) NOT NULL,
    payload_hash VARCHAR(64) NOT NULL,
    processed_at TIMESTAMPTZ DEFAULT NOW(),
    status VARCHAR(20) DEFAULT 'SUCCESS' CHECK (status IN ('SUCCESS', 'FAILED', 'RETRY'))
);

CREATE INDEX idx_event_log_processed_at ON event_log(processed_at);
CREATE INDEX idx_event_log_status ON event_log(status);

-- ============================================================================
-- SCORING OUTPUT TABLE
-- ============================================================================
CREATE TABLE IF NOT EXISTS scoring_output (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    supplier_id UUID NOT NULL REFERENCES supplier_profile(id) ON DELETE CASCADE,
    ics_score INTEGER NOT NULL CHECK (ics_score BETWEEN 300 AND 900),
    risk_grade VARCHAR(5) NOT NULL CHECK (risk_grade IN ('A', 'B', 'C', 'D', 'E')),
    default_probability_90d DECIMAL(5,4) NOT NULL CHECK (default_probability_90d BETWEEN 0.001 AND 0.450),
    default_probability_180d DECIMAL(5,4) NOT NULL CHECK (default_probability_180d BETWEEN 0.001 AND 0.450),
    supply_chain_stability_index DECIMAL(5,2) NOT NULL CHECK (supply_chain_stability_index BETWEEN 0 AND 100),
    recommended_limit_inr BIGINT NOT NULL,
    pricing_spread_percent DECIMAL(5,2) NOT NULL,
    base_rate_percent DECIMAL(5,2) NOT NULL,
    final_rate_percent DECIMAL(5,2) NOT NULL,
    collateral_required BOOLEAN DEFAULT FALSE,
    methodology_version VARCHAR(50) NOT NULL,
    calculated_at TIMESTAMPTZ DEFAULT NOW(),
    expires_at TIMESTAMPTZ
);

CREATE UNIQUE INDEX idx_scoring_supplier ON scoring_output(supplier_id, calculated_at DESC);
CREATE INDEX idx_scoring_ics ON scoring_output(ics_score);
CREATE INDEX idx_scoring_grade ON scoring_output(risk_grade);

-- ============================================================================
-- CONSENT LOG TABLE (DPDP Compliance)
-- ============================================================================
CREATE TABLE IF NOT EXISTS consent_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    supplier_id UUID NOT NULL REFERENCES supplier_profile(id) ON DELETE CASCADE,
    purpose VARCHAR(100) NOT NULL,
    granted BOOLEAN NOT NULL,
    revoked_at TIMESTAMPTZ,
    expiry_date TIMESTAMPTZ,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_consent_supplier ON consent_log(supplier_id);
CREATE INDEX idx_consent_purpose ON consent_log(purpose);
CREATE INDEX idx_consent_granted ON consent_log(granted);

-- ============================================================================
-- AUDIT LOG TABLE (Immutable)
-- ============================================================================
CREATE TABLE IF NOT EXISTS audit_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    actor_id UUID,
    action VARCHAR(100) NOT NULL,
    resource_type VARCHAR(50) NOT NULL,
    resource_id UUID,
    old_value JSONB,
    new_value JSONB,
    ip_address INET,
    user_agent TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_audit_actor ON audit_log(actor_id);
CREATE INDEX idx_audit_action ON audit_log(action);
CREATE INDEX idx_audit_resource ON audit_log(resource_type, resource_id);
CREATE INDEX idx_audit_created_at ON audit_log(created_at);

-- ============================================================================
-- DEAD LETTER QUEUE TABLE
-- ============================================================================
CREATE TABLE IF NOT EXISTS dead_letter_queue (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_type VARCHAR(100) NOT NULL,
    original_payload JSONB NOT NULL,
    error_message TEXT NOT NULL,
    retry_count INTEGER DEFAULT 0,
    last_retry_at TIMESTAMPTZ,
    resolved BOOLEAN DEFAULT FALSE,
    resolved_at TIMESTAMPTZ,
    resolved_by UUID,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_dlq_event_type ON dead_letter_queue(event_type);
CREATE INDEX idx_dlq_resolved ON dead_letter_queue(resolved);
CREATE INDEX idx_dlq_created_at ON dead_letter_queue(created_at);

-- ============================================================================
-- TRIGGER FUNCTIONS FOR UPDATED_AT
-- ============================================================================
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Apply triggers to tables with updated_at
CREATE TRIGGER update_supplier_profile_updated_at
    BEFORE UPDATE ON supplier_profile
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_material_passport_updated_at
    BEFORE UPDATE ON material_passport
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- ============================================================================
-- INITIAL DATA SEEDING (Optional - for development)
-- ============================================================================
-- No hardcoded data inserted per requirements
