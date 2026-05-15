const { app, BrowserWindow, Menu, ipcMain, shell, dialog, clipboard, globalShortcut } = require('electron')
const fs = require('node:fs')
const path = require('node:path')

try {
  require('electron-reloader')(module, {
    ignore: ['node_modules', 'native', 'icons', 'scripts']
  })
} catch (_) {}

const version = '0.5.0'
let native
let mainWindow
let settingsWindow
let quitting = false
let registeredToggleAccelerator = null

const defaultAppSettings = {
  shortcuts: {
    toggleWindow: 'Alt+1'
  }
}

function getNative() {
  if (!native) {
    const nativePath = app.isPackaged
      ? path.join(process.resourcesPath, 'native')
      : path.join(__dirname, '../../native')
    native = require(nativePath)
  }
  return native
}

function getAppSettingsPath() {
  return path.join(app.getPath('userData'), 'app-settings.json')
}

function normalizeToggleAccelerator(accelerator) {
  const value = String(accelerator || '').trim()
  if (!value || value === 'Command+Command') {
    return defaultAppSettings.shortcuts.toggleWindow
  }
  return value
}

function readAppSettings() {
  try {
    const content = fs.readFileSync(getAppSettingsPath(), 'utf8')
    const parsed = JSON.parse(content)
    return {
      shortcuts: {
        toggleWindow: normalizeToggleAccelerator(parsed.shortcuts?.toggleWindow)
      }
    }
  } catch (_error) {
    return defaultAppSettings
  }
}

function writeAppSettings(settings) {
  const sanitized = {
    shortcuts: {
      toggleWindow: normalizeToggleAccelerator(settings.shortcuts?.toggleWindow)
    }
  }
  fs.mkdirSync(path.dirname(getAppSettingsPath()), { recursive: true })
  fs.writeFileSync(getAppSettingsPath(), JSON.stringify(sanitized, null, 2))
  return sanitized
}

function toggleMainWindow() {
  if (!mainWindow || mainWindow.isDestroyed()) {
    createWindow()
    return
  }
  if (mainWindow.isVisible() && mainWindow.isFocused()) {
    mainWindow.hide()
    return
  }
  mainWindow.show()
  if (process.platform === 'darwin') {
    app.dock.show()
  }
  mainWindow.focus()
}

function registerToggleShortcut(accelerator) {
  if (registeredToggleAccelerator) {
    globalShortcut.unregister(registeredToggleAccelerator)
    registeredToggleAccelerator = null
  }

  if (!accelerator) {
    return {
      registered: false,
      accelerator: '',
      message: '快捷键为空，未注册。'
    }
  }

  try {
    const ok = globalShortcut.register(accelerator, toggleMainWindow)
    if (ok) {
      registeredToggleAccelerator = accelerator
      return { registered: true, accelerator, message: '快捷键已注册。' }
    }
  } catch (error) {
    return {
      registered: false,
      accelerator,
      message: `快捷键格式无效或不受系统支持：${String(error && error.message ? error.message : error)}`
    }
  }

  return { registered: false, accelerator, message: '快捷键注册失败，可能已被系统或其他应用占用。' }
}

function applyAppShortcuts() {
  const settings = readAppSettings()
  return registerToggleShortcut(settings.shortcuts.toggleWindow)
}

function createWindow() {
  mainWindow = new BrowserWindow({
    width: 1024,
    height: 740,
    minWidth: 600,
    minHeight: 500,
    fullscreen: false,
    title: 'Wikit Desktop',
    titleBarStyle: process.platform === 'darwin' ? 'hiddenInset' : 'default',
    trafficLightPosition: { x: 2, y: 2 },
    backgroundColor: '#ffffff',
    webPreferences: {
      preload: path.join(__dirname, 'preload.js'),
      contextIsolation: true,
      nodeIntegration: false,
      sandbox: false
    }
  })

  mainWindow.loadFile(path.join(__dirname, 'index.html'))

  mainWindow.on('close', (event) => {
    if (quitting) {
      return
    }

    const answer = dialog.showMessageBoxSync(mainWindow, {
      type: 'question',
      buttons: ['Cancel', 'Quit'],
      defaultId: 0,
      cancelId: 0,
      title: 'Wikit Desktop',
      message: 'Are you sure that you want to exit wikit desktop?'
    })

    if (answer !== 1) {
      event.preventDefault()
      return
    }

    event.preventDefault()
    quitting = true
    try {
      getNative().stopPreviewServer()
    } catch (_error) {
    }
    app.exit(0)
  })

  mainWindow.on('closed', () => {
    mainWindow = null
  })
}

