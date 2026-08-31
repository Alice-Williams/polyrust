$ErrorActionPreference = 'Stop'
$root = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$dockerfile = Get-Content -Raw (Join-Path $root '.devcontainer\Dockerfile')
$module = Get-Content -Raw (Join-Path $root 'MODULE.bazel')
$devcontainer = Get-Content -Raw (Join-Path $root '.devcontainer\devcontainer.json') | ConvertFrom-Json

$requiredDockerPins = @(
    'FROM rust:1.98.0-trixie',
    'ARG BAZELISK_VERSION=1.29.0',
    'ARG NODE_VERSION=24.20.0',
    'ARG TYPESCRIPT_VERSION=7.0.2',
    'ARG PRETTIER_VERSION=3.9.6',
    'ARG RUFF_VERSION=0.16.5',
    'ARG MYPY_VERSION=2.3.1',
    'ARG PYTEST_VERSION=9.1.1'
)
foreach ($pin in $requiredDockerPins) {
    if (-not $dockerfile.Contains($pin)) { throw "missing container pin: $pin" }
}
foreach ($pin in @('versions = ["1.98.0"]', 'version = "1.25.14"')) {
    if (-not $module.Contains($pin)) { throw "missing Bazel toolchain pin: $pin" }
}
if ($devcontainer.workspaceFolder -ne '/workspace') { throw 'unexpected container workspace' }
if ($devcontainer.remoteUser -ne 'root') { throw 'unexpected container user' }

$status = & git -C $root status --porcelain=v1
if ($LASTEXITCODE -ne 0) { throw 'git status failed' }
if ($status) { throw "checkout changed during Windows contract test: $status" }
Write-Host 'Windows checkout and Linux dev-container contract passed.'

