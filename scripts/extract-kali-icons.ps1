$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.IO.Compression.FileSystem
$taskRoot = Split-Path -Parent $PSScriptRoot
$taskZip = [IO.Compression.ZipFile]::OpenRead((Join-Path $taskRoot 'artifacts/kali-reference/theme-icons.zip'))
try {
    $taskEntries = @{}
    foreach ($taskEntry in $taskZip.Entries) {
        $taskRelative = $taskEntry.FullName -replace '^.*?/share/icons/Flat-Remix-Blue-Dark/', ''
        $taskEntries[$taskRelative] = $taskEntry
    }
    function Read-ThemeIcon([string]$taskPath, [int]$taskDepth = 0) {
        if ($taskDepth -gt 12 -or -not $taskEntries.ContainsKey($taskPath)) { return $null }
        $taskReader = [IO.StreamReader]::new($taskEntries[$taskPath].Open())
        try { $taskContent = $taskReader.ReadToEnd() } finally { $taskReader.Dispose() }
        if ($taskContent -match '<svg') { return $taskContent }
        $taskParent = $taskPath.Substring(0, $taskPath.LastIndexOf('/') + 1)
        $taskResolved = [Uri]::new([Uri]'https://icons.invalid/', $taskParent + $taskContent.Trim()).AbsolutePath.TrimStart('/')
        return Read-ThemeIcon $taskResolved ($taskDepth + 1)
    }
    $taskNames = @('utilities-terminal', 'user-desktop', 'system-file-manager', 'accessories-text-editor', 'firefox-esr', 'firefox', 'web-browser', 'applications-other', 'applications-accessories', 'starred', 'document-open-recent', 'folder', 'kali-panel-menu', 'preferences-system', 'system-log-out', 'system-lock-screen', 'system-shutdown', 'applications-system', 'applications-internet', 'applications-development', 'accessories-calculator', 'folder-favorites', 'exploit-db', 'network-wired-symbolic', 'audio-volume-high-symbolic', 'audio-volume-muted-symbolic', 'notification-symbolic', 'battery-full-charged-symbolic', 'system-lock-screen-symbolic', 'system-log-out-symbolic')
    $taskOutput = Join-Path $taskRoot 'artifacts/kali-reference/theme'
    [IO.Directory]::CreateDirectory($taskOutput) | Out-Null
    foreach ($taskName in $taskNames) {
        $taskMatch = $taskEntries.Keys | Where-Object { $_ -match ('(^|/)' + [regex]::Escape($taskName) + '\.svg$') } | Sort-Object @{Expression={ if ($_ -like 'apps/scalable/*') {0} elseif ($_ -like 'places/scalable/*') {1} else {2} }}, Length | Select-Object -First 1
        if ($taskMatch) {
            $taskContent = Read-ThemeIcon $taskMatch
            if ($taskContent) { [IO.File]::WriteAllText((Join-Path $taskOutput ($taskName + '.svg')), $taskContent) }
        }
    }
    Get-ChildItem -LiteralPath $taskOutput -Name
} finally { $taskZip.Dispose() }
