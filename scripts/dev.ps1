param([ValidateSet('dev','check','build','build:web','test')][string]$Task='dev')
$ErrorActionPreference='Stop'
$taskRoot = Split-Path -Parent $PSScriptRoot
Set-Location -LiteralPath $taskRoot
if (Test-Path -LiteralPath (Join-Path $taskRoot '.tools/cargo/bin/cargo.exe')) {
    $env:CARGO_HOME=Join-Path $taskRoot '.tools/cargo'
    $env:RUSTUP_HOME=Join-Path $taskRoot '.tools/rustup'
    $env:PATH="$env:CARGO_HOME\bin;$env:PATH"
}
$taskPnpm=Get-Command pnpm -ErrorAction SilentlyContinue
if (-not $taskPnpm) {
    $taskFallback=Join-Path $env:USERPROFILE '.cache/codex-runtimes/codex-primary-runtime/dependencies/bin/fallback'
    if (Test-Path -LiteralPath (Join-Path $taskFallback 'pnpm.cmd')) { $env:PATH="$taskFallback;$env:PATH" }
}
& pnpm $Task
exit $LASTEXITCODE
