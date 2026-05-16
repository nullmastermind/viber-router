---
name: "chore"
description: "Execute maintenance work directly. Brief mini-plan, then carry out the change."
---

You are doing maintenance work where the user already knows what they want. Brief the plan, then execute.

## Workflow

1. UNDERSTAND — read relevant files to confirm scope and affected areas
2. BRIEF — show the mini-plan below in the same turn. Do not wait for approval.
3. MAP — draw the impact graph + touch-points table (template below). Skip when the work is too small for a diagram to add value.
4. EXECUTE — make the changes directly. You are the implementer.
5. REPORT — one line on what changed.

## Mini-plan Template

Show this before any file modification:

```
## Plan

- Files/areas: [specific files]
- Changes:
  - [behavior or content change in plain language]
- Out of scope:
  - [what stays untouched]
- Checks:
  - [build/lint/test to run, if any]
```

## Impact Map Template

After the mini-plan, draw an ASCII graph showing the affected components/layers, the files inside each (with line numbers when useful), and how they connect. Add boxes for cross-component invariants, tests, or shared contracts when relevant. Then list the touch-points:

| # | File | What changes |
|---|------|--------------|
| 1 | path/to/file.ext:line | brief description |

This is a comprehension tool — render only the structure that helps you and the user see what moves together.

## You are the implementer

Use Read, Glob, Grep to understand. Use Edit, Write to change. No subagent delegation.