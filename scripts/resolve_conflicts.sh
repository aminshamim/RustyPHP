#!/bin/bash

# Comprehensive conflict resolution script for RustyPHP repository
# This script handles common conflict scenarios

echo "🔧 RustyPHP Conflict Resolution Tool"
echo "=================================="

# Function to check for conflicts in a file
check_conflicts() {
    local file="$1"
    if grep -q "<<<<<<< HEAD\|=======\|>>>>>>>" "$file"; then
        return 0  # Conflicts found
    else
        return 1  # No conflicts
    fi
}

# Function to resolve README title conflicts
resolve_readme_conflicts() {
    echo "📝 Resolving README.md conflicts..."
    
    if check_conflicts "README.md"; then
        echo "Found conflicts in README.md"
        
        # Backup original file
        cp README.md README.md.backup
        
        # Resolve conflicts by keeping the version with emoji and logo
        awk '
        /<<<<<<< HEAD/,/=======/ {
            if (/<<<<<<< HEAD/ || /=======$/) next
            if (/^# RustyPHP$/) next  # Remove version without emoji
            print
        }
        /=======/,/>>>>>>> / {
            if (/=======$/ || />>>>>>> /) next
            print
        }
        !/<<<<<<< HEAD/ && !/=======$/ && !/>>>>>>> / {
            if (!/^# RustyPHP$/ || /🦀/) print
        }
        ' README.md.backup > README.md
        
        echo "✅ README.md conflicts resolved"
        echo "   Kept: '# RustyPHP 🦀' with logo"
        rm README.md.backup
    else
        echo "✅ No conflicts found in README.md"
    fi
}

# Function to sync branches
sync_branches() {
    echo "🔄 Syncing branches with main..."
    
    local current_branch=$(git branch --show-current)
    echo "Current branch: $current_branch"
    
    # Update main first
    git checkout main
    git pull origin main
    
    # Update arithmetic branch
    if git show-ref --verify --quiet refs/heads/arithmetic; then
        echo "Updating arithmetic branch..."
        git checkout arithmetic
        git merge main
        git push origin arithmetic
    fi
    
    # Return to original branch
    git checkout "$current_branch"
    echo "✅ Branches synchronized"
}

# Main execution
echo "Checking repository status..."
git status

echo ""
echo "Options:"
echo "1. Resolve README conflicts only"
echo "2. Sync all branches" 
echo "3. Do both"
echo ""

read -p "Choose option (1-3): " choice

case $choice in
    1)
        resolve_readme_conflicts
        ;;
    2)
        sync_branches
        ;;
    3)
        resolve_readme_conflicts
        sync_branches
        ;;
    *)
        echo "Invalid option. Running conflict check only..."
        resolve_readme_conflicts
        ;;
esac

echo ""
echo "🎉 Conflict resolution complete!"
echo "📋 Next steps:"
echo "   - Review changes: git diff"
echo "   - Commit if needed: git add . && git commit -m 'Resolve conflicts'"
echo "   - Push changes: git push origin $(git branch --show-current)"
