@echo off
cd /d "%~dp0"
:find_root
if not exist Cargo.toml (
    if "%cd%"=="%~d0\" exit /b 1
    pushd ..
    goto find_root
)
@echo ON
cargo run -- lua-examples\windows-notepad-helloworld.lua

