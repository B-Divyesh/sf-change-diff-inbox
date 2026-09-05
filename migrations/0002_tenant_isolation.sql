ALTER TABLE sources ADD COLUMN tenant_id TEXT NOT NULL DEFAULT 'legacy';
ALTER TABLE changes ADD COLUMN tenant_id TEXT NOT NULL DEFAULT 'legacy';

CREATE TABLE IF NOT EXISTS tenants (
  id TEXT PRIMARY KEY NOT NULL,
  kind TEXT NOT NULL CHECK (kind IN ('real', 'demo')),
  created_at TEXT NOT NULL,
  expires_at TEXT
);

CREATE INDEX IF NOT EXISTS idx_sources_tenant ON sources(tenant_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_changes_tenant ON changes(tenant_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_tenants_expiry ON tenants(kind, expires_at);
