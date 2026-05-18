param(
  [Parameter(Mandatory)]
  [string]$InstallerPath,
  [string]$PublicKeyPath  = ".\mtw-public.key",
  [string]$PrivateKeyPath = "C:\Users\MadTi\mtw-release-private.key",
  [string]$OutDir         = "."
)

$ErrorActionPreference = "Stop"

# --- validate inputs ---
if (!(Test-Path $InstallerPath)) {
  Write-Host "FAIL: Installer not found: $InstallerPath" -ForegroundColor Red
  exit 1
}

if (!(Get-Command minisign -ErrorAction SilentlyContinue)) {
  Write-Host "FAIL: minisign not found in PATH. Install from https://jedisct1.github.io/minisign/" -ForegroundColor Red
  exit 1
}

if (!(Test-Path $PrivateKeyPath)) {
  Write-Host "FAIL: Private key not found: $PrivateKeyPath" -ForegroundColor Red
  Write-Host "  Generate with: minisign -G -p mtw-public.key -s mtw-release-private.key" -ForegroundColor Yellow
  exit 1
}

# --- hashes ---
$installerName = Split-Path $InstallerPath -Leaf
$installerHash = (Get-FileHash $InstallerPath -Algorithm SHA256).Hash.ToLower()

Write-Host "Hashing $installerName ..." -ForegroundColor Cyan
Write-Host "  SHA256: $installerHash"

# --- manifest ---
$manifest = [ordered]@{
  publisher    = "Mad Tinker's Workshop"
  app          = "TinkerVault 42"
  version      = "42.1.0"
  vault_format = "TVLT42-1"
  geo_lock     = "GeoAstroLock42-Nautical57"
  files        = @(
    [ordered]@{
      name   = $installerName
      sha256 = $installerHash
    }
  )
  released_utc = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")
}

$manifestPath = Join-Path $OutDir "release-manifest.json"
$manifest | ConvertTo-Json -Depth 5 | Set-Content $manifestPath -Encoding UTF8
Write-Host "Manifest written: $manifestPath" -ForegroundColor Cyan

# --- sign manifest with minisign ---
# minisign reads the key password from MINISIGN_PASSWORD env var if set.
# Set it before calling this script: $env:MINISIGN_PASSWORD = "your-password"
$sigPath = "$manifestPath.minisig"
Write-Host "Signing manifest with Minisign ..." -ForegroundColor Cyan

minisign -S -s $PrivateKeyPath -m $manifestPath -x $sigPath

$exitCode = $LASTEXITCODE
$env:MINISIGN_PASSWORD = $null
if ($exitCode -ne 0) {
  Write-Host "FAIL: minisign signing failed (exit $exitCode)." -ForegroundColor Red
  exit 1
}

Write-Host "Signature written: $sigPath" -ForegroundColor Cyan

# --- verify round-trip ---
Write-Host "Verifying signature round-trip ..." -ForegroundColor Cyan

if (!(Test-Path $PublicKeyPath)) {
  Write-Host "WARN: Public key not found at $PublicKeyPath, skipping verify." -ForegroundColor Yellow
} else {
  minisign -V -p $PublicKeyPath -m $manifestPath -x $sigPath
  if ($LASTEXITCODE -ne 0) {
    Write-Host "FAIL: Round-trip verify failed." -ForegroundColor Red
    exit 1
  }
  Write-Host "Verify OK." -ForegroundColor Green
}

Write-Host ""
Write-Host "PASS: Signing complete." -ForegroundColor Green
Write-Host "  Manifest : $manifestPath"
Write-Host "  Signature: $sigPath"
