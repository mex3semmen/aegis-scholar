Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Get-RepoRoot {
    Split-Path -Parent $PSScriptRoot
}

$repoRoot = Get-RepoRoot
$appFile = Join-Path $repoRoot 'src\App.tsx'
$workspaceDir = Join-Path $repoRoot 'src\workspaces'
$workspaceFiles = @()

if (Test-Path $workspaceDir) {
    $workspaceFiles = Get-ChildItem -Path $workspaceDir -Filter '*.tsx' | Select-Object -ExpandProperty FullName
}

$filesToScan = @($appFile) + $workspaceFiles
$filesToScan = $filesToScan | Where-Object { Test-Path $_ }

$patterns = @(
    'SourceRecord.path',
    '.aegis',
    'locators',
    'relative_path',
    'export_answer_artifacts',
    'inspect_answer_artifact_export_bundle',
    'build_evidence_pack',
    'build_answer_draft',
    'build_grounded_answer',
    'build_final_answer',
    'invoke('
)

Write-Host 'V1 scope check review leads'
Write-Host "Repo root: $repoRoot"
Write-Host 'Scanned files:'
$filesToScan | ForEach-Object { Write-Host " - $_" }

foreach ($pattern in $patterns) {
    Write-Host ""
    Write-Host "Pattern: $pattern"
    $matches = @()
    if ($filesToScan.Count -gt 0) {
        $matches = @(Select-String -Path $filesToScan -Pattern $pattern -SimpleMatch)
    }

    if ($matches.Count -eq 0) {
        Write-Host '  No matches.'
        continue
    }

    foreach ($match in $matches) {
        Write-Host ("  {0}:{1}: {2}" -f $match.Path, $match.LineNumber, $match.Line.Trim())
    }
}

Write-Host ''
Write-Host 'Review guidance: matches are leads for manual review, not automatic failures.'
