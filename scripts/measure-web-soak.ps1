param(
    [Parameter(Mandatory = $true)][int]$GameProcessId,
    [ValidateRange(1, 720)][int]$Minutes = 120,
    [ValidateRange(5, 60)][int]$IntervalSeconds = 30
)
$ErrorActionPreference = 'Stop'
$workspace = Split-Path -Parent $PSScriptRoot
$directory = Join-Path $workspace 'artifacts/web-audit'
New-Item -ItemType Directory -Path $directory -Force | Out-Null
$process = Get-Process -Id $GameProcessId
if ($process.ProcessName -ne 'game-hacker') { throw 'Select the game-hacker process, not another application.' }
$output = Join-Path $directory ("desktop-soak-{0}.csv" -f (Get-Date -Format 'yyyyMMdd-HHmmss'))
$started = Get-Date
$end = $started.AddMinutes($Minutes)
Write-Output "Recording the game process and descendant WebView processes to $output. Global listener and SQLite samples are recorded separately by the native QA harness."
while ((Get-Date) -lt $end) {
    $current = Get-Process -Id $GameProcessId -ErrorAction SilentlyContinue
    if (!$current -or $current.StartTime -ne $process.StartTime) { break }
    $processRows = @(Get-CimInstance Win32_Process -Property ProcessId,ParentProcessId,Name)
    $descendantIds = [System.Collections.Generic.HashSet[int]]::new()
    [void]$descendantIds.Add($GameProcessId)
    do {
        $changed = $false
        foreach ($row in $processRows) {
            if ($descendantIds.Contains([int]$row.ParentProcessId) -and $descendantIds.Add([int]$row.ProcessId)) { $changed = $true }
        }
    } while ($changed)
    $family = @(Get-Process -Id @($descendantIds) -ErrorAction SilentlyContinue)
    [pscustomobject]@{
        elapsedSeconds = [math]::Round(((Get-Date) - $started).TotalSeconds)
        workingSetBytes = $current.WorkingSet64
        privateBytes = $current.PrivateMemorySize64
        handles = $current.HandleCount
        cpuSeconds = $current.TotalProcessorTime.TotalSeconds
        threads = $current.Threads.Count
        responding = $current.Responding
        processCount = $family.Count
        treeWorkingSetBytes = ($family | Measure-Object WorkingSet64 -Sum).Sum
        treePrivateBytes = ($family | Measure-Object PrivateMemorySize64 -Sum).Sum
        treeHandles = ($family | Measure-Object HandleCount -Sum).Sum
        treeCpuSeconds = ($family | ForEach-Object { $_.TotalProcessorTime.TotalSeconds } | Measure-Object -Sum).Sum
    } | Export-Csv -LiteralPath $output -NoTypeInformation -Append -Encoding utf8
    Start-Sleep -Seconds $IntervalSeconds
}
Write-Output "Measurement finished: $output"
