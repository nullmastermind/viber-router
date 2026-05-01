## ADDED Requirements

### Requirement: Subscription tables show TPM limit
The subscription tables in the group detail page SHALL display each key subscription's optional `tpm_limit` in a TPM Limit column.

#### Scenario: Subscription has TPM limit
- **WHEN** a key subscription has `tpm_limit: 100000`
- **THEN** the group detail subscription table SHALL display `100000` in the TPM Limit column for that subscription

#### Scenario: Subscription has no TPM limit
- **WHEN** a key subscription has `tpm_limit: null`
- **THEN** the group detail subscription table SHALL display an empty or unlimited value for the TPM Limit column
