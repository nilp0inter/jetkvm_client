--[[
    This Lua script demonstrates how to connect to the JetKVM control server,
    authenticate, fetch details of the active process on the remote system,
    and then conditionally execute commands based on the active process.

    Breakdown of the script:
    
    1. Create a JetKVM control server client:
       - The JetKvmControlSvrClient() constructor initializes a new client instance.
    
    2. Connect to the server:
       - svr:connect(host, port, username, certificate_path) attempts to connect to the server.
       - It returns a boolean 'success' flag and a 'message' with additional details or error information.
    
    3. Check connection/authentication:
       - If the connection fails (success is false), the script prints an error message and terminates.
    
    4. Fetch active process details:
       - svr:send_command("active_process") sends a command to retrieve details about the active process on the remote system.
       - The result should include an 'executable_name' field if successful.
    
    5. Validate the result:
       - If the response is missing or the active process details cannot be determined,
         the script prints an error message and exits.
    
    6. Process the active process details:
       - The script prints the name of the active process.
       - If the active process is 'cmd.exe' (Command Prompt), it sends a text command "echo hello world"
         followed by a simulated Return key press to execute the command.
       - Otherwise, it prints a message indicating that cmd.exe is not the active process.
    
    Important functions used:
       - send_text(text): Sends text input as simulated keystrokes to the remote system.
       - send_return(): Simulates pressing the Return (Enter) key to execute a command.
--]]

-- Create the JetKVM control server client
local svr = JetKvmControlSvrClient()

-- Attempt to connect to the server using the provided host, port, username, and certificate path.
-- 'localhost' and '8080' specify the server address, 'dave' is the username, and CERT_PATH is the path to the certificate.
local success, message = svr:connect("localhost", "8080", "dave", CERT_PATH)
print("Connect result:", success, "Message:", message)

-- Check if authentication failed. If not successful, print the error message and exit the script.
if not success then
    print("Failed to authenticate:", message)
    return
end

-- Fetch the active process details from the remote system.
-- This command should return a table containing details about the active process,
-- including at least an 'executable_name' field.
local result = svr:send_command("active_process")

-- Ensure the response is valid. If the result is nil or does not include an executable name,
-- print an error message and exit.
if not result or not result.executable_name then
    print("Failed to retrieve active process details.")
    return
end

-- Print the active process's executable name.
print("Executable Name:", result.executable_name)

-- Check if the active process is the Command Prompt (cmd.exe).
-- Convert the executable name to lower case to ensure case-insensitive comparison.
if result.executable_name:lower() == "cmd.exe" then
    print("Command Prompt (cmd.exe) is the active process!")
    -- If cmd.exe is active, send a text command "echo hello world" to the command prompt.
    send_text("echo hello world")
    -- Simulate a Return key press to execute the command.
    send_return()
else
    -- If the active process is not cmd.exe, inform the user by printing the current active process.
    print("Command Prompt (cmd.exe) is NOT the active process. Current:", result.executable_name)
end