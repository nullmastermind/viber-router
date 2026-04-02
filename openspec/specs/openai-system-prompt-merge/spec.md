## ADDED Requirements

### Requirement: Server system_prompt injected into OpenAI messages array
When the request path starts with `/v1/chat/` and the server has a non-null `system_prompt`, the proxy SHALL inject the server system prompt into the request body's `messages` array before forwarding. If the first message has `"role": "system"`, the server prompt SHALL be appended to its `content` string with `"\n\n"` separator. If no system message exists, a new `{"role":"system","content":"<server_prompt>"}` object SHALL be prepended to the `messages` array.

#### Scenario: OpenAI request with no existing system message
- **WHEN** the request path is `/v1/chat/completions`, the server has `system_prompt: "Always respond in Vietnamese"`, and the request body has `messages: [{"role":"user","content":"Hello"}]`
- **THEN** the proxy SHALL forward `messages: [{"role":"system","content":"Always respond in Vietnamese"},{"role":"user","content":"Hello"}]`

#### Scenario: OpenAI request with existing system message
- **WHEN** the request path is `/v1/chat/completions`, the server has `system_prompt: "Always respond in Vietnamese"`, and the request body has `messages: [{"role":"system","content":"You are helpful"},{"role":"user","content":"Hello"}]`
- **THEN** the proxy SHALL forward `messages: [{"role":"system","content":"You are helpful\n\nAlways respond in Vietnamese"},{"role":"user","content":"Hello"}]`

#### Scenario: OpenAI request with no server system_prompt
- **WHEN** the request path is `/v1/chat/completions` and the server has `system_prompt: null`
- **THEN** the proxy SHALL forward the `messages` array unchanged

#### Scenario: Anthropic system prompt merge unaffected
- **WHEN** the request path is `/v1/messages`
- **THEN** the proxy SHALL use the existing Anthropic top-level `system` field merge logic unchanged
