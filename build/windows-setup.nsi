; NSIS for Nuchain Node Installer

;--------------------------------
;Include Modern UI

  !include "MUI2.nsh"

;--------------------------------
;General

  ;Name and file
  Name "Nuchain Node"
  OutFile "nuchain-win64-setup.exe"
  Unicode True
  ; RequestExecutionLevel admin

  ; !include LogicLib.nsh

  ;Default installation folder
  ; InstallDir "$PROGRAMFILES\Nuchain"
  InstallDir "$LOCALAPPDATA\nuchain"
  
  ;Get installation folder from registry if available
  InstallDirRegKey HKCU "Software\Nuchain" ""

  ;Request application privileges for Windows Vista
  RequestExecutionLevel user

;--------------------------------
;Variables

  Var StartMenuFolder
  Var ComputerName
  Var ProgramDir
  Var DatabaseDir

;--------------------------------
;Interface Settings

  !define MUI_ABORTWARNING

;--------------------------------
;Pages

  #!insertmacro MUI_PAGE_LICENSE "${NSISDIR}\Docs\Modern UI\License.txt"
  !insertmacro MUI_PAGE_COMPONENTS

  !define MUI_DIRECTORYPAGE_VARIABLE $ProgramDir
  !define MUI_PAGE_CUSTOMFUNCTION_PRE DirectoryPre
  !insertmacro MUI_PAGE_DIRECTORY

  # Second directory page.
  !define MUI_DIRECTORYPAGE_VARIABLE $DatabaseDir
  !define MUI_PAGE_CUSTOMFUNCTION_PRE DirectoryPre
  !define MUI_PAGE_HEADER_TEXT "Please select directory for store data"
  !define MUI_PAGE_HEADER_SUBTEXT "The location should be large and fast enough to store Nuchain data"

  !insertmacro MUI_PAGE_DIRECTORY
  ;Assign descriptions to sections
  ; !insertmacro MUI_HEADER_TEXT "Select data directory" "this location should be large enough to store Nuchain data"
  
  Function DirectoryPre
    StrCpy $ProgramDir "$LOCALAPPDATA\nuchain"
    StrCpy $INSTDIR $ProgramDir
    StrCpy $DatabaseDir "$LOCALAPPDATA\nuchain\db"
  FunctionEnd

    
  ;Nuchain Page Configuration
  !define MUI_STARTMENUPAGE_REGISTRY_ROOT "HKCU" 
  !define MUI_STARTMENUPAGE_REGISTRY_KEY "Software\Nuchain" 
  !define MUI_STARTMENUPAGE_REGISTRY_VALUENAME "Nuchain"
  
  !insertmacro MUI_PAGE_STARTMENU Application $StartMenuFolder
  
  !insertmacro MUI_PAGE_INSTFILES
  
  !insertmacro MUI_UNPAGE_CONFIRM
  !insertmacro MUI_UNPAGE_INSTFILES

;--------------------------------
;Languages
 
  !insertmacro MUI_LANGUAGE "English"

Function .onInit
  ReadRegStr $0 HKLM "System\CurrentControlSet\Control\ComputerName\ActiveComputerName" "ComputerName"
  StrCpy $1 $0 4 3
  Push $0
  Pop $ComputerName
FunctionEnd


;--------------------------------
;Installer Sections


Section "Nuchain Binary" SecBin

  SetOutPath "$INSTDIR"
  
  ;ADD YOUR OWN FILES HERE...
  File /oname=nuchain.exe ..\bin_archives\nuchain.exe
  File C:\Windows\SYSTEM32\VCRUNTIME140.dll
  File C:\Windows\SYSTEM32\MSVCP140.dll

  ;Store installation folder
  WriteRegStr HKCU "Software\Nuchain" "" $INSTDIR
  
  ;Create uninstaller
  WriteUninstaller "$INSTDIR\Uninstall.exe"
  
  !insertmacro MUI_STARTMENU_WRITE_BEGIN Application
    
    ;Create shortcuts
    CreateDirectory "$SMPROGRAMS\$StartMenuFolder"

    CreateShortcut "$SMPROGRAMS\$StartMenuFolder\nuchain.lnk" "$INSTDIR\nuchain.exe" \
        "--validator --base-path=$DatabaseDir --unsafe-pruning --pruning=1000 --name=WIN-$ComputerName --telemetry-url='wss://telemetry.nuchain.network/submit 0'" "$INSTDIR\nuchain.exe" 2 SW_SHOWNORMAL \
        ALT|CONTROL|SHIFT|F5 "Nuchain Node Starter"

    ; add to startup menu
    CreateShortcut "$SMSTARTUP\nuchain.lnk" "$INSTDIR\nuchain.exe" \
        "--validator --base-path=$DatabaseDir --unsafe-pruning --pruning=1000 --name=WIN-$ComputerName --telemetry-url='wss://telemetry.nuchain.network/submit 0'" "$INSTDIR\nuchain.exe" 2 SW_SHOWNORMAL \
        ALT|CONTROL|SHIFT|F5 "Nuchain Node Starter"

    CreateShortcut "$SMPROGRAMS\$StartMenuFolder\Uninstall.lnk" "$INSTDIR\Uninstall.exe"
  
  !insertmacro MUI_STARTMENU_WRITE_END

SectionEnd

Section "Validator Toolkit" SecValidatorTool

  SetOutPath "$INSTDIR"
  File /oname=rotatekey.exe ..\bin_archives\rotatekey.exe

  CreateShortcut "$SMPROGRAMS\$StartMenuFolder\rotatekey.lnk" "$INSTDIR\rotatekey.exe" \
      "" "$INSTDIR\rotatekey.exe" 2 SW_SHOWNORMAL

SectionEnd

Section "" SecDatabase
  SetOutPath "$DatabaseDir"
SectionEnd

;--------------------------------
;Uninstaller Section

Section "Uninstall"
  
  !insertmacro MUI_STARTMENU_GETFOLDER Application $StartMenuFolder

  Delete "$SMPROGRAMS\$StartMenuFolder\Uninstall.lnk"
  RMDir /r "$SMPROGRAMS\$StartMenuFolder"

  Delete "$ProgramDir\nuchain.exe"
  Delete "$ProgramDir\rotatekey.exe"
  Delete "$ProgramDir\msvcp140.dll"
  Delete "$ProgramDir\vcruntime140.dll"
  RMDir /r /REBOOTOK "$ProgramDir"
  RMDir "$LOCALAPPDATA\nuchain"

  Delete "$ProgramDir\Uninstall.exe"

  DeleteRegKey /ifempty HKCU "Software\Nuchain"

SectionEnd

