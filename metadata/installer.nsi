# NSIS Installer Script for Church Presenter
# Built with metadata: Thruqe, danielpeter0039@gmail.com

!ifndef VERSION
  !define VERSION "0.1.0"
!endif

!define APP_NAME "Church Presenter"
!define COMP_NAME "Thruqe"
!define APP_VERSION "${VERSION}"
!define OUT_FILE "church-presenter-setup.exe"
!define ICON_FILE "play.ico"
!define HELP_URL "mailto:danielpeter0039@gmail.com"

# Set compression
SetCompressor lzma

# Include modern UI
!include "MUI2.nsh"

# MUI Settings
!define MUI_ABORTWARNING
!define MUI_ICON "${ICON_FILE}"
!define MUI_UNICON "${ICON_FILE}"

# Pages
!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH

# Uninstaller pages
!insertmacro MUI_UNPAGE_WELCOME
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_UNPAGE_FINISH

# Language
!insertmacro MUI_LANGUAGE "English"

Name "${APP_NAME}"
OutFile "../${OUT_FILE}"
InstallDir "$PROGRAMFILES64\${APP_NAME}"
InstallDirRegKey HKLM "Software\${COMP_NAME}\${APP_NAME}" "Install_Dir"
RequestExecutionLevel admin

Section "Install"
    SetOutPath "$INSTDIR"
    
    # Copy all files from the bundle directory
    File /r "../dist-windows-bundle\*"

    # Write registry keys for uninstaller
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APP_NAME}" "DisplayName" "${APP_NAME}"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APP_NAME}" "UninstallString" '"$INSTDIR\uninstall.exe"'
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APP_NAME}" "DisplayIcon" "$INSTDIR\church-presenter.exe"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APP_NAME}" "Publisher" "${COMP_NAME}"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APP_NAME}" "DisplayVersion" "${APP_VERSION}"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APP_NAME}" "HelpLink" "${HELP_URL}"
    
    # Create shortcuts
    CreateShortcut "$SMPROGRAMS\${APP_NAME}.lnk" "$INSTDIR\church-presenter.exe" "" "$INSTDIR\church-presenter.exe" 0
    CreateShortcut "$DESKTOP\${APP_NAME}.lnk" "$INSTDIR\church-presenter.exe" "" "$INSTDIR\church-presenter.exe" 0

    WriteUninstaller "$INSTDIR\uninstall.exe"
SectionEnd

Section "Uninstall"
    # Remove shortcuts
    Delete "$SMPROGRAMS\${APP_NAME}.lnk"
    Delete "$DESKTOP\${APP_NAME}.lnk"

    # Remove files and directories
    RMDir /r "$INSTDIR"

    # Remove registry keys
    DeleteRegKey HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APP_NAME}"
    DeleteRegKey HKLM "Software\${COMP_NAME}\${APP_NAME}"
SectionEnd
