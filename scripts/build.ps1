<#
.SYNOPSIS
Builds the DLL and CLI, then assembles and signs the sparse package into `dist\`. Does not need elevation.
#>
$ErrorActionPreference = 'Stop'
$root = Split-Path $PSScriptRoot -Parent
$dist = Join-Path $root 'dist'
$pkg = Join-Path $dist 'pkg'
$subject = 'CN=Chuyang Chen'   # must equal Publisher in packaging\AppxManifest.xml

$kit = Get-ChildItem 'C:\Program Files (x86)\Windows Kits\10\bin\10.*\x64' -Directory |
    Where-Object { Test-Path (Join-Path $_.FullName 'makeappx.exe') } |
    Sort-Object Name | Select-Object -Last 1
if (-not $kit) { throw 'Windows SDK (makeappx.exe, signtool.exe) not found' }

Push-Location $root
try { cargo build --release; if ($LASTEXITCODE) { throw 'cargo build failed' } } finally { Pop-Location }

Remove-Item $dist -Recurse -Force -ErrorAction SilentlyContinue
New-Item -ItemType Directory $pkg | Out-Null
Copy-Item (Join-Path $root 'target\release\weztermhere_shell.dll') $dist
Copy-Item (Join-Path $root 'target\release\weztermhere.exe') $dist
Copy-Item (Join-Path $root 'packaging\AppxManifest.xml') $pkg
Copy-Item (Join-Path $root 'packaging\Assets') $pkg -Recurse

$cert = Get-ChildItem Cert:\CurrentUser\My | Where-Object { $_.Subject -eq $subject -and $_.FriendlyName -eq 'weztermhere' } | Select-Object -First 1
if (-not $cert) {
    $cert = New-SelfSignedCertificate -Type Custom -Subject $subject -FriendlyName 'weztermhere' `
        -KeyUsage DigitalSignature -CertStoreLocation Cert:\CurrentUser\My -NotAfter (Get-Date).AddYears(5) `
        -TextExtension @('2.5.29.37={text}1.3.6.1.5.5.7.3.3', '2.5.29.19={text}')
}
Export-Certificate -Cert $cert -FilePath (Join-Path $dist 'weztermhere.cer') | Out-Null

$msix = Join-Path $dist 'weztermhere.msix'
& (Join-Path $kit.FullName 'makeappx.exe') pack /d $pkg /p $msix /nv /o
if ($LASTEXITCODE) { throw 'makeappx failed' }
& (Join-Path $kit.FullName 'signtool.exe') sign /fd SHA256 /sha1 $cert.Thumbprint $msix
if ($LASTEXITCODE) { throw 'signtool failed' }
Write-Host "Built $msix. Next: run scripts\install.ps1 from an elevated PowerShell."
