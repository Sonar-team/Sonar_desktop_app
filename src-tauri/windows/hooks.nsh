!include "LogicLib.nsh"
!include "x64.nsh"

; Reproductibilité du setup.exe (#119) : par défaut (`SetDateSave on`), NSIS
; stocke la date de modification de chaque fichier empaqueté dans le header
; compressé. Or Tauri régénère des fichiers de travail pendant le bundling
; (target/release/nsis/x64/), APRÈS la normalisation des mtimes du
; beforeBundleCommand : leurs dates portent l'heure de chaque build et font
; diverger le flux LZMA solide dès son premier bloc — d'où des écarts de
; taille erratiques entre deux builds identiques (354, 1333, 2735 octets
; observés). Prouvé par contre-épreuve : mtimes différentes, installateurs
; identiques avec `off`, divergents avec `on`.
; Effet assumé : les fichiers installés portent la date d'installation au
; lieu de leur date de build — sans sémantique pour SONAR.
; Ce fichier étant inclus en tête du script généré, la directive s'applique
; à toute la compilation.
SetDateSave off

!define NPCAP_DOWNLOAD_URL "https://npcap.com/#download"

; Retourne dans $R0 : "absent", "incompatible" ou "ok".
; SONAR lie wpcap.dll/Packet.dll implicitement : la présence du service Npcap
; ne suffit pas, le mode compatible WinPcap doit aussi être activé.
Function SONAR_CheckNpcap
  StrCpy $R0 "absent"

  ; Le service distingue Npcap de l'ancien WinPcap.
  ClearErrors
  ReadRegDWORD $R1 HKLM "SYSTEM\CurrentControlSet\Services\npcap" "Start"
  ${If} ${Errors}
    Return
  ${EndIf}
  StrCpy $R0 "incompatible"

  ; Depuis Npcap 0.93, les options sont stockées sous Parameters.
  ClearErrors
  ReadRegDWORD $R1 HKLM "SYSTEM\CurrentControlSet\Services\npcap\Parameters" "WinPcapCompatible"
  ${If} ${Errors}
    ; Repli pour les anciennes installations Npcap.
    ClearErrors
    ReadRegDWORD $R1 HKLM "SYSTEM\CurrentControlSet\Services\npcap" "WinPcapCompatible"
  ${EndIf}
  ${If} ${Errors}
    Return
  ${EndIf}
  ${If} $R1 != 1
    Return
  ${EndIf}

  ; Le setup Tauri est un processus 32 bits, même pour une cible x64 :
  ; désactiver la redirection afin de contrôler le vrai System32.
  ${If} ${RunningX64}
    ${DisableX64FSRedirection}
  ${EndIf}
  ${If} ${FileExists} "$SYSDIR\Npcap\wpcap.dll"
  ${AndIf} ${FileExists} "$SYSDIR\Npcap\Packet.dll"
  ${AndIf} ${FileExists} "$SYSDIR\wpcap.dll"
  ${AndIf} ${FileExists} "$SYSDIR\Packet.dll"
    StrCpy $R0 "ok"
  ${EndIf}
  ${If} ${RunningX64}
    ${EnableX64FSRedirection}
  ${EndIf}
FunctionEnd

!macro NSIS_HOOK_PREINSTALL
  Call SONAR_CheckNpcap
  StrCmp $R0 "ok" sonar_npcap_done

  StrCmp $R0 "incompatible" 0 sonar_npcap_absent
  StrCpy $R1 "Npcap est installé, mais le mode 'WinPcap API-compatible Mode' requis par SONAR n'est pas disponible.$\r$\n$\r$\nNpcap n'est pas inclus dans SONAR. Modifie ou réinstalle Npcap avant de lancer SONAR.$\r$\n$\r$\nOuvrir la page officielle Npcap ?"
  Goto sonar_npcap_prompt

sonar_npcap_absent:
  StrCpy $R1 "Npcap n'est pas détecté. Il n'est pas inclus dans SONAR et doit être installé séparément avec l'option 'WinPcap API-compatible Mode' avant de lancer l'application.$\r$\n$\r$\nOuvrir la page officielle Npcap ?"

sonar_npcap_prompt:
  ; Une installation silencieuse ne doit jamais rester bloquée sur une boîte
  ; de dialogue. Le contrôle est conservé dans le journal de l'installeur.
  DetailPrint "$R1"
  IfSilent sonar_npcap_done 0
  MessageBox MB_YESNO|MB_ICONEXCLAMATION "$R1" IDYES sonar_npcap_open IDNO sonar_npcap_done

sonar_npcap_open:
  ExecShell "open" "${NPCAP_DOWNLOAD_URL}"

sonar_npcap_done:
!macroend
