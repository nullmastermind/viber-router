# Viber Router Installer for Codex CLI (Windows PowerShell 5.1+)
# Configures Codex CLI to use Viber Router

$ErrorActionPreference = "Stop"

# Configuration (auto-populated by server)
$ENDPOINT_URL = "{{ENDPOINT_URL}}"
$API_KEY = '{{API_KEY}}'
$CODEX_SMALL = "{{SMALL}}"
$CODEX_MEDIUM = "{{MEDIUM}}"
$CODEX_LARGE = "{{LARGE}}"

Write-Host "================================" -ForegroundColor Blue
Write-Host "  Viber Router - Codex CLI Setup" -ForegroundColor Blue
Write-Host "================================" -ForegroundColor Blue
Write-Host ""

# Validate configuration
if ($ENDPOINT_URL -eq "__" + "ENDPOINT_URL__" -or [string]::IsNullOrEmpty($ENDPOINT_URL)) {
    Write-Host "Error: Endpoint URL not configured" -ForegroundColor Red
    Write-Host "Please use the install link from your Viber Router dashboard."
    exit 1
}

if ($API_KEY -eq "__" + "API_KEY__" -or [string]::IsNullOrEmpty($API_KEY)) {
    Write-Host "Error: API key not configured" -ForegroundColor Red
    Write-Host "Please use the install link from your Viber Router dashboard."
    exit 1
}

if ([string]::IsNullOrEmpty($CODEX_SMALL)) {
    Write-Host "Error: Small model not configured" -ForegroundColor Red
    exit 1
}

if ([string]::IsNullOrEmpty($CODEX_MEDIUM)) {
    Write-Host "Error: Medium (default) model not configured" -ForegroundColor Red
    exit 1
}

if ([string]::IsNullOrEmpty($CODEX_LARGE)) {
    Write-Host "Error: Large model not configured" -ForegroundColor Red
    exit 1
}

# Mask API key for display
$MASKED_KEY = $API_KEY.Substring(0, [Math]::Min(10, $API_KEY.Length))
Write-Host "Endpoint URL:     " -NoNewline
Write-Host "$ENDPOINT_URL" -ForegroundColor Green
Write-Host "API Key:          " -NoNewline
Write-Host "$MASKED_KEY..." -ForegroundColor Green
Write-Host "Small (Fast):     " -NoNewline
Write-Host "$CODEX_SMALL" -ForegroundColor Green
Write-Host "Medium (Default): " -NoNewline
Write-Host "$CODEX_MEDIUM" -ForegroundColor Green
Write-Host "Large (Powerful): " -NoNewline
Write-Host "$CODEX_LARGE" -ForegroundColor Green
Write-Host ""

# Check Node.js >= 22
Write-Host "Checking prerequisites..." -ForegroundColor Blue

try {
    $nodeVersion = node --version 2>$null
} catch {
    $nodeVersion = $null
}

if (-not $nodeVersion) {
    Write-Host "Error: Node.js is not installed" -ForegroundColor Red
    Write-Host ""
    Write-Host "Codex CLI requires Node.js 22 or later."
    Write-Host "Download from: https://nodejs.org"
    exit 1
}

$nodeMajor = [int]($nodeVersion -replace '^v','').Split('.')[0]
if ($nodeMajor -lt 22) {
    Write-Host "Error: Node.js 22+ is required (found $nodeVersion)" -ForegroundColor Red
    Write-Host "Download from: https://nodejs.org"
    exit 1
}
Write-Host "  " -NoNewline
Write-Host "OK" -ForegroundColor Green -NoNewline
Write-Host " Node.js $nodeVersion"

# Check npm
try {
    $npmVersion = npm --version 2>$null
} catch {
    $npmVersion = $null
}

if (-not $npmVersion) {
    Write-Host "Error: npm is not installed" -ForegroundColor Red
    Write-Host "Please install npm (usually comes with Node.js)."
    exit 1
}
Write-Host "  " -NoNewline
Write-Host "OK" -ForegroundColor Green -NoNewline
Write-Host " npm $npmVersion"

# Install Codex CLI if not found
Write-Host ""
# Ensure npm global prefix is on PATH (Windows shims live at prefix dir directly)
$npmGlobalBin = try { (npm config get prefix 2>$null).Trim() } catch { $null }
if ($npmGlobalBin -and (Test-Path $npmGlobalBin)) {
    $env:PATH = "$npmGlobalBin;$env:PATH"
}

