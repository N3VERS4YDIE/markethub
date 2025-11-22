#!/usr/bin/env bash

set -e

echo "🧪 Running tests with coverage..."

cargo llvm-cov --html \
  --ignore-filename-regex '(main\.rs|lib\.rs|server\.rs|state\.rs|config\.rs|error\.rs|mod\.rs|handlers/|middeleware/)'

echo ""
echo "✅ Coverage report generated!"
echo "📊 Open: target/llvm-cov/html/index.html"
echo ""
echo "To view in browser:"
echo "  xdg-open target/llvm-cov/html/index.html"
