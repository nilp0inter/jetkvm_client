#!/bin/zsh

# Prevent recursive invocation
if [[ -n "$IS_RUNNING" ]]; then
    echo "Recursive invocation detected. Script: ${0}, called from PID: $PPID ($(ps -p $PPID -o command=))"
    exit 1
fi
export IS_RUNNING=1

# Save the original directory
ORIG_DIR="$PWD"

# Get the absolute path of this script
SCRIPT_DIR="${0:A:h}"
SCRIPT_NAME="${0:t}"

# Change to the script's directory
cd "$SCRIPT_DIR" || { echo "Could not change directory"; exit 1; }

# Ascend to the project root by looking for Cargo.toml
while [ ! -f "Cargo.toml" ]; do
    if [ "$PWD" = "/" ]; then
        echo "Cargo.toml not found in any parent directory. Exiting."
        cd "$ORIG_DIR"
        exit 1
    fi
    cd ..
done

# Replace .sh with .lua in the script's name
LUA_SCRIPT_NAME="${SCRIPT_NAME%.sh}"

# Call run_macos_sample.sh with the relative lua script path plus all other arguments
"$SCRIPT_DIR/run_macos_sample.sh" "$LUA_SCRIPT_NAME" 
echo "$SCRIPT_DIR/run_macos_sample.sh" "$LUA_SCRIPT_NAME" 
#"$@"

# Restore original directory
cd "$ORIG_DIR"

