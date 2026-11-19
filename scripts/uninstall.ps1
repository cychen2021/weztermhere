<#
.SYNOPSIS
Removes the sparse package and its certificate from the trust store. Run from an elevated PowerShell.
#>
#Requires -RunAsAdministrator
$ErrorActionPreference = 'Stop'
Get-AppxPackage -Name weztermhere | Remove-AppxPackage
Get-ChildItem Cert:\LocalMachine\TrustedPeople | Where-Object { $_.Subject -eq 'CN=Chuyang Chen' -and $_.FriendlyName -eq 'weztermhere' } | Remove-Item
Write-Host 'Uninstalled.'