function createSettingsWindow() {
  if (settingsWindow && !settingsWindow.isDestroyed()) {
    settingsWindow.focus()
    return
  }

  settingsWindow = new BrowserWindow({
    width: 680,
    height: 620,
    resizable: false,
    fullscreen: false,
    title: 'Settings',
    titleBarStyle: process.platform === 'darwin' ? 'hiddenInset' : 'default',
    trafficLightPosition: { x: 16, y: 16 },
    backgroundColor: '#f6f7f9',
    parent: mainWindow || undefined,
    modal: false,
    webPreferences: {
      preload: path.join(__dirname, 'preload.js'),
      contextIsolation: true,
      nodeIntegration: false,
      sandbox: false
    }
  })

  settingsWindow.loadFile(path.join(__dirname, 'settings.html'))
  settingsWindow.on('closed', () => {
    settingsWindow = null
  })
}

function createMenu() {
  const template = [
    {
      label: 'File',
      submenu: [
        {
          label: 'Configuration',
          click: async () => {
            try {
              const configDir = getNative().getConfigDir()
              await shell.openPath(configDir)
            } catch (error) {
              dialog.showErrorBox('Wikit Desktop', String(error))
            }
          }
        },
        {
          label: 'Translation Settings',
          click: createSettingsWindow
        },
        { type: 'separator' },
        { role: 'quit', label: 'Quit' }
      ]
    }
  ]

  if (process.platform !== 'linux') {
    template.push({
      label: 'Edit',
      submenu: [
        { role: 'copy' },
        { role: 'cut' },
        { role: 'paste' },
        { role: 'undo' },
        { role: 'redo' },
        { role: 'selectAll' }
      ]
    })
  }

  template.push({
    label: 'Help',
    submenu: [
      {
        label: 'Home Page',
        click: () => shell.openExternal('https://github.com/ikey4u/wikit')
      },
      {
        label: 'Report Bug',
        click: () => shell.openExternal('https://github.com/ikey4u/wikit/issues/new')
      },
      {
        label: 'Manual',
        click: () => shell.openExternal('https://github.com/ikey4u/wikit/wiki')
      },
      {
        label: 'About',
        click: () => {
          dialog.showMessageBox(mainWindow, {
            type: 'info',
            title: 'Wikit Desktop',
            message: 'Wikit Desktop',
            detail: `A universal dictionary\nv${version}\nhttps://github.com/ikey4u/wikit`
          })
        }
      }
    ]
  })

  Menu.setApplicationMenu(Menu.buildFromTemplate(template))
}

