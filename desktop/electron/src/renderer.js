const translateTab = document.getElementById('translateTab')
const dictTab = document.getElementById('dictTab')
const toolsTab = document.getElementById('toolsTab')
const settingsButton = document.getElementById('settingsButton')
const openTranslationSettings = document.getElementById('openTranslationSettings')
const translatePage = document.getElementById('translatePage')
const wordPage = document.getElementById('wordPage')
const toolsPage = document.getElementById('toolsPage')
const translationInput = document.getElementById('translationInput')
const translateAction = document.getElementById('translateAction')
const clearTranslationAction = document.getElementById('clearTranslationAction')
const translationOutput = document.getElementById('translationOutput')
const translationHistoryList = document.getElementById('translationHistoryList')
const sourceLanguageSelect = document.getElementById('sourceLanguageSelect')
const targetLanguageSelect = document.getElementById('targetLanguageSelect')
const activeModelLabel = document.getElementById('activeModelLabel')
const dictSelect = document.getElementById('dictSelect')
const searchInput = document.getElementById('searchInput')
const noDictionary = document.getElementById('noDictionary')
const importWikitDictBtn = document.getElementById('importWikitDictBtn')
const importMdxDictBtn = document.getElementById('importMdxDictBtn')
const importDictStatus = document.getElementById('importDictStatus')
const openConfigDirBtn = document.getElementById('openConfigDirBtn')
const addDictMenuBtn = document.getElementById('addDictMenuBtn')
const addDictMenu = document.getElementById('addDictMenu')
const toolbarImportWikitBtn = document.getElementById('toolbarImportWikitBtn')
const toolbarImportMdxBtn = document.getElementById('toolbarImportMdxBtn')
const toolbarImportStatus = document.getElementById('toolbarImportStatus')
const wordContent = document.getElementById('wordContent')
const meaningPlaceholder = document.getElementById('meaningPlaceholder')
const candidateList = document.getElementById('candidateList')
const noCandidate = document.getElementById('noCandidate')
const meaningFrame = document.getElementById('meaningFrame')
const previewFrame = document.getElementById('previewFrame')
const suggestionPopup = document.getElementById('suggestionPopup')
const dictMakerTab = document.getElementById('dictMakerTab')
const mdxConverterTab = document.getElementById('mdxConverterTab')
const moreToolsTab = document.getElementById('moreToolsTab')
const dictMakerContent = document.getElementById('dictMakerContent')
const mdxConverterContent = document.getElementById('mdxConverterContent')
const dictMakerSearch = document.getElementById('dictMakerSearch')
const entriesList = document.getElementById('entriesList')
const contentEditor = document.getElementById('contentEditor')
const cssEditor = document.getElementById('cssEditor')
const jsEditor = document.getElementById('jsEditor')
const contentType = document.getElementById('contentType')
const editorTypeSelector = document.querySelector('.editor-type-selector')
const togglePreviewBtn = document.getElementById('togglePreviewBtn')
const previewIframe = document.getElementById('previewIframe')
const saveEntryBtn = document.getElementById('saveEntryBtn')
const deleteEntryBtn = document.getElementById('deleteEntryBtn')
const addEntryBtn = document.getElementById('addEntryBtn')
const saveStatus = document.getElementById('saveStatus')
const showCssPreview = document.getElementById('showCssPreview')
const showJsPreview = document.getElementById('showJsPreview')

const dictEditorTab = document.getElementById('dictEditorTab')
const dictEditorContent = document.getElementById('dictEditorContent')
const deDictSelect = document.getElementById('deDictSelect')
const deOpenLocalDictBtn = document.getElementById('deOpenLocalDictBtn')
const deEditMetaBtn = document.getElementById('deEditMetaBtn')
const deSearchInput = document.getElementById('deSearchInput')
const deEntryList = document.getElementById('deEntryList')
const deViewer = document.getElementById('deViewer')
const deTogglePreviewBtn = document.getElementById('deTogglePreviewBtn')
const deForceFormatSelect = document.getElementById('deForceFormatSelect')
const deEditMode = document.getElementById('deEditMode')
const dePreviewMode = document.getElementById('dePreviewMode')
const deHtmlEditor = document.getElementById('deHtmlEditor')
const deCssEditor = document.getElementById('deCssEditor')
const deJsEditor = document.getElementById('deJsEditor')
const deSaveBtn = document.getElementById('deSaveBtn')
const deSaveStatus = document.getElementById('deSaveStatus')
const deExportBtn = document.getElementById('deExportBtn')
const selectMdxFileBtn = document.getElementById('selectMdxFile')
const mdxConverterStatus = document.getElementById('mdxConverterStatus')
const mdxConverterProgress = document.getElementById('mdxConverterProgress')
const openMdxOutputBtn = document.getElementById('openMdxOutputBtn')

let lookupTimer = null
let latestLookup = null
let currentResponse = null
let previewSocket = null
let activeMode = 'translate'
let currentTranslationSettings = null
let latestTranslation = null
let translationTimer = null
let copyFeedbackTimer = null

let currentTool = 'dict-maker'
let currentDictDir = null
let entries = {}
let currentEntry = null
let isModified = false
let previewTimer = null

let _dmResolve = null
let _dmDone = false

let deCurrentDictId = null
let deCurrentDictInfo = null
let deCurrentWord = null
let deCurrentDefinition = null
let deSearchTimer = null
let deDictMeta = null
let dePreviewModeEnabled = false
let deForcedPreviewFormat = 'html'
let mdxConverterOutputPath = ''

function _dmCleanup(val) {
  if (_dmDone) return
  _dmDone = true
  document.getElementById('dmPromptOverlay').classList.add('is-hidden')
  document.getElementById('dmPromptInput').classList.remove('is-hidden')
  if (_dmResolve) {
    const r = _dmResolve
    _dmResolve = null
    r(val)
  }
}

function dmPrompt(label, defaultValue) {
  return new Promise((resolve) => {
    _dmResolve = resolve
    _dmDone = false
    const overlay = document.getElementById('dmPromptOverlay')
    const input = document.getElementById('dmPromptInput')
    const labelEl = document.getElementById('dmPromptLabel')
    const okBtn = document.getElementById('dmPromptOk')
    const cancelBtn = document.getElementById('dmPromptCancel')
    labelEl.textContent = label
    input.value = defaultValue || ''
    input.classList.remove('is-hidden')
    overlay.classList.remove('is-hidden')
    input.focus()
    okBtn.onclick = () => _dmCleanup(input.value)
    cancelBtn.onclick = () => _dmCleanup(null)
    input.onkeydown = (e) => {
      if (e.key === 'Enter') _dmCleanup(input.value)
      if (e.key === 'Escape') _dmCleanup(null)
    }
  })
}

