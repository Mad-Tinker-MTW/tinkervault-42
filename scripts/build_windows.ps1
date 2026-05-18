$ErrorActionPreference = "Stop"

Write-Host "Building TinkerVault 42 Final Core..." -ForegroundColor Cyan

if (!(Test-Path ".\src-tauri\icons\icon.ico")) {
  throw "Missing src-tauri\icons\icon.ico"
}

npm install
npm run tauri build

Write-Host "Build complete." -ForegroundColor Green
Write-Host "Raw EXE: src-tauri\target\release\tinkervault-42.exe" -ForegroundColor Green
Write-Host "Installers: src-tauri\target\release\bundle" -ForegroundColor Green
