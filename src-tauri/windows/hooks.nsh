!include "LogicLib.nsh"

!macro NSIS_HOOK_PREINSTALL

  ;  Détection Npcap via registre "éditeur" (peut varier)
  ClearErrors
  ReadRegStr $0 HKLM "SOFTWARE\Npcap" "InstallDir"
  ${If} ${Errors}
    ClearErrors
    ReadRegStr $0 HKLM "SOFTWARE\WOW6432Node\Npcap" "InstallDir"
  ${EndIf}

  ;  Si pas trouvé, fallback sur la clé Uninstall (souvent la plus fiable)
  ${If} $0 == ""
    ClearErrors
    ReadRegStr $1 HKLM "SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\Npcap" "DisplayName"
    ${If} ${Errors}
      ClearErrors
      ReadRegStr $1 HKLM "SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall\Npcap" "DisplayName"
    ${EndIf}
    ${If} $1 != ""
      StrCpy $0 "FOUND"
    ${EndIf}
  ${EndIf}

  ;  Si toujours absent => proposer installation
  ${If} $0 == ""
    MessageBox MB_YESNO|MB_ICONQUESTION \
      "Npcap n'est pas installé. SONAR en a besoin pour capturer le trafic réseau.$\r$\n$\r$\nSouhaites-tu installer Npcap maintenant ?" \
      IDYES +2
      Goto done

    ; Lance l'installeur Npcap embarqué dans l'installeur SONAR (resources)
    ; NB: nécessite élévation admin (driver).
    ExecShell "runas" "$INSTDIR\windows\npcap-1.85.exe" ""

  ${EndIf}

done:
!macroend