$codexCmd = Get-Command codex -ErrorAction SilentlyContinue
if ($codexCmd) {
    Write-Host "Codex CLI already installed" -ForegroundColor Blue
} else {
    Write-Host "Installing Codex CLI..." -ForegroundColor Blue
    npm install -g @openai/codex
    $codexCmd = Get-Command codex -ErrorAction SilentlyContinue
    if ($codexCmd) {
        Write-Host "  " -NoNewline
        Write-Host "OK" -ForegroundColor Green -NoNewline
        Write-Host " Codex CLI installed"
    } else {
        Write-Host "Warning: Codex CLI installed but not found on PATH" -ForegroundColor Yellow
        Write-Host "You may need to restart your terminal or add npm global bin to PATH."
    }
}

# Setup ~/.codex directory
Write-Host ""
Write-Host "Configuring Codex CLI..." -ForegroundColor Blue
$codexDir = Join-Path $env:USERPROFILE ".codex"
if (-not (Test-Path $codexDir)) {
    New-Item -ItemType Directory -Path $codexDir -Force | Out-Null
}

# Backup existing config.toml
$configPath = Join-Path $codexDir "config.toml"
if (Test-Path $configPath) {
    $backupPath = "$configPath.backup.$(Get-Date -Format 'yyyyMMddHHmmss')"
    Copy-Item $configPath $backupPath
    Write-Host "  Backed up: $configPath" -ForegroundColor Yellow
}

# Write config.toml
$configContent = @"
model = "$CODEX_MEDIUM"
model_provider = "viberrouter"
model_catalog_json = "~/.codex/models.json"

[model_providers.viberrouter]
name = "Viber Router"
base_url = "$ENDPOINT_URL"
experimental_bearer_token = "$API_KEY"
wire_api = "responses"

[features]
apps = false
"@
[System.IO.File]::WriteAllText($configPath, $configContent, [System.Text.UTF8Encoding]::new($false))
Write-Host "  " -NoNewline
Write-Host "OK" -ForegroundColor Green -NoNewline
Write-Host " Written config.toml"

# Write models.json (content pre-rendered by server via Go template)
$modelsPath = Join-Path $codexDir "models.json"
$modelsContent = @'
{
  "models": [
    {
      "slug": "{{SMALL}}",
      "display_name": "{{SMALL}}",
      "description": "{{SMALL}} via Viber Router (Fast)",
      "default_reasoning_level": "medium",
      "supported_reasoning_levels": [
        {
          "effort": "low",
          "description": "Minimal reasoning"
        },
        {
          "effort": "medium",
          "description": "Balanced reasoning"
        },
        {
          "effort": "high",
          "description": "Deep reasoning"
        },
        {
          "effort": "xhigh",
          "description": "Maximum reasoning"
        }
      ],
      "shell_type": "shell_command",
      "visibility": "list",
      "supported_in_api": true,
      "priority": 1,
      "availability_nux": null,
      "upgrade": null,
      "base_instructions": "",
      "model_messages": null,
      "supports_reasoning_summaries": false,
      "default_reasoning_summary": "auto",
      "support_verbosity": true,
      "default_verbosity": null,
      "apply_patch_tool_type": null,
      "web_search_tool_type": "text",
      "truncation_policy": {
        "mode": "tokens",
        "limit": 400000
      },
      "supports_parallel_tool_calls": true,
      "supports_image_detail_original": false,
      "context_window": 400000,
      "auto_compact_token_limit": null,
      "effective_context_window_percent": 95,
      "experimental_supported_tools": [],
      "input_modalities": [
        "text",
        "image"
      ],
      "supports_search_tool": false
    },
    {
      "slug": "{{MEDIUM}}",
      "display_name": "{{MEDIUM}}",
      "description": "{{MEDIUM}} via Viber Router (Default)",
      "default_reasoning_level": "medium",
      "supported_reasoning_levels": [
        {
          "effort": "low",
          "description": "Minimal reasoning"
        },
        {
          "effort": "medium",
          "description": "Balanced reasoning"
        },
        {
          "effort": "high",
          "description": "Deep reasoning"
        },
        {
          "effort": "xhigh",
          "description": "Maximum reasoning"
        }
      ],
      "shell_type": "shell_command",
      "visibility": "list",
      "supported_in_api": true,
      "priority": 2,
      "availability_nux": null,
      "upgrade": null,
      "base_instructions": "",
      "model_messages": null,
      "supports_reasoning_summaries": false,
      "default_reasoning_summary": "auto",
      "support_verbosity": true,
      "default_verbosity": null,
      "apply_patch_tool_type": null,
      "web_search_tool_type": "text",
      "truncation_policy": {
        "mode": "tokens",
        "limit": 400000
      },
      "supports_parallel_tool_calls": true,
      "supports_image_detail_original": false,
      "context_window": 400000,
      "auto_compact_token_limit": null,
      "effective_context_window_percent": 95,
      "experimental_supported_tools": [],
      "input_modalities": [
        "text",
        "image"
      ],
      "supports_search_tool": false
    },
    {
      "slug": "{{LARGE}}",
      "display_name": "{{LARGE}}",
      "description": "{{LARGE}} via Viber Router (Powerful)",
      "default_reasoning_level": "medium",
      "supported_reasoning_levels": [
        {
          "effort": "low",
          "description": "Minimal reasoning"
        },
        {
          "effort": "medium",
          "description": "Balanced reasoning"
        },
        {
          "effort": "high",
          "description": "Deep reasoning"
        },
        {
          "effort": "xhigh",
          "description": "Maximum reasoning"
        }
      ],
      "shell_type": "shell_command",
      "visibility": "list",
      "supported_in_api": true,
      "priority": 3,
      "availability_nux": null,
      "upgrade": null,
      "base_instructions": "",
      "model_messages": null,
      "supports_reasoning_summaries": false,
      "default_reasoning_summary": "auto",
      "support_verbosity": true,
      "default_verbosity": null,
      "apply_patch_tool_type": null,
      "web_search_tool_type": "text",
      "truncation_policy": {
        "mode": "tokens",
        "limit": 400000
      },
      "supports_parallel_tool_calls": true,
      "supports_image_detail_original": false,
      "context_window": 400000,
      "auto_compact_token_limit": null,
      "effective_context_window_percent": 95,
      "experimental_supported_tools": [],
      "input_modalities": [
        "text",
        "image"
      ],
      "supports_search_tool": false
    }
  ]
}
'@
[System.IO.File]::WriteAllText($modelsPath, $modelsContent, [System.Text.UTF8Encoding]::new($false))
Write-Host "  " -NoNewline
Write-Host "OK" -ForegroundColor Green -NoNewline
Write-Host " Written models.json"

