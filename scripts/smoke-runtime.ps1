[CmdletBinding()]
param(
    [string]$TranscriptPath = ""
)

$ErrorActionPreference = "Stop"
$script:ownedMarkerPaths = [System.Collections.Generic.HashSet[string]]::new(
    [System.StringComparer]::OrdinalIgnoreCase
)
$script:ownedProcesses = [System.Collections.Generic.List[System.Diagnostics.Process]]::new()
$script:smokeFailed = $false

Add-Type @"
using System;
using System.Runtime.InteropServices;
using System.Text;

public static class WubiLexSmokeNativeMethods
{
    private delegate bool EnumWindowsProc(IntPtr window, IntPtr parameter);

    [DllImport("user32.dll")]
    private static extern bool EnumWindows(EnumWindowsProc callback, IntPtr parameter);

    [DllImport("user32.dll")]
    private static extern bool IsWindowVisible(IntPtr window);

    [DllImport("user32.dll", SetLastError = true)]
    private static extern uint GetWindowThreadProcessId(IntPtr window, out uint processId);

    [DllImport("user32.dll", CharSet = CharSet.Unicode)]
    private static extern int GetWindowText(IntPtr window, StringBuilder text, int maximumCount);

    public static bool HasVisibleWindow(uint expectedProcessId, string expectedTitle)
    {
        bool found = false;
        EnumWindows((window, parameter) =>
        {
            uint processId;
            GetWindowThreadProcessId(window, out processId);
            if (processId != expectedProcessId || !IsWindowVisible(window))
            {
                return true;
            }

            var title = new StringBuilder(256);
            GetWindowText(window, title, title.Capacity);
            if (String.Equals(title.ToString(), expectedTitle, StringComparison.Ordinal))
            {
                found = true;
                return false;
            }
            return true;
        }, IntPtr.Zero);
        return found;
    }
}
"@

function Test-IsAdministrator {
    $identity = [Security.Principal.WindowsIdentity]::GetCurrent()
    $principal = [Security.Principal.WindowsPrincipal]::new($identity)
    return $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
}

if (-not (Test-IsAdministrator)) {
    $repositoryRoot = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot ".."))
    $transcript = [IO.Path]::GetFullPath((Join-Path $repositoryRoot "target\smoke-runtime-transcript.log"))
    Remove-Item -LiteralPath $transcript -Force -ErrorAction SilentlyContinue
    $arguments = @(
        "-NoProfile",
        "-ExecutionPolicy", "Bypass",
        "-File", "`"$PSCommandPath`"",
        "-TranscriptPath", "`"$transcript`""
    )
    $elevated = Start-Process powershell.exe -Verb RunAs -ArgumentList $arguments -Wait -PassThru -WindowStyle Hidden
    if (Test-Path -LiteralPath $transcript -PathType Leaf) {
        Get-Content -LiteralPath $transcript
        Remove-Item -LiteralPath $transcript -Force
    }
    exit $elevated.ExitCode
}

$repositoryRoot = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot ".."))
$transcribing = $false
if ($TranscriptPath) {
    $resolvedTranscript = [IO.Path]::GetFullPath($TranscriptPath)
    $expectedTranscript = [IO.Path]::GetFullPath(
        (Join-Path $repositoryRoot "target\smoke-runtime-transcript.log")
    )
    if (-not $resolvedTranscript.Equals($expectedTranscript, [StringComparison]::OrdinalIgnoreCase)) {
        throw "Transcript path is outside the owned smoke target."
    }
    Start-Transcript -LiteralPath $resolvedTranscript -Force | Out-Null
    $transcribing = $true
}
$executable = [IO.Path]::GetFullPath((Join-Path $repositoryRoot "target\debug\wubilex-app.exe"))
$expectedTargetRoot = [IO.Path]::GetFullPath((Join-Path $repositoryRoot "target"))
if (-not $executable.StartsWith($expectedTargetRoot, [StringComparison]::OrdinalIgnoreCase)) {
    throw "Resolved executable escaped the repository target directory."
}
if (-not (Test-Path -LiteralPath $executable -PathType Leaf)) {
    throw "Debug executable is missing: $executable"
}

