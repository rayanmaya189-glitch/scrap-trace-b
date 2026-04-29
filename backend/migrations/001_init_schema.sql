-- B-Trace Protocol v1.0 - Initial Schema Migration
-- Creates core tables for supplier management, material traceability, digital handshakes, and credit scoring

-- Enable required extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- Supplier Profile Table
CREATE TABLE IF NOT EXISTS supplier_profile (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    phone VARCHAR(15) NOT NULL UNIQUE,
    name VARCHAR(100),
    business_name VARCHAR(200),
    role VARCHAR(50) NOT NULL CHECK (role IN ('dealer', 'buyer', 'exporter', 'nbfc', 'auditor')),
    pincode VARCHAR(6),
    gst_number VARCHAR(15),
    is_verified BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_supplier_phone ON supplier_profile(phone);
CREATE INDEX idx_supplier_role ON supplier_profile(role);
CREATE INDEX idx_supplier_verified ON supplier_profile(is_verified);

-- Material Passport Table
CREATE TABLE IF NOT EXISTS material_passport (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    material_type VARCHAR(50) NOT NULL,
    batch_weight_kg DECIMAL(10,2) NOT NULL CHECK (batch_weight_kg > 0),
    material_grade VARCHAR(20) NOT NULL,
    source_pincode VARCHAR(6) NOT NULL,
    supplier_id UUID NOT NULL REFERENCES supplier_profile(id) ON DELETE CASCADE,
    buyer_id UUID REFERENCES supplier_profile(id),
    metadata JSONB DEFAULT '{}'::jsonb,
    cbam_fields JSONB DEFAULT '{}'::jsonb,
    status VARCHAR(20) DEFAULT 'PENDING' CHECK (status IN ('PENDING', 'CONFIRMED', 'DISPUTED', 'COMPLETED', 'CANCELLED')),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_material_supplier ON material_passport(supplier_id);
CREATE INDEX idx_material_buyer ON material_passport(buyer_id);
CREATE INDEX idx_material_status ON material_passport(status);
CREATE INDEX idx_material_created ON material_passport(created_at DESC);

-- Digital Handshake Table
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
    timestamp_utc TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_handshake_material ON digital_handshake(material_id);
CREATE INDEX idx_handshake_sync ON digital_handshake(sync_status);
CREATE INDEX idx_handshake_timestamp ON digital_handshake(timestamp_utc DESC);

-- Scoring Output Table
CREATE TABLE IF NOT EXISTS scoring_output (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    supplier_id UUID NOT NULL UNIQUE REFERENCES supplier_profile(id) ON DELETE CASCADE,
    ics_score INTEGER NOT NULL CHECK (ics_score >= 300 AND ics_score <= 900),
    risk_grade VARCHAR(5) NOT NULL CHECK (risk_grade IN ('A', 'B', 'C', 'D', 'E')),
    default_probability_90d DECIMAL(5,4),
    default_probability_180d DECIMAL(5,4),
    stability_index DECIMAL(5,2),
    recommended_limit_inr DECIMAL(15,2),
    pricing_spread_percent DECIMAL(5,2),
    base_rate_percent DECIMAL(5,2),
    final_rate_percent DECIMAL(5,2),
    collateral_required BOOLEAN DEFAULT FALSE,
    methodology_version VARCHAR(50) NOT NULL,
    calculated_at TIMESTAMPTZ DEFAULT NOW(),
    expires_at TIMESTAMPTZ
);

CREATE INDEX idx_scoring_supplier ON scoring_output(supplier_id);
CREATE INDEX idx_scoring_grade ON scoring_output(risk_grade);
CREATE INDEX idx_scoring_expires ON scoring_output(expires_at);

-- Consent Log Table
CREATE TABLE IF NOT EXISTS consent_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    supplier_id UUID NOT NULL REFERENCES supplier_profile(id) ON DELETE CASCADE,
    purpose VARCHAR(100) NOT NULL,
    granted BOOLEAN NOT NULL DEFAULT TRUE,
    revoked_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_consent_supplier ON consent_log(supplier_id);
CREATE INDEX idx_consent_purpose ON consent_log(purpose);

-- Event Log Table (for idempotency)
CREATE TABLE IF NOT EXISTS event_log (
    idempotency_key VARCHAR(64) PRIMARY KEY,
    subject VARCHAR(100) NOT NULL,
    payload_hash VARCHAR(64) NOT NULL,
    processed_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_event_processed ON event_log(processed_at DESC);

-- Audit Log Table
CREATE TABLE IF NOT EXISTS audit_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    actor_id UUID REFERENCES supplier_profile(id),
    action VARCHAR(50) NOT NULL,
    resource_type VARCHAR(50) NOT NULL,
    resource_id UUID,
    details JSONB DEFAULT '{}'::jsonb,
    ip_address INET,
    user_agent TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_audit_actor ON audit_log(actor_id);
CREATE INDEX idx_audit_action ON audit_log(action);
CREATE INDEX idx_audit_resource ON audit_log(resource_type, resource_id);
CREATE INDEX idx_audit_created ON audit_log(created_at DESC);

-- Create updated_at trigger function
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Add triggers for updated_at
DROP TRIGGER IF EXISTS update_supplier_updated_at ON supplier_profile;
CREATE TRIGGER update_supplier_updated_at
    BEFORE UPDATE ON supplier_profile
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

DROP TRIGGER IF EXISTS update_material_updated_at ON material_passport;
CREATE TRIGGER update_material_updated_at
    BEFORE UPDATE ON material_passport
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

COMMENT ON TABLE supplier_profile IS 'Stores profile information for all supply chain participants';
COMMENT ON TABLE material_passport IS 'Tracks material batches with cryptographic provenance';
COMMENT ON TABLE digital_handshake IS 'Records mutual verification between suppliers and buyers';
COMMENT ON TABLE scoring_output IS 'Stores computed proxy credit scores for suppliers';
COMMENT ON TABLE consent_log IS 'Audit trail for DPDP compliance and data access consent';
COMMENT ON TABLE event_log IS 'Idempotency tracking for exactly-once event processing';
COMMENT ON TABLE audit_log IS 'Comprehensive audit trail for security and compliance';
