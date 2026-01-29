#!/bin/bash
# =============================================================================
# CI Setup Script - Centralized dependency installation
# =============================================================================
# This script installs all required dependencies for CI jobs.
# Usage: bash .github/setup-ci.sh [--install-audit] [--install-coverage] [--install-diagrams]
#
# Dependencies installed by default:
#   - protobuf-compiler (required for milvus-sdk-rust)
#
# Optional dependencies (via flags):
#   - cargo-audit (for security audits)
#   - cargo-tarpaulin (for coverage)
#   - plantuml (for diagram generation)
# =============================================================================

set -e

# Detect OS
OS=$(uname -s)

# Color output for visibility
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}=== CI Setup: Installing dependencies ===${NC}"

# Install protobuf-compiler (always required)
echo -e "${BLUE}Installing protobuf-compiler...${NC}"
case "$OS" in
  Linux)
    # Check if already installed
    if ! command -v protoc &> /dev/null; then
      sudo apt-get update -qq
      sudo apt-get install -y protobuf-compiler
    fi
    ;;
  Darwin)
    # Check if already installed
    if ! command -v protoc &> /dev/null; then
      brew install protobuf
    fi
    ;;
  MINGW*|MSYS*|CYGWIN*)
    # Check if already installed
    if ! command -v protoc &> /dev/null; then
      choco install protoc -y
    fi
    ;;
  *)
    echo "Unsupported OS: $OS"
    exit 1
    ;;
esac
echo -e "${GREEN}✓ protobuf-compiler ready${NC}"

# Parse optional flags
while [[ $# -gt 0 ]]; do
  case $1 in
    --install-audit)
      echo -e "${BLUE}Installing cargo-audit...${NC}"
      cargo install cargo-audit --locked
      echo -e "${GREEN}✓ cargo-audit installed${NC}"
      shift
      ;;
    --install-coverage)
      echo -e "${BLUE}Installing cargo-tarpaulin...${NC}"
      cargo install cargo-tarpaulin --locked
      echo -e "${GREEN}✓ cargo-tarpaulin installed${NC}"
      shift
      ;;
    --install-diagrams)
      echo -e "${BLUE}Installing plantuml...${NC}"
      if command -v plantuml &> /dev/null; then
        echo -e "${GREEN}✓ plantuml already installed${NC}"
      else
        case "$OS" in
          Linux)
            sudo apt-get install -y plantuml
            ;;
          Darwin)
            brew install plantuml
            ;;
          *)
            echo "PlantUML installation not supported on this OS"
            exit 1
            ;;
        esac
        echo -e "${GREEN}✓ plantuml installed${NC}"
      fi
      shift
      ;;
    *)
      echo "Unknown option: $1"
      exit 1
      ;;
  esac
done

echo -e "${GREEN}=== CI Setup: Dependencies ready ===${NC}"
