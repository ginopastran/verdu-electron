Unicode true
ManifestDPIAware true
SetCompressor /SOLID lzma
RequestExecutionLevel user

!ifndef VERSION
  !error "Falta /DVERSION=<x.y.z>"
!endif
!ifndef EJECUTABLE
  !error "Falta /DEJECUTABLE=<ruta de andexmarket.exe>"
!endif
!ifndef SALIDA
  !error "Falta /DSALIDA=<ruta del instalador>"
!endif

!define PRODUCTO "AndexMarket"
!define EDITOR "AndexTech"
!define EXE "AndexMarket.exe"
!define DESINSTALADOR "Desinstalar AndexMarket.exe"
!define CLAVE_DESINSTALAR "Software\Microsoft\Windows\CurrentVersion\Uninstall\${PRODUCTO}"
!define ELECTRON_GUID "239e3efe-4668-5c7f-bb6c-7bb107adc208"
!define ELECTRON_DESINSTALAR "Software\Microsoft\Windows\CurrentVersion\Uninstall\${ELECTRON_GUID}"

!include "MUI2.nsh"
!include "LogicLib.nsh"
!include "FileFunc.nsh"

Name "${PRODUCTO}"
OutFile "${SALIDA}"
InstallDir "$LOCALAPPDATA\${PRODUCTO}"
BrandingText "${EDITOR}"

VIProductVersion "${VERSION}.0"
VIAddVersionKey /LANG=0 "ProductName" "${PRODUCTO}"
VIAddVersionKey /LANG=0 "CompanyName" "${EDITOR}"
VIAddVersionKey /LANG=0 "FileDescription" "Instalador de ${PRODUCTO}"
VIAddVersionKey /LANG=0 "FileVersion" "${VERSION}"
VIAddVersionKey /LANG=0 "ProductVersion" "${VERSION}"
VIAddVersionKey /LANG=0 "LegalCopyright" "${EDITOR}"

Var Pasivo
Var Reabrir

!define MUI_ICON "${__FILEDIR__}\icono.ico"
!define MUI_UNICON "${__FILEDIR__}\icono.ico"
!define MUI_ABORTWARNING

!define MUI_PAGE_CUSTOMFUNCTION_PRE SaltarSiPasivo
!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_INSTFILES
!define MUI_PAGE_CUSTOMFUNCTION_PRE SaltarSiAutomatico
!define MUI_FINISHPAGE_RUN "$INSTDIR\${EXE}"
!define MUI_FINISHPAGE_RUN_TEXT "Abrir ${PRODUCTO}"
!insertmacro MUI_PAGE_FINISH

!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES

!insertmacro MUI_LANGUAGE "Spanish"

!macro OPCION NOMBRE VARIABLE
  ClearErrors
  ${GetOptions} $R0 "${NOMBRE}" $R1
  ${IfNot} ${Errors}
    StrCpy ${VARIABLE} 1
  ${EndIf}
!macroend

Function .onInit
  InitPluginsDir
  StrCpy $Pasivo 0
  StrCpy $Reabrir 0
  ${GetParameters} $R0
  !insertmacro OPCION "/P" $Pasivo
  !insertmacro OPCION "/R" $Reabrir
  !insertmacro OPCION "--updated" $Pasivo
  !insertmacro OPCION "--updated" $Reabrir
  !insertmacro OPCION "--force-run" $Reabrir
FunctionEnd

Function SaltarSiPasivo
  ${If} $Pasivo == 1
    Abort
  ${EndIf}
FunctionEnd

Function SaltarSiAutomatico
  ${If} $Pasivo == 1
  ${OrIf} $Reabrir == 1
    Abort
  ${EndIf}
FunctionEnd

