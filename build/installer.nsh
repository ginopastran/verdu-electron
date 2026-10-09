!macro customInstall
  DetailPrint "Instalando drivers de balanza USB-serie..."
  nsExec::ExecToLog '"$WINDIR\Sysnative\pnputil.exe" /add-driver "$INSTDIR\resources\resources\drivers\*.inf" /subdirs /install'
  Pop $0
  DetailPrint "pnputil terminó con código $0"
!macroend
