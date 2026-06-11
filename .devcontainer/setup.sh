#!/usr/bin/env bash
# Runs once when the devcontainer is created (see postCreateCommand in devcontainer.json).
set -euo pipefail

sudo apt-get update
sudo apt-get install -y \
  libwebkit2gtk-4.1-dev \
  libssl-dev \
  libgtk-3-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev \
  libxdo-dev \
  build-essential \
  patchelf

npm ci

# CodeRabbit CLI, used by .claude/rules/coderabbit-before-pr.md
curl -fsSL https://cli.coderabbit.ai/install.sh | sh

# Authenticate non-interactively if CODERABBIT_API_KEY is set as a Codespaces secret
if [ -n "${CODERABBIT_API_KEY:-}" ]; then
  coderabbit auth login --api-key "$CODERABBIT_API_KEY"
fi

# Claude Code picks up ANTHROPIC_API_KEY (or CLAUDE_CODE_OAUTH_TOKEN) automatically from
# the environment, so set one of these as a Codespaces secret to skip the `claude login` flow.
