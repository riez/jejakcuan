# Data Handling Procedures

## Purpose

This document outlines procedures for handling personal data in compliance with Indonesia's PDP Law.

## Data Collection

### Consent Collection

```typescript
// Required consent fields
interface ConsentRecord {
  user_id: string;
  consent_type: 'registration' | 'marketing' | 'analytics' | 'notifications';
  granted: boolean;
  timestamp: Date;
  ip_address: string;
  version: string;
}
```

### Minimum Data Principle

Only collect data necessary for service:

| Service | Required Data | Optional Data |
|---------|--------------|---------------|
| Account | Email, Password | Name, Phone |
| Watchlist | Stock symbols | Notes |
| Alerts | Alert config | Webhook URL |
| Portfolio | Holdings | Cost basis |

## Data Access

### Access Request Handling

1. **Verify Identity** - 2FA required for data requests
2. **Log Request** - Audit event created
3. **Process** - Within 3x24 hours
4. **Deliver** - Secure download link (24h expiry)

### Export Format

```json
{
  "export_metadata": {
    "user_id": "user_123",
    "generated_at": "2024-01-15T10:00:00Z",
    "format_version": "1.0"
  },
  "profile": { ... },
  "watchlist": [ ... ],
  "alerts": [ ... ],
  "activity_log": [ ... ]
}
```

## Data Modification

### Update Procedures

1. Validate input data
2. Log previous value (audit)
3. Apply update
4. Confirm to user
5. Sync dependent systems

### Deletion Procedures

```sql
-- Soft delete (immediate)
UPDATE users SET deleted_at = NOW(), email = CONCAT('deleted_', id) 
WHERE id = $1;

-- Data anonymization (scheduled)
UPDATE audit_logs SET actor = '{"user_id": "anonymized"}' 
WHERE actor->>'user_id' = $1 AND timestamp < NOW() - INTERVAL '90 days';

-- Hard delete (after retention period)
DELETE FROM users WHERE deleted_at < NOW() - INTERVAL '30 days';
```

## Data Storage

### Classification

| Level | Examples | Controls |
|-------|----------|----------|
| Public | Stock prices | None required |
| Internal | Aggregated analytics | Access control |
| Confidential | User profiles | Encryption + ACL |
| Restricted | Passwords, tokens | Encryption + Hashing |

### Encryption Requirements

```yaml
at_rest:
  algorithm: AES-256-GCM
  key_rotation: 90 days
  
in_transit:
  protocol: TLS 1.3
  ciphers:
    - TLS_AES_256_GCM_SHA384
    - TLS_CHACHA20_POLY1305_SHA256

passwords:
  algorithm: Argon2id
  memory: 64 MB
  iterations: 3
  parallelism: 4
```

## Data Sharing

### Internal Sharing

- Need-to-know basis
- Role-based access enforced
- Audit log required

### External Sharing

1. **Check legal basis** - Consent or legal requirement
2. **Verify recipient** - DPA in place
3. **Minimize data** - Only required fields
4. **Secure transfer** - Encrypted channel
5. **Log transfer** - Audit record

### API Data Exposure

```rust
// Never expose in API responses:
// - password_hash
// - reset_tokens
// - api_secrets
// - internal_ids (use public_id)

#[derive(Serialize)]
pub struct UserResponse {
    pub id: Uuid,           // Public ID
    pub email: String,
    pub name: Option<String>,
    // Excluded: password_hash, created_at, etc.
}
```

## Incident Response

### Classification

| Severity | Criteria | Response Time |
|----------|----------|---------------|
| Critical | Active breach, mass data | 1 hour |
| High | Potential breach, sensitive data | 4 hours |
| Medium | Policy violation, limited data | 24 hours |
| Low | Minor issue, no data exposed | 72 hours |

### Response Steps

```
1. IDENTIFY
   - What happened?
   - What data affected?
   - How many users?

2. CONTAIN
   - Stop ongoing breach
   - Preserve evidence
   - Isolate affected systems

3. ERADICATE
   - Remove threat
   - Patch vulnerability
   - Reset credentials

4. RECOVER
   - Restore services
   - Verify integrity
   - Monitor closely

5. NOTIFY
   - Regulatory body (72h)
   - Affected users
   - Management

6. REVIEW
   - Root cause analysis
   - Update procedures
   - Document lessons
```

## Training Requirements

### All Staff

- PDP Law overview
- Data handling basics
- Incident reporting
- Annual refresh

### Technical Staff

- Secure coding practices
- Encryption implementation
- Audit logging
- Access control

### Customer Support

- Data request handling
- Identity verification
- Escalation procedures

## Audit Schedule

| Audit Type | Frequency | Scope |
|------------|-----------|-------|
| Access review | Monthly | User permissions |
| Log review | Weekly | Security events |
| Compliance check | Quarterly | Full procedures |
| External audit | Annual | Independent assessment |

## Document Control

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2024-01-15 | JejakCuan | Initial release |
