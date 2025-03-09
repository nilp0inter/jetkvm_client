#!/bin/zsh
# Save the original directory so we can restore it later.
ORIG_DIR="$PWD"
SCRIPT_DIR="${0:A:h}"

# Change directory to the location of this script.
cd "$SCRIPT_DIR" || { echo "Failed to change directory to script location"; exit 1; }

# ---------------------------------------------------------------------------
# This script tests if jetkvm_control_svr is running by invoking Cargo in test mode.
# The command attempts to connect and authenticate using:
#   cargo run -p jetkvm_control_svr --example jetkvm_control_svr_client -- -t --password dave
# If the server is running, the command exits with 0.
# If not, the script prompts the user to start the server via the run_svr.sh script.
# ---------------------------------------------------------------------------

echo "Testing if jetkvm_control_svr is running..."
# Run the test command.
cargo run -p jetkvm_control_svr --example jetkvm_control_svr_client -- -t --password dave
TEST_EXIT_CODE=$?

if [ $TEST_EXIT_CODE -ne 0 ]; then
    echo ""
    echo "jetkvm_control_svr is not running."
    read "userChoice?Do you want to start jetkvm_control_svr now? (y/n): "
    if [[ "${userChoice:l}" == y* ]]; then
        # Launch the server using the run_svr.sh script in a new terminal window.
        # We assume run_svr_mac.sh is in the same directory as this script.
        RUN_SVR_SCRIPT="$SCRIPT_DIR/run_svr_mac.sh"
        if [ ! -x "$RUN_SVR_SCRIPT" ]; then
            echo "Server script not found or not executable: $RUN_SVR_SCRIPT"
            cd "$ORIG_DIR"
            exit 1
        fi

        # Check if Alacritty is installed. If so, use it; otherwise, use Terminal.
        if [ -d "/Applications/Alacritty.app" ]; then
            /Applications/Alacritty.app/Contents/MacOS/alacritty -e bash -ic "$RUN_SVR_SCRIPT" &
        else
            osascript <<EOF
tell application "Terminal"
    do script "$RUN_SVR_SCRIPT"
    activate
end tell
EOF
        fi
        # Allow a few seconds for the server to initialize.
        sleep 6
    else
        echo "Please start jetkvm_control_svr and try again."
        cd "$ORIG_DIR"
        exit 1
    fi
else
    echo ""
    echo "jetkvm_control_svr is running."
fi

# ---------------------------------------------------------------------------
# Find the project root by locating Cargo.toml.
# This loop ascends directories until Cargo.toml is found, which indicates the project root.
# Using a relative path (e.g., for cert.pem) ensures that our paths remain valid regardless
# of the starting directory.
# ---------------------------------------------------------------------------
while [ ! -f "Cargo.toml" ]; do
    if [ "$PWD" = "/" ]; then
        echo "Cargo.toml not found in any parent directory. Exiting."
        cd "$ORIG_DIR"
        exit 1
    fi
    cd ..
done

# ---------------------------------------------------------------------------
# Once the project root is found, and the svr is up, run the sample Lua script.
# ---------------------------------------------------------------------------
# Use forward slashes for paths on macOS.
cargo run -- lua-examples/macos_terminal_helloworld.lua

# Restore the original directory.
cd "$ORIG_DIR"