!macro DESINSTALAR_ELECTRON RAIZ ALCANCE
  ReadRegStr $R0 ${RAIZ} "${ELECTRON_DESINSTALAR}" "UninstallString"
  ReadRegStr $R1 ${RAIZ} "Software\${ELECTRON_GUID}" "InstallLocation"
  ${If} $R0 != ""
  ${AndIf} $R1 != ""
  ${AndIf} ${FileExists} "$R1\Uninstall AndexMarket.exe"
    DetailPrint "Quitando la versión anterior de AndexMarket..."
    nsExec::Exec 'taskkill /F /IM "AndexMarket.exe"'
    Pop $R2
    CopyFiles /SILENT "$R1\Uninstall AndexMarket.exe" "$PLUGINSDIR\desinstalar-electron.exe"
    ClearErrors
    ExecWait '"$PLUGINSDIR\desinstalar-electron.exe" /S ${ALCANCE} /KEEP_APP_DATA _?=$R1' $R2
    ${If} ${Errors}
      ExecShellWait "runas" "$PLUGINSDIR\desinstalar-electron.exe" '/S ${ALCANCE} /KEEP_APP_DATA _?=$R1' SW_HIDE
    ${EndIf}
    Delete "$R1\Uninstall AndexMarket.exe"
    RMDir "$R1"
  ${EndIf}
!macroend

Section "${PRODUCTO}"
  ${If} $Pasivo == 1
    SetAutoClose true
  ${EndIf}

  !insertmacro DESINSTALAR_ELECTRON HKLM "/allusers"
  !insertmacro DESINSTALAR_ELECTRON HKCU "/currentuser"

  nsExec::Exec 'taskkill /F /IM "${EXE}"'
  Pop $R2
  Sleep 500

  SetOutPath "$INSTDIR"
  File "/oname=${EXE}" "${EJECUTABLE}"
  SetOutPath "$INSTDIR\drivers\ch340"
  File "${__FILEDIR__}\..\drivers\ch340\*.*"
  SetOutPath "$INSTDIR"
  WriteUninstaller "$INSTDIR\${DESINSTALADOR}"

  CreateShortCut "$DESKTOP\${PRODUCTO}.lnk" "$INSTDIR\${EXE}"
  CreateShortCut "$SMPROGRAMS\${PRODUCTO}.lnk" "$INSTDIR\${EXE}"

  WriteRegStr HKCU "${CLAVE_DESINSTALAR}" "DisplayName" "${PRODUCTO}"
  WriteRegStr HKCU "${CLAVE_DESINSTALAR}" "DisplayVersion" "${VERSION}"
  WriteRegStr HKCU "${CLAVE_DESINSTALAR}" "Publisher" "${EDITOR}"
  WriteRegStr HKCU "${CLAVE_DESINSTALAR}" "DisplayIcon" "$INSTDIR\${EXE}"
  WriteRegStr HKCU "${CLAVE_DESINSTALAR}" "InstallLocation" "$INSTDIR"
  WriteRegStr HKCU "${CLAVE_DESINSTALAR}" "UninstallString" '"$INSTDIR\${DESINSTALADOR}"'
  WriteRegStr HKCU "${CLAVE_DESINSTALAR}" "QuietUninstallString" '"$INSTDIR\${DESINSTALADOR}" /S'
  WriteRegDWORD HKCU "${CLAVE_DESINSTALAR}" "NoModify" 1
  WriteRegDWORD HKCU "${CLAVE_DESINSTALAR}" "NoRepair" 1
  ${GetSize} "$INSTDIR" "/S=0K" $R0 $R1 $R2
  IntFmt $R0 "0x%08X" $R0
  WriteRegDWORD HKCU "${CLAVE_DESINSTALAR}" "EstimatedSize" $R0

  ${IfNot} ${FileExists} "$WINDIR\Sysnative\DriverStore\FileRepository\ch341ser.inf_*"
    DetailPrint "Instalando drivers de balanza USB-serie..."
    ExecShellWait "runas" "$WINDIR\Sysnative\pnputil.exe" '/add-driver "$INSTDIR\drivers\ch340\*.inf" /subdirs /install' SW_HIDE
  ${EndIf}

  ${If} $Reabrir == 1
    Exec '"$INSTDIR\${EXE}"'
  ${EndIf}
SectionEnd

Section "Uninstall"
  nsExec::Exec 'taskkill /F /IM "${EXE}"'
  Pop $R2
  Sleep 500
  Delete "$INSTDIR\${EXE}"
  RMDir /r "$INSTDIR\drivers"
  Delete "$INSTDIR\${DESINSTALADOR}"
  RMDir "$INSTDIR"
  Delete "$DESKTOP\${PRODUCTO}.lnk"
  Delete "$SMPROGRAMS\${PRODUCTO}.lnk"
  DeleteRegKey HKCU "${CLAVE_DESINSTALAR}"
SectionEnd
