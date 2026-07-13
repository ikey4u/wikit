const { contextBridge, ipcRenderer } = require('electron')

function sendLog(level, scope, message, meta) {
  ipcRenderer.send('app:log', { level, scope, message, meta })
}

contextBridge.exposeInMainWorld('wikit', {
  log: (scope, level, message, meta) => sendLog(level, scope, message, meta),
  getDictList: () => ipcRenderer.invoke('dict:list'),
  loadLocalDict: (path) => ipcRenderer.invoke('dict:load-local', path),
  removeLocalDict: (dictid) => ipcRenderer.invoke('dict:remove-local', dictid),
  lookup: (dictid, word) => ipcRenderer.invoke('dict:lookup', dictid, word),
  lookupResource: (dictid, key) => ipcRenderer.invoke('dict:lookup-resource', dictid, key),
  getDictInfo: (dictid) => ipcRenderer.invoke('dict:get-info', dictid),
  searchDict: (dictid, word) => ipcRenderer.invoke('dict:search', dictid, word),
  republishLocalDict: (dictid, style, script, outputPath, name, desc) =>
    ipcRenderer.invoke('dict:republish-local', dictid, style, script, outputPath, name, desc),
  ffiHello: (name) => ipcRenderer.invoke('native:ffi-hello', name),
  startStaticFileServer: () => ipcRenderer.invoke('static:start'),
  openDirectory: () => ipcRenderer.invoke('dialog:open-directory'),
  openFile: (filters) => ipcRenderer.invoke('dialog:open-file', filters),
  openConfigDir: () => ipcRenderer.invoke('app:open-config-dir'),
  startPreviewServer: (dir) => ipcRenderer.invoke('preview:start', dir),
  stopPreviewServer: () => ipcRenderer.invoke('preview:stop'),
  isPreviewServerUp: () => ipcRenderer.invoke('preview:is-up'),
  openSettingsWindow: () => ipcRenderer.invoke('settings:open'),
  getTranslationSettings: () => ipcRenderer.invoke('translation:get-settings'),
  saveTranslationSettings: (settingsJson) => ipcRenderer.invoke('translation:save-settings', settingsJson),
  translateText: (requestJson) => ipcRenderer.invoke('translation:translate', requestJson),
  testTranslationConnection: (settingsJson) => ipcRenderer.invoke('translation:test-connection', settingsJson),
  writeClipboardText: (text) => ipcRenderer.invoke('clipboard:write-text', text),
  getAppSettings: () => ipcRenderer.invoke('app-settings:get'),
  saveAppSettings: (settingsJson) => ipcRenderer.invoke('app-settings:save', settingsJson),
  readTextFile: (path) => ipcRenderer.invoke('fs:read-text-file', path),
  writeTextFile: (path, content) => ipcRenderer.invoke('fs:write-text-file', path, content),
  copyFile: (src, dst) => ipcRenderer.invoke('fs:copy-file', src, dst),
  revealPath: (path) => ipcRenderer.invoke('shell:reveal-path', path),
  buildWikit: (srcfile, outfile) => ipcRenderer.invoke('dict:build-wikit', srcfile, outfile),
  onBuildProgress: (callback) => {
    const listener = (_event, progress) => callback(progress)
    ipcRenderer.on('dict:build-progress', listener)
    return () => ipcRenderer.removeListener('dict:build-progress', listener)
  },
  onTranslationSettingsUpdated: (callback) => {
    const listener = (_event, payload) => callback(payload)
    ipcRenderer.on('translation-settings-updated', listener)
    return () => ipcRenderer.removeListener('translation-settings-updated', listener)
  },
  emitJsEvent: (payload) => ipcRenderer.send('js-event', payload),
  onRustEvent: (callback) => {
    const listener = (_event, payload) => callback(payload)
    ipcRenderer.on('rust-event', listener)
    return () => ipcRenderer.removeListener('rust-event', listener)
  }
})
