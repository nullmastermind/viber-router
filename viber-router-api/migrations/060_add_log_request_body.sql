-- Whether the proxy stores the full request body in proxy logs (used to rebuild
-- the cURL command on the Proxy Error Logs page). Request bodies can be large and
-- expensive to store, so this defaults to false: when off, an empty JSON object
-- "{}" is stored instead of the real body. Headers are unaffected.
ALTER TABLE settings
    ADD COLUMN IF NOT EXISTS log_request_body BOOLEAN NOT NULL DEFAULT false;
