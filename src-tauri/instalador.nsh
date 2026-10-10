!define ELECTRON_GUID "239e3efe-4668-5c7f-bb6c-7bb107adc208"
!define ELECTRON_DESINSTALAR "Software\Microsoft\Windows\CurrentVersion\Uninstall\${ELECTRON_GUID}"

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
    ExecWait '"$PLUGINSDIR\desinstalar-electron.exe" /S ${ALCANCE} /KEEP_APP_DATA _?=$R1' $R2
    Delete "$R1\Uninstall AndexMarket.exe"
    RMDir "$R1"
  ${EndIf}
!macroend

!macro NSIS_HOOK_PREINSTALL
  !insertmacro DESINSTALAR_ELECTRON HKLM "/allusers"
  !insertmacro DESINSTALAR_ELECTRON HKCU "/currentuser"
!macroend

!macro NSIS_HOOK_POSTINSTALL
  ${IfNot} ${FileExists} "$WINDIR\Sysnative\DriverStore\FileRepository\ch341ser.inf_*"
    DetailPrint "Instalando drivers de balanza USB-serie..."
    ExecShellWait "runas" "$WINDIR\Sysnative\pnputil.exe" '/add-driver "$INSTDIR\drivers\ch340\*.inf" /subdirs /install' SW_HIDE
  ${EndIf}
!macroend