$originalSmokeDataRoot = $env:WUBILEX_SMOKE_DATA_ROOT
$smokeAppDataRoot = [IO.Path]::GetFullPath(
    (Join-Path $expectedTargetRoot "smoke-runtime-appdata")
)
if (-not $smokeAppDataRoot.StartsWith($expectedTargetRoot, [StringComparison]::OrdinalIgnoreCase)) {
    throw "Resolved smoke APPDATA escaped the repository target directory."
}
if (Test-Path -LiteralPath $smokeAppDataRoot) {
    Remove-Item -LiteralPath $smokeAppDataRoot -Recurse -Force
}
New-Item -ItemType Directory -Path $smokeAppDataRoot | Out-Null
$env:WUBILEX_SMOKE_DATA_ROOT = $smokeAppDataRoot

$appDataDirectory = Join-Path $smokeAppDataRoot "com.wubilex.tool"
$sessionDirectory = Join-Path $appDataDirectory "sessions"
$logDirectory = Join-Path $appDataDirectory "logs"

function Get-MarkerPaths {
    if (-not (Test-Path -LiteralPath $sessionDirectory -PathType Container)) {
        return @()
    }
    return @(
        Get-ChildItem -LiteralPath $sessionDirectory -File -ErrorAction Stop |
            Where-Object { $_.Name -match '^wubilex-session-[A-Za-z0-9-]+\.json$' } |
            ForEach-Object { $_.FullName }
    )
}

function Get-NewMarkerPaths([string[]]$baseline) {
    return @(Get-MarkerPaths | Where-Object { $_ -notin $baseline })
}

function Wait-ForSingleNewMarker([string[]]$baseline, [string]$description) {
    Wait-Until { @(Get-NewMarkerPaths -baseline $baseline).Count -eq 1 } $description
    $markers = @(Get-NewMarkerPaths -baseline $baseline)
    if ($markers.Count -ne 1) {
        throw "Expected one marker for $description, found $($markers.Count)."
    }
    return [string]$markers[0]
}

function Wait-Until([scriptblock]$condition, [string]$description, [int]$seconds = 15) {
    $deadline = [DateTime]::UtcNow.AddSeconds($seconds)
    do {
        if (& $condition) {
            return
        }
        Start-Sleep -Milliseconds 200
    } while ([DateTime]::UtcNow -lt $deadline)
    throw "Timed out waiting for $description."
}

function Start-OwnedProcess([string[]]$arguments) {
    $process = if ($null -eq $arguments -or $arguments.Count -eq 0) {
        Start-Process -FilePath $executable -PassThru
    } else {
        Start-Process -FilePath $executable -ArgumentList $arguments -PassThru
    }
    $script:ownedProcesses.Add($process)
    return $process
}

function Get-LogEvents([string]$eventName) {
    if (-not (Test-Path -LiteralPath $logDirectory -PathType Container)) {
        return @()
    }
    $records = foreach ($file in Get-ChildItem -LiteralPath $logDirectory -File -Filter "wubilex.*.jsonl") {
        foreach ($line in Get-Content -LiteralPath $file.FullName -ErrorAction Stop) {
            try {
                $record = $line | ConvertFrom-Json -ErrorAction Stop
                if ($record.event -eq $eventName) {
                    $record
                }
            } catch {
                continue
            }
        }
    }
    return @($records)
}

function Wait-ForExit([Diagnostics.Process]$process, [string]$description) {
    if (-not $process.WaitForExit(15000)) {
        throw "Timed out waiting for $description to exit."
    }
}

function Test-MainWindowVisible([Diagnostics.Process]$process) {
    $process.Refresh()
    return [WubiLexSmokeNativeMethods]::HasVisibleWindow([uint32]$process.Id, "WubiLex")
}

