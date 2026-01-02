#!/bin/sh
# Setup git hooks for development

git config core.hooksPath .githooks
chmod +x .githooks/*

echo "Git hooks configured! Pre-commit hook will run cargo fmt and clippy."

