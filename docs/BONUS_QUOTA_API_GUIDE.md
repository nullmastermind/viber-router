# Bonus Quota API Guide

This guide is for developers building AI service providers that integrate with Viber Router's bonus subscription quota system.

---

## Overview

When a user has an active bonus subscription, Viber Router calls your `quota_url` endpoint to fetch current quota utilization. The response is shown on the user's public usage page.

- Method: `GET`
- Timeout: 5 seconds
- Called by: Viber Router's backend on each public usage page load

---

## Request

Viber Router sends a plain GET request to your configured URL:

```
GET https://your-service.example.com/quota
```

Custom headers (e.g. for auth) can be configured via `bonus_quota_headers` in the admin UI.

---

## Response

Return `200 OK` with JSON:

```json
{
  "quotas": [
    {
      "name": "Daily requests",
      "utilization": 0.75,
      "reset_at": "2026-04-26T00:00:00Z",
      "description": "Max 1000 requests per day"
    },
    {
      "name": "Monthly tokens",
      "utilization": 0.42,
      "reset_at": "2026-05-01T00:00:00Z"
    }
  ]
}
```

**Fields:**

| Field | Type | Required | Description |
|---|---|---|---|
| `name` | string | yes | Display name for the quota |
| `utilization` | float | yes | Used fraction: `0.0` = none used, `1.0` = fully used |
| `reset_at` | string | no | ISO 8601 datetime when quota resets |
| `description` | string | no | Optional detail text |

---

## Error Handling

If your endpoint is unreachable, times out, or returns a non-2xx status, Viber Router treats it as an empty quota list — no error is shown to the user. Returning `{ "quotas": [] }` is also acceptable.

---

## Code Example

**Python (FastAPI):**

```python
from fastapi import FastAPI, Request
from datetime import datetime, timezone

app = FastAPI()

@app.get("/quota")
async def get_quota(request: Request):
    # Optionally validate auth header
    token = request.headers.get("Authorization")
    if token != "Bearer your-secret-token":
        return {"quotas": []}

    return {
        "quotas": [
            {
                "name": "Daily requests",
                "utilization": 0.75,
                "reset_at": "2026-04-26T00:00:00Z",
                "description": "750 / 1000 requests used today"
            }
        ]
    }
```

**Node.js (Express):**

```js
app.get('/quota', (req, res) => {
  res.json({
    quotas: [
      {
        name: 'Daily requests',
        utilization: 0.75,
        reset_at: '2026-04-26T00:00:00Z',
        description: '750 / 1000 requests used today'
      }
    ]
  })
})
```

---

## Configuration (Admin UI)

In the Viber Router admin panel, when assigning a bonus subscription:

1. Set **Quota URL** to your endpoint (e.g. `https://your-service.example.com/quota`)
2. Set **Quota Headers** (optional) as a JSON object for any auth headers:
   ```json
   { "Authorization": "Bearer your-secret-token" }
   ```

These map to `bonus_quota_url` and `bonus_quota_headers` on the subscription record.
