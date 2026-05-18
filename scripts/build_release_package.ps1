$ErrorActionPreference = "Stop"

$version      = "42.1.0"
$appName      = "TinkerVault42"
$zipName      = "${appName}_${version}_release.zip"
$releaseDir   = Join-Path $PSScriptRoot "..\release"
$bundleDir    = Join-Path $PSScriptRoot "..\src-tauri\target\release\bundle\nsis"
$manifestSrc  = Join-Path $PSScriptRoot "..\release-manifest.json"
$sigSrc       = Join-Path $PSScriptRoot "..\release-manifest.json.minisig"

# --- locate installer ---
$installer = Get-ChildItem $bundleDir -Filter "*.exe" -ErrorAction SilentlyContinue |
  Where-Object { $_.Name -notmatch "uninstall" } |
  Sort-Object LastWriteTime -Descending |
  Select-Object -First 1

if (!$installer) {
  Write-Host "FAIL: No installer EXE found in $bundleDir" -ForegroundColor Red
  Write-Host "  Run npm run build:windows first." -ForegroundColor Yellow
  exit 1
}

foreach ($required in @($manifestSrc, $sigSrc)) {
  if (!(Test-Path $required)) {
    Write-Host "FAIL: Required file missing: $required" -ForegroundColor Red
    Write-Host "  Run scripts\sign_manifest.ps1 -InstallerPath '$($installer.FullName)' first." -ForegroundColor Yellow
    exit 1
  }
}

# --- prepare release folder ---
if (Test-Path $releaseDir) { Remove-Item $releaseDir -Recurse -Force }
New-Item -ItemType Directory -Path $releaseDir | Out-Null
Write-Host "Release folder: $releaseDir" -ForegroundColor Cyan

# --- copy files ---
Copy-Item $installer.FullName        $releaseDir
Copy-Item $manifestSrc               $releaseDir
Copy-Item $sigSrc                    $releaseDir

# --- SHA256SUMS.txt ---
$sumsPath = Join-Path $releaseDir "SHA256SUMS.txt"
Get-ChildItem $releaseDir -File | Where-Object { $_.Name -ne "SHA256SUMS.txt" } | ForEach-Object {
  $h = (Get-FileHash $_.FullName -Algorithm SHA256).Hash.ToLower()
  "$h  $($_.Name)"
} | Set-Content $sumsPath -Encoding UTF8
Write-Host "SHA256SUMS.txt written." -ForegroundColor Cyan

# --- VERIFY.txt ---
$pubKeyNote = "mtw-release.pub (distribute separately or embed in docs)"
$verifyContent = @"
TinkerVault 42 v$version — Release Verification
================================================

STEP 1 — Verify the manifest signature with Minisign
  minisign -V -p $pubKeyNote -m release-manifest.json -x release-manifest.json.minisig

  If this passes, the manifest was signed by Mad Tinker's Workshop and has not been tampered with.

STEP 2 — Verify the installer hash matches the manifest
  The SHA256 of the installer EXE is listed in release-manifest.json under "files".
  Cross-check against SHA256SUMS.txt in this package.

  PowerShell:
    (Get-FileHash '.\$($installer.Name)' -Algorithm SHA256).Hash.ToLower()

  The output must match the sha256 value in release-manifest.json exactly.

STEP 3 — Install
  Only run the installer after both checks pass.

Minisign download: https://jedisct1.github.io/minisign/
Public key:        Obtain from Mad Tinker's Workshop directly or from the MTW repo.
"@

$verifyContent | Set-Content (Join-Path $releaseDir "VERIFY.txt") -Encoding UTF8
Write-Host "VERIFY.txt written." -ForegroundColor Cyan

# --- zip ---
$zipPath = Join-Path (Split-Path $releaseDir -Parent) $zipName
if (Test-Path $zipPath) { Remove-Item $zipPath -Force }
Compress-Archive -Path "$releaseDir\*" -DestinationPath $zipPath
Write-Host "Package zipped: $zipPath" -ForegroundColor Cyan

# --- summary ---
Write-Host ""
Write-Host "PASS: Release package ready." -ForegroundColor Green
Write-Host "  Installer : $($installer.Name)"
Write-Host "  Manifest  : release-manifest.json"
Write-Host "  Signature : release-manifest.json.minisig"
Write-Host "  Checksums : SHA256SUMS.txt"
Write-Host "  Verify    : VERIFY.txt"
Write-Host "  Zip       : $zipName"
