[setup]
AppName=RustSync Client
AppVersion=0.1.0
DefaultDirName={autopf}\RustSync
DefaultGroupName=RustSync
OutputBaseFilename=RustSyncInstaller
Compression=lzma2/max
SolidCompression=yes

[files]
Source: "..\..\target\release\client.exe"; DestName: "rustsync.exe"; DestDir: "{app}"


[registry]
Root: HKCU; Subkey: "Environment"; ValueType: expandsz; ValueName: "Path"; \
  ValueData: "{olddata};{app}";


; [Run]
; Filename: "{app}\client.exe"; Description: "Run Backup Client"; Flags: nowait postinstall skipifsilent
