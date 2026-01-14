# PDP Law Compliance Documentation

## Overview

JejakCuan implements comprehensive data protection measures compliant with Indonesia's Personal Data Protection Law (UU No. 27 Tahun 2022 tentang Perlindungan Data Pribadi).

## Data Categories

### Personal Data Processed

| Category | Data Type | Purpose | Retention |
|----------|-----------|---------|-----------|
| User Identity | Name, Email | Account management | Active + 5 years |
| Authentication | Password hash, Sessions | Security | Active + 1 year |
| Contact | Phone number | Notifications | Active + 1 year |
| Financial | Portfolio, Watchlist | Core service | Active + 5 years |
| Usage | Activity logs, IP address | Analytics, Security | 90 days |

### Sensitive Data

- No biometric data collected
- No health data collected
- No political/religious data collected

## Legal Basis for Processing

1. **Consent** - User registration agreement
2. **Contractual Necessity** - Service delivery
3. **Legal Obligation** - Financial record keeping (OJK requirements)
4. **Legitimate Interest** - Security, fraud prevention

## Data Subject Rights

JejakCuan supports all PDP Law required rights:

### 1. Right to Information (Hak Informasi)
- Privacy policy accessible at `/privacy`
- Data processing purposes clearly stated
- Third-party sharing disclosed

### 2. Right to Access (Hak Akses)
- Users can view all personal data via Profile settings
- Data export available in JSON/CSV format
- Request processing: within 3x24 hours

### 3. Right to Rectification (Hak Koreksi)
- Users can update profile information directly
- Support for data correction requests
- Audit trail maintained

### 4. Right to Erasure (Hak Penghapusan)
- Account deletion available via settings
- Data anonymization for retained analytics
- Processing: within 3x24 hours

### 5. Right to Portability (Hak Portabilitas)
- Data export in machine-readable format
- Includes: portfolio, watchlist, alerts, activity

### 6. Right to Withdraw Consent (Hak Menarik Persetujuan)
- Granular consent management
- Marketing opt-out available
- Non-essential processing can be disabled

### 7. Right to Object (Hak Keberatan)
- Profiling opt-out available
- Automated decision-making disclosure

## Data Security Measures

### Technical Controls

```
1. Encryption
   - Data at rest: AES-256
   - Data in transit: TLS 1.3
   - Password hashing: Argon2id

2. Access Control
   - Role-based access (RBAC)
   - Multi-factor authentication available
   - Session management with secure tokens

3. Network Security
   - WAF protection
   - DDoS mitigation
   - Rate limiting

4. Monitoring
   - Comprehensive audit logging
   - Anomaly detection
   - Real-time alerting
```

### Organizational Controls

1. **Data Protection Officer** - Designated DPO
2. **Access Management** - Need-to-know basis
3. **Employee Training** - Annual privacy training
4. **Incident Response** - 72-hour notification SLA
5. **Vendor Management** - DPA with all processors

## Data Retention Policy

| Data Category | Retention Period | Basis |
|---------------|------------------|-------|
| Authentication logs | 1 year | Security |
| Data access logs | 180 days | PDP Compliance |
| Security incident logs | 2 years | Legal requirement |
| API access logs | 90 days | Operations |
| User account data | Active + 5 years | Legal requirement |
| Transaction records | 5 years | OJK requirement |

### Retention Implementation

```rust
// crates/audit/src/retention.rs
RetentionPolicy::pdp_compliant() {
    auth_logs_days: 365,
    data_access_days: 180,
    security_logs_days: 730,
    api_logs_days: 90,
    default_days: 180,
}
```

## Data Processing Records

### Internal Processing

| Process | Purpose | Legal Basis |
|---------|---------|-------------|
| Stock tracking | Core service | Contract |
| Alert notifications | User service | Consent |
| Portfolio analysis | Feature | Consent |
| Security monitoring | Protection | Legitimate interest |

### Third-Party Processors

| Processor | Purpose | Location | DPA |
|-----------|---------|----------|-----|
| IDX | Market data | Indonesia | Yes |
| KSEI | Shareholding | Indonesia | Yes |
| Sectors.app | Analytics | Indonesia | Yes |
| Telegram | Notifications | Global | Yes |

## Cross-Border Data Transfer

### Current Status
- Primary data storage: Indonesia
- No cross-border transfer by default
- International services (Telegram) use user consent

### Safeguards
- Adequacy assessment required
- Standard contractual clauses
- User notification for transfers

## Data Breach Response

### Notification Procedure

1. **Detection** - Automated monitoring + manual reporting
2. **Assessment** - Within 24 hours of detection
3. **Containment** - Immediate action
4. **Notification**:
   - Regulatory: Within 72 hours
   - Data subjects: Without undue delay
5. **Remediation** - Root cause analysis, fixes
6. **Documentation** - Complete incident record

### Breach Log Template

```json
{
  "incident_id": "BRH-2024-001",
  "detected_at": "2024-01-15T10:00:00Z",
  "type": "unauthorized_access",
  "affected_data": ["email", "name"],
  "affected_count": 0,
  "severity": "low|medium|high|critical",
  "notification_sent": true,
  "resolved_at": null,
  "root_cause": "",
  "remediation": []
}
```

## Audit Trail

### Events Logged

```rust
// crates/audit/src/events.rs
EventCategory {
    Authentication,   // Login/logout events
    Authorization,    // Access control decisions
    DataAccess,       // Read operations on personal data
    DataModification, // Write operations on personal data
    SystemConfig,     // Configuration changes
    Security,         // Security events
    ApiAccess,        // API requests
    DataExport,       // Data portability requests
    Consent,          // Consent changes
}
```

### Log Integrity
- Tamper-evident logging
- Centralized log management
- Retention per policy

## Compliance Checklist

### Technical Requirements

- [x] Encryption at rest (AES-256)
- [x] Encryption in transit (TLS 1.3)
- [x] Access control (RBAC)
- [x] Audit logging
- [x] Data retention automation
- [x] Consent management
- [x] Data export capability
- [x] Account deletion
- [x] Anomaly detection

### Organizational Requirements

- [ ] Privacy policy published
- [ ] DPO appointed
- [ ] DPIA conducted
- [ ] Staff training completed
- [ ] Vendor DPAs signed
- [ ] Incident response tested
- [ ] Annual compliance review

## Contact

**Data Protection Officer**
- Email: dpo@jejakcuan.id
- Phone: +62-xxx-xxx-xxxx

**Regulatory Authority**
- Kementerian Komunikasi dan Informatika
- https://kominfo.go.id

## Document History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2024-01-15 | Initial release |

---

*This document is reviewed quarterly and updated as regulations change.*
