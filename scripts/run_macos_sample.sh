#!/bin/zsh

# Prevent recursive invocation
#if [[ -n "$IS_RUNNING" ]]; then
#    echo "Recursive invocation detected. Script: ${0}, called from PID: $PPID ($(ps -p $PPID -o command=))"
#    exit 1
#fi
#export IS_RUNNING=1

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

LUA_EXAMPLES_DIR="lua-examples"

# Get the Lua script name from the first argument and replace any extension with .lua
if [ -z "$1" ]; then
    echo "Usage: $0 <lua-script-name> [additional args...]"
    cd "$ORIG_DIR"
    exit 1
fi

LUA_SCRIPT_NAME="${1%.*}.lua"
shift

# Add lua-examples prefix if not already present
if [[ "$LUA_SCRIPT_NAME" != "$LUA_EXAMPLES_DIR"/* ]]; then
    LUA_SCRIPT_RELATIVE_PATH="$LUA_EXAMPLES_DIR/$LUA_SCRIPT_NAME"
else
    LUA_SCRIPT_RELATIVE_PATH="$LUA_SCRIPT_NAME"
fi

# Run cargo with lua script and remaining arguments
cargo run -- "$LUA_SCRIPT_RELATIVE_PATH" "$@"

echo "$SCRIPT_DIR/run_macos_sample.sh" "$LUA_SCRIPT_RELATIVE_PATH" "$@"
echo "Current working directory: $PWD"

# Restore original directory
cd "$ORIG_DIR"

