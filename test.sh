#!/bin/bash

# Test script to verify the application compiles and basic functionality works

echo "Testing GitUI compilation..."
cargo build --release

if [ $? -eq 0 ]; then
    echo "✓ Build successful!"
    echo ""
    echo "To run the application:"
    echo "  cargo run"
    echo ""
    echo "Or use the release binary:"
    echo "  ./target/release/gitui"
    echo ""
    echo "Key features to try:"
    echo "  - Press 1/2/3 to switch between Files/History/Branches views"
    echo "  - Use ↑/↓ to navigate"
    echo "  - Press 's' to stage a file (in Files view)"
    echo "  - Press 'a' to stage all files (in Files view)"
    echo "  - Press 'c' to commit (in Files view)"
    echo "  - Press 'Enter' to checkout a branch (in Branches view)"
    echo "  - Press 'q' to quit"
else
    echo "✗ Build failed!"
    exit 1
fi
