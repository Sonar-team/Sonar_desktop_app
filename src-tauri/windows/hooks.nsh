!include "LogicLib.nsh"
!include "FileFunc.nsh"

!macro NSIS_HOOK_PREINSTALL

  ; Indicateur WinPcap-compat : wpcap.dll doit être présent
  ; (install Npcap en "WinPcap API-compatible Mode")
  ${If} ${FileExists} "$SYSDIR\wpcap.dll"
    Goto done
  ${EndIf}

  MessageBox MB_YESNO|MB_ICONQUESTION \
    "Npcap (mode WinPcap compatible) n'est pas détecté. SONAR en a besoin pour capturer le trafic réseau.$\r$\n$\r$\nSouhaites-tu installer Npcap maintenant ?" \
    IDYES +2
    Goto done

  ; Lance l'installeur embarqué (admin requis)
  ExecShell "runas" "$INSTDIR\windows\npcap-1.86.exe" ""

done:
!macroend
