# Install qiui binary (+ optional Codex skill) from GitHub Releases (CntierTeam/QIUI).
# Usage:
#   irm https://raw.githubusercontent.com/CntierTeam/QIUI/main/scripts/install.ps1 | iex
#   .\scripts\install.ps1 -Version v0.1.1 -Force
#   .\scripts\install.ps1 -BinOnly
#   .\scripts\install.ps1 -Uninstall

[CmdletBinding()]
param(
    [string]$Repo = $(if ($env:QIUI_REPO) { $env:QIUI_REPO } else { "CntierTeam/QIUI" }),
    [string]$Version = $(if ($env:QIUI_VERSION) { $env:QIUI_VERSION } else { "latest" }),
    [string]$Prefix = $(if ($env:QIUI_PREFIX) { $env:QIUI_PREFIX } else { Join-Path $env:LOCALAPPDATA "qiui" }),
    [switch]$BinOnly,
    [switch]$SkillOnly,
    [switch]$Force,
    [switch]$Uninstall
)

$ErrorActionPreference = "Stop"

$BinDir = Join-Path $Prefix "bin"
$BinPath = Join-Path $BinDir "qiui.exe"
$CodexHome = if ($env:CODEX_HOME) { $env:CODEX_HOME } else { Join-Path $env:USERPROFILE ".codex" }
$SkillDst = Join-Path $CodexHome "skills\qiui"
$Api = "https://api.github.com/repos/$Repo"
$InstallBin = -not $SkillOnly.IsPresent
$InstallSkill = -not $BinOnly.IsPresent

function Get-ReleaseJson {
    $headers = @{ "Accept" = "application/vnd.github+json"; "User-Agent" = "qiui-install" }
    if ($env:GITHUB_TOKEN) { $headers["Authorization"] = "Bearer $($env:GITHUB_TOKEN)" }
    $url = if ($Version -eq "latest") { "$Api/releases/latest" } else { "$Api/releases/tags/$Version" }
    return Invoke-RestMethod -Uri $url -Headers $headers
}

function Ensure-Replace([string]$Path) {
    if (Test-Path $Path) {
        if (-not $Force) {
            throw "already exists: $Path (re-run with -Force)"
        }
        Remove-Item -Recurse -Force $Path
    }
}

if ($Uninstall) {
    if (Test-Path $BinPath) {
        Remove-Item -Force $BinPath
        Write-Host "removed $BinPath"
    }
    if (Test-Path $SkillDst) {
        Remove-Item -Recurse -Force $SkillDst
        Write-Host "removed $SkillDst"
    }
    return
}

$release = Get-ReleaseJson
Write-Host "release: $Repo@$($release.tag_name)"

$tmp = Join-Path ([System.IO.Path]::GetTempPath()) ("qiui-install-" + [guid]::NewGuid().ToString("N"))
New-Item -ItemType Directory -Force -Path $tmp | Out-Null
try {
    if ($InstallBin) {
        $assetName = "qiui-x86_64-pc-windows-msvc.zip"
        $asset = $release.assets | Where-Object { $_.name -eq $assetName } | Select-Object -First 1
        if (-not $asset) {
            throw "asset not found in release: $assetName"
        }
        $zip = Join-Path $tmp $assetName
        Write-Host "downloading $assetName"
        Invoke-WebRequest -Uri $asset.browser_download_url -OutFile $zip
        Expand-Archive -Path $zip -DestinationPath $tmp -Force
        $exe = Get-ChildItem -Path $tmp -Recurse -Filter qiui.exe | Select-Object -First 1
        if (-not $exe) { throw "qiui.exe missing in archive" }
        New-Item -ItemType Directory -Force -Path $BinDir | Out-Null
        Ensure-Replace $BinPath
        Copy-Item $exe.FullName $BinPath -Force
        Write-Host "binary: $BinPath"
    }

    if ($InstallSkill) {
        $skillName = "qiui-skill.tar.gz"
        $skillAsset = $release.assets | Where-Object { $_.name -eq $skillName } | Select-Object -First 1
        if (-not $skillAsset) {
            throw "asset not found in release: $skillName"
        }
        $tg = Join-Path $tmp $skillName
        Write-Host "downloading $skillName"
        Invoke-WebRequest -Uri $skillAsset.browser_download_url -OutFile $tg
        $skillExtract = Join-Path $tmp "skill"
        New-Item -ItemType Directory -Force -Path $skillExtract | Out-Null
        # Prefer tar (Windows 10+); fall back to failing with a clear hint.
        if (Get-Command tar -ErrorAction SilentlyContinue) {
            & tar -C $skillExtract -xzf $tg
        } else {
            throw "tar not found; install Windows tar or extract qiui-skill.tar.gz manually into $SkillDst"
        }
        $src = Join-Path $skillExtract "qiui"
        if (-not (Test-Path (Join-Path $src "SKILL.md"))) {
            throw "skill payload missing SKILL.md"
        }
        New-Item -ItemType Directory -Force -Path (Split-Path $SkillDst -Parent) | Out-Null
        Ensure-Replace $SkillDst
        Copy-Item -Recurse $src $SkillDst
        Write-Host "skill: $SkillDst"
    }
}
finally {
    Remove-Item -Recurse -Force $tmp -ErrorAction SilentlyContinue
}

Write-Host ""
Write-Host "Done."
if ($InstallBin) {
    $pathParts = $env:PATH -split ';'
    if ($pathParts -notcontains $BinDir) {
        Write-Host "note: add to PATH → $BinDir"
        Write-Host "  [Environment]::SetEnvironmentVariable('Path', `$env:Path + ';$BinDir', 'User')"
    }
    Write-Host "try: qiui profiles"
}
if ($InstallSkill) {
    Write-Host "Codex skill: `$qiui (restart Codex / new session if already running)"
}
