; Script de Inno Setup para BTW-One Verificador de Precios

[Setup]
AppName=BTW-One Verificador de Precios
AppVersion=1.0.0
AppPublisher=BTW-One
AppPublisherURL=https://btw-one.com
DefaultDirName={autopf}\BTW-One\VerificadorPrecios
DefaultGroupName=BTW-One Verificador
OutputBaseFilename=BTW-One_Verificador_Setup
Compression=lzma2/max
SolidCompression=yes
PrivilegesRequired=admin
ArchitecturesInstallIn64BitMode=x64

[Files]
Source: "target\release\verifgsigma.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "config.example.json"; DestName: "config.json"; DestDir: "{app}"; Flags: onlyifdoesntexist


[Icons]
Name: "{autostartup}\BTW-One Verificador Server"; Filename: "{app}\verifgsigma.exe"; WorkingDir: "{app}"
Name: "{group}\BTW-One Verificador de Precios"; Filename: "{app}\verifgsigma.exe"; WorkingDir: "{app}"
Name: "{commondesktop}\BTW-One Verificador de Precios"; Filename: "{app}\verifgsigma.exe"; WorkingDir: "{app}"

[Run]
Filename: "{app}\verifgsigma.exe"; Description: "Iniciar Servidor Verificador de Precios"; Flags: nowait postinstall skipifsilent
