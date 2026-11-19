<#
.SYNOPSIS
Trusts the signing certificate and registers the sparse package. Run from an elevated PowerShell after `build.ps1`.
#>
#Requires -RunAsAdministrator
$ErrorActionPreference = 'Stop'
$dist = Join-Path (Split-Path $PSScriptRoot -Parent) 'dist'

Import-Certificate -FilePath (Join-Path $dist 'weztermhere.cer') -CertStoreLocation Cert:\LocalMachine\TrustedPeople | Out-Null
Get-AppxPackage -Name weztermhere | Remove-AppxPackage
Add-AppxPackage -Path (Join-Path $dist 'weztermhere.msix') -ExternalLocation $dist
Write-Host 'Installed. Restart Explorer (Stop-Process -Name explorer) if the entry does not show up.'
