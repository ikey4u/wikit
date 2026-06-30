const { app, BrowserWindow, Menu, ipcMain, shell, dialog, clipboard, globalShortcut } = require('electron')
const fs = require('node:fs')
const path = require('node:path')

const APP_NAME = 'Bootsmind Wikit'
const USER_DATA_DIR_NAME = 'bootsmind-wikit'

function configureUserDataPath() {
  app.setName(APP_NAME)
  app.setPath('userData', path.join(app.getPath('appData'), USER_DATA_DIR_NAME))
}

configureUserDataPath()

const { initLogger, createLogger } = require('./logger')

const bootstrapLog = createLogger('bootstrap')
const appLog = createLogger('app')
const windowLog = createLogger('window')
const nativeBridgeLog = createLogger('native-bridge')
const staticServerLog = createLogger('static-server')
const previewServerLog = createLogger('preview-server')
const processLog = createLogger('process')
const devLog = createLogger('dev-hot-reload')

if (!app.isPackaged) {
  try {
    require('electron-reloader')(module, {
      ignore: ['node_modules', 'native', 'icons', 'scripts']
    })
    devLog.info('electron-reloader enabled')
  } catch (error) {
    devLog.warn('electron-reloader unavailable', { error: String(error) })
  }
}

const version = '0.5.0'
let native
let mainWindow
let settingsWindow
let quitting = false
let registeredToggleAccelerator = null
let rendererReloadAttempts = 0

