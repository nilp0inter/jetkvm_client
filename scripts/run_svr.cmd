@echo off
setlocal EnableDelayedExpansion

if "%~1"=="" (
    set JETKVM_PASSWORD=dave
) else (
    set JETKVM_PASSWORD=%~1
)

cd /d "%~dp0"
:find_root
if not exist Cargo.toml (
    if "%cd%"=="%~d0\" exit /b 1
    pushd ..
    goto find_root
)

if exist cert.pem (
    goto run_server
) else (
    echo cert.pem not found.
    set /p answer="Would you like to initialize a new certificate? (Y/n): "
    if "!answer!"=="" set answer=Y
    echo "dave - !answer!"
    if /I "!answer!"=="Y" (
        start /wait "" cargo run -p jetkvm_control_svr -- -P %JETKVM_PASSWORD% --init-cert
        if not exist cert.pem (
            echo Certificate initialization failed. Exiting.
            exit /b 1
        ) else (
            goto run_server
        )
    ) else (
        echo Certificate not initialized. Exiting.
        exit /b 1
    )
)

:run_server
start cmd /k "cargo run -p jetkvm_control_svr -- -P %JETKVM_PASSWORD% & pause & exit"
popd
