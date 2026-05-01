# Frontend Manual Verification Notes

## Plans page

- Confirmed the plan data model includes nullable `tpm_limit` and the plans table defines a TPM Limit column.
- Confirmed the create/edit dialog includes a clearable TPM Limit numeric input with an unlimited/null state.
- Confirmed edit population copies `row.tpm_limit` into form state, and reset clears it back to `null`.
- Confirmed create/update payloads include `tpm_limit` as a finite number when set and `null` when empty/unlimited.
- Confirmed each plan row includes a Sync TPM action that calls `POST /api/admin/subscription-plans/{id}/sync-tpm`, reloads plans on success, and reports the updated subscription count.

## Group detail page

- Confirmed the key subscription data model includes nullable `tpm_limit`.
- Confirmed the subscription table defines a TPM Limit column and formats `null` as an unlimited/empty em dash value.

## Automated check coverage

- `just check` covers `vue-tsc --noEmit` and `biome lint ./src`, validating the added Vue/TypeScript TPM fields, columns, payloads, and sync handler compile and lint cleanly.
