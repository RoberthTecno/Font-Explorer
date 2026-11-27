import { check } from '@tauri-apps/plugin-updater';

export async function checkForUpdates() {
  try {
    const update = await check();

    if (!update) {
      return {
        available: false,
        version: null
      };
    }

    return {
      available: true,
      version: update.version,
      install: async () => {
        await update.downloadAndInstall();
      }
    };

  } catch (error) {
    console.error('Error checking updates:', error);
    throw error;
  }
}