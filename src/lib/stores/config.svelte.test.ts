import { describe, it, expect } from 'vitest';
import { ConfigStore } from './config.svelte';

describe('ConfigStore', () => {
  it('initializes with default values', () => {
    const store = new ConfigStore();
    expect(store.cliCommand).toBe('gemini');
    expect(store.rootDir).toBe('');
    expect(store.yoloMode).toBe(false);
  });

  it('updates app config', () => {
    const store = new ConfigStore();
    store.updateAppConfig({ root_dir: '/test/path' });
    expect(store.rootDir).toBe('/test/path');
  });

  it('updates clara config', () => {
    const store = new ConfigStore();
    store.updateClaraConfig({
      cli_command: 'custom-cli',
      model: 'test-model',
      working_dir: '/work',
      cli_args: [],
      workspace_history: []
    });
    expect(store.cliCommand).toBe('custom-cli');
    expect(store.cliModel).toBe('test-model');
    expect(store.cliWorkingDir).toBe('/work');
  });
});
