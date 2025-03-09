--[[
    This Lua script connects to the JetKVM control server,
    authenticates, fetches details of the active process on the remote system,
    and then conditionally executes commands based on the active process.

    Modified behavior:
    - Checks if the active process is "Terminal" or "Alacritty".
    - Prints the name of the active process accordingly.

    Breakdown:
    1. Create a JetKVM control server client.
    2. Connect and authenticate.
    3. Fetch active process details.
    4. Print the active process.
    5. Check if Terminal or Alacritty is active.
---------------------------------------------------------------------------
]]--

-- Create a JetKVM control server client
local svr = JetKvmControlSvrClient()

-- Connect to the server
local success, message = svr:connect('localhost', '8080', 'dave', CERT_PATH)
if not success then
    print("Connection failed:", message)
    return
end

-- Fetch the active process details
local result = svr:send_command("active_process")
if not result or not result.executable_name then
    print("Failed to get active process details.")
    return
end

-- Print the current active process
print("Current active process:", result.executable_name)

-- Check for Terminal or Alacritty (case-insensitive)
local active_process = result.executable_name:lower()
if active_process == "terminal" or active_process == "alacritty" then
    print("Terminal emulator (" .. result.executable_name .. ") is the active process.")
else
    print("Terminal emulator is NOT the active process. Current:", result.executable_name)
end

