---
name: review
description: What to review — omit for uncommitted changes, pass a PR/MR URL, or describe a feature/area to review
---

Before launching the subagent, gather context from the current conversation:

1. If user provided a scope argument (PR URL, MR URL, or feature description):
   - Pass it directly as the review scope
2. If no arguments provided:
   - Default scope is uncommitted git changes
3. If prior implementation or fix was just done in this conversation:
   - Mention what was changed so the reviewer has context

Brief the user, then launch Agent tool with `subagent_type: "osf-review"`.

Pass the scope and any relevant conversation context in the agent prompt.