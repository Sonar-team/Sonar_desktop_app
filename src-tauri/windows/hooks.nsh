!include "LogicLib.nsh"
!include "FileFunc.nsh"

!macro VerifyNpcapDependency result_var
  StrCpy ${result_var} "0"
  ${If} ${FileExists} "$SYSDIR\wpcap.dll"
  ${AndIf} ${FileExists} "$SYSDIR\Packet.dll"
    StrCpy ${result_var} "1"
  ${EndIf}

  ${If} ${result_var} == "0"
  ${AndIf} ${FileExists} "$WINDIR\Sysnative\wpcap.dll"
  ${AndIf} ${FileExists} "$WINDIR\Sysnative\Packet.dll"
    StrCpy ${result_var} "1"
  ${EndIf}
!macroend

!macro NSIS_HOOK_POSTINSTALL

  ; Npcap en mode WinPcap compatible fournit wpcap.dll et Packet.dll.
  !insertmacro VerifyNpcapDependency $R0
  ${If} $R0 == "1"
    Goto done
  ${EndIf}

  MessageBox MB_YESNO|MB_ICONQUESTION \
    "Npcap (mode WinPcap compatible) n'est pas détecté. SONAR en a besoin pour capturer le trafic réseau.$\r$\n$\r$\nSouhaites-tu installer Npcap maintenant ?" \
    IDYES +2
    Goto done

  ; L'installeur Npcap est copié avec les ressources SONAR avant ce hook.
  ClearErrors
  ExecShellWait "runas" "$INSTDIR\windows\npcap-1.87.exe" "/winpcap_mode=yes" SW_SHOWNORMAL
  ${If} ${Errors}
    MessageBox MB_ICONEXCLAMATION \
      "L'installation de Npcap a échoué ou a été annulée. La capture réseau SONAR peut ne pas fonctionner."
    Goto done
  ${EndIf}

  !insertmacro VerifyNpcapDependency $R0
  ${If} $R0 != "1"
    MessageBox MB_ICONEXCLAMATION \
      "Npcap a été lancé, mais SONAR ne détecte toujours pas wpcap.dll et Packet.dll. Vérifie que Npcap est installé en mode WinPcap compatible."
  ${EndIf}

done:
!macroend
