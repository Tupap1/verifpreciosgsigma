# PowerShell script to install verifgsigma.exe to Windows Startup (Inicio de Windows)
$WshShell = New-Object -comObject WScript.Shell
$StartupFolder = $WshShell.SpecialFolders("Startup")
$ShortcutPath = Join-Path -Path $StartupFolder -ChildPath "VerificadorGsigmaServer.lnk"
$TargetPath = Resolve-Path "target\release\verifgsigma.exe"

$Shortcut = $WshShell.CreateShortcut($ShortcutPath)
$Shortcut.TargetPath = $TargetPath
$Shortcut.WorkingDirectory = Split-Path -Path $TargetPath
$Shortcut.WindowStyle = 7 # Minimized/Hidden
$Shortcut.Description = "Servidor Verificador de Precios Gsigma (Segundo Plano)"
$Shortcut.Save()

Write-Host "✅ Servidor agregado al Inicio de Windows exitosamente en: $ShortcutPath"
