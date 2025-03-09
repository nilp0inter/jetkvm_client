@echo off
REM Save the original directory so we can restore it later.
set "ORIG_DIR=%CD%"

REM Change directory to the location of this script.
cd /d "%~dp0"

REM ---------------------------------------------------------------------------
REM This script tests if jetkvm_control_svr is running by invoking Cargo in test mode.
REM The command attempts to connect and authenticate using:
REM    cargo run -p jetkvm_control_svr --example jetkvm_control_svr_client -- -t --password dave
REM If the server is running, the command exits with 0.
REM If not, the script prompts the user to start the server via scripts\run_svr.cmd.
REM ---------------------------------------------------------------------------

echo Testing if jetkvm_control_svr is running...
cargo run -p jetkvm_control_svr --example jetkvm_control_svr_client -- -t --password dave
if errorlevel 1 (
    echo.
    echo jetkvm_control_svr is not running.
    set /p userChoice="Do you want to start jetkvm_control_svr now? (y/n): "
    if /i "%userChoice:~0,1%"=="y" (
        REM Launch the server using the run_svr.cmd script in a new command window.
        start "" cmd /k "run_svr.cmd"
        REM Allow a few seconds for the server to initialize.
        timeout /t 5 >nul
    ) else (
        echo Please start jetkvm_control_svr and try again.
        REM Restore the original directory before exiting.
        cd /d "%ORIG_DIR%"
        exit /b 1
    )
) else (
    echo.
    echo jetkvm_control_svr is running.
)

REM ---------------------------------------------------------------------------
REM Find the project root by locating Cargo.toml.
REM This loop ascends directories until Cargo.toml is found, which indicates the root of the project.
REM Using a relative path (e.g., for cert.pem) ensures that our paths remain valid regardless
REM of the starting directory.
REM ---------------------------------------------------------------------------
:find_root
if not exist Cargo.toml (
    if "%CD%"=="%~d0\" exit /b 1
    pushd ..
    goto find_root
)
@echo ON

REM Once the project root is found, run the sample Lua script.
cargo run -- lua-examples\windows-is_cmd_running.lua

REM Restore the original directory.
cd /d "%ORIG_DIR%"

