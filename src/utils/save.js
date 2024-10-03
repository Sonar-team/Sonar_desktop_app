import { save } from '@tauri-apps/api/dialog';
import { invoke } from '@tauri-apps/api';
import { message } from '@tauri-apps/api/dialog';

// Fonction pour sauvegarder en CSV
export async function SaveAsCsv(getCurrentDate, niveauConfidentialite, installationName) {
  try {
    const filePath = await save({
      filters: [{ name: '.csv', extensions: ['csv'] }],
      title: 'Sauvegarder la matrice de flux',
      defaultPath: `${getCurrentDate()}_${niveauConfidentialite}_${installationName}.csv`
    });

    if (filePath) {
      await invoke('save_packets_to_csv', { file_path: filePath });
      return true; // Succès de la sauvegarde
    } else {
      return false; // Annulation ou échec
    }
  } catch (error) {
    console.error('Erreur lors de la sauvegarde CSV:', error);
    return false;
  }
}

// Fonction pour sauvegarder en XLSX
export async function SaveAsXlsx(getCurrentDate, niveauConfidentialite, installationName) {
  try {
    const filePath = await save({
      filters: [{ name: '.xlsx', extensions: ['xlsx'] }],
      title: 'Sauvegarder la matrice de flux',
      defaultPath: `${getCurrentDate()}_${niveauConfidentialite}_${installationName}.xlsx`
    });

    if (filePath) {
      await invoke('save_packets_to_excel', { file_path: filePath });
      return true; // Succès de la sauvegarde
    } else {
      return false; // Annulation ou échec
    }
  } catch (error) {
    console.error('Erreur lors de la sauvegarde XLSX:', error);
    return false;
  }
}

// Fonction triggerSave pour gérer la sauvegarde en fonction du format sélectionné
export async function triggerSave(selectedFormat, getCurrentDate, niveauConfidentialite, installationName) {
  let saveResult = false; // Variable pour stocker le résultat de la sauvegarde

  if (selectedFormat === 'csv') {
    saveResult = await SaveAsCsv(getCurrentDate, niveauConfidentialite, installationName);
  } else if (selectedFormat === 'xlsx') {
    saveResult = await SaveAsXlsx(getCurrentDate, niveauConfidentialite, installationName);
  }

  if (saveResult) {
    await message('La sauvegarde a été effectuée avec succès.', {
      title: 'Confirmation',
      type: 'info'
    });
  } else {
    await message('Erreur lors de la sauvegarde.', {
      title: 'Erreur',
      type: 'error'
    });
  }

  return saveResult; // Retourner la confirmation de la sauvegarde
}


export async function getDesktopDirPath() {
    try {
      const dir = await desktopDir();
      console.log("App Data Directory: ", dir);
      return dir;
    } catch (error) {
      console.error("Error getting app data directory: ", error);
    }
  }