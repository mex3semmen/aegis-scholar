param(
    [switch] $Fast,
    [switch] $BackendOnly
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Get-RepoRoot {
    Split-Path -Parent $PSScriptRoot
}

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

if ($Fast -and $BackendOnly) {
    throw 'Use either -Fast or -BackendOnly, not both.'
}

function Invoke-Verification {
    param(
        [Parameter(Mandatory = $true)]
        [string] $Root,
        [Parameter(Mandatory = $true)]
        [switch] $FastMode,
        [Parameter(Mandatory = $true)]
        [switch] $BackendOnlyMode
    )

    Push-Location $Root
    try {
        function New-Step {
            param(
                [Parameter(Mandatory = $true)]
                [string] $Label,
                [Parameter(Mandatory = $true)]
                [scriptblock] $Action
            )

            [pscustomobject]@{
                Label = $Label
                Action = $Action
            }
        }

        $steps = if ($FastMode) {
            @(
                New-Step 'npm run build' { npm run build }
                New-Step 'npm run test:browser-smoke' { npm run test:browser-smoke }
                New-Step 'cargo check' { cargo check --manifest-path .\src-tauri\Cargo.toml }
                New-Step 'git diff --check' { git diff --check }
            )
        } elseif ($BackendOnlyMode) {
            @(
                New-Step 'cargo test final_answer' { cargo test --manifest-path .\src-tauri\Cargo.toml final_answer -- --nocapture }
                New-Step 'cargo test answer' { cargo test --manifest-path .\src-tauri\Cargo.toml answer -- --nocapture }
                New-Step 'cargo test pipeline' { cargo test --manifest-path .\src-tauri\Cargo.toml pipeline -- --nocapture }
                New-Step 'cargo check' { cargo check --manifest-path .\src-tauri\Cargo.toml }
                New-Step 'git diff --check' { git diff --check }
            )
        } else {
            @(
                New-Step 'npm run build' { npm run build }
                New-Step 'npm run test:browser-smoke' { npm run test:browser-smoke }
                New-Step 'cargo test final_answer' { cargo test --manifest-path .\src-tauri\Cargo.toml final_answer -- --nocapture }
                New-Step 'cargo test answer' { cargo test --manifest-path .\src-tauri\Cargo.toml answer -- --nocapture }
                New-Step 'cargo test pipeline' { cargo test --manifest-path .\src-tauri\Cargo.toml pipeline -- --nocapture }
                New-Step 'cargo check' { cargo check --manifest-path .\src-tauri\Cargo.toml }
                New-Step 'git diff --check' { git diff --check }
            )
        }

        foreach ($step in $steps) {
            Invoke-Step -Label $step.Label -Action $step.Action
        }
    }
    finally {
        Pop-Location
    }
}

Invoke-Verification -Root (Get-RepoRoot) -FastMode:$Fast -BackendOnlyMode:$BackendOnly
