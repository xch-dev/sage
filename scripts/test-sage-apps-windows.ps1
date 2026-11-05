$ErrorActionPreference = 'Stop'
$repo = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$manifest = Join-Path $PSScriptRoot 'sage-apps-test.manifest'

# The test harness has no application manifest. Sage's UI dependencies import
# TaskDialogIndirect, which Windows loads from Common Controls version 6.
$mt = Get-Command mt.exe -ErrorAction SilentlyContinue | Select-Object -First 1 -ExpandProperty Source
if (-not $mt) {
    $kits = Join-Path ${env:ProgramFiles(x86)} 'Windows Kits\10\bin'
    $mt = Get-ChildItem -LiteralPath $kits -Filter mt.exe -Recurse -File |
        Where-Object { $_.DirectoryName -like '*\x64' } |
        Sort-Object FullName -Descending |
        Select-Object -First 1 -ExpandProperty FullName
}
if (-not $mt) { throw 'Windows SDK mt.exe (x64) is required.' }

Push-Location $repo
try {
    & cargo test -p sage-apps --lib sandbox --no-run
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

    $exe = Get-ChildItem -LiteralPath (Join-Path $repo 'target\debug\deps') -Filter 'sage_apps-*.exe' -File |
        Sort-Object LastWriteTimeUtc -Descending |
        Select-Object -First 1
    if (-not $exe) { throw 'Sage apps test executable was not produced.' }

    & $mt -manifest $manifest "-outputresource:$($exe.FullName);#1"
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

    & $exe.FullName sandbox --nocapture
    exit $LASTEXITCODE
} finally {
    Pop-Location
}
