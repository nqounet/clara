import type { AppConfig, ClaraConfig } from '$lib/types/clara';
import { invoke } from '@tauri-apps/api/core';
import { open as openDialog } from '@tauri-apps/plugin-dialog';

export class ConfigStore {
  // ClaraConfig settings
  cliCommand = $state('gemini');
  cliModel = $state('');
  cliWorkingDir = $state('');
  workspaceHistory = $state<string[]>([]);
  claraSettingsMsg = $state('');
  claraSettingsMsgIsError = $state(false);

  // Vault Settings
  rootDir = $state('');
  vaultMsg = $state('');
  isVaultMsgError = $state(false);

  // YOLO
  yoloMode = $state(false);
  yoloSentMsg = $state('');
  
  appVersion = $state('');
  isSaving = $state(false);

  updateAppConfig(config: AppConfig) {
    this.rootDir = config.root_dir;
  }

  updateClaraConfig(config: ClaraConfig) {
    this.cliCommand = config.cli_command;
    this.cliModel = config.model ?? '';
    this.cliWorkingDir = config.working_dir ?? '';
    this.workspaceHistory = config.workspace_history ?? [];
  }

  async pickFolder(): Promise<string | null> {
    const selected = await openDialog({ directory: true, multiple: false });
    return typeof selected === "string" ? selected : null;
  }

  async pickVaultDir() {
    const path = await this.pickFolder();
    if (path) this.rootDir = path;
  }

  async pickWorkspaceDir() {
    const path = await this.pickFolder();
    if (path) this.cliWorkingDir = path;
  }

  async handleUpdateClaraConfig(onSuccess?: () => void) {
    if (this.isSaving) return;
    
    // Validate that CLI command is not empty
    if (!this.cliCommand.trim()) {
      this.claraSettingsMsgIsError = true;
      this.claraSettingsMsg = "CLIコマンドは必須です。";
      return;
    }

    if (!this.cliWorkingDir.trim()) {
      this.claraSettingsMsgIsError = true;
      this.claraSettingsMsg = "Workspaceディレクトリは必須です。";
      return;
    }
    
    this.isSaving = true;
    try {
      this.claraSettingsMsgIsError = false;
      this.claraSettingsMsg = "保存中...";
      const updated = await invoke<ClaraConfig>("update_clara_config", {
        cliCommand: this.cliCommand,
        model: this.cliModel.trim() || null,
        workingDir: this.cliWorkingDir.trim() || null,
      });
      this.cliCommand = updated.cli_command;
      this.cliModel = updated.model ?? "";
      this.cliWorkingDir = updated.working_dir ?? "";
      this.workspaceHistory = updated.workspace_history ?? [];
      this.claraSettingsMsg = "✓ 保存しました";
      setTimeout(() => {
        this.claraSettingsMsg = "";
        if (onSuccess) onSuccess();
      }, 1500);
    } catch (e) {
      this.claraSettingsMsgIsError = true;
      this.claraSettingsMsg = String(e);
    } finally {
      this.isSaving = false;
    }
  }

  async removeWorkspaceHistory(path: string) {
    try {
      const updated = await invoke<ClaraConfig>("remove_workspace_history", { path });
      this.workspaceHistory = updated.workspace_history ?? [];
    } catch (e) {
      console.error("Failed to remove history:", e);
    }
  }
}