ipcMain.handle('app:open-config-dir', async () => {
  try {
    const configDir = getNative().getConfigDir()
    await shell.openPath(configDir)
    return true
  } catch (error) {
    return false
  }
})
ipcMain.handle('dict:list', () => getNative().getDictList())
ipcMain.handle('dict:load-local', (_event, filePath) => getNative().loadLocalDictionary(String(filePath)))
ipcMain.handle('dict:lookup', (_event, dictid, word) => getNative().lookup(dictid, word))
ipcMain.handle('dict:get-info', (_event, dictid) => getNative().getDictInfo(dictid))
ipcMain.handle('dict:search', (_event, dictid, word) => getNative().searchDict(dictid, word))
ipcMain.handle('translation:get-settings', () => getNative().getTranslationSettings())
ipcMain.handle('translation:save-settings', (_event, settingsJson) => {
  const settings = getNative().saveTranslationSettings(settingsJson)
  if (mainWindow && !mainWindow.isDestroyed()) {
    mainWindow.webContents.send('translation-settings-updated', settings)
  }
  return settings
})
ipcMain.handle('translation:translate', (_event, requestJson) => getNative().translateText(requestJson))
ipcMain.handle('translation:test-connection', (_event, settingsJson) => getNative().testTranslationConnection(settingsJson))
ipcMain.handle('clipboard:write-text', (_event, text) => clipboard.writeText(String(text || '')))
ipcMain.handle('shell:reveal-path', (_event, targetPath) => {
  shell.showItemInFolder(String(targetPath || ''))
  return true
})
ipcMain.handle('app-settings:get', () => {
  const settings = readAppSettings()
  return JSON.stringify({
    ...settings,
    shortcutState: registerToggleShortcut(settings.shortcuts.toggleWindow)
  })
})
ipcMain.handle('app-settings:save', (_event, settingsJson) => {
  let incoming
  try {
    incoming = JSON.parse(settingsJson)
  } catch (_error) {
    incoming = defaultAppSettings
  }
  const settings = writeAppSettings(incoming)
  return JSON.stringify({
    ...settings,
    shortcutState: registerToggleShortcut(settings.shortcuts.toggleWindow)
  })
})
ipcMain.handle('native:ffi-hello', (_event, name) => getNative().ffiHello(name))
ipcMain.handle('static:start', () => getNative().startStaticFileServer())
ipcMain.handle('preview:start', (_event, dir) => getNative().startPreviewServer(dir))
ipcMain.handle('preview:stop', () => getNative().stopPreviewServer())
ipcMain.handle('preview:is-up', () => getNative().isPreviewServerUp())
ipcMain.handle('settings:open', () => createSettingsWindow())
ipcMain.handle('dialog:open-directory', async () => {
  const result = await dialog.showOpenDialog(mainWindow, {
    properties: ['openDirectory']
  })

  if (result.canceled || result.filePaths.length === 0) {
    return null
  }

  return result.filePaths[0]
})
ipcMain.handle('dialog:open-file', async () => {
  const result = await dialog.showOpenDialog(mainWindow, {
    properties: ['openFile'],
    filters: [
      { name: 'Dictionary Files', extensions: ['txt', 'csv', 'wikit', 'mdx', 'md'] },
      { name: 'All Files', extensions: ['*'] }
    ]
  })

  if (result.canceled || result.filePaths.length === 0) {
    return null
  }

  return result.filePaths[0]
})
ipcMain.on('js-event', (event, payload) => {
  console.log(`got js-event with message '${JSON.stringify(payload)}'`)
  event.sender.send('rust-event', 'something else')
})
ipcMain.handle('fs:read-text-file', async (_event, filePath) => {
  try {
    return fs.readFileSync(String(filePath), 'utf8')
  } catch (_error) {
    return null
  }
})
ipcMain.handle('fs:write-text-file', async (_event, filePath, content) => {
  try {
    const dir = path.dirname(String(filePath))
    fs.mkdirSync(dir, { recursive: true })
    fs.writeFileSync(String(filePath), String(content || ''), 'utf8')
    return true
  } catch (error) {
    return false
  }
})
ipcMain.handle('fs:copy-file', async (_event, src, dst) => {
  try {
    const dir = path.dirname(String(dst))
    fs.mkdirSync(dir, { recursive: true })
    fs.copyFileSync(String(src), String(dst))
    return true
  } catch (error) {
    return false
  }
})
ipcMain.handle('dict:build-wikit', async (event, srcfile, outfile) => {
  try {
    const raw = await getNative().buildDictionary(
      String(srcfile),
      String(outfile),
      (_err, progress) => {
        if (event && !event.sender.isDestroyed()) {
          event.sender.send('dict:build-progress', progress)
        }
      }
    )
    return JSON.parse(raw)
  } catch (error) {
    console.error('build wikit failed:', error)
    return { ok: false, error: error.message || 'Unknown error' }
  }
})

app.whenReady().then(() => {
  getNative().startStaticFileServer()
  createMenu()
  createWindow()
  applyAppShortcuts()

  app.on('activate', () => {
    if (BrowserWindow.getAllWindows().length === 0) {
      createWindow()
    }
  })
})

app.on('before-quit', () => {
  quitting = true
  globalShortcut.unregisterAll()
  try {
    getNative().stopPreviewServer()
  } catch (_error) {
  }
})

app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') {
    app.quit()
  }
})
