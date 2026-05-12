const translateTab = document.getElementById('translateTab')
const dictTab = document.getElementById('dictTab')
const settingsButton = document.getElementById('settingsButton')
const openTranslationSettings = document.getElementById('openTranslationSettings')
const translatePage = document.getElementById('translatePage')
const wordPage = document.getElementById('wordPage')
const translationInput = document.getElementById('translationInput')
const translateAction = document.getElementById('translateAction')
const clearTranslationAction = document.getElementById('clearTranslationAction')
const translationOutput = document.getElementById('translationOutput')
const sourceLanguageSelect = document.getElementById('sourceLanguageSelect')
const targetLanguageSelect = document.getElementById('targetLanguageSelect')
const activeModelLabel = document.getElementById('activeModelLabel')
const dictSelect = document.getElementById('dictSelect')
const searchInput = document.getElementById('searchInput')
const noDictionary = document.getElementById('noDictionary')
const wordContent = document.getElementById('wordContent')
const meaningPlaceholder = document.getElementById('meaningPlaceholder')
const candidateList = document.getElementById('candidateList')
const noCandidate = document.getElementById('noCandidate')
const meaningFrame = document.getElementById('meaningFrame')
const previewFrame = document.getElementById('previewFrame')
const suggestionPopup = document.getElementById('suggestionPopup')

let lookupTimer = null
let latestLookup = null
let currentResponse = null
let previewSocket = null
let activeMode = 'translate'
let currentTranslationSettings = null
let latestTranslation = null

function show(element) {
  element.classList.remove('is-hidden')
}

function hide(element) {
  element.classList.add('is-hidden')
}

function setActiveMode(mode) {
  activeMode = mode
  translateTab.classList.toggle('is-active', mode === 'translate')
  dictTab.classList.toggle('is-active', mode === 'dictionary')
  translatePage.classList.toggle('is-hidden', mode !== 'translate')
  wordPage.classList.toggle('is-hidden', mode !== 'dictionary')

  if (mode === 'translate') {
    translationInput.focus()
  } else {
    searchInput.focus()
  }
}

function showPlaceholder(text) {
  meaningPlaceholder.textContent = text
  show(meaningPlaceholder)
  hide(candidateList)
  hide(noCandidate)
  hide(meaningFrame)
  hide(previewFrame)
  hide(suggestionPopup)
}

function renderCandidates(words, response) {
  candidateList.replaceChildren()
  hide(suggestionPopup)
  hide(meaningPlaceholder)
  hide(meaningFrame)
  hide(previewFrame)

  if (!words.length) {
    hide(candidateList)
    show(noCandidate)
    return
  }

  hide(noCandidate)
  show(candidateList)

  for (const word of words) {
    const item = document.createElement('div')
    const button = document.createElement('button')
    button.className = 'button'
    button.type = 'button'
    button.textContent = word
    button.addEventListener('click', () => {
      searchInput.value = word
      renderMeaning(word, response)
    })
    item.appendChild(button)
    candidateList.appendChild(item)
  }
}

function renderMeaning(word, response) {
  if (!response || !response.words || !response.words[word]) {
    return
  }

  const chromeStyle = '<style>html{background:#fff;}body{margin:0;padding:16px 18px 32px;color:#1f2328;font-family:-apple-system,BlinkMacSystemFont,"Segoe UI","Helvetica Neue",Arial,sans-serif;line-height:1.58;}body>*:first-child{margin-top:0;}a{color:#0969da;}</style>'
  const content = `<!DOCTYPE html><html><head><meta charset="UTF-8">${response.script}${response.style}${chromeStyle}</head><body>${response.words[word]}</body></html>`
  hide(meaningPlaceholder)
  hide(candidateList)
  hide(noCandidate)
  hide(previewFrame)
  show(meaningFrame)
  meaningFrame.srcdoc = content
}

async function loadDictionaries() {
  try {
    const dictionaries = await window.wikit.getDictList()
    dictSelect.replaceChildren()

    for (const dictionary of dictionaries) {
      const option = document.createElement('option')
      option.value = dictionary.id
      option.textContent = dictionary.name
      dictSelect.appendChild(option)
    }

    if (dictionaries.length) {
      hide(noDictionary)
      show(wordContent)
      showPlaceholder('Type a word to look up ...')
    } else {
      hide(wordContent)
      show(noDictionary)
    }
  } catch (_error) {
    hide(wordContent)
    show(noDictionary)
  }
}

async function lookupCurrentWord() {
  const dictid = dictSelect.value
  const word = searchInput.value.trim().toLowerCase()

  if (!dictid || !word) {
    currentResponse = null
    showPlaceholder('Type a word to look up ...')
    return
  }

  const token = Symbol(word)
  latestLookup = token

  try {
    const response = await window.wikit.lookup(dictid, word)

    if (latestLookup !== token) {
      return
    }

    currentResponse = response
    const words = Object.keys(response.words || {}).sort()

    if (response.words && response.words[word]) {
      renderMeaning(word, response)
    } else {
      renderCandidates(words, response)
    }
  } catch (_error) {
    currentResponse = null
    renderCandidates([], null)
  }
}

function scheduleLookup() {
  if (activeMode !== 'dictionary') {
    return
  }
  if (lookupTimer) {
    clearTimeout(lookupTimer)
  }
  lookupTimer = setTimeout(lookupCurrentWord, 50)
}

function openSettingsWindow() {
  window.wikit.openSettingsWindow().catch(() => {})
}

