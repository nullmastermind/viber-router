#!/bin/sh

# Viber Router Installer for Codex CLI (Linux/macOS)
# Configures Codex CLI to use Viber Router

set -e

# Configuration (auto-populated by server)
ENDPOINT_URL="{{ENDPOINT_URL}}"
API_KEY='{{API_KEY}}'
CODEX_SMALL="{{SMALL}}"
CODEX_MEDIUM="{{MEDIUM}}"
CODEX_LARGE="{{LARGE}}"

# Colors
RED=$(printf '\033[0;31m')
GREEN=$(printf '\033[0;32m')
YELLOW=$(printf '\033[1;33m')
BLUE=$(printf '\033[0;34m')
NC=$(printf '\033[0m')

echo "${BLUE}================================${NC}"
echo "${BLUE}  Viber Router - Codex CLI Setup${NC}"
echo "${BLUE}================================${NC}"
echo ""

# Validate configuration
if [ "$ENDPOINT_URL" = "__""ENDPOINT_URL__" ] || [ -z "$ENDPOINT_URL" ]; then
    echo "${RED}Error: Endpoint URL not configured${NC}"
    echo "Please use the install link from your Viber Router dashboard."
    exit 1
fi

if [ "$API_KEY" = "__""API_KEY__" ] || [ -z "$API_KEY" ]; then
    echo "${RED}Error: API key not configured${NC}"
    echo "Please use the install link from your Viber Router dashboard."
    exit 1
fi

if [ "$CODEX_SMALL" = "__""CODEX_SMALL__" ] || [ -z "$CODEX_SMALL" ]; then
    echo "${RED}Error: Small model not configured${NC}"
    exit 1
fi

if [ "$CODEX_MEDIUM" = "__""CODEX_MEDIUM__" ] || [ -z "$CODEX_MEDIUM" ]; then
    echo "${RED}Error: Medium (default) model not configured${NC}"
    exit 1
fi

if [ "$CODEX_LARGE" = "__""CODEX_LARGE__" ] || [ -z "$CODEX_LARGE" ]; then
    echo "${RED}Error: Large model not configured${NC}"
    exit 1
fi

# Mask API key for display
MASKED_KEY=$(echo "$API_KEY" | cut -c 1-10)
echo "Endpoint URL:    ${GREEN}$ENDPOINT_URL${NC}"
echo "API Key:         ${GREEN}${MASKED_KEY}...${NC}"
echo "Small (Fast):    ${GREEN}$CODEX_SMALL${NC}"
echo "Medium (Default):${GREEN} $CODEX_MEDIUM${NC}"
echo "Large (Powerful):${GREEN} $CODEX_LARGE${NC}"
echo ""

# Check Node.js >= 22
echo "${BLUE}Checking prerequisites...${NC}"

if ! command -v node >/dev/null 2>&1; then
    echo "${RED}Error: Node.js is not installed${NC}"
    echo ""
    echo "Codex CLI requires Node.js 22 or later."
    echo "Install Node.js from: ${BLUE}https://nodejs.org${NC}"
    echo ""
    echo "  ${BLUE}macOS:${NC}        brew install node"
    echo "  ${BLUE}Ubuntu/Debian:${NC} curl -fsSL https://deb.nodesource.com/setup_22.x | sudo -E bash - && sudo apt-get install -y nodejs"
    exit 1
fi

NODE_MAJOR=$(node --version | sed 's/^v//' | cut -d. -f1)
if [ "$NODE_MAJOR" -lt 22 ] 2>/dev/null; then
    echo "${RED}Error: Node.js 22+ is required (found $(node --version))${NC}"
    echo ""
    echo "Please upgrade Node.js: ${BLUE}https://nodejs.org${NC}"
    exit 1
fi
echo "  ${GREEN}✓ Node.js $(node --version)${NC}"

# Check npm
if ! command -v npm >/dev/null 2>&1; then
    echo "${RED}Error: npm is not installed${NC}"
    echo "Please install npm (usually comes with Node.js)."
    exit 1
fi
echo "  ${GREEN}✓ npm $(npm --version)${NC}"

# Install Codex CLI if not found
echo ""
# Ensure npm global bin is on PATH (needed for nvm/custom prefix)
NPM_GLOBAL_BIN=$(npm bin -g 2>/dev/null || npm prefix -g 2>/dev/null | xargs -I{} echo {}/bin)
if [ -n "$NPM_GLOBAL_BIN" ] && [ -d "$NPM_GLOBAL_BIN" ]; then
    export PATH="$NPM_GLOBAL_BIN:$PATH"
fi

if command -v codex >/dev/null 2>&1; then
    echo "${BLUE}Codex CLI already installed${NC}"
else
    echo "${BLUE}Installing Codex CLI...${NC}"
    npm install -g @openai/codex
    if command -v codex >/dev/null 2>&1; then
        echo "  ${GREEN}✓ Codex CLI installed${NC}"
    else
        echo "${YELLOW}Warning: Codex CLI installed but not found on PATH${NC}"
        echo "You may need to restart your terminal or add npm global bin to PATH."
        echo "  npm global bin: ${BLUE}$NPM_GLOBAL_BIN${NC}"
    fi
fi

# Setup ~/.codex directory
echo ""
echo "${BLUE}Configuring Codex CLI...${NC}"
CODEX_DIR="$HOME/.codex"
mkdir -p "$CODEX_DIR"

# Backup existing config.toml
if [ -f "$CODEX_DIR/config.toml" ]; then
    cp "$CODEX_DIR/config.toml" "$CODEX_DIR/config.toml.backup.$(date +%Y%m%d%H%M%S)"
    echo "  ${YELLOW}Backed up: ~/.codex/config.toml${NC}"
fi

# Write config.toml
cat > "$CODEX_DIR/config.toml" << TOML_EOF
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
TOML_EOF
echo "  ${GREEN}✓ Written ~/.codex/config.toml${NC}"

# Write models.json (content pre-rendered by server via Go template)
cat > "$CODEX_DIR/models.json" << 'MODELS_EOF'
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
MODELS_EOF
echo "  ${GREEN}✓ Written ~/.codex/models.json${NC}"

# Write auth.json only if missing or empty (preserve existing auth state)
if [ ! -f "$CODEX_DIR/auth.json" ] || [ ! -s "$CODEX_DIR/auth.json" ]; then
    echo '{}' > "$CODEX_DIR/auth.json"
    echo "  ${GREEN}✓ Written ~/.codex/auth.json${NC}"
else
    echo "  ${GREEN}✓ Kept existing ~/.codex/auth.json${NC}"
fi

# Remove models cache to prevent stale OpenAI models
if [ -f "$CODEX_DIR/models_cache.json" ]; then
    rm -f "$CODEX_DIR/models_cache.json"
    echo "  ${GREEN}✓ Removed ~/.codex/models_cache.json${NC}"
fi

echo ""
echo "${GREEN}================================${NC}"
echo "${GREEN}  Configuration Complete!${NC}"
echo "${GREEN}================================${NC}"
echo ""
echo "Codex CLI is now configured to use Viber Router:"
echo "  Endpoint:          ${BLUE}$ENDPOINT_URL${NC}"
echo "  API Key:           ${BLUE}${MASKED_KEY}...${NC}"
echo "  Small (Fast):      ${BLUE}$CODEX_SMALL${NC}"
echo "  Medium (Default):  ${BLUE}$CODEX_MEDIUM${NC}"
echo "  Large (Powerful):  ${BLUE}$CODEX_LARGE${NC}"
echo ""
echo "${YELLOW}Next steps:${NC}"
echo "  Run: ${BLUE}codex${NC}"
echo ""
