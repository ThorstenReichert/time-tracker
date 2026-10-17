# One-command feedback loop (plan.md "Proof"): format, lint, tests for Rust and the UI.
# Usage: pwsh scripts/check.ps1        Ends with "ALL CHECKS PASSED" or stops at the first failure.
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
Push-Location $root
try {
    function Step([string]$name, [scriptblock]$cmd) {
        Write-Host "==> $name" -ForegroundColor Cyan
        & $cmd
        if ($LASTEXITCODE -ne 0) { throw "FAILED: $name (exit $LASTEXITCODE)" }
    }

    if (-not (Test-Path 'ui/node_modules')) { Step 'npm ci' { npm --prefix ui ci } }

    Step 'cargo fmt'    { cargo fmt --all --check }
    Step 'cargo clippy' { cargo clippy --workspace --all-targets --quiet -- -D warnings }
    Step 'cargo test'   { cargo test --workspace --quiet }
    Step 'ui typecheck' { npm --prefix ui run --silent typecheck }
    Step 'ui test'      { npm --prefix ui run --silent test }

    Write-Host 'ALL CHECKS PASSED' -ForegroundColor Green
}
finally {
    Pop-Location
}