function dmConfirm(message) {
  return new Promise((resolve) => {
    _dmResolve = resolve
    _dmDone = false
    const overlay = document.getElementById('dmPromptOverlay')
    const input = document.getElementById('dmPromptInput')
    const labelEl = document.getElementById('dmPromptLabel')
    const okBtn = document.getElementById('dmPromptOk')
    const cancelBtn = document.getElementById('dmPromptCancel')
    labelEl.textContent = message
    input.classList.add('is-hidden')
    overlay.classList.remove('is-hidden')
    okBtn.focus()
    okBtn.onclick = () => _dmCleanup(true)
    cancelBtn.onclick = () => _dmCleanup(false)
    okBtn.onkeydown = (e) => {
      if (e.key === 'Enter') _dmCleanup(true)
      if (e.key === 'Escape') _dmCleanup(false)
    }
  })
}

const translationHistoryKey = 'wikit.translation.history'
const translationLanguageKey = 'wikit.translation.languages'
const maxTranslationHistory = 50
const translationDebounceDelay = 700
const languageLabels = {
  auto: '自动检测',
  zh: '中文',
  en: 'English',
  ja: '日本語',
  ko: '한국어'
}

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
  toolsTab.classList.toggle('is-active', mode === 'tools')
  translatePage.classList.toggle('is-hidden', mode !== 'translate')
  wordPage.classList.toggle('is-hidden', mode !== 'dictionary')
  toolsPage.classList.toggle('is-hidden', mode !== 'tools')

  if (mode === 'translate') {
    translationInput.focus()
  } else if (mode === 'dictionary') {
    if (wordContent.classList.contains('is-hidden')) {
      importWikitDictBtn?.focus()
    } else {
      searchInput.focus()
    }
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

function isLookupPanelActive() {
  return !wordContent.classList.contains('is-hidden')
}

function setImportControlsDisabled(disabled) {
  for (const button of [
    importWikitDictBtn,
    importMdxDictBtn,
    toolbarImportWikitBtn,
    toolbarImportMdxBtn,
    addDictMenuBtn
  ]) {
    if (button) {
      button.disabled = disabled
    }
  }
}

function applyImportStatus(element, text, type, baseClass) {
  element.textContent = text
  element.className = baseClass
  if (type) {
    element.classList.add(`is-${type}`)
  }
  show(element)
}

function showImportDictStatus(text, type) {
  if (isLookupPanelActive()) {
    applyImportStatus(toolbarImportStatus, text, type, 'dict-toolbar-status')
    hide(importDictStatus)
    return
  }
  applyImportStatus(importDictStatus, text, type, 'dict-empty-status')
  hide(toolbarImportStatus)
}

function hideImportDictStatus() {
  importDictStatus.textContent = ''
  importDictStatus.className = 'dict-empty-status is-hidden'
  toolbarImportStatus.textContent = ''
  toolbarImportStatus.className = 'dict-toolbar-status is-hidden'
}

function closeAddDictMenu() {
  hide(addDictMenu)
  addDictMenuBtn.setAttribute('aria-expanded', 'false')
}

function toggleAddDictMenu() {
  const willOpen = addDictMenu.classList.contains('is-hidden')
  if (willOpen) {
    show(addDictMenu)
    addDictMenuBtn.setAttribute('aria-expanded', 'true')
  } else {
    closeAddDictMenu()
  }
}

async function importDictionary(expectedSuffix) {
  closeAddDictMenu()

  const filePath = await window.wikit.openFile()
  if (!filePath) {
    return
  }

  const suffix = (filePath.split('.').pop() || '').toLowerCase()
  if (suffix !== expectedSuffix) {
    showImportDictStatus(`请选择 .${expectedSuffix} 词典文件`, 'error')
    return
  }

  setImportControlsDisabled(true)
  showImportDictStatus(
    expectedSuffix === 'mdx' ? '正在转换 MDX 词典，请稍候…' : '正在加载词典…',
    ''
  )

  try {
    const dict = await window.wikit.loadLocalDict(filePath)
    await loadDictionaries()
    if (dict && dict.id) {
      dictSelect.value = dict.id
    }
    showImportDictStatus(
      `已导入：${dict && dict.name ? dict.name : '词典'}`,
      'success'
    )
    searchInput.focus()
  } catch (error) {
    showImportDictStatus('导入失败：' + getErrorMessage(error), 'error')
  } finally {
    setImportControlsDisabled(false)
  }
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
      hideImportDictStatus()
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

function getLanguageLabel(value) {
  return languageLabels[value] || value
}

function closeCustomSelects(except = null) {
  document.querySelectorAll('.custom-select.is-open').forEach((select) => {
    if (select !== except) {
      select.classList.remove('is-open')
      const trigger = select.querySelector('.select-trigger')
      if (trigger) {
        trigger.setAttribute('aria-expanded', 'false')
      }
    }
  })
}

function setCustomSelectValue(input, value) {
  const select = input.closest('.custom-select')
  if (!select) {
    input.value = value
    return
  }
  const options = Array.from(select.querySelectorAll('.select-option'))
  const selected = options.find((option) => option.dataset.value === value) || options[0]
  if (!selected) {
    return
  }
  input.value = selected.dataset.value
  const valueLabel = select.querySelector('.select-value')
  if (valueLabel) {
    valueLabel.textContent = selected.textContent.trim()
  }
  for (const option of options) {
    const active = option === selected
    option.classList.toggle('is-selected', active)
    option.setAttribute('aria-selected', String(active))
  }
}

function saveTranslationLanguages() {
  localStorage.setItem(translationLanguageKey, JSON.stringify({
    source: sourceLanguageSelect.value,
    target: targetLanguageSelect.value
  }))
}

function loadTranslationLanguages() {
  const languages = parseJson(localStorage.getItem(translationLanguageKey), {})
  if (languages && languageLabels[languages.source]) {
    setCustomSelectValue(sourceLanguageSelect, languages.source)
  }
  if (languages && languageLabels[languages.target] && languages.target !== 'auto') {
    setCustomSelectValue(targetLanguageSelect, languages.target)
  }
}

function handleTranslationLanguageChange() {
  saveTranslationLanguages()
  scheduleTranslation()
}

function initCustomSelects() {
  document.querySelectorAll('.custom-select').forEach((select) => {
    const input = select.querySelector('input')
    const trigger = select.querySelector('.select-trigger')
    const options = Array.from(select.querySelectorAll('.select-option'))
    if (!input || !trigger || !options.length) {
      return
    }
    setCustomSelectValue(input, input.value)
    trigger.addEventListener('click', (event) => {
      event.stopPropagation()
      const open = !select.classList.contains('is-open')
      closeCustomSelects(select)
      select.classList.toggle('is-open', open)
      trigger.setAttribute('aria-expanded', String(open))
    })
    trigger.addEventListener('keydown', (event) => {
      if (event.key === 'Escape') {
        closeCustomSelects()
      }
    })
    for (const option of options) {
      option.addEventListener('click', (event) => {
        event.stopPropagation()
        setCustomSelectValue(input, option.dataset.value)
        input.dispatchEvent(new Event('change', { bubbles: true }))
        closeCustomSelects()
      })
    }
  })
  document.addEventListener('click', () => closeCustomSelects())
}

function readTranslationHistory() {
  const history = parseJson(localStorage.getItem(translationHistoryKey), [])
  return Array.isArray(history) ? history : []
}

function saveTranslationHistory(history) {
  localStorage.setItem(translationHistoryKey, JSON.stringify(history.slice(0, maxTranslationHistory)))
}

function renderTranslationHistory() {
  translationHistoryList.replaceChildren()
  const history = readTranslationHistory()
  if (!history.length) {
    const empty = document.createElement('div')
    empty.className = 'history-empty'
    empty.textContent = '暂无历史记录'
    translationHistoryList.appendChild(empty)
    return
  }
  for (const item of history) {
    const button = document.createElement('button')
    button.className = 'history-item'
    button.type = 'button'
    button.addEventListener('click', () => {
      setCustomSelectValue(sourceLanguageSelect, item.source_language)
      setCustomSelectValue(targetLanguageSelect, item.target_language)
      translationInput.value = item.source_text
      renderTranslationResult(item.translated_text)
      translationOutput.focus()
    })

    const meta = document.createElement('div')
    meta.className = 'history-meta'
    meta.textContent = `${getLanguageLabel(item.source_language)} → ${getLanguageLabel(item.target_language)}`

    const source = document.createElement('div')
    source.className = 'history-text'
    source.textContent = item.source_text

    const target = document.createElement('div')
    target.className = 'history-result'
    target.textContent = item.translated_text

    button.append(meta, source, target)
    translationHistoryList.appendChild(button)
  }
}

function addTranslationHistory(record) {
  const history = readTranslationHistory().filter((item) => (
    item.source_text !== record.source_text ||
    item.source_language !== record.source_language ||
    item.translated_text !== record.translated_text ||
    item.target_language !== record.target_language
  ))
  history.unshift(record)
  saveTranslationHistory(history)
  renderTranslationHistory()
}

function updateActiveModelLabel(settings) {
  currentTranslationSettings = settings
  if (!activeModelLabel) {
    return
  }
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

function getTranslationOutputText() {
  const result = translationOutput.querySelector('.translation-result')
  return result ? result.textContent.trim() : ''
}

async function copyTranslationOutput() {
  translationOutput.focus()
  const text = getTranslationOutputText()
  if (!text) {
    return
  }
  await window.wikit.writeClipboardText(text)
  translationOutput.classList.add('is-copied')
  if (copyFeedbackTimer) {
    clearTimeout(copyFeedbackTimer)
  }
  copyFeedbackTimer = setTimeout(() => {
    translationOutput.classList.remove('is-copied')
  }, 900)
}

function renderTranslationPlaceholder(message) {
  translationOutput.replaceChildren()
  const placeholder = document.createElement('div')
  placeholder.className = 'output-placeholder'
  placeholder.textContent = message
  translationOutput.appendChild(placeholder)
}

function scheduleTranslation() {
  if (activeMode !== 'translate') {
    return
  }
  if (translationTimer) {
    clearTimeout(translationTimer)
  }
  const text = translationInput.value.trim()
  latestTranslation = Symbol('pending-translation')
  translateAction.disabled = false
  translateAction.textContent = '翻译'
  if (!text) {
    renderTranslationPlaceholder('输入文本后会自动翻译。')
    return
  }
  translationTimer = setTimeout(() => {
    requestTranslation()
  }, translationDebounceDelay)
}

async function requestTranslation() {
  if (translationTimer) {
    clearTimeout(translationTimer)
    translationTimer = null
  }
  const text = translationInput.value.trim()
  if (!text) {
    latestTranslation = null
    renderTranslationPlaceholder('输入文本后会自动翻译。')
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
    addTranslationHistory({
      source_text: text,
      source_language: sourceLanguageSelect.value,
      translated_text: response.text,
      target_language: targetLanguageSelect.value,
      created_at: new Date().toISOString()
    })
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
  if (translationTimer) {
    clearTimeout(translationTimer)
    translationTimer = null
  }
  latestTranslation = null
  translateAction.disabled = false
  translateAction.textContent = '翻译'
  translationInput.value = ''
  renderTranslationPlaceholder('输入文本后会自动翻译。')
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

translateTab.addEventListener('click', () => setActiveMode('translate'))
dictTab.addEventListener('click', () => setActiveMode('dictionary'))
toolsTab.addEventListener('click', () => setActiveMode('tools'))
settingsButton.addEventListener('click', openSettingsWindow)
openTranslationSettings.addEventListener('click', openSettingsWindow)
translateAction.addEventListener('click', requestTranslation)
clearTranslationAction.addEventListener('click', clearTranslation)
translationInput.addEventListener('input', scheduleTranslation)
sourceLanguageSelect.addEventListener('change', handleTranslationLanguageChange)
targetLanguageSelect.addEventListener('change', handleTranslationLanguageChange)
document.addEventListener('keydown', (event) => {
  if (activeMode !== 'translate' || event.key !== 'Enter') {
    return
  }
  if (event.metaKey || event.ctrlKey) {
    event.preventDefault()
    copyTranslationOutput().catch(() => {})
    return
  }
  const target = event.target
  if (target !== translationInput && !event.altKey && !event.shiftKey) {
    event.preventDefault()
    translationInput.focus()
  }
})
searchInput.addEventListener('input', scheduleLookup)
dictSelect.addEventListener('change', lookupCurrentWord)
importWikitDictBtn.addEventListener('click', () => importDictionary('wikit'))
importMdxDictBtn.addEventListener('click', () => importDictionary('mdx'))
toolbarImportWikitBtn.addEventListener('click', () => importDictionary('wikit'))
toolbarImportMdxBtn.addEventListener('click', () => importDictionary('mdx'))
addDictMenuBtn.addEventListener('click', (event) => {
  event.stopPropagation()
  toggleAddDictMenu()
})
openConfigDirBtn.addEventListener('click', () => {
  window.wikit.openConfigDir().catch(() => {})
})
document.addEventListener('click', (event) => {
  if (!addDictMenu || addDictMenu.classList.contains('is-hidden')) {
    return
  }
  if (event.target.closest('.dict-add-control')) {
    return
  }
  closeAddDictMenu()
})
previewFrame.addEventListener('load', connectPreviewSocket)

window.wikit.onTranslationSettingsUpdated((settingsJson) => {
  updateActiveModelLabel(parseJson(settingsJson, null))
})
window.wikit.startPreviewer = startPreviewer
window.wikit.stopPreviewer = stopPreviewer
window.wikit.startStaticFileServer().catch(() => {})

dictMakerTab.addEventListener('click', () => setActiveTool('dict-maker'))
dictEditorTab.addEventListener('click', () => setActiveTool('dict-editor'))
mdxConverterTab.addEventListener('click', () => setActiveTool('mdx-converter'))
moreToolsTab.addEventListener('click', toggleMoreMenu)
document.addEventListener('click', (event) => {
  if (!moreToolsTab.contains(event.target)) {
    closeMoreMenu()
  }
})
document.querySelectorAll('.tools-menu-item').forEach((item) => {
  item.addEventListener('click', (event) => {
    const tool = event.target.dataset.tool
    if (tool) {
      setActiveTool(tool)
      closeMoreMenu()
    }
  })
})

document.getElementById('openDictDir').addEventListener('click', openDictDirectory)
document.getElementById('editMetaBtn').addEventListener('click', openMetaEditor)
addEntryBtn.addEventListener('click', addNewEntry)
saveEntryBtn.addEventListener('click', saveCurrentEntry)
deleteEntryBtn.addEventListener('click', deleteCurrentEntry)
document.getElementById('batchImportBtn').addEventListener('click', openBatchImport)

contentEditor.addEventListener('input', () => {
  markModified()
  schedulePreview()
})
cssEditor.addEventListener('input', () => {
  markModified()
  schedulePreview()
})
jsEditor.addEventListener('input', () => {
  markModified()
  schedulePreview()
})
showCssPreview.addEventListener('change', schedulePreview)
showJsPreview.addEventListener('change', schedulePreview)

document.querySelectorAll('.editor-tab').forEach((tab) => {
  tab.addEventListener('click', (event) => {
    const editor = event.target.dataset.editor
    document.querySelectorAll('.editor-tab').forEach((t) => t.classList.remove('is-active'))
    event.target.classList.add('is-active')
    const isContentEditor = editor === 'content'
    contentEditor.classList.toggle('is-hidden', !isContentEditor)
    cssEditor.classList.toggle('is-hidden', editor !== 'css')
    jsEditor.classList.toggle('is-hidden', editor !== 'js')
    editorTypeSelector.classList.remove('is-hidden')
    contentType.classList.toggle('is-hidden', !isContentEditor)
    if (!isContentEditor) setDmPreviewMode(false)
  })
})

function setActiveTool(tool) {
  currentTool = tool
  dictMakerTab.classList.toggle('is-active', tool === 'dict-maker')
  dictEditorTab.classList.toggle('is-active', tool === 'dict-editor')
  mdxConverterTab.classList.toggle('is-active', tool === 'mdx-converter')
  dictMakerContent.classList.toggle('is-hidden', tool !== 'dict-maker')
  dictEditorContent.classList.toggle('is-hidden', tool !== 'dict-editor')
  mdxConverterContent.classList.toggle('is-hidden', tool !== 'mdx-converter')
  if (tool === 'dict-editor' && deDictSelect.options.length <= 1) {
    loadDeDictList()
  }
}

function toggleMoreMenu() {
  moreToolsTab.classList.toggle('is-open')
}

function closeMoreMenu() {
  moreToolsTab.classList.remove('is-open')
}

function showSaveStatus(text, type) {
  saveStatus.textContent = text
  saveStatus.className = 'save-status'
  if (type) {
    saveStatus.classList.add(`is-${type}`)
  }
}

function clearSaveStatus() {
  saveStatus.textContent = ''
  saveStatus.className = 'save-status'
}

function markModified() {
  isModified = true
  showSaveStatus('未保存', '')
}

async function openDictDirectory() {
  const dir = await window.wikit.openDirectory()
  if (dir) {
    currentDictDir = dir
    const openDictDirBtn = document.getElementById('openDictDir')
    if (openDictDirBtn) openDictDirBtn.textContent = dir.split('/').pop() || dir
    await loadEntries()
  }
}

function parseToml(text) {
  const result = {}
  if (!text || !text.trim()) return result
  const lines = text.split('\n')
  for (const line of lines) {
    const m = line.match(/^(\w+)\s*=\s*"(.*)"/)
    if (m) { result[m[1]] = m[2]; continue }
    const a = line.match(/^(\w+)\s*=\s*\[(.*)\]/)
    if (a) { result[a[1]] = a[2].split(',').map(s => s.trim().replace(/^"|"$/g, '')).filter(Boolean); continue }
    const b = line.match(/^(\w+)\s*=\s*(.+)/)
    if (b) { result[b[1]] = b[2].trim() }
  }
  return result
}

function buildToml(data) {
  const lines = []
  lines.push(`name = "${data.name || ''}"`)
  lines.push(`version = "${data.version || '1.0'}"`)
  const authors = (data.authors || []).map(a => `"${a}"`).join(', ')
  lines.push(`authors = [${authors}]`)
  const distributors = (data.distributors || []).map(d => `"${d}"`).join(', ')
  lines.push(`distributors = [${distributors}]`)
  lines.push(`description = "${data.description || ''}"`)
  lines.push(`homepage = "${data.homepage || ''}"`)
  lines.push(`css = "${data.css || ''}"`)
  lines.push(`js = "${data.js || ''}"`)
  return lines.join('\n')
}

async function openMetaEditor() {
  if (!currentDictDir) {
    showSaveStatus('请先选择词典目录', 'error')
    return
  }
  const dirName = currentDictDir.split('/').pop() || currentDictDir
  const tomlPath = `${currentDictDir}/${dirName}.toml`
  let data = { name: dirName, version: '1.0', authors: [''], distributors: [''], description: '', homepage: '', css: '', js: '' }
  try {
    const content = await window.wikit.readTextFile(tomlPath)
    if (content) {
      const parsed = parseToml(content)
      if (parsed.name) data.name = parsed.name
      if (parsed.version) data.version = parsed.version
      if (parsed.authors) data.authors = parsed.authors
      if (parsed.distributors) data.distributors = parsed.distributors
      if (parsed.description) data.description = parsed.description
      if (parsed.homepage) data.homepage = parsed.homepage
      if (parsed.css) data.css = parsed.css
      if (parsed.js) data.js = parsed.js
    }
  } catch (_e) {}
  document.getElementById('metaName').value = data.name || ''
  document.getElementById('metaVersion').value = data.version || ''
  document.getElementById('metaAuthors').value = Array.isArray(data.authors) ? data.authors.join(', ') : (data.authors || '')
  document.getElementById('metaDistributors').value = Array.isArray(data.distributors) ? data.distributors.join(', ') : (data.distributors || '')
  document.getElementById('metaDescription').value = data.description || ''
  document.getElementById('metaHomepage').value = data.homepage || ''
  document.getElementById('metaEditOverlay').classList.remove('is-hidden')
  document.getElementById('metaName').focus()
}

document.getElementById('metaEditCancel').addEventListener('click', () => {
  document.getElementById('metaEditOverlay').classList.add('is-hidden')
})

document.getElementById('metaEditSave').addEventListener('click', async () => {
  if (!currentDictDir) return
  const dirName = currentDictDir.split('/').pop() || currentDictDir
  const tomlPath = `${currentDictDir}/${dirName}.toml`
  const data = {
    name: document.getElementById('metaName').value.trim(),
    version: document.getElementById('metaVersion').value.trim() || '1.0',
    authors: document.getElementById('metaAuthors').value.split(',').map(s => s.trim()).filter(Boolean),
    distributors: document.getElementById('metaDistributors').value.split(',').map(s => s.trim()).filter(Boolean),
    description: document.getElementById('metaDescription').value.trim(),
    homepage: document.getElementById('metaHomepage').value.trim(),
    css: '',
    js: '',
  }
  if (!data.authors.length) data.authors = ['']
  if (!data.distributors.length) data.distributors = ['']
  const ok = await window.wikit.writeTextFile(tomlPath, buildToml(data))
  document.getElementById('metaEditOverlay').classList.add('is-hidden')
  showSaveStatus(ok ? '元信息已保存' : '保存失败', ok ? 'success' : 'error')
})

async function loadEntries() {
  if (!currentDictDir) return
  entries = {}
  try {
    const content = await window.wikit.readTextFile(`${currentDictDir}/entries.txt`)
    if (content) {
      const parts = content.split('</>').map(s => s.trim()).filter(Boolean)
      for (const part of parts) {
        const lines = part.split('\n')
        if (lines.length < 2) continue
        const keyword = lines[0].trim()
        const rest = lines.slice(1).join('\n').trim()
        entries[keyword] = { keyword, content: rest }
      }
    }
  } catch (_e) {}
  renderEntriesList()
}

function renderEntriesList() {
  entriesList.replaceChildren()
  const query = dictMakerSearch.value.trim().toLowerCase()
  const visibleKeys = Object.keys(entries).filter(key => !query || key.toLowerCase().includes(query))
  if (Object.keys(entries).length === 0 || visibleKeys.length === 0) {
    const empty = document.createElement('div')
    empty.className = 'entries-empty'
    empty.textContent = Object.keys(entries).length === 0
      ? (currentDictDir ? '暂无词条，点击右上角添加' : '请先选择词典目录')
      : '未找到词条'
    entriesList.appendChild(empty)
    return
  }
  for (const key of visibleKeys) {
    const item = document.createElement('div')
    item.className = 'entry-item'
    item.textContent = key
    item.addEventListener('click', () => selectEntry(key))
    entriesList.appendChild(item)
  }
}

function selectEntry(keyword) {
  currentEntry = keyword
  renderEntryEditor()
  document.querySelectorAll('.entry-item').forEach(el => {
    el.classList.toggle('is-selected', el.textContent === keyword)
  })
  clearSaveStatus()
  isModified = false
}

function renderEntryEditor() {
  const entry = entries[currentEntry]
  if (!entry) {
    contentEditor.textContent = ''
    cssEditor.textContent = ''
    jsEditor.textContent = ''
    previewIframe.srcdoc = ''
    return
  }
  contentEditor.textContent = entry.content || ''
  cssEditor.textContent = ''
  jsEditor.textContent = ''
  renderPreview()
}

async function addNewEntry() {
  if (!currentDictDir) {
    await openDictDirectory()
    if (!currentDictDir) return
  }
  const keyword = await dmPrompt('输入词条关键词:')
  if (!keyword) return
  const safeKey = keyword.trim()
  if (!safeKey) return
  if (entries[safeKey]) {
    showSaveStatus('关键词已存在', 'error')
    return
  }
  entries[safeKey] = { keyword: safeKey, content: '' }
  currentEntry = safeKey
  renderEntriesList()
  renderEntryEditor()
  selectEntry(safeKey)
  markModified()
}

async function saveCurrentEntry() {
  if (!currentEntry || !currentDictDir) return
  const entry = entries[currentEntry]
  if (!entry) return
  entry.content = contentEditor.textContent
  const lines = []
  for (const k of Object.keys(entries)) {
    const e = entries[k]
    lines.push(`${e.keyword}\n${e.content}`)
  }
  const fileContent = lines.join('\n</>\n')
  const ok = await window.wikit.writeTextFile(`${currentDictDir}/entries.txt`, fileContent)
  showSaveStatus(ok ? '已保存' : '保存失败', ok ? 'success' : 'error')
  isModified = false
  renderEntriesList()
}

async function deleteCurrentEntry() {
  if (!currentEntry || !currentDictDir) return
  const confirmed = await dmConfirm(`确定删除词条 "${currentEntry}"?`)
  if (!confirmed) return
  delete entries[currentEntry]
  currentEntry = null
  renderEntriesList()
  renderEntryEditor()
  const lines = []
  for (const k of Object.keys(entries)) {
    const e = entries[k]
    lines.push(`${e.keyword}\n${e.content}`)
  }
  const fileContent = lines.join('\n</>\n')
  const ok = await window.wikit.writeTextFile(`${currentDictDir}/entries.txt`, fileContent)
  showSaveStatus(ok ? '已删除' : '删除失败', ok ? 'success' : 'error')
  isModified = false
}

function convertMarkdownToHtml(md) {
  let html = md
  html = html.replace(/^### (.+)$/gm, '<h3>$1</h3>')
  html = html.replace(/^## (.+)$/gm, '<h2>$1</h2>')
  html = html.replace(/^# (.+)$/gm, '<h1>$1</h1>')
  html = html.replace(/\*\*(.+?)\*\*/g, '<strong>$1</strong>')
  html = html.replace(/\*(.+?)\*/g, '<em>$1</em>')
  html = html.replace(/`(.+?)`/g, '<code>$1</code>')
  html = html.replace(/\n/g, '<br>\n')
  return html
}

function schedulePreview() {
  if (previewTimer) clearTimeout(previewTimer)
  previewTimer = setTimeout(renderPreview, 300)
}

function renderPreview() {
  const format = contentType.value
  let bodyHtml = contentEditor.textContent || ''
  if (format === 'markdown') {
    bodyHtml = convertMarkdownToHtml(bodyHtml)
  }
  const css = cssEditor.textContent || ''
  const js = jsEditor.textContent || ''
  const includeCss = showCssPreview.checked && css
  const includeJs = showJsPreview.checked && js
  let doc = `<!DOCTYPE html><html><head><meta charset="UTF-8">`
  if (includeCss) {
    doc += `<style>${css}</style>`
  }
  doc += `</head><body>${bodyHtml}`
  if (includeJs) {
    doc += `<script>${js}<\/script>`
  }
  doc += '</body></html>'
  previewIframe.srcdoc = doc
}

function parseWikitEntries(text) {
  const result = []
  const parts = text.split('</>').map(s => s.trim()).filter(Boolean)
  for (const part of parts) {
    const lines = part.split('\n')
    if (lines.length < 1) continue
    const keyword = lines[0].trim()
    if (!keyword) continue
    const content = lines.length > 1 ? lines.slice(1).join('\n').trim() : ''
    result.push({ keyword, content })
  }
  return result
}

async function openBatchImport() {
  if (!currentDictDir) {
    showSaveStatus('请先选择词典目录', 'error')
    return
  }
  const overlay = document.getElementById('batchImportOverlay')
  document.getElementById('batchImportText').value = ''
  document.getElementById('batchImportFileName').textContent = ''
  overlay.classList.remove('is-hidden')
  document.getElementById('batchImportText').focus()
}

async function executeBatchImport() {
  const text = document.getElementById('batchImportText').value.trim()
  if (!text) {
    showSaveStatus('没有可导入的内容', 'error')
    return
  }
  const parsed = parseWikitEntries(text)
  let added = 0
  let skipped = 0
  for (const entry of parsed) {
    if (!entry.keyword || entries[entry.keyword]) {
      skipped++
      continue
    }
    entries[entry.keyword] = { keyword: entry.keyword, content: entry.content }
    added++
  }
  if (added > 0) {
    const lines = []
    for (const k of Object.keys(entries)) {
      const e = entries[k]
      lines.push(`${e.keyword}\n${e.content}`)
    }
    const fileContent = lines.join('\n</>\n')
    const ok = await window.wikit.writeTextFile(`${currentDictDir}/entries.txt`, fileContent)
    if (ok) {
      showSaveStatus(`已导入 ${added} 条词条${skipped > 0 ? `，跳过 ${skipped} 条` : ''}`, 'success')
      renderEntriesList()
      if (!currentEntry) {
        const firstKey = Object.keys(entries)[0]
        if (firstKey) selectEntry(firstKey)
      }
    } else {
      showSaveStatus('写入文件失败', 'error')
    }
  } else {
    showSaveStatus(skipped > 0 ? `全部 ${skipped} 条均已存在，无新增` : '未解析到有效词条', 'error')
  }
  document.getElementById('batchImportOverlay').classList.add('is-hidden')
}

document.getElementById('batchImportCancel').addEventListener('click', () => {
  document.getElementById('batchImportOverlay').classList.add('is-hidden')
})

document.getElementById('batchImportConfirm').addEventListener('click', executeBatchImport)

document.getElementById('batchImportFileBtn').addEventListener('click', async () => {
  const filePath = await window.wikit.openFile()
  if (!filePath) return
  try {
    const content = await window.wikit.readTextFile(filePath)
    document.getElementById('batchImportText').value = content
    document.getElementById('batchImportFileName').textContent = filePath.split('/').pop() || filePath
  } catch (_e) {
    showSaveStatus('读取文件失败', 'error')
  }
})

let dmPreviewMode = false

function setDmPreviewMode(enabled) {
  dmPreviewMode = enabled
  togglePreviewBtn.classList.toggle('is-active', dmPreviewMode)
  togglePreviewBtn.textContent = dmPreviewMode ? '编辑' : '预览'
  document.getElementById('dmEditMode').classList.toggle('is-hidden', dmPreviewMode)
  document.getElementById('dmPreviewMode').classList.toggle('is-hidden', !dmPreviewMode)
  if (dmPreviewMode) renderPreview()
}

togglePreviewBtn.addEventListener('click', () => {
  setDmPreviewMode(!dmPreviewMode)
})

async function buildDictionary() {
  if (!currentDictDir) {
    showSaveStatus('请先选择词典目录', 'error')
    return
  }
  const dirName = currentDictDir.split('/').pop() || currentDictDir
  const tomlPath = `${currentDictDir}/${dirName}.toml`
  const txtPath = `${currentDictDir}/entries.txt`
  const outPath = `${currentDictDir}/${dirName}.wikit`
  let txtExists
  try { await window.wikit.readTextFile(txtPath); txtExists = true } catch (_e) { txtExists = false }
  if (!txtExists) {
    showSaveStatus('未找到 entries.txt', 'error')
    return
  }
  let tomlExists
  try { await window.wikit.readTextFile(tomlPath); tomlExists = true } catch (_e) { tomlExists = false }
  if (!tomlExists) {
    const defaultToml = buildToml({ name: dirName, version: '1.0', authors: [''], distributors: [''], description: '', homepage: '', css: '', js: '' })
    await window.wikit.writeTextFile(tomlPath, defaultToml)
  }
  const btn = document.getElementById('buildDictBtn')
  btn.disabled = true
  btn.textContent = '构建中...'
  showBuildProgress(0)
  let unlistenProgress = null
  try {
    unlistenProgress = window.wikit.onBuildProgress((pct) => {
      showBuildProgress(pct * 100)
    })
    const result = await window.wikit.buildWikit(txtPath, outPath)
    if (result.ok) {
      showBuildProgress(100)
      showSaveStatus('已构建: ' + dirName + '.wikit', 'success')
    } else {
      showSaveStatus(result.error || '构建失败', 'error')
    }
  } catch (_e) {
    showSaveStatus('构建失败', 'error')
  } finally {
    if (unlistenProgress) unlistenProgress()
    hideBuildProgress()
    btn.disabled = false
    btn.textContent = '构建词典'
  }
}

function showBuildProgress(pct) {
  let bar = document.getElementById('buildProgressBar')
  if (!bar) {
    bar = document.createElement('div')
    bar.id = 'buildProgressBar'
    bar.className = 'dm-build-progress'
    bar.innerHTML = '<div class="dm-progress-fill"></div><span class="dm-progress-text">0%</span>'
    document.querySelector('.dm-bottombar').prepend(bar)
  }
  const fill = bar.querySelector('.dm-progress-fill')
  const text = bar.querySelector('.dm-progress-text')
  fill.style.width = Math.min(Math.round(pct), 100) + '%'
  text.textContent = Math.min(Math.round(pct), 100) + '%'
}

function hideBuildProgress() {
  const bar = document.getElementById('buildProgressBar')
  if (bar) setTimeout(() => bar.remove(), 1500)
}

document.getElementById('buildDictBtn').addEventListener('click', buildDictionary)

function showMdxConverterStatus(text, type) {
  mdxConverterStatus.textContent = text
  mdxConverterStatus.className = 'mdx-converter-status'
  if (type) mdxConverterStatus.classList.add(`is-${type}`)
}

function showMdxConverterProgress(pct) {
  mdxConverterProgress.classList.remove('is-hidden')
  const value = Math.min(Math.max(Math.round(pct), 0), 100)
  mdxConverterProgress.querySelector('.dm-progress-fill').style.width = value + '%'
  mdxConverterProgress.querySelector('.dm-progress-text').textContent = value + '%'
}

function resetMdxConverterProgress() {
  mdxConverterProgress.classList.add('is-hidden')
  mdxConverterProgress.querySelector('.dm-progress-fill').style.width = '0%'
  mdxConverterProgress.querySelector('.dm-progress-text').textContent = '0%'
}

async function convertMdxDictionary(filePath) {
  const suffix = (filePath.split('.').pop() || '').toLowerCase()
  if (suffix !== 'mdx') {
    showMdxConverterStatus('请选择 MDX 文件', 'error')
    return
  }
  const fileName = filePath.split(/[\\/]/).pop() || 'dictionary.mdx'
  const baseName = fileName.replace(/\.[^.]+$/, '') || 'dictionary'
  const sourceDir = filePath.replace(/[\\/][^\\/]*$/, '')
  const outPath = `${sourceDir}/${baseName}.wikit`
  mdxConverterOutputPath = ''
  openMdxOutputBtn.classList.add('is-hidden')
  selectMdxFileBtn.disabled = true
  selectMdxFileBtn.textContent = '转换中...'
  showMdxConverterStatus('正在转换: ' + fileName, '')
  resetMdxConverterProgress()
  showMdxConverterProgress(0)
  let unlistenProgress = null
  try {
    unlistenProgress = window.wikit.onBuildProgress((pct) => {
      showMdxConverterProgress(pct * 100)
    })
    const result = await window.wikit.buildWikit(filePath, outPath)
    if (result.ok) {
      mdxConverterOutputPath = result.output || outPath
      showMdxConverterProgress(100)
      showMdxConverterStatus('已转换: ' + baseName + '.wikit', 'success')
      openMdxOutputBtn.classList.remove('is-hidden')
    } else {
      showMdxConverterStatus(result.error || '转换失败', 'error')
    }
  } catch (error) {
    const message = getErrorMessage(error)
    showMdxConverterStatus('转换失败: ' + (message || '未知错误'), 'error')
  } finally {
    if (unlistenProgress) unlistenProgress()
    selectMdxFileBtn.disabled = false
    selectMdxFileBtn.textContent = '选择 MDX 文件'
  }
}

selectMdxFileBtn.addEventListener('click', async () => {
  const filePath = await window.wikit.openFile()
  if (filePath) convertMdxDictionary(filePath)
})

openMdxOutputBtn.addEventListener('click', () => {
  if (mdxConverterOutputPath) window.wikit.revealPath(mdxConverterOutputPath)
})

initCustomSelects()
loadTranslationLanguages()
renderTranslationHistory()
loadDictionaries()
loadTranslationSettings()
setActiveMode('translate')
setActiveTool('dict-maker')

function addDeDictOption(dict) {
  let opt = Array.from(deDictSelect.options).find(option => option.value === dict.id)
  if (!opt) {
    opt = document.createElement('option')
    deDictSelect.appendChild(opt)
  }
  opt.value = dict.id
  opt.textContent = dict.name
  return opt
}

async function loadDeDictList() {
  deDictSelect.replaceChildren()
  const placeholder = document.createElement('option')
  placeholder.value = ''
  placeholder.textContent = '选择词典...'
  deDictSelect.appendChild(placeholder)
  try {
    const dicts = await window.wikit.getDictList()
    for (const dict of dicts) {
      addDeDictOption(dict)
    }
  } catch (_e) {}
}

deDictSelect.addEventListener('change', async () => {
  const dictid = deDictSelect.value
  setDePreviewMode(false)
  deEditMetaBtn.classList.add('is-hidden')
  closeDeMetaOverlay()
  if (!dictid) {
    deCurrentDictId = null
    deCurrentDictInfo = null
    deCurrentWord = null
    deCurrentDefinition = null
    deDictMeta = null
    deEntryList.replaceChildren()
    deEntryList.innerHTML = '<div class="de-empty">请选择词典</div>'
    deViewer.srcdoc = ''
    deHtmlEditor.textContent = ''
    deCssEditor.textContent = ''
    deJsEditor.textContent = ''
    deSaveBtn.disabled = true
    return
  }
  deCurrentDictId = dictid
  try {
    deCurrentDictInfo = await window.wikit.getDictInfo(dictid)
    deDictMeta = { css: deCurrentDictInfo.style || '', js: deCurrentDictInfo.script || '' }
    deEditMetaBtn.classList.remove('is-hidden')
  } catch (error) {
    const message = getErrorMessage(error)
    console.error('load dictionary info failed:', error)
    deCurrentDictInfo = null
    deDictMeta = null
    deShowSaveStatus('加载词典信息失败: ' + (message || '未知错误'), 'error')
  }
  deCurrentWord = null
  deCurrentDefinition = null
  deEntryList.replaceChildren()
  deEntryList.innerHTML = '<div class="de-empty">输入关键词搜索</div>'
  deViewer.srcdoc = ''
  deHtmlEditor.textContent = ''
  deCssEditor.textContent = deDictMeta ? deDictMeta.css : ''
  deJsEditor.textContent = deDictMeta ? deDictMeta.js : ''
  deSaveBtn.disabled = true
  deSearchInput.value = ''
  deSearchInput.focus()
})

deOpenLocalDictBtn.addEventListener('click', async () => {
  const filePath = await window.wikit.openFile()
  if (!filePath) return
  const suffix = (filePath.split('.').pop() || '').toLowerCase()
  if (suffix !== 'wikit' && suffix !== 'mdx') {
    deShowSaveStatus('请选择 .wikit 或 .mdx 词典文件', 'error')
    return
  }
  deShowSaveStatus(suffix === 'mdx' ? '正在转换并加载 MDX 词典...' : '正在加载本地词典...', '')
  try {
    const dict = await window.wikit.loadLocalDict(filePath)
    addDeDictOption(dict)
    deDictSelect.value = dict.id
    deDictSelect.dispatchEvent(new Event('change'))
    deShowSaveStatus('已加载: ' + dict.name, 'success')
  } catch (error) {
    const message = getErrorMessage(error) || '未知错误'
    console.error('load local dictionary failed:', error)
    deEntryList.replaceChildren()
    deEntryList.innerHTML = `<div class="de-empty">本地词典加载失败<br>${escapePreviewText(message)}</div>`
    deShowSaveStatus('本地词典加载失败: ' + message, 'error')
  }
})

deSearchInput.addEventListener('input', () => {
  if (deSearchTimer) clearTimeout(deSearchTimer)
  deSearchTimer = setTimeout(dePerformSearch, 200)
})

async function dePerformSearch() {
  if (!deCurrentDictId) return
  const word = deSearchInput.value.trim()
  if (!word) {
    deEntryList.replaceChildren()
    deEntryList.innerHTML = '<div class="de-empty">输入关键词搜索</div>'
    return
  }
  try {
    const entries = await window.wikit.searchDict(deCurrentDictId, word)
    deEntryList.replaceChildren()
    if (entries.length === 0) {
      deEntryList.innerHTML = '<div class="de-empty">未找到词条</div>'
      return
    }
    for (const entry of entries) {
      const item = document.createElement('div')
      item.className = 'de-entry-item'
      item.textContent = entry.word
      item.addEventListener('click', () => deSelectEntry(entry.word, entry.definition))
      deEntryList.appendChild(item)
    }
  } catch (_e) {
    deEntryList.replaceChildren()
    deEntryList.innerHTML = '<div class="de-empty">搜索失败</div>'
  }
}

function setDePreviewMode(enabled) {
  dePreviewModeEnabled = enabled
  deTogglePreviewBtn.classList.toggle('is-active', dePreviewModeEnabled)
  deTogglePreviewBtn.textContent = dePreviewModeEnabled ? '编辑' : '预览'
  deEditMode.classList.toggle('is-hidden', dePreviewModeEnabled)
  dePreviewMode.classList.toggle('is-hidden', !dePreviewModeEnabled)
  if (dePreviewModeEnabled) deRenderViewer()
}

function deSelectEntry(word, definition) {
  deCurrentWord = word
  deCurrentDefinition = definition
  document.querySelectorAll('.de-entry-item').forEach(el => {
    el.classList.toggle('is-selected', el.textContent === word)
  })
  deHtmlEditor.textContent = definition || ''
  deSaveBtn.disabled = false
  deRenderViewer()
}

function escapePreviewText(text) {
  return text
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;')
}

function getDePreviewBodyHtml() {
  deForcedPreviewFormat = deForceFormatSelect.value || 'html'
  const body = deHtmlEditor.textContent || ''
  if (deForcedPreviewFormat === 'md') return convertMarkdownToHtml(body)
  if (deForcedPreviewFormat === 'txt') return `<pre class="wikit-plain-preview">${escapePreviewText(body)}</pre>`
  return body
}

function deRenderViewer() {
  const html = getDePreviewBodyHtml()
  const css = deCssEditor.textContent || deDictMeta?.css || ''
  const js = deJsEditor.textContent || deDictMeta?.js || ''
  const chromeStyle = '<style>html{background:#fff;}body{margin:0;padding:16px 18px 32px;color:#1f2328;font-family:-apple-system,BlinkMacSystemFont,"Segoe UI","Helvetica Neue",Arial,sans-serif;line-height:1.58;}body>*:first-child{margin-top:0;}a{color:#0969da;}.wikit-plain-preview{margin:0;white-space:pre-wrap;word-break:break-word;font:inherit;line-height:inherit;}</style>'
  let doc = `<!DOCTYPE html><html><head><meta charset="UTF-8">${css ? '<style>' + css + '</style>' : ''}${chromeStyle}</head><body>${html}`
  if (js) doc += `<script>${js}<\/script>`
  doc += '</body></html>'
  deViewer.srcdoc = doc
}

document.querySelectorAll('.de-editor-tab').forEach(tab => {
  tab.addEventListener('click', (event) => {
    const editor = event.target.dataset.deEditor
    const isHtmlEditor = editor === 'html'
    document.querySelectorAll('.de-editor-tab').forEach(t => t.classList.remove('is-active'))
    event.target.classList.add('is-active')
    deHtmlEditor.classList.toggle('is-hidden', !isHtmlEditor)
    deCssEditor.classList.toggle('is-hidden', editor !== 'css')
    deJsEditor.classList.toggle('is-hidden', editor !== 'js')
    deForceFormatSelect.classList.toggle('is-hidden', !isHtmlEditor)
    setDePreviewMode(false)
  })
})

deTogglePreviewBtn.addEventListener('click', () => {
  setDePreviewMode(!dePreviewModeEnabled)
})

deForceFormatSelect.addEventListener('change', () => {
  deForcedPreviewFormat = deForceFormatSelect.value
  if (dePreviewModeEnabled) deRenderViewer()
})

deHtmlEditor.addEventListener('input', () => deRenderViewer())
deCssEditor.addEventListener('input', () => deRenderViewer())
deJsEditor.addEventListener('input', () => deRenderViewer())

deEditMetaBtn.addEventListener('click', () => {
  if (!deCurrentDictInfo) {
    deShowSaveStatus('请先选择词典', 'error')
    return
  }
  document.getElementById('deMetaName').value = deCurrentDictInfo.name || ''
  document.getElementById('deMetaDesc').value = deCurrentDictInfo.desc || ''
  document.getElementById('deMetaCss').value = deCurrentDictInfo.style || ''
  document.getElementById('deMetaJs').value = deCurrentDictInfo.script || ''
  document.getElementById('deMetaOverlay').classList.remove('is-hidden')
})

function closeDeMetaOverlay() {
  document.getElementById('deMetaOverlay').classList.add('is-hidden')
}

document.getElementById('deMetaClose').addEventListener('click', closeDeMetaOverlay)
document.getElementById('deMetaCloseTop').addEventListener('click', closeDeMetaOverlay)

deSaveBtn.addEventListener('click', () => {
  deShowSaveStatus('wikit/mdx 词典为只读格式，修改仅在本会话生效', 'error')
})

deExportBtn.addEventListener('click', async () => {
  if (!deCurrentDictId) {
    deShowSaveStatus('请先选择词典', 'error')
    return
  }
  const dir = await window.wikit.openDirectory()
  if (!dir) return
  const dictName = deCurrentDictInfo ? deCurrentDictInfo.name : deCurrentDictId.split('/').pop() || 'dict'
  const outPath = `${dir}/${dictName}.wikit`
  try {
    const ok = await window.wikit.copyFile(deCurrentDictId, outPath)
    deShowSaveStatus(ok ? '已导出: ' + dictName + '.wikit' : '导出失败', ok ? 'success' : 'error')
  } catch (_e) {
    deShowSaveStatus('导出失败', 'error')
  }
})

function getErrorMessage(error) {
  if (!error) return ''
  if (typeof error === 'string') return error
  if (error.message) return error.message
  try {
    return JSON.stringify(error)
  } catch (_e) {
    return String(error)
  }
}

function deShowSaveStatus(text, type) {
  deSaveStatus.textContent = text
  deSaveStatus.title = text
  deSaveStatus.className = 'save-status'
  if (type) deSaveStatus.classList.add(`is-${type}`)
}
