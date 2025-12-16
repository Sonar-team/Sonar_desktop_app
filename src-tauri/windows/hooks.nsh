!macro NSIS_HOOK_PREINSTALL
  ; Check if Npcap is installed
  ReadRegStr $0 HKLM "SOFTWARE\Npcap" ""
  ${If} $0 == ""
    MessageBox MB_OK|MB_ICONINFORMATION "Npcap is not installed. Please install Npcap for full functionality."
  ${EndIf}
!macroend