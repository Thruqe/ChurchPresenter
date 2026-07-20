; Inno Setup Script for Church Presenter
; https://jrsoftware.org/isinfo.php

#ifndef MyAppVersion
  #define MyAppVersion "0.1.0"
#endif

#define MyAppName      "Church Presenter"
#define MyAppPublisher "Thruqe"
#define MyAppURL       "https://github.com/Thruqe/ChurchPresenter"
#define MyAppExeName   "church-presenter.exe"
#define MyAppIconName  "church-presenter.ico"

[Setup]
; Basic identity
AppId={{6F3A2B1C-4D5E-4F6A-8B9C-0D1E2F3A4B5C}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}/issues
AppUpdatesURL={#MyAppURL}/releases

; Install into 64-bit Program Files
DefaultDirName={autopf64}\{#MyAppName}
DefaultGroupName={#MyAppName}
DisableProgramGroupPage=yes

; Output
OutputDir=..
OutputBaseFilename=church-presenter-setup
SetupIconFile=..\dist-windows-bundle\{#MyAppIconName}
UninstallDisplayIcon={app}\{#MyAppIconName}

; Compression
Compression=lzma2/ultra64
SolidCompression=yes
LZMANumBlockThreads=4

; Require 64-bit Windows
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible

; Modern wizard look
WizardStyle=modern
WizardResizable=no

; Request admin so we can write to Program Files
PrivilegesRequired=admin
PrivilegesRequiredOverridesAllowed=dialog

; Minimum OS: Windows 10
MinVersion=10.0

; Misc
DisableWelcomePage=no
DisableReadyPage=no
ShowLanguageDialog=no

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
; Copy everything from the bundle directory (exe, all DLLs, GTK assets, NDI DLL, icon, etc.)
Source: "..\dist-windows-bundle\*"; DestDir: "{app}"; Flags: ignoreversion recursesubdirs createallsubdirs

[Icons]
; Start Menu shortcut
Name: "{group}\{#MyAppName}";     Filename: "{app}\{#MyAppExeName}"; IconFilename: "{app}\{#MyAppIconName}"
; Desktop shortcut (optional task)
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; IconFilename: "{app}\{#MyAppIconName}"; Tasks: desktopicon

[Run]
; Offer to launch the app after install
Filename: "{app}\{#MyAppExeName}"; Description: "{cm:LaunchProgram,{#StringChange(MyAppName, '&', '&&')}}"; Flags: nowait postinstall skipifsilent

[Registry]
; Programs & Features / Apps & Features entry
Root: HKLM; Subkey: "Software\Microsoft\Windows\CurrentVersion\Uninstall\{#MyAppName}"; \
  ValueType: string; ValueName: "DisplayIcon"; ValueData: "{app}\{#MyAppIconName}"; Flags: uninsdeletekey
Root: HKLM; Subkey: "Software\Microsoft\Windows\CurrentVersion\Uninstall\{#MyAppName}"; \
  ValueType: string; ValueName: "Publisher"; ValueData: "{#MyAppPublisher}"
Root: HKLM; Subkey: "Software\Microsoft\Windows\CurrentVersion\Uninstall\{#MyAppName}"; \
  ValueType: string; ValueName: "URLInfoAbout"; ValueData: "{#MyAppURL}"
