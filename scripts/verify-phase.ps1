Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Get-RepoRoot {
    Split-Path -Parent $PSScriptRoot
}

function Invoke-NativeStep {
    param(
        [Parameter(Mandatory = $true)]
        [string] $Label,
        [Parameter(Mandatory = $true)]
        [string] $FilePath,
        [string[]] $Arguments = @()
    )

    Write-Host "==> $Label"
    & $FilePath @Arguments
    if ($LASTEXITCODE -ne 0) {
        throw "$Label failed with exit code $LASTEXITCODE"
    }
}

$repoRoot = Get-RepoRoot

Push-Location $repoRoot
try {
    Invoke-NativeStep -Label 'git status --short --branch' -FilePath 'git' -Arguments @('status', '--short', '--branch')
    Invoke-NativeStep -Label 'git diff --name-status' -FilePath 'git' -Arguments @('diff', '--name-status')
    Invoke-NativeStep -Label 'git ls-files --others --exclude-standard' -FilePath 'git' -Arguments @('ls-files', '--others', '--exclude-standard')
    Invoke-NativeStep -Label 'git stash list' -FilePath 'git' -Arguments @('stash', 'list')
    Invoke-NativeStep -Label 'npx tsc --noEmit' -FilePath 'npx' -Arguments @('tsc', '--noEmit')
    Invoke-NativeStep -Label 'npm run build' -FilePath 'npm' -Arguments @('run', 'build')
    Invoke-NativeStep -Label 'cargo check --manifest-path .\src-tauri\Cargo.toml' -FilePath 'cargo' -Arguments @('check', '--manifest-path', '.\src-tauri\Cargo.toml')
    Invoke-NativeStep -Label 'git diff --check' -FilePath 'git' -Arguments @('diff', '--check')
}
finally {
    Pop-Location
}
