param(
  [string]$ProjectRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path,
  [string]$OutDir = ""
)

$ErrorActionPreference = "Stop"

function Get-VersionFromCargoToml([string]$cargoTomlPath) {
  $content = Get-Content -Raw -Encoding UTF8 $cargoTomlPath
  if ($content -match 'version\s*=\s*"([^"]+)"') {
    return $Matches[1]
  }
  return "0.0.0"
}

Write-Host "ProjectRoot: $ProjectRoot"

$cargoToml = Join-Path $ProjectRoot "Cargo.toml"
if (!(Test-Path $cargoToml)) {
  throw "Cargo.toml not found at $cargoToml"
}

$version = Get-VersionFromCargoToml $cargoToml
Write-Host "Version: $version"

if ($OutDir -eq "") {
  $OutDir = Join-Path $ProjectRoot ("dist\v" + $version)
}

$distRoot = $OutDir
$distApp  = Join-Path $distRoot "tongyi-translator"
$zipPath  = Join-Path $distRoot ("tongyi-translator-v" + $version + "-windows-x64.zip")

Write-Host "Dist: $distApp"
New-Item -ItemType Directory -Force -Path $distApp | Out-Null

Push-Location $ProjectRoot
try {
  Write-Host "Building release..."
  cargo build --release

  $exeSrc = Join-Path $ProjectRoot "target\release\tongyi-translator.exe"
  if (!(Test-Path $exeSrc)) {
    throw "Release exe not found: $exeSrc"
  }

  Write-Host "Copying files..."
  Copy-Item -Force $exeSrc (Join-Path $distApp "tongyi-translator.exe")

  # scripts (needed for Marian offline)
  $scriptsSrc = Join-Path $ProjectRoot "scripts"
  if (Test-Path $scriptsSrc) {
    New-Item -ItemType Directory -Force -Path (Join-Path $distApp "scripts") | Out-Null
    Copy-Item -Force (Join-Path $scriptsSrc "marian_translate.py")       (Join-Path $distApp "scripts\marian_translate.py")       -ErrorAction SilentlyContinue
    Copy-Item -Force (Join-Path $scriptsSrc "marian_download_models.py") (Join-Path $distApp "scripts\marian_download_models.py") -ErrorAction SilentlyContinue
  }

  # docs
  $readme = Join-Path $ProjectRoot "README.md"
  if (Test-Path $readme) { Copy-Item -Force $readme (Join-Path $distApp "README.md") }

  $design = Join-Path $ProjectRoot "design.md"
  if (Test-Path $design) { Copy-Item -Force $design (Join-Path $distApp "design.md") }

  $summary = Join-Path $ProjectRoot "summary.txt"
  if (Test-Path $summary) { Copy-Item -Force $summary (Join-Path $distApp "summary.txt") }

  $license = Join-Path $ProjectRoot "LICENSE"
  if (Test-Path $license) { Copy-Item -Force $license (Join-Path $distApp "LICENSE") }

  # config example (optional)
  $cfgExample = Join-Path $ProjectRoot "config.toml.example"
  if (Test-Path $cfgExample) {
    Copy-Item -Force $cfgExample (Join-Path $distApp "config.toml.example")
  }

  # Create empty models folder (models are too large to ship)
  New-Item -ItemType Directory -Force -Path (Join-Path $distApp "models") | Out-Null

  # Zip
  if (Test-Path $zipPath) { Remove-Item -Force $zipPath }
  Write-Host "Creating zip: $zipPath"
  Compress-Archive -Path (Join-Path $distApp "*") -DestinationPath $zipPath

  Write-Host "Done."
  Write-Host "Output folder: $distApp"
  Write-Host "Zip: $zipPath"
}
finally {
  Pop-Location
}
