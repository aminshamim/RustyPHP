#!/bin/bash

# Script to resolve common README title conflicts
# This script will automatically resolve conflicts in favor of "RustyPHP 🦀"

echo "Checking for title conflicts in README.md..."

if grep -q "<<<<<<< HEAD" README.md; then
    echo "Found conflict markers. Resolving..."
    
    # Create a temporary file with the resolved content
    sed '/<<<<<<< HEAD/,/=======/{
        /<<<<<<< HEAD/d
        /RustyPHP$/d
        /=======/d
    }
    />>>>>>> /d' README.md > README_temp.md
    
    # Replace the original file
    mv README_temp.md README.md
    
    echo "Conflict resolved! Title is now: '# RustyPHP 🦀'"
    echo "Please review the changes and commit if everything looks correct."
else
    echo "No conflicts found in README.md"
fi
