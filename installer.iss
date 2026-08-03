; Script de Inno Setup para verifGsigma (BTW-One)

[Setup]
AppId={{C6D2B4A1-8E92-4F81-9A33-72C5B5238210}
AppName=verifGsigma
AppVersion=1.4.10
AppPublisher=BTW-One
AppPublisherURL=https://btw-one.com
DefaultDirName={autopf}\BTW-One\verifGsigma
DefaultGroupName=BTW-One\verifGsigma
OutputBaseFilename=verifGsigma_Setup
Compression=lzma2/max
SolidCompression=yes
PrivilegesRequired=admin
ArchitecturesInstallIn64BitMode=x64compatible
DisableDirPage=no
DisableProgramGroupPage=no
AlwaysShowDirOnReadyPage=yes

[Files]
Source: "target\release\verifgsigma.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "config.example.json"; DestName: "config.json"; DestDir: "{app}"; Flags: onlyifdoesntexist

[Icons]
Name: "{group}\verifGsigma Servidor Local"; Filename: "{app}\verifgsigma.exe"; WorkingDir: "{app}"

[Run]
; Detener y eliminar servicios/procesos previos para evitar ejecuciones o puertos duplicados
Filename: "taskkill.exe"; Parameters: "/F /IM verifgsigma.exe"; Flags: runhidden
Filename: "sc.exe"; Parameters: "stop verifgsigma"; Flags: runhidden
Filename: "sc.exe"; Parameters: "delete verifgsigma"; Flags: runhidden
Filename: "sc.exe"; Parameters: "stop verifGsigma"; Flags: runhidden
Filename: "sc.exe"; Parameters: "delete verifGsigma"; Flags: runhidden

; Registrar el nuevo servicio de Windows verifGsigma con auto-recuperacion
Filename: "sc.exe"; Parameters: "create verifGsigma binPath= ""{app}\verifgsigma.exe"" start= auto DisplayName= ""verifGsigma Servidor Local (BTW-One)"""; Flags: runhidden
Filename: "sc.exe"; Parameters: "description verifGsigma ""Servidor Local Verificador de Precios por BTW-One"""; Flags: runhidden
Filename: "sc.exe"; Parameters: "failure verifGsigma reset= 86400 actions= restart/5000/restart/5000/restart/5000"; Flags: runhidden
Filename: "sc.exe"; Parameters: "start verifGsigma"; Flags: runhidden

[UninstallRun]
Filename: "taskkill.exe"; Parameters: "/F /IM verifgsigma.exe"; Flags: runhidden; RunOnceId: "KillProcess"
Filename: "sc.exe"; Parameters: "stop verifGsigma"; Flags: runhidden; RunOnceId: "StopService"
Filename: "sc.exe"; Parameters: "delete verifGsigma"; Flags: runhidden; RunOnceId: "DeleteService"

[Code]
var
  ConfigPage: TInputQueryWizardPage;

function ShouldSkipPage(PageID: Integer): Boolean;
begin
  if (ConfigPage <> nil) and (PageID = ConfigPage.ID) then
    Result := False
  else
    Result := False;
end;

procedure InitializeWizard;
begin
  // Crear pagina personalizada de configuracion de MySQL y Puerto justo despues de Bienvenidos (wpWelcome)
  ConfigPage := CreateInputQueryPage(wpWelcome,
    'Configuración del Servidor verifGsigma (BTW-One)',
    'Parámetros de Red y Base de Datos MySQL',
    'Por favor especifique el puerto de red y los datos de conexión MySQL para esta sucursal.');

  ConfigPage.Add('Puerto HTTP del Servidor:', False);
  ConfigPage.Add('Host de MySQL (IP o localhost):', False);
  ConfigPage.Add('Puerto de MySQL:', False);
  ConfigPage.Add('Usuario de MySQL:', False);
  ConfigPage.Add('Contraseña de MySQL:', True);
  ConfigPage.Add('Nombre de Base de Datos MySQL:', False);
  ConfigPage.Add('Código de Sucursal:', False);

  // Valores predeterminados iniciales
  ConfigPage.Values[0] := '8080';
  ConfigPage.Values[1] := 'localhost';
  ConfigPage.Values[2] := '3306';
  ConfigPage.Values[3] := 'root';
  ConfigPage.Values[4] := '';
  ConfigPage.Values[5] := 'pv';
  ConfigPage.Values[6] := '01';
end;

procedure CurStepChanged(CurStep: TSetupStep);
var
  ConfigContent: String;
  ConfigFilePath: String;
  PuertoStr, HostStr, DbPortStr, UserStr, PassStr, DbNameStr, SucursalStr: String;
  DbUrlStr: String;
begin
  if CurStep = ssPostInstall then
  begin
    PuertoStr := ConfigPage.Values[0];
    HostStr := ConfigPage.Values[1];
    DbPortStr := ConfigPage.Values[2];
    UserStr := ConfigPage.Values[3];
    PassStr := ConfigPage.Values[4];
    DbNameStr := ConfigPage.Values[5];
    SucursalStr := ConfigPage.Values[6];

    if PuertoStr = '' then PuertoStr := '8080';
    if HostStr = '' then HostStr := 'localhost';
    if DbPortStr = '' then DbPortStr := '3306';
    if UserStr = '' then UserStr := 'root';
    if DbNameStr = '' then DbNameStr := 'pv';
    if SucursalStr = '' then SucursalStr := '01';

    // Construir la cadena de conexion mysql://usuario:contraseña@host:puerto/dbname
    DbUrlStr := 'mysql://' + UserStr + ':' + PassStr + '@' + HostStr + ':' + DbPortStr + '/' + DbNameStr;

    ConfigContent := '{' + #13#10 +
      '  "puerto": ' + PuertoStr + ',' + #13#10 +
      '  "sucursal": "' + SucursalStr + '",' + #13#10 +
      '  "db_url": "' + DbUrlStr + '",' + #13#10 +
      '  "auto_update": true,' + #13#10 +
      '  "repo_owner": "Tupap1",' + #13#10 +
      '  "repo_name": "verifpreciosgsigma"' + #13#10 +
      '}';

    ConfigFilePath := ExpandConstant('{app}\config.json');

    // Escribir el archivo config.json con los datos confirmados por el usuario
    SaveStringToFile(ConfigFilePath, ConfigContent, False);
  end;
end;
