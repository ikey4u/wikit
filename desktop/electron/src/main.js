const { app, BrowserWindow, Menu, ipcMain, shell, dialog } = require('electron')
const path = require('node:path')

const version = '0.5.0'
let native
let mainWindow
let settingsWindow
let quitting = false

function getNative() {
  if (!native) {
    const nativePath = app.isPackaged
      ? path.join(process.resourcesPath, 'native')
      : path.join(__dirname, '../../native')
    native = require(nativePath)
  }
  return native
}

function createWindow() {
  mainWindow = new BrowserWindow({
    width: 1024,
    height: 740,
    resizable: false,
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
    width: 520,
    height: 620,
    resizable: false,
    fullscreen: false,
    title: 'Translation Settings',
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

ipcMain.handle('dict:list', () => getNative().getDictList())
ipcMain.handle('dict:lookup', (_event, dictid, word) => getNative().lookup(dictid, word))
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
ipcMain.on('js-event', (event, payload) => {
  console.log(`got js-event with message '${JSON.stringify(payload)}'`)
  event.sender.send('rust-event', 'something else')
})

app.whenReady().then(() => {
  getNative().startStaticFileServer()
  createMenu()
  createWindow()

  app.on('activate', () => {
    if (BrowserWindow.getAllWindows().length === 0) {
      createWindow()
    }
  })
})

app.on('before-quit', () => {
  quitting = true
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