try {
    $existingProcesses = @(
        Get-Process -Name "wubilex-app" -ErrorAction SilentlyContinue |
            Where-Object { $_.Path -eq $executable }
    )
    if ($existingProcesses.Count -ne 0) {
        throw "Close the existing debug WubiLex process before running smoke validation."
    }

    $baselineMarkers = @(Get-MarkerPaths)
    $secondaryEventBaseline = @(Get-LogEvents "secondary_launch_received").Count
    $argumentNoticeBaseline = @(Get-LogEvents "launch_argument_notice").Count
    $trayCreatedBaseline = @(Get-LogEvents "tray_created").Count
    $delayScheduledBaseline = @(Get-LogEvents "tray_delay_scheduled").Count
    $delayCancelledBaseline = @(Get-LogEvents "tray_delay_cancelled").Count

    Write-Host "[1/4] Starting hidden primary instance"
    $primary = Start-OwnedProcess @("/tray")
    $primaryMarker = Wait-ForSingleNewMarker -baseline $baselineMarkers -description "primary session marker"
    Write-Host "      primary marker created"
    [void]$script:ownedMarkerPaths.Add($primaryMarker)
    $primary.Refresh()
    Write-Host "      primary process is running"
    if ($primary.HasExited) {
        throw "Hidden primary instance exited unexpectedly."
    }
    if (Test-MainWindowVisible $primary) {
        throw "/tray primary window was visible before a secondary request."
    }
    Write-Host "      primary window is hidden"

    Write-Host "[2/4] Verifying second-instance handoff and visible warning"
    $secondary = Start-OwnedProcess @("--navigate", "/settings/runtime")
    Wait-ForExit $secondary "navigation secondary instance"
    Wait-Until {
        Test-MainWindowVisible $primary
    } "primary window activation"
    Start-Sleep -Milliseconds 3500
    $delayWasScheduled = @(
        Get-LogEvents "tray_delay_scheduled"
    ).Count -gt $delayScheduledBaseline
    if (
        $delayWasScheduled -and
        @(Get-LogEvents "tray_delay_cancelled").Count -le $delayCancelledBaseline
    ) {
        throw "A scheduled tray delay was not cancelled by the secondary request."
    }
    if (@(Get-LogEvents "tray_created").Count -ne $trayCreatedBaseline) {
        throw "A tray icon was created after the hidden launch was restored."
    }

    $invalid = Start-OwnedProcess @("--unsupported-smoke-argument")
    Wait-ForExit $invalid "invalid-argument secondary instance"
    Wait-Until {
        @(Get-LogEvents "secondary_launch_received").Count -ge ($secondaryEventBaseline + 2)
    } "two secondary launch log records"
    Wait-Until {
        @(Get-LogEvents "launch_argument_notice").Count -ge ($argumentNoticeBaseline + 1)
    } "redacted invalid-argument log record"

    Write-Host "[3/4] Verifying close-to-tray and second-instance restore"
    if (-not $primary.CloseMainWindow()) {
        throw "Primary window did not accept a normal close request."
    }
    Wait-Until { -not (Test-MainWindowVisible $primary) } "primary window hide"
    $primary.Refresh()
    if ($primary.HasExited) {
        throw "Default close action exited instead of hiding to tray."
    }
    Wait-Until {
        @(Get-LogEvents "tray_created").Count -eq ($trayCreatedBaseline + 1)
    } "single owned tray creation"

    $restore = Start-OwnedProcess @()
    Wait-ForExit $restore "restore secondary instance"
    Wait-Until { Test-MainWindowVisible $primary } "close-to-tray restore"
    if (@(Get-LogEvents "tray_created").Count -ne ($trayCreatedBaseline + 1)) {
        throw "Restore created a duplicate tray icon."
    }

    Write-Host "[4/4] Verifying abnormal evidence and closeAction=exit cleanup"
    Stop-Process -Id $primary.Id -Force
    Wait-ForExit $primary "forced abnormal primary instance"
    if (-not (Test-Path -LiteralPath $primaryMarker -PathType Leaf)) {
        throw "Forced termination did not preserve its session marker."
    }

    $configPath = Join-Path $appDataDirectory "config.toml"
    if (-not (Test-Path -LiteralPath $configPath -PathType Leaf)) {
        throw "Isolated runtime config was not created."
    }
    $configText = Get-Content -LiteralPath $configPath -Raw
    $closeActionPattern = '(?m)^closeAction = "minimizeToTray"$'
    $closeActionMatches = [regex]::Matches($configText, $closeActionPattern)
    if ($closeActionMatches.Count -ne 1) {
        throw "Expected one default closeAction in the isolated runtime config."
    }
    $exitConfig = [regex]::Replace(
        $configText,
        $closeActionPattern,
        'closeAction = "exit"',
        1
    )
    [IO.File]::WriteAllText(
        $configPath,
        $exitConfig,
        [Text.UTF8Encoding]::new($false)
    )

    $beforeRecoveryMarkers = @(Get-MarkerPaths)
    $recoveryLogBaseline = @(Get-LogEvents "application_started").Count
    $recoveryTrayBaseline = @(Get-LogEvents "tray_created").Count
    $recovery = Start-OwnedProcess @()
    $recoveryMarker = Wait-ForSingleNewMarker -baseline $beforeRecoveryMarkers -description "recovery session marker"
    [void]$script:ownedMarkerPaths.Add($recoveryMarker)
    Wait-Until {
        $events = @(Get-LogEvents "application_started")
        $events.Count -gt $recoveryLogBaseline -and
            [int]$events[-1].previous_abnormal_session_count -ge 1
    } "abnormal-session detection log record"
    Wait-Until { Test-MainWindowVisible $recovery } "normal recovery window"
    Start-Sleep -Milliseconds 800
    if (@(Get-LogEvents "tray_created").Count -ne $recoveryTrayBaseline) {
        throw "Normal visible startup created a tray icon before hide."
    }
    if (-not $recovery.CloseMainWindow()) {
        throw "Recovery window did not accept a normal close request."
    }
    Wait-ForExit $recovery "clean recovery instance"
    Wait-Until { -not (Test-Path -LiteralPath $recoveryMarker) } "recovery marker removal"
    [void]$script:ownedMarkerPaths.Remove($recoveryMarker)

    Remove-Item -LiteralPath $primaryMarker -Force
    [void]$script:ownedMarkerPaths.Remove($primaryMarker)
    Write-Host "      visible checklist: drag/double-click/title buttons, taskbar/tray left-click, two-item tray menu, tray exit, DPI/multi-monitor restore"
    Write-Host "runtime smoke: passed"
} catch {
    $script:smokeFailed = $true
    Write-Error $_
} finally {
    foreach ($process in $script:ownedProcesses) {
        try {
            $process.Refresh()
            if (-not $process.HasExited) {
                Stop-Process -Id $process.Id -Force -ErrorAction SilentlyContinue
            }
        } catch {
            continue
        }
    }
    foreach ($path in $script:ownedMarkerPaths) {
        if ([string]::IsNullOrWhiteSpace($path)) {
            continue
        }
        $resolvedParent = [IO.Path]::GetFullPath((Split-Path -Parent $path))
        $expectedParent = [IO.Path]::GetFullPath($sessionDirectory)
        $name = Split-Path -Leaf $path
        if (
            $resolvedParent.Equals($expectedParent, [StringComparison]::OrdinalIgnoreCase) -and
            $name -match '^wubilex-session-[A-Za-z0-9-]+\.json$'
        ) {
            Remove-Item -LiteralPath $path -Force -ErrorAction SilentlyContinue
        }
    }
    if ($transcribing) {
        Stop-Transcript | Out-Null
    }
    $env:WUBILEX_SMOKE_DATA_ROOT = $originalSmokeDataRoot
    if (Test-Path -LiteralPath $smokeAppDataRoot) {
        Remove-Item -LiteralPath $smokeAppDataRoot -Recurse -Force -ErrorAction SilentlyContinue
    }
}

if ($script:smokeFailed) {
    exit 1
}
