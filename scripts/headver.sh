#!/bin/bash
# HeadVer versioning system implementation
# Format: {head}.{yearweek}.{build}
# https://github.com/line/headver

set -e

# Configuration
HEAD_VERSION_FILE=".headver"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Change to project root
cd "$PROJECT_ROOT"

# Function to get head version
get_head_version() {
    if [ -f "$HEAD_VERSION_FILE" ]; then
        cat "$HEAD_VERSION_FILE"
    else
        echo "1"
    fi
}

# Function to calculate yearweek
get_yearweek() {
    # Format: 2-digit year + 2-digit week number
    YEAR=$(date +%y)
    WEEK=$(date +%V)
    printf "%s%02d" "$YEAR" "$WEEK"
}

# Function to get build number
get_build_number() {
    if [ -n "$BUILD_NUMBER" ]; then
        # Use CI/CD build number if available
        echo "$BUILD_NUMBER"
    elif [ -n "$GITHUB_RUN_NUMBER" ]; then
        # Use GitHub Actions run number if available
        echo "$GITHUB_RUN_NUMBER"
    else
        # Use commit count for this week as build number
        # Get the start of the current week (Monday)
        if [[ "$OSTYPE" == "darwin"* ]]; then
            # macOS
            WEEK_START=$(date -v-monday +%Y-%m-%d 2>/dev/null || date +%Y-%m-%d)
        else
            # Linux
            WEEK_START=$(date -d "last monday" +%Y-%m-%d 2>/dev/null || date +%Y-%m-%d)
        fi
        
        # Count commits since week start
        BUILD=$(git rev-list --count --since="$WEEK_START" HEAD 2>/dev/null || echo "0")
        echo "$BUILD"
    fi
}

# Parse command line arguments
case "$1" in
    --help|-h)
        cat << EOF
HeadVer Version Generator

Usage: $0 [OPTIONS]

Options:
    --help, -h          Show this help message
    --head              Show only the head version
    --yearweek          Show only the yearweek part
    --build             Show only the build number
    --set-head VERSION  Set the head version
    --bump-head         Increment the head version by 1
    --format FORMAT     Output in specific format (full|short|json)
    
Without options, outputs the full version string.

Examples:
    $0                  # Output: 1.2534.0
    $0 --head           # Output: 1
    $0 --yearweek       # Output: 2534
    $0 --build          # Output: 0
    $0 --set-head 2     # Set head version to 2
    $0 --bump-head      # Increment head version
    $0 --format json    # Output: {"head":"1","yearweek":"2534","build":"0","version":"1.2534.0"}

Environment Variables:
    BUILD_NUMBER        Override build number (useful in CI/CD)
    GITHUB_RUN_NUMBER   GitHub Actions run number (auto-detected)

More info: https://github.com/line/headver
EOF
        exit 0
        ;;
    
    --head)
        get_head_version
        ;;
    
    --yearweek)
        get_yearweek
        ;;
    
    --build)
        get_build_number
        ;;
    
    --set-head)
        if [ -z "$2" ]; then
            echo "Error: --set-head requires a version number" >&2
            echo "Usage: $0 --set-head <version>" >&2
            exit 1
        fi
        echo "$2" > "$HEAD_VERSION_FILE"
        echo "Head version set to $2"
        ;;
    
    --bump-head)
        CURRENT_HEAD=$(get_head_version)
        NEW_HEAD=$((CURRENT_HEAD + 1))
        echo "$NEW_HEAD" > "$HEAD_VERSION_FILE"
        echo "Head version bumped from $CURRENT_HEAD to $NEW_HEAD"
        ;;
    
    --format)
        HEAD=$(get_head_version)
        YEARWEEK=$(get_yearweek)
        BUILD=$(get_build_number)
        VERSION="${HEAD}.${YEARWEEK}.${BUILD}"
        
        case "$2" in
            json)
                printf '{"head":"%s","yearweek":"%s","build":"%s","version":"%s"}\n' \
                    "$HEAD" "$YEARWEEK" "$BUILD" "$VERSION"
                ;;
            short)
                echo "${HEAD}.${YEARWEEK}"
                ;;
            full|*)
                echo "$VERSION"
                ;;
        esac
        ;;
    
    *)
        # Default: output full version
        HEAD=$(get_head_version)
        YEARWEEK=$(get_yearweek)
        BUILD=$(get_build_number)
        echo "${HEAD}.${YEARWEEK}.${BUILD}"
        ;;
esac