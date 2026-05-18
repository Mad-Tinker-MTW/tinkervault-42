$ErrorActionPreference = "Stop"

param(
  [string]$ReleaseFile,
  [string]$PrivateKeyPath = ".\mtw-release-private.pem"
)

if (!(Test-Path $ReleaseFile)) {
  throw "Release file not found: $ReleaseFile"
}

$hash = (Get-FileHash $ReleaseFile -Algorithm SHA256).Hash.ToLower()
$manifest = @{
  publisher = "Mad Tinker's Workshop"
  app = "TinkerVault"
  version = "42.1.0"
  vault_format = "TVLT42-1"
  file = (Split-Path $ReleaseFile -Leaf)
  sha256 = $hash
  released_utc = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")
} | ConvertTo-Json -Depth 5

$manifestPath = ".\release-manifest.json"
$manifest | Set-Content $manifestPath -Encoding UTF8

Write-Host "Manifest created: $manifestPath"
Write-Host "SHA256: $hash"
Write-Host ""
Write-Host "Signature step placeholder:"
Write-Host "Use OpenSSL, minisign, age-plugin-yubikey, or another chosen signing tool."
Write-Host "Keep the private key offline. Embed only public key in app later."