# Write auth.json only if missing or empty (preserve existing auth state)
$authPath = Join-Path $codexDir "auth.json"
if (-not (Test-Path $authPath) -or (Get-Item $authPath).Length -eq 0) {
    [System.IO.File]::WriteAllText($authPath, "{}", [System.Text.UTF8Encoding]::new($false))
    Write-Host "  " -NoNewline
    Write-Host "OK" -ForegroundColor Green -NoNewline
    Write-Host " Written auth.json"
} else {
    Write-Host "  " -NoNewline
    Write-Host "OK" -ForegroundColor Green -NoNewline
    Write-Host " Kept existing auth.json"
}

# Remove models cache to prevent stale OpenAI models
$cachePath = Join-Path $codexDir "models_cache.json"
if (Test-Path $cachePath) {
    Remove-Item $cachePath -Force
    Write-Host "  " -NoNewline
    Write-Host "OK" -ForegroundColor Green -NoNewline
    Write-Host " Removed models_cache.json"
}

Write-Host ""
Write-Host "================================" -ForegroundColor Green
Write-Host "  Configuration Complete!" -ForegroundColor Green
Write-Host "================================" -ForegroundColor Green
Write-Host ""
Write-Host "Codex CLI is now configured to use Viber Router:"
Write-Host "  Endpoint:          " -NoNewline
Write-Host "$ENDPOINT_URL" -ForegroundColor Blue
Write-Host "  API Key:           " -NoNewline
Write-Host "$MASKED_KEY..." -ForegroundColor Blue
Write-Host "  Small (Fast):      " -NoNewline
Write-Host "$CODEX_SMALL" -ForegroundColor Blue
Write-Host "  Medium (Default):  " -NoNewline
Write-Host "$CODEX_MEDIUM" -ForegroundColor Blue
Write-Host "  Large (Powerful):  " -NoNewline
Write-Host "$CODEX_LARGE" -ForegroundColor Blue
Write-Host ""
Write-Host "Next steps:" -ForegroundColor Yellow
Write-Host "  Run: " -NoNewline
Write-Host "codex" -ForegroundColor Blue
Write-Host ""
