<#
.SYNOPSIS
Installs or uninstalls Brume for the current Windows user without elevation.
Preserves write permissions for seamless in-app autoupdates.
#>
param(
    [switch]$Uninstall,
    [switch]$Quiet
)

$ErrorActionPreference = 'Stop'
$appName = 'Brume'
$legacyAppName = 'Serein'
$publisher = 'Brume contributors'
$website = 'https://github.com/Lynx-1ST/Brume'
$installDir = Join-Path $env:LOCALAPPDATA "Programs\$appName"
$uninstallKey = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\$appName"
$legacyUninstallKey = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\$legacyAppName"
$shortcut = Join-Path ([Environment]::GetFolderPath('Programs')) "$appName.lnk"
$legacyShortcut = Join-Path ([Environment]::GetFolderPath('Programs')) "$legacyAppName.lnk"

# Check if Brume/serein is currently running
$running = Get-Process brume, serein -ErrorAction SilentlyContinue
if ($running) {
    if ($Quiet) {
        $running | Stop-Process -Force
    } else {
        throw "Brume is currently running. Please close Brume before running setup."
    }
}

if ($Uninstall) {
    if (Test-Path -LiteralPath $shortcut) {
        Remove-Item -LiteralPath $shortcut -Force
    }
    if (Test-Path -LiteralPath $legacyShortcut) {
        Remove-Item -LiteralPath $legacyShortcut -Force
    }
    if (Test-Path -LiteralPath $uninstallKey) {
        Remove-Item -LiteralPath $uninstallKey -Recurse -Force
    }
    if (Test-Path -LiteralPath $legacyUninstallKey) {
        Remove-Item -LiteralPath $legacyUninstallKey -Recurse -Force
    }
    $runKey = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Run'
    if ((Get-ItemProperty -LiteralPath $runKey -Name $appName -ErrorAction SilentlyContinue)) {
        Remove-ItemProperty -LiteralPath $runKey -Name $appName -Force
    }
    if ((Get-ItemProperty -LiteralPath $runKey -Name $legacyAppName -ErrorAction SilentlyContinue)) {
        Remove-ItemProperty -LiteralPath $runKey -Name $legacyAppName -Force
    }
    if (Test-Path -LiteralPath $installDir) {
        Remove-Item -LiteralPath $installDir -Recurse -Force
    }
    if (!$Quiet) {
        Write-Host "Brume was successfully uninstalled."
    }
    return
}

# Install
$distDir = $PSScriptRoot
$executableName = if (Test-Path -LiteralPath (Join-Path $distDir 'brume.exe')) { 'brume.exe' } else { 'serein.exe' }
$executable = Join-Path $distDir $executableName
if (!(Test-Path -LiteralPath $executable)) {
    $candidate = Join-Path (Join-Path $distDir '..\..\dist') 'brume.exe'
    if (!(Test-Path -LiteralPath $candidate)) {
        $candidate = Join-Path (Join-Path $distDir '..\..\dist') 'serein.exe'
    }
    if (Test-Path -LiteralPath $candidate) {
        $distDir = (Resolve-Path (Join-Path $distDir '..\..\dist')).Path
        $executable = $candidate
        $executableName = Split-Path -Leaf $candidate
    } else {
        throw "Neither brume.exe nor serein.exe found in $distDir. Run this script from the release package directory or build the project first."
    }
}

[IO.Directory]::CreateDirectory($installDir) | Out-Null

# Copy payload
Copy-Item -Path "$distDir\*" -Destination $installDir -Recurse -Force
if ($PSCommandPath -and (Test-Path -LiteralPath $PSCommandPath)) {
    Copy-Item -LiteralPath $PSCommandPath -Destination (Join-Path $installDir 'setup.ps1') -Force
}

# Create Start Menu shortcut with AUMID
$notificationScript = Join-Path $installDir 'install-notifications.ps1'
if (Test-Path -LiteralPath $notificationScript) {
    & $notificationScript -Force
}

# Determine version
$version = '0.1.0'
try {
    $versionInfo = (Get-Item -LiteralPath (Join-Path $installDir $executableName)).VersionInfo.ProductVersion
    if ($versionInfo) { $version = $versionInfo }
} catch {}

# Register in Windows Add/Remove Programs
if (!(Test-Path -LiteralPath $uninstallKey)) {
    New-Item -Path $uninstallKey -Force | Out-Null
}

$uninstallCmd = "powershell.exe -NoProfile -ExecutionPolicy Bypass -File `"$installDir\setup.ps1`" -Uninstall"
Set-ItemProperty -LiteralPath $uninstallKey -Name 'DisplayName' -Value $appName
Set-ItemProperty -LiteralPath $uninstallKey -Name 'DisplayVersion' -Value $version
Set-ItemProperty -LiteralPath $uninstallKey -Name 'Publisher' -Value $publisher
Set-ItemProperty -LiteralPath $uninstallKey -Name 'DisplayIcon' -Value "$installDir\$executableName,0"
Set-ItemProperty -LiteralPath $uninstallKey -Name 'InstallLocation' -Value $installDir
Set-ItemProperty -LiteralPath $uninstallKey -Name 'UninstallString' -Value $uninstallCmd
Set-ItemProperty -LiteralPath $uninstallKey -Name 'QuietUninstallString' -Value "$uninstallCmd -Quiet"
Set-ItemProperty -LiteralPath $uninstallKey -Name 'URLInfoAbout' -Value $website
Set-ItemProperty -LiteralPath $uninstallKey -Name 'NoModify' -Value 1 -Type DWord
Set-ItemProperty -LiteralPath $uninstallKey -Name 'NoRepair' -Value 1 -Type DWord

if (!$Quiet) {
    Write-Host "Brume $version installed successfully to $installDir"
}
