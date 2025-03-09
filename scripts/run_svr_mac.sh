#!/bin/zsh

# Set a default password if not already set (adjust as needed)
JETKVM_PASSWORD="${JETKVM_PASSWORD:-dave}"

# Determine the script's directory using zsh's parameter expansion
SCRIPT_DIR="${0:A:h}"

# Change directory to the script's location
cd "$SCRIPT_DIR" || { echo "Failed to change directory to script location"; exit 1; }

# Search upward for Cargo.toml
while [ ! -f "Cargo.toml" ]; do
  if [ "$(pwd)" = "/" ]; then
    echo "Cargo.toml not found in any parent directory. Exiting."
    exit 1
  fi
  cd ..
done

# Save the directory where Cargo.toml was found (project root)
PROJECT_ROOT="$(pwd)"

# Define the command to run; include changing to the project root directory first
SERVER_COMMAND='cd "'"$PROJECT_ROOT"'" && cargo run -p jetkvm_control_svr -- -P "'"$JETKVM_PASSWORD"'" ; echo "Press Enter to exit..."; read'

# Check if Alacritty exists in the /Applications folder
if [ -d "/Applications/Alacritty.app" ]; then
    # Launch Alacritty and run the command with the proper working directory
    /Applications/Alacritty.app/Contents/MacOS/alacritty -e bash -c "$SERVER_COMMAND" &
else
    # Fallback: use Terminal with AppleScript
    osascript <<EOF
tell application "Terminal"
    do script "$SERVER_COMMAND"
    activate
end tell
EOF
fi

