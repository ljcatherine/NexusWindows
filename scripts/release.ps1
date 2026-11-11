<#
.SYNOPSIS
  Build, test, package and publish a Nexus Desktop release to the
  GitHub repository ljcatherine/NexusWindows.

.DESCRIPTION
  * Version line comes from the workspace Cargo.toml (major.minor). The patch is
    the next one after existing vMAJOR.MINOR.* tags (0.1.0 -> 0.1.1 -> ...).
    Never changes major/minor; pass -AllowVersionLineChange only with owner approval.
  * Runs `cargo test --workspace` and stops on failure.
  * Builds the release exe, a portable zip, and (when WiX is usable) an MSI.
  * Writes SHA-256 sums and publishes everything with `gh release create`.
  * No GitHub Actions or hosted build services are used.

.EXAMPLE
  powershell -ExecutionPolicy Bypass -File scripts\release.ps1
  powershell -ExecutionPolicy Bypass -File scripts\release.ps1 -DryRun
#>
param(
  [switch]$DryRun,
  [switch]$SkipTests,
  [switch]$NoMsi,
  [string]$Notes = ""
)
$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
Set-Location $root
$env:PATH = "$env:USERPROFILE\.cargo\bin;$env:USERPROFILE\.dotnet\tools;$env:PATH"

# ---- version -------------------------------------------------------------
$cargo = Get-Content Cargo.toml -Raw
if ($cargo -notmatch '(?m)^version = "(\d+)\.(\d+)\.(\d+)"') { throw "workspace version not found" }
$major = [int]$Matches[1]; $minor = [int]$Matches[2]; $patch = [int]$Matches[3]
$tags = @(git tag --list "v$major.$minor.*")
$used = @($tags | ForEach-Object { if ($_ -match "^v$major\.$minor\.(\d+)$") { [int]$Matches[1] } })
if ($used.Count -gt 0) {
  $next = ([int]($used | Measure-Object -Maximum).Maximum) + 1
  if ($next -gt $patch) { $patch = $next }
}
$version = "$major.$minor.$patch"
$tag = "v$version"
Write-Host "Releasing Nexus Desktop $version" -ForegroundColor Cyan

if ((git status --porcelain) -and -not $DryRun) { throw "Working tree is not clean; commit first." }

# Write the version into Cargo.toml if it changed.
$newCargo = $cargo -replace '(?m)^version = "\d+\.\d+\.\d+"', "version = `"$version`""
if ($newCargo -ne $cargo -and -not $DryRun) {
  Set-Content Cargo.toml $newCargo -NoNewline
  cargo update --workspace -q
  git add Cargo.toml Cargo.lock
  git commit -q -m "Release $tag"
}

# ---- test + build ----------------------------------------------------------
if (-not $SkipTests) {
  cargo test --workspace
  if ($LASTEXITCODE -ne 0) { throw "tests failed" }
}
cargo build --release -p nexus-app
if ($LASTEXITCODE -ne 0) { throw "build failed" }

$dist = Join-Path $root "dist\$tag"
New-Item -ItemType Directory -Force $dist | Out-Null
$exe = Join-Path $dist "NexusDesktop-$version-x64.exe"
Copy-Item target\release\NexusDesktop.exe $exe -Force

# Portable zip: exe + docs.
$stage = Join-Path $dist "portable"
New-Item -ItemType Directory -Force $stage | Out-Null
Copy-Item target\release\NexusDesktop.exe (Join-Path $stage "NexusDesktop.exe") -Force
Copy-Item README.md, docs\STATUS.md, docs\SECURITY-MODEL.md $stage -Force
$zip = Join-Path $dist "NexusDesktop-$version-x64-portable.zip"
if (Test-Path $zip) { Remove-Item $zip }
Compress-Archive -Path "$stage\*" -DestinationPath $zip

# MSI (optional; WiX v7 requires the owner to have accepted its OSMF EULA).
$assets = @($exe, $zip)
# Linux server helper (static musl build, see docs/BUILD-AND-RELEASE.md)
cargo build -p nexus-ro --release --target x86_64-unknown-linux-musl
if ($LASTEXITCODE -ne 0) { throw "nexus-ro build failed" }
$ro = Join-Path $dist "nexus-ro-$version-x86_64-linux-musl"
Copy-Item target\x86_64-unknown-linux-musl\release\nexus-ro $ro -Force
$assets += $ro
$msi = Join-Path $dist "NexusDesktop-$version-x64.msi"
if (-not $NoMsi -and (Get-Command wix -ErrorAction SilentlyContinue)) {
  wix build installer\nexus.wxs -arch x64 -d Version=$version -d ExePath=target\release\NexusDesktop.exe -d IconPath=crates\nexus-app\assets\nexus.ico -o $msi
  if ($LASTEXITCODE -eq 0) { $assets += $msi } else { Write-Warning "MSI not built (see WiX output). Publishing without installer." }
}

# Checksums + metadata.
$sums = Join-Path $dist "SHA256SUMS.txt"
$assets | ForEach-Object { "{0}  {1}" -f (Get-FileHash $_ -Algorithm SHA256).Hash.ToLower(), (Split-Path $_ -Leaf) } | Set-Content $sums
$commit = git rev-parse HEAD
$meta = Join-Path $dist "build-metadata.json"
@{ version = $version; commit = $commit; rustc = (rustc --version); built = (Get-Date).ToUniversalTime().ToString("o"); target = "x86_64-pc-windows-msvc"; signed = $false } | ConvertTo-Json | Set-Content $meta
$assets += @($sums, $meta)

if ($DryRun) { Write-Host "Dry run: built $($assets -join ', ')"; exit 0 }

# ---- publish ---------------------------------------------------------------
git tag -a $tag -m "Nexus Desktop $version"
git push origin HEAD
git push origin $tag
$body = @"
Nexus Desktop $version for Windows x64 (unsigned).

Commit: $commit

$Notes

Assets
- NexusDesktop-$version-x64.exe - portable executable
- NexusDesktop-$version-x64-portable.zip - exe with README, STATUS and security model
$(if ($assets -contains $msi) { "- NexusDesktop-$version-x64.msi - per-user installer (no admin rights)" } else { "- (no MSI in this release)" })
- nexus-ro-$version-x86_64-linux-musl - optional server helper (see docs/SECURITY-MODEL.md); not installed anywhere by this release
- SHA256SUMS.txt, build-metadata.json

The executable is not code-signed; Windows SmartScreen may warn on first run.
See docs/STATUS.md for verified features and known limitations.
"@
gh release create $tag @assets --repo ljcatherine/NexusWindows --title "Nexus Desktop $version (unsigned)" --notes $body
Write-Host "Published $tag" -ForegroundColor Green