function parseJson(value, fallback) {
  try {
    return JSON.parse(value)
  } catch (_error) {
    return fallback
  }
}

function updateActiveModelLabel(settings) {
  currentTranslationSettings = settings
  if (!settings || !settings.model) {
    activeModelLabel.textContent = '未配置模型'
    return
  }
  activeModelLabel.textContent = `${settings.provider} / ${settings.model}`
}

async function loadTranslationSettings() {
  try {
    const settingsJson = await window.wikit.getTranslationSettings()
    updateActiveModelLabel(parseJson(settingsJson, null))
  } catch (_error) {
    updateActiveModelLabel(null)
  }
}

function renderTranslationResult(text) {
  translationOutput.replaceChildren()
  const result = document.createElement('div')
  result.className = 'translation-result'
  result.textContent = text
  translationOutput.appendChild(result)
}

function renderTranslationPlaceholder(message) {
  translationOutput.replaceChildren()
  const placeholder = document.createElement('div')
  placeholder.className = 'output-placeholder'
  placeholder.textContent = message
  translationOutput.appendChild(placeholder)
}

async function requestTranslation() {
  const text = translationInput.value.trim()
  if (!text) {
    renderTranslationPlaceholder('请输入要翻译的文本。')
    return
  }

  const token = Symbol(text)
  latestTranslation = token
  translateAction.disabled = true
  translateAction.textContent = '翻译中...'
  renderTranslationPlaceholder('正在请求模型，请稍候。')

  try {
    const responseJson = await window.wikit.translateText(JSON.stringify({
      text,
      source: sourceLanguageSelect.value,
      target: targetLanguageSelect.value
    }))

    if (latestTranslation !== token) {
      return
    }

    const response = parseJson(responseJson, null)
    if (!response || !response.text) {
      renderTranslationPlaceholder('模型没有返回可用译文。')
      return
    }

    renderTranslationResult(response.text)
    updateActiveModelLabel({
      provider: response.provider,
      model: response.model
    })
  } catch (error) {
    if (latestTranslation === token) {
      renderTranslationPlaceholder(String(error && error.message ? error.message : error))
    }
  } finally {
    if (latestTranslation === token) {
      translateAction.disabled = false
      translateAction.textContent = '翻译'
    }
  }
}

function clearTranslation() {
  translationInput.value = ''
  renderTranslationPlaceholder('选择模型后，翻译结果会显示在这里。')
  translationInput.focus()
}

function connectPreviewSocket() {
  if (previewSocket) {
    previewSocket.close()
  }

  previewSocket = new WebSocket('ws://127.0.0.1:8088/wss')
  previewSocket.addEventListener('open', () => {
    previewSocket.send('WIKIT_PREVIEWER_CONNECT')
  })
  previewSocket.addEventListener('message', (event) => {
    if (typeof event.data === 'string' && event.data.trim() === 'CMD:RELOAD') {
      previewFrame.src = previewFrame.src
    }
  })
}

async function waitForPreviewServer() {
  for (let i = 0; i < 30; i += 1) {
    try {
      const response = await fetch('http://127.0.0.1:8088')
      if (response.status === 200) {
        return true
      }
    } catch (_error) {
    }
    await new Promise((resolve) => setTimeout(resolve, 1000))
  }
  return false
}

async function startPreviewer() {
  let started = await window.wikit.isPreviewServerUp()
  if (!started) {
    const sourceDir = await window.wikit.openDirectory()
    if (!sourceDir) {
      return
    }
    await window.wikit.startPreviewServer(sourceDir)
    started = await waitForPreviewServer()
  }

  setActiveMode('dictionary')

  if (started) {
    hide(meaningPlaceholder)
    hide(candidateList)
    hide(noCandidate)
    hide(meaningFrame)
    show(previewFrame)
    previewFrame.src = 'http://127.0.0.1:8088'
  }
}

async function stopPreviewer() {
  await window.wikit.stopPreviewServer()
  if (previewSocket) {
    previewSocket.close()
    previewSocket = null
  }
  previewFrame.removeAttribute('src')
  hide(previewFrame)
  showPlaceholder('Type a word to look up ...')
}

windowClose.addEventListener('click', () => window.wikit.closeWindow().catch(() => {}))
windowMinimize.addEventListener('click', () => window.wikit.minimizeWindow().catch(() => {}))
windowZoom.addEventListener('click', () => window.wikit.zoomWindow().catch(() => {}))
translateTab.addEventListener('click', () => setActiveMode('translate'))
dictTab.addEventListener('click', () => setActiveMode('dictionary'))
settingsButton.addEventListener('click', openSettingsWindow)
openTranslationSettings.addEventListener('click', openSettingsWindow)
translateAction.addEventListener('click', requestTranslation)
clearTranslationAction.addEventListener('click', clearTranslation)
translationInput.addEventListener('keydown', (event) => {
  if ((event.metaKey || event.ctrlKey) && event.key === 'Enter') {
    event.preventDefault()
    requestTranslation()
  }
})
searchInput.addEventListener('input', scheduleLookup)
dictSelect.addEventListener('change', lookupCurrentWord)
previewFrame.addEventListener('load', connectPreviewSocket)

window.wikit.onTranslationSettingsUpdated((settingsJson) => {
  updateActiveModelLabel(parseJson(settingsJson, null))
})
window.wikit.startPreviewer = startPreviewer
window.wikit.stopPreviewer = stopPreviewer
window.wikit.startStaticFileServer().catch(() => {})
loadDictionaries()
loadTranslationSettings()
setActiveMode('translate')
