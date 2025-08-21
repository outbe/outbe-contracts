#!/bin/bash
#
# Script to install git hooks for the repository
#

echo "Setting up git hooks..."

if git config --local core.hooksPath hooks 2>/dev/null; then
    chmod +x hooks/pre-commit
    echo "✓ Git configured to use hooks from hooks/ directory"
    echo "✓ Pre-commit hook is now active"
else
    if [ -f "hooks/pre-commit" ]; then
        cp hooks/pre-commit .git/hooks/pre-commit
        chmod +x .git/hooks/pre-commit
        echo "✓ Pre-commit hook copied and made executable"
    else
        echo "✗ Pre-commit hook not found in hooks/ directory"
        exit 1
    fi
fi

echo ""
echo "🎉 Git hooks installed successfully!"
echo "The pre-commit hook will now run:"
echo "  - cargo fmt (code formatting)"
echo "  - cargo +1.86.0 clippy --all-targets -- -D warnings (linting)"
echo ""
echo "To run setup again: make install"
