@echo off
echo Starting Rust Cool Nginx Static File Server...
echo Visit http://127.0.0.1:8080 in your browser
echo Press Ctrl+C to stop the server
echo.
target\release\rust-cool-nginx.exe static . 127.0.0.1:8080
pause