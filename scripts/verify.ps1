Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Invoke-Step {
    param(
        [Parameter(Mandatory = $true)]
        [string] $Label,
        [Parameter(Mandatory = $true)]
        [scriptblock] $Action
    )

    Write-Host "==> $Label"
    & $Action
    if ($LASTEXITCODE -ne 0) {
        throw "$Label failed with exit code $LASTEXITCODE"
    }
}

Invoke-Step 'npm run build' { npm run build }
Invoke-Step 'cargo test final_answer' { cargo test --manifest-path .\src-tauri\Cargo.toml final_answer -- --nocapture }
Invoke-Step 'cargo test answer' { cargo test --manifest-path .\src-tauri\Cargo.toml answer -- --nocapture }
Invoke-Step 'cargo test pipeline' { cargo test --manifest-path .\src-tauri\Cargo.toml pipeline -- --nocapture }
Invoke-Step 'cargo check' { cargo check --manifest-path .\src-tauri\Cargo.toml }
Invoke-Step 'git diff --check' { git diff --check }