const gotSingleInstanceLock = app.requestSingleInstanceLock()
if (!gotSingleInstanceLock) {
  app.quit()
}

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
    nativeBridgeLog.info('loading native module', { nativePath, isPackaged: app.isPackaged })
    try {
      native = require(nativePath)
      nativeBridgeLog.info('native module loaded')
    } catch (error) {
      nativeBridgeLog.error('native module load failed', { nativePath, error: String(error) })
      throw error
    }
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
  windowLog.info('creating main window')
  mainWindow = new BrowserWindow({
    width: 1024,
    height: 740,
    minWidth: 600,
    minHeight: 500,
    fullscreen: false,
    title: APP_NAME,
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

  const indexPath = path.join(__dirname, 'index.html')
  const preloadPath = path.join(__dirname, 'preload.js')
  windowLog.info('loading page', { indexPath, preloadPath })
  mainWindow.loadFile(indexPath)

  mainWindow.webContents.on('console-message', (event) => {
    const level = Number(event.level)
    const message = event.message || ''
    const source = event.sourceId || ''
    const line = event.lineNumber || 0
    const scope = 'renderer.console'
    const meta = { source, line }
    if (level >= 3) {
      windowLog.error(message, meta)
      return
    }
    if (level === 2) {
      windowLog.warn(message, meta)
      return
    }
    windowLog.info(message, meta)
  })

  mainWindow.webContents.on('did-finish-load', () => {
    windowLog.info('finished loading', { url: mainWindow.webContents.getURL() })
  })
  mainWindow.webContents.on('did-fail-load', (_event, errorCode, errorDescription, validatedURL, isMainFrame) => {
    windowLog.error('failed to load', {
      errorCode,
      errorDescription,
      validatedURL,
      isMainFrame
    })
  })
  mainWindow.webContents.on('render-process-gone', (_event, details) => {
    windowLog.error('render process gone', details)
    if (quitting || !mainWindow || mainWindow.isDestroyed()) {
      return
    }
    if (rendererReloadAttempts >= 1) {
      dialog.showErrorBox(
        APP_NAME,
        `界面进程异常退出（${details.reason || 'unknown'}），请完全退出后重新打开。\n\n日志：${path.join(app.getPath('userData'), 'logs', 'main.log')}`
      )
      return
    }
    rendererReloadAttempts += 1
    windowLog.warn('reloading window after renderer crash', {
      attempt: rendererReloadAttempts,
      reason: details.reason,
      exitCode: details.exitCode
    })
    mainWindow.loadFile(indexPath)
  })
  mainWindow.on('unresponsive', () => {
    windowLog.warn('became unresponsive')
  })
  mainWindow.on('responsive', () => {
    windowLog.info('became responsive again')
  })

  mainWindow.on('close', (event) => {
    if (quitting) {
      return
    }

    const answer = dialog.showMessageBoxSync(mainWindow, {
      type: 'question',
      buttons: ['Cancel', 'Quit'],
      defaultId: 0,
      cancelId: 0,
      title: APP_NAME,
      message: `Are you sure you want to quit ${APP_NAME}?`
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
    windowLog.info('closed')
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
              dialog.showErrorBox(APP_NAME, String(error))
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
        label: 'Open Log File',
        click: () => {
          const logFilePath = path.join(app.getPath('userData'), 'logs', 'main.log')
          if (fs.existsSync(logFilePath)) {
            shell.openPath(logFilePath)
            return
          }
          dialog.showMessageBox(mainWindow, {
            type: 'info',
            title: APP_NAME,
            message: '日志文件尚未生成',
            detail: logFilePath
          })
        }
      },
      {
        label: 'About',
        click: () => {
          dialog.showMessageBox(mainWindow, {
            type: 'info',
            title: APP_NAME,
            message: APP_NAME,
            detail: `${APP_NAME}\nA universal dictionary\nv${version}\nhttps://github.com/ikey4u/wikit`
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
ipcMain.handle('static:start', () => {
  const port = getNative().startStaticFileServer()
  staticServerLog.info('start requested via ipc', { port })
  return port
})
ipcMain.handle('preview:start', (_event, dir) => {
  const port = getNative().startPreviewServer(dir)
  previewServerLog.info('start requested via ipc', { port, dir: String(dir || '') })
  return port
})
ipcMain.handle('preview:stop', () => {
  previewServerLog.info('stop requested via ipc')
  return getNative().stopPreviewServer()
})
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

ipcMain.on('app:log', (_event, payload) => {
  const level = payload && payload.level ? String(payload.level).toUpperCase() : 'INFO'
  const scope = payload && payload.scope ? String(payload.scope) : 'renderer.unknown'
  const message = payload && payload.message ? String(payload.message) : ''
  const meta = payload && payload.meta !== undefined ? payload.meta : undefined
  const scopedLog = createLogger(scope)
  if (level === 'ERROR') {
    scopedLog.error(message, meta)
    return
  }
  if (level === 'WARN') {
    scopedLog.warn(message, meta)
    return
  }
  scopedLog.info(message, meta)
})

function bootstrap() {
  const logFilePath = initLogger(app.getPath('userData'))
  bootstrapLog.info('started', {
    version,
    isPackaged: app.isPackaged,
    pid: process.pid,
    userData: app.getPath('userData'),
    logFilePath
  })

  try {
    getNative().initNativeLogger(logFilePath)
    nativeBridgeLog.info('logger initialized', { logFilePath })
  } catch (error) {
    nativeBridgeLog.error('logger initialization failed', { error: String(error) })
  }

  let staticPort
  try {
    staticPort = getNative().startStaticFileServer()
    staticServerLog.info('started', { port: staticPort })
  } catch (error) {
    staticServerLog.error('failed to start', { error: String(error) })
    dialog.showErrorBox(
      APP_NAME,
      `静态资源服务启动失败，应用可能显示白屏。\n\n${String(error)}\n\n日志：${logFilePath}`
    )
    throw error
  }

  createMenu()
  createWindow()
  applyAppShortcuts()
  bootstrapLog.info('completed', { staticPort })
}

process.on('uncaughtException', (error) => {
  processLog.error('uncaught exception', { error: String(error) })
})

process.on('unhandledRejection', (reason) => {
  processLog.error('unhandled rejection', { reason: String(reason) })
})

app.whenReady().then(() => {
  if (!gotSingleInstanceLock) {
    return
  }

  try {
    bootstrap()
  } catch (error) {
    bootstrapLog.error('failed', { error: String(error) })
  }

  app.on('second-instance', () => {
    appLog.info('second instance requested, focusing existing window')
    if (!mainWindow || mainWindow.isDestroyed()) {
      createWindow()
      return
    }
    if (mainWindow.isMinimized()) {
      mainWindow.restore()
    }
    mainWindow.show()
    if (process.platform === 'darwin') {
      app.dock.show()
    }
    mainWindow.focus()
  })

  app.on('activate', () => {
    appLog.info('activated', { windowCount: BrowserWindow.getAllWindows().length })
    if (BrowserWindow.getAllWindows().length === 0) {
      createWindow()
    }
  })
})

app.on('before-quit', () => {
  quitting = true
  appLog.info('quitting')
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
