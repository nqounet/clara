import { UiStore } from './ui.svelte';
import { ConfigStore } from './config.svelte';
import { AtomStore } from './atom.svelte';
import { SearchStore } from './search.svelte';

export const uiStore = new UiStore();
export const configStore = new ConfigStore();
export const atomStore = new AtomStore();
export const searchStore = new SearchStore();

// App-level initialization combining them
import { invoke } from '@tauri-apps/api/core';
import { getVersion } from '@tauri-apps/api/app';
import type { AppConfig, ClaraConfig } from '$lib/types/clara';

export async function initApp() {
  try {
    const [appConfig, claraConfig, version] = await Promise.all([
      invoke<AppConfig>('get_app_config'),
      invoke<ClaraConfig>('get_clara_config'),
      getVersion(),
    ]);

    configStore.updateAppConfig(appConfig);
    configStore.updateClaraConfig(claraConfig);
    configStore.appVersion = version;

    await atomStore.fetchRecentAtoms();
    await atomStore.setupStreamingListener();
  } catch (e) {
    uiStore.errorMsg = `初期化に失敗しました: ${e}`;
    console.error('初期化に失敗しました', e);
  }
}

export function closeModal() {
  uiStore.closeModal();
  configStore.claraSettingsMsg = "";
  configStore.claraSettingsMsgIsError = false;
  configStore.vaultMsg = "";
  configStore.isVaultMsgError = false;
}

export async function handleUpdateRootDir() {
  if (configStore.isSaving) return;
  configStore.isSaving = true;
  try {
    configStore.vaultMsg = "切り替え中...";
    configStore.isVaultMsgError = false;
    const config = await invoke<AppConfig>("update_root_dir", { newPath: configStore.rootDir });
    configStore.updateAppConfig(config);
    configStore.vaultMsg = "✓ Vaultを切り替えました";

    atomStore.clearContext();
    atomStore.prompt = "";
    uiStore.errorMsg = "";
    searchStore.clearSearch();
    await atomStore.fetchRecentAtoms();

    setTimeout(() => {
      configStore.vaultMsg = "";
      uiStore.closeModalSilent();
    }, 1500);
  } catch (e) {
    configStore.isVaultMsgError = true;
    configStore.vaultMsg = String(e);
  } finally {
    configStore.isSaving = false;
  }
}

export async function handleSend() {
  uiStore.errorMsg = "";
  const { success, errorMsg } = await atomStore.handleSend(configStore.yoloMode);
  if (!success && errorMsg) {
    uiStore.errorMsg = errorMsg;
  } else if (success && configStore.yoloMode) {
    configStore.yoloSentMsg = "🔥 YOLOモードで送信しました。次の送信もYOLOが有効です。";
    setTimeout(() => {
      configStore.yoloSentMsg = "";
    }, 4000);
  }
}
