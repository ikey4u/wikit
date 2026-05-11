const { app, BrowserWindow, Menu, ipcMain, shell, dialog } = require('electron')
const path = require('node:path')

const version = '0.5.0'
let native
let mainWindow
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
    trafficLightPosition: { x: 12, y: 11 },
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
ipcMain.handle('native:ffi-hello', (_event, name) => getNative().ffiHello(name))
ipcMain.handle('static:start', () => getNative().startStaticFileServer())
ipcMain.handle('preview:start', (_event, dir) => getNative().startPreviewServer(dir))
ipcMain.handle('preview:stop', () => getNative().stopPreviewServer())
ipcMain.handle('preview:is-up', () => getNative().isPreviewServerUp())
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
