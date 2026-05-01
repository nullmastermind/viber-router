## Purpose
TBD

## Requirements
### Requirement: Server has a supported_models filter list
Each server assignment in a group SHALL have a `supported_models` field (array of model name strings). An empty array means the server accepts all models. A non-empty array means the server only accepts requests whose model matches an entry in the list or a key in the server's `model_mappings`.

#### Scenario: Empty supported_models — server accepts all models
- **WHEN** a server assignment has `supported_models = []`
- **THEN** the proxy SHALL NOT skip that server based on the requested model

#### Scenario: Non-empty supported_models — model in list
- **WHEN** a server assignment has `supported_models = ["gpt-4o", "gpt-4o-mini"]` and the request model is `"gpt-4o"`
- **THEN** the proxy SHALL NOT skip that server based on the model filter

#### Scenario: Non-empty supported_models — model not in list
- **WHEN** a server assignment has `supported_models = ["gpt-4o"]` and the request model is `"claude-3-5-sonnet"`
- **THEN** the proxy SHALL skip that server silently and continue to the next server in the failover chain

#### Scenario: Model covered by model_mappings key — implicitly supported
- **WHEN** a server assignment has `supported_models = ["gpt-4o"]` and `model_mappings = {"claude-3-5-sonnet": "gpt-4o"}` and the request model is `"claude-3-5-sonnet"`
- **THEN** the proxy SHALL NOT skip that server (the mapping key implies support)

#### Scenario: No model in request — filter not applied
- **WHEN** a server assignment has a non-empty `supported_models` list and the request body contains no `model` field
- **THEN** the proxy SHALL NOT skip that server based on the model filter

#### Scenario: All servers filtered — exhausted error returned
- **WHEN** every enabled server in a group has a non-empty `supported_models` list that excludes the requested model
- **THEN** the proxy SHALL return the existing "all servers exhausted" error response (no new error type)

### Requirement: supported_models check uses original request model
The proxy SHALL evaluate `supported_models` against the model name from the original request body, before any `model_mappings` transformation is applied.

#### Scenario: Check uses pre-mapping model name
- **WHEN** a server has `model_mappings = {"claude-3-5-sonnet": "gpt-4o"}` and `supported_models = ["claude-3-5-sonnet"]` and the request model is `"claude-3-5-sonnet"`
- **THEN** the proxy SHALL NOT skip the server (original model matches the list)
