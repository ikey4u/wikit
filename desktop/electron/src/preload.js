const { contextBridge, ipcRenderer } = require('electron')

contextBridge.exposeInMainWorld('wikit', {
  getDictList: () => ipcRenderer.invoke('dict:list'),
  lookup: (dictid, word) => ipcRenderer.invoke('dict:lookup', dictid, word),
  ffiHello: (name) => ipcRenderer.invoke('native:ffi-hello', name),
  startStaticFileServer: () => ipcRenderer.invoke('static:start'),
  openDirectory: () => ipcRenderer.invoke('dialog:open-directory'),
  startPreviewServer: (dir) => ipcRenderer.invoke('preview:start', dir),
  stopPreviewServer: () => ipcRenderer.invoke('preview:stop'),
  isPreviewServerUp: () => ipcRenderer.invoke('preview:is-up'),
  openSettingsWindow: () => ipcRenderer.invoke('settings:open'),
  getTranslationSettings: () => ipcRenderer.invoke('translation:get-settings'),
  saveTranslationSettings: (settingsJson) => ipcRenderer.invoke('translation:save-settings', settingsJson),
  translateText: (requestJson) => ipcRenderer.invoke('translation:translate', requestJson),
  testTranslationConnection: (settingsJson) => ipcRenderer.invoke('translation:test-connection', settingsJson),
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
