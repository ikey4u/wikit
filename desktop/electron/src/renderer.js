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
const swapLanguagesAction = document.getElementById('swapLanguagesAction')
const activeModelLabel = document.getElementById('activeModelLabel')
const dictSelect = document.getElementById('dictSelect')
const searchInput = document.getElementById('searchInput')
const noDictionary = document.getElementById('noDictionary')
const importWikitDictBtn = document.getElementById('importWikitDictBtn')
const importMdxDictBtn = document.getElementById('importMdxDictBtn')
const importDictStatus = document.getElementById('importDictStatus')
const importDictProgress = document.getElementById('importDictProgress')
const openConfigDirBtn = document.getElementById('openConfigDirBtn')
const addDictMenuBtn = document.getElementById('dictMenuBtn')
const dictMenuBtn = addDictMenuBtn
const addDictMenu = document.getElementById('dictMenu')
const dictMenu = addDictMenu
const toolbarImportWikitBtn = document.getElementById('toolbarImportWikitBtn')
const toolbarImportMdxBtn = document.getElementById('toolbarImportMdxBtn')
const toolbarEditDictBtn = document.getElementById('toolbarEditDictBtn')
const toolbarRemoveDictBtn = document.getElementById('toolbarRemoveDictBtn')
const toolbarImportStatus = document.getElementById('toolbarImportStatus')
const toolbarImportProgress = document.getElementById('toolbarImportProgress')
const wordContent = document.getElementById('wordContent')
const meaningPlaceholder = document.getElementById('meaningPlaceholder')
const candidateList = document.getElementById('candidateList')
const noCandidate = document.getElementById('noCandidate')
const meaningFrame = document.getElementById('meaningFrame')
const previewFrame = document.getElementById('previewFrame')
const suggestionPopup = document.getElementById('suggestionPopup')
const lookupBackBtn = document.getElementById('lookupBackBtn')
const lookupForwardBtn = document.getElementById('lookupForwardBtn')
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

function reportRendererLog(scope, level, message, meta) {
  if (window.wikit && typeof window.wikit.log === 'function') {
    window.wikit.log(scope, level, message, meta)
  }
}

window.addEventListener('error', (event) => {
  reportRendererLog('renderer.runtime', 'error', 'window error', {
    message: event.message,
    filename: event.filename,
    lineno: event.lineno,
    colno: event.colno
  })
})

window.addEventListener('unhandledrejection', (event) => {
  reportRendererLog('renderer.runtime', 'error', 'unhandled rejection', {
    reason: String(event.reason)
  })
})

reportRendererLog('renderer.bootstrap', 'info', 'script started')

let lookupTimer = null
let latestLookup = null
let currentResponse = null
let pendingMeaningHash = ''
let lookupHistoryStack = []
let lookupHistoryIndex = -1
let lookupHistoryNavigating = false
const maxLookupHistory = 80
let previewSocket = null
let activeMode = 'translate'
let currentTranslationSettings = null
let latestTranslation = null
let copyFeedbackTimer = null

let currentTool = 'dict-maker'
let currentDictDir = null
let entries = {}
let currentEntry = null
let isModified = false
let previewTimer = null
let previewPort = null
let dictMetaCssRef = ''
let dictMetaJsRef = ''

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

const translationHistoryKey = 'bootsmind-wikit.translation.history'
const translationLanguageKey = 'bootsmind-wikit.translation.languages'
const maxTranslationHistory = 50
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

async function navigateToWord(word, options = {}) {
  const target = normalizeLookupWord(word)
  if (!target) return

  const dictid = options.dictid || dictSelect.value
  if (!dictid) return

  if (options.dictid && dictSelect.value !== options.dictid) {
    const hasDict = Array.from(dictSelect.options).some((opt) => opt.value === options.dictid)
    if (hasDict) {
      dictSelect.value = options.dictid
    }
  }

  searchInput.value = target
  hide(suggestionPopup)
  await lookupCurrentWord({
    pushHistory: options.pushHistory !== false,
    hash: options.hash || '',
    preferredWord: target,
    forceOpen: options.forceOpen === true,
  })
}

function escapeHtmlText(value) {
  return String(value || '')
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
}

function highlightSnippet(snippet, query) {
  const text = String(snippet || '')
  const q = String(query || '').trim()
  if (!text || !q) return escapeHtmlText(text)
  try {
    const re = new RegExp(`(${q.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')})`, 'gi')
    return escapeHtmlText(text).replace(re, '<mark>$1</mark>')
  } catch (_error) {
    return escapeHtmlText(text)
  }
}

function headwordMatchRank(word, query) {
  const w = String(word || '').trim().toLowerCase()
  const q = String(query || '').trim().toLowerCase()
  if (!w || !q) return 100
  if (w === q) return 0
  if (w.startsWith(q)) return 1
  if (q.startsWith(w)) return 2
  try {
    const escaped = q.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
    if (new RegExp(`(?:^|[^a-z0-9\\u00c0-\\u024f])${escaped}(?:$|[^a-z0-9\\u00c0-\\u024f])`, 'i').test(w)) {
      return 3
    }
  } catch (_error) {
    // fall through
  }
  if (w.includes(q)) return 4
  return 5
}

function sortHeadwordsByRelevance(words, query) {
  return [...(Array.isArray(words) ? words : [])].sort((a, b) => {
    const rankDiff = headwordMatchRank(a, query) - headwordMatchRank(b, query)
    if (rankDiff !== 0) return rankDiff
    const lenDiff = String(a).length - String(b).length
    if (lenDiff !== 0) return lenDiff
    return String(a).localeCompare(String(b), undefined, { sensitivity: 'base' })
  })
}

function sortSearchEntriesByRelevance(entries, query) {
  return [...(Array.isArray(entries) ? entries : [])].sort((a, b) => {
    const aKind = a?.kind === 'body' ? 1 : 0
    const bKind = b?.kind === 'body' ? 1 : 0
    if (aKind !== bKind) return aKind - bKind
    const rankDiff = headwordMatchRank(a?.word, query) - headwordMatchRank(b?.word, query)
    if (rankDiff !== 0) return rankDiff
    return String(a?.word || '').localeCompare(String(b?.word || ''), undefined, { sensitivity: 'base' })
  })
}

function openCandidateEntry(word, response, html) {
  const target = String(word || '').trim()
  if (!target) return

  const cachedHtml = html || (response && response.words && (
    response.words[target] ||
    response.words[Object.keys(response.words).find((key) => key.toLowerCase() === target.toLowerCase()) || '']
  ))

  if (cachedHtml) {
    const dictid = dictSelect.value
    searchInput.value = target
    hide(suggestionPopup)
    if (dictid) {
      pushLookupHistory(target, dictid)
    }
    currentResponse = response || currentResponse
    renderMeaning(target, response || currentResponse, { html: cachedHtml })
    return
  }

  navigateToWord(target, { pushHistory: true, forceOpen: true }).catch(() => {})
}

function renderCandidates(words, response, bodyHits = [], query = '') {
  candidateList.replaceChildren()
  hide(suggestionPopup)
  hide(meaningPlaceholder)
  hide(meaningFrame)
  hide(previewFrame)

  const headwords = Array.isArray(words) ? words : []
  const bodies = Array.isArray(bodyHits) ? bodyHits : []
  if (!headwords.length && !bodies.length) {
    hide(candidateList)
    show(noCandidate)
    return
  }

  hide(noCandidate)
  show(candidateList)

  for (const word of headwords) {
    const item = document.createElement('div')
    item.className = 'candidate-row'
    const button = document.createElement('button')
    button.className = 'candidate-item'
    button.type = 'button'
    button.innerHTML = `
      <span class="candidate-item-head">
        <span class="candidate-item-word">${escapeHtmlText(word)}</span>
        <span class="candidate-item-kind is-headword">词头</span>
      </span>
    `
    button.addEventListener('click', (event) => {
      event.preventDefault()
      event.stopPropagation()
      openCandidateEntry(word, response)
    })
    item.appendChild(button)
    candidateList.appendChild(item)
  }

  for (const hit of bodies) {
    const word = hit.word || hit.headword
    if (!word) continue
    const snippet = hit.snippet || ''
    const item = document.createElement('div')
    item.className = 'candidate-row'
    const button = document.createElement('button')
    button.className = 'candidate-item is-body'
    button.type = 'button'
    button.title = snippet || word
    button.innerHTML = `
      <span class="candidate-item-head">
        <span class="candidate-item-word">${escapeHtmlText(word)}</span>
        <span class="candidate-item-kind">正文</span>
      </span>
      ${snippet ? `<span class="candidate-item-snippet">${highlightSnippet(snippet, query)}</span>` : ''}
    `
    button.addEventListener('click', (event) => {
      event.preventDefault()
      event.stopPropagation()
      openCandidateEntry(word, response, hit.definition)
    })
    item.appendChild(button)
    candidateList.appendChild(item)
  }
}

function updateLookupHistoryButtons() {
  if (lookupBackBtn) {
    lookupBackBtn.disabled = lookupHistoryIndex <= 0
  }
  if (lookupForwardBtn) {
    lookupForwardBtn.disabled = lookupHistoryIndex < 0 || lookupHistoryIndex >= lookupHistoryStack.length - 1
  }
}

function clearLookupHistory() {
  lookupHistoryStack = []
  lookupHistoryIndex = -1
  lookupHistoryNavigating = false
  updateLookupHistoryButtons()
}

function pushLookupHistory(word, dictid) {
  if (lookupHistoryNavigating) return
  const normalized = String(word || '').trim()
  const dict = String(dictid || '').trim()
  if (!normalized || !dict) return

  const current = lookupHistoryIndex >= 0 ? lookupHistoryStack[lookupHistoryIndex] : null
  if (current && current.word === normalized && current.dictid === dict) {
    updateLookupHistoryButtons()
    return
  }

  lookupHistoryStack = lookupHistoryStack.slice(0, lookupHistoryIndex + 1)
  lookupHistoryStack.push({ word: normalized, dictid: dict })
  if (lookupHistoryStack.length > maxLookupHistory) {
    const overflow = lookupHistoryStack.length - maxLookupHistory
    lookupHistoryStack = lookupHistoryStack.slice(overflow)
  }
  lookupHistoryIndex = lookupHistoryStack.length - 1
  updateLookupHistoryButtons()
}

function goLookupHistory(delta) {
  const next = lookupHistoryIndex + delta
  if (next < 0 || next >= lookupHistoryStack.length) return
  const entry = lookupHistoryStack[next]
  if (!entry) return
  lookupHistoryIndex = next
  lookupHistoryNavigating = true
  updateLookupHistoryButtons()
  navigateToWord(entry.word, {
    dictid: entry.dictid,
    pushHistory: false,
    forceOpen: true,
  }).finally(() => {
    lookupHistoryNavigating = false
    updateLookupHistoryButtons()
  })
}

function decodeHrefPart(value) {
  try {
    return decodeURIComponent(value)
  } catch (_error) {
    return value
  }
}

function parseEntryHref(href) {
  let rest = String(href || '').trim()
  // Some MDX/Oxford entries store doubled prefixes like entry://entry://word.
  while (/^entry:/i.test(rest)) {
    rest = rest.replace(/^entry:\/*/i, '')
  }
  rest = decodeHrefPart(rest)
  const hashIndex = rest.indexOf('#')
  let word = rest
  let hash = ''
  if (hashIndex >= 0) {
    word = rest.slice(0, hashIndex)
    hash = rest.slice(hashIndex + 1)
  }
  word = word.replace(/^\/+/, '').replace(/\/+$/, '').trim()
  return { word, hash }
}

function parseSoundHref(href) {
  return decodeHrefPart(
    String(href || '')
      .trim()
      .replace(/^(?:sound|snd):\/*/i, '')
  )
    .replace(/^[\\/]+/, '')
    .trim()
}

function normalizeLookupWord(word) {
  let value = String(word || '').trim()
  while (/^entry:/i.test(value)) {
    value = value.replace(/^entry:\/*/i, '')
  }
  return value.replace(/^\/+/, '').replace(/\/+$/, '').trim()
}

function findExactWordKey(words, word) {
  if (!words || !word) return null
  if (Object.prototype.hasOwnProperty.call(words, word)) return word
  const lower = word.toLowerCase()
  return Object.keys(words).find((key) => key.toLowerCase() === lower) || null
}

function parseLinkRedirect(body) {
  const match = String(body || '').trim().match(/^@@@LINK=\s*(.+?)\s*$/i)
  return match ? normalizeLookupWord(match[1]) : null
}

async function resolveLookupEntry(dictid, word, response, depth = 0) {
  const words = response?.words || {}
  const exactKey = findExactWordKey(words, word)
  if (!exactKey) {
    return { word, response, html: null, resolved: false }
  }

  const body = words[exactKey]
  const redirect = parseLinkRedirect(body)
  if (!redirect) {
    return { word: exactKey, response, html: body, resolved: true }
  }
  if (depth >= 6) {
    return { word: exactKey, response, html: body, resolved: true }
  }

  const cachedTarget = findExactWordKey(words, redirect)
  if (cachedTarget && !parseLinkRedirect(words[cachedTarget])) {
    return { word: cachedTarget, response, html: words[cachedTarget], resolved: true }
  }

  const nextResponse = await window.wikit.lookup(dictid, redirect)
  return resolveLookupEntry(dictid, redirect, nextResponse, depth + 1)
}

function scrollMeaningToHash(hash) {
  if (!hash || !meaningFrame) return
  try {
    const doc = meaningFrame.contentDocument
    if (!doc) return
    const el = doc.getElementById(hash) || doc.querySelector(`[name="${CSS.escape(hash)}"]`)
    if (el && typeof el.scrollIntoView === 'function') {
      el.scrollIntoView({ block: 'start' })
    }
  } catch (_error) {
    // ignore cross-document access failures
  }
}

function escapeHtmlAttr(value) {
  return String(value || '')
    .replace(/&/g, '&amp;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;')
    .replace(/</g, '&lt;')
}

function sanitizeMeaningHtml(html) {
  let out = String(html || '')
  // Convert entry/sound links so the iframe never navigates to blocked custom schemes.
  // Also cover unquoted hrefs and Oxford's snd:// alias.
  out = out.replace(/\shref\s*=\s*(["'])((?:entry|sound|snd):[\s\S]*?)\1/gi, (_match, _quote, href) => (
    ` href="#" data-wikit-nav="${escapeHtmlAttr(href)}"`
  ))
  out = out.replace(/\shref\s*=\s*((?:entry|sound|snd):[^\s>]+)/gi, (_match, href) => (
    ` href="#" data-wikit-nav="${escapeHtmlAttr(href)}"`
  ))
  // Preserve known OALD actions as data attributes; drop other inline handlers (CSP blocks them).
  out = out.replace(
    /\s+onclick\s*=\s*(["'])\s*(toggle_active|toggle_enlarger)\s*\(\s*this\s*\)\s*;?\s*\1/gi,
    ' data-wikit-action="$2" role="button" tabindex="0"'
  )
  out = out.replace(/\s+on[a-z]+\s*=\s*("[^"]*"|'[^']*'|[^\s>]+)/gi, '')
  return out
}

function toggleMeaningUnbox(titleEl) {
  const panel = titleEl.closest('.unbox, [unbox]')
  if (!panel) return false
  panel.classList.toggle('is-active')
  return true
}

function toggleMeaningEnlarger(el) {
  const root = el.id === 'ox-enlarge' ? el : (el.closest('#ox-enlarge') || el)
  if (!root) return false
  const full = root.querySelector('img.fullsize')
  const thumb = root.querySelector('img.thumb')
  if (!full || !thumb) return false
  const currentlyHidden = /display\s*:\s*none/i.test(full.getAttribute('style') || '') || full.style.display === 'none'
  if (currentlyHidden) {
    full.style.display = ''
    thumb.style.display = 'none'
  } else {
    full.style.display = 'none'
    thumb.style.display = ''
  }
  const label = root.querySelector('.ox-enlarge-label')
  if (label) {
    label.textContent = currentlyHidden ? 'click to reduce image size' : 'enlarge image'
  }
  return true
}

function handleMeaningFrameClick(event) {
  const target = event.target
  if (!target || typeof target.closest !== 'function') return

  const actionEl = target.closest('[data-wikit-action]')
  const action = actionEl?.getAttribute('data-wikit-action') || ''
  if (action === 'toggle_active' || target.closest('.box_title, pnc.heading')) {
    const title = action === 'toggle_active' ? actionEl : target.closest('.box_title, pnc.heading, [data-wikit-action="toggle_active"]')
    if (title && toggleMeaningUnbox(title)) {
      event.preventDefault()
      event.stopPropagation()
      return
    }
  }
  if (action === 'toggle_enlarger' || target.closest('#ox-enlarge')) {
    const el = actionEl || target.closest('#ox-enlarge')
    if (el && toggleMeaningEnlarger(el)) {
      event.preventDefault()
      event.stopPropagation()
      return
    }
  }

  const anchor = target.closest('a')
  if (!anchor) return

  const nav = anchor.getAttribute('data-wikit-nav')
  const href = nav || anchor.getAttribute('href') || ''

  if (nav || /^(?:entry|sound|snd):/i.test(href)) {
    event.preventDefault()
    event.stopPropagation()
    handleDictionaryNavMessage(nav || href).catch(() => {})
    return
  }

  if (href.charAt(0) === '#') {
    event.preventDefault()
    event.stopPropagation()
    const id = decodeHrefPart(href.slice(1))
    if (!id) return
    try {
      const doc = meaningFrame.contentDocument
      if (!doc) return
      const el = doc.getElementById(id) || doc.querySelector(`[name="${CSS.escape(id)}"]`)
      if (el && typeof el.scrollIntoView === 'function') {
        el.scrollIntoView({ block: 'start' })
      }
    } catch (_error) {
      // ignore
    }
  }
}

function bindMeaningFrameNavigation() {
  try {
    const doc = meaningFrame.contentDocument
    if (!doc || !doc.documentElement) return
    if (doc.documentElement.dataset.wikitNavBound === '1') return
    doc.documentElement.dataset.wikitNavBound = '1'
    doc.addEventListener('click', handleMeaningFrameClick, true)
    doc.addEventListener('auxclick', handleMeaningFrameClick, true)
  } catch (_error) {
    // ignore
  }
}

function renderMeaning(word, response, options = {}) {
  const html = options.html || (response && response.words && response.words[word])
  if (!html) {
    return
  }

  pendingMeaningHash = options.hash || ''

  // Base chrome first; dictionary CSS after so it can override.
  // Do not inject dictionary <script> into srcdoc: CSP blocks inline scripts.
  const chromeStyle = '<style>html{background:#fff;}body{margin:0;padding:16px 18px 32px;color:#1f2328;font-family:-apple-system,BlinkMacSystemFont,"Segoe UI","Helvetica Neue",Arial,sans-serif;line-height:1.58;}body>*:first-child{margin-top:0;}a{color:#0969da;cursor:pointer;}</style>'
  const bodyHtml = sanitizeMeaningHtml(html)
  const content = `<!DOCTYPE html><html><head><meta charset="UTF-8">${chromeStyle}${response.style || ''}</head><body>${bodyHtml}</body></html>`
  hide(meaningPlaceholder)
  hide(candidateList)
  hide(noCandidate)
  hide(previewFrame)
  show(meaningFrame)
  meaningFrame.onload = () => {
    bindMeaningFrameNavigation()
    resolveMeaningMediaResources().catch(() => {})
    if (pendingMeaningHash) {
      scrollMeaningToHash(pendingMeaningHash)
      pendingMeaningHash = ''
    }
  }
  meaningFrame.srcdoc = content
}

function isExternalOrDataUrl(url) {
  return /^(?:https?:|data:|blob:|file:|#|\/\/)/i.test(String(url || '').trim())
}

async function resolveMeaningMediaResources() {
  const dictid = dictSelect && dictSelect.value
  if (!dictid || !window.wikit.lookupResource) return
  let doc
  try {
    doc = meaningFrame.contentDocument
  } catch (_error) {
    return
  }
  if (!doc) return

  const nodes = Array.from(doc.querySelectorAll('img[src], source[src], audio[src], video[src]'))
  await Promise.all(nodes.map(async (el) => {
    const src = el.getAttribute('src') || ''
    if (!src || isExternalOrDataUrl(src)) return
    const key = src.replace(/^file:\/*/i, '').trim()
    if (!key) return
    try {
      const res = await window.wikit.lookupResource(dictid, key)
      if (res && res.found && res.url) {
        el.setAttribute('src', res.url)
      }
    } catch (_error) {
      // ignore missing resources
    }
  }))
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
    dictMenuBtn
  ]) {
    if (button) {
      button.disabled = disabled
    }
  }
  if (!disabled) {
    updateDictMenuState()
  }
}

function updateDictMenuState() {
  const hasDict = !!(dictSelect && dictSelect.value && dictSelect.options.length)
  if (toolbarEditDictBtn) toolbarEditDictBtn.disabled = !hasDict
  if (toolbarRemoveDictBtn) toolbarRemoveDictBtn.disabled = !hasDict
}

function resetLookupSurface() {
  currentResponse = null
  latestLookup = null
  pendingMeaningHash = ''
  if (searchInput) searchInput.value = ''
  hide(suggestionPopup)
  clearLookupHistory()
  showPlaceholder('Type a word to look up ...')
}

async function playDictionarySound(resource) {
  const name = String(resource || '').trim()
  if (!name) return
  const dictid = dictSelect && dictSelect.value
  if (!dictid || !window.wikit.lookupResource) {
    reportRendererLog('renderer.dict-sound', 'warn', 'sound lookup unavailable', { name })
    return
  }
  try {
    const res = await window.wikit.lookupResource(dictid, name)
    if (!res || !res.found || !res.url) {
      reportRendererLog('renderer.dict-sound', 'warn', 'sound resource not found', { name })
      return
    }
    const audio = new Audio(res.url)
    await audio.play()
  } catch (error) {
    reportRendererLog('renderer.dict-sound', 'error', 'failed to play sound', {
      name,
      error: String(error && error.message ? error.message : error),
    })
  }
}

async function handleDictionaryNavMessage(href) {
  const raw = String(href || '').trim()
  if (/^entry:/i.test(raw)) {
    const { word, hash } = parseEntryHref(raw)
    if (!word) return
    await navigateToWord(word, { pushHistory: true, hash, forceOpen: true })
    return
  }
  if (/^(?:sound|snd):/i.test(raw)) {
    await playDictionarySound(parseSoundHref(raw))
  }
}

function closeAddDictMenu() {
  if (!dictMenu || !dictMenuBtn) return
  hide(dictMenu)
  dictMenuBtn.setAttribute('aria-expanded', 'false')
}

function toggleAddDictMenu() {
  if (!dictMenu || !dictMenuBtn) return
  const willOpen = dictMenu.classList.contains('is-hidden')
  if (willOpen) {
    updateDictMenuState()
    show(dictMenu)
    dictMenuBtn.setAttribute('aria-expanded', 'true')
  } else {
    closeAddDictMenu()
  }
}

let importStatusTimer = null

function applyImportStatus(element, text, type, baseClass) {
  element.textContent = text
  element.className = baseClass
  if (type) {
    element.classList.add(`is-${type}`)
  }
  show(element)
}

function showImportDictStatus(text, type) {
  if (importStatusTimer) {
    clearTimeout(importStatusTimer)
    importStatusTimer = null
  }

  if (isLookupPanelActive()) {
    applyImportStatus(toolbarImportStatus, text, type, 'dict-toolbar-status')
    hide(importDictStatus)
  } else {
    applyImportStatus(importDictStatus, text, type, 'dict-empty-status')
    hide(toolbarImportStatus)
  }

  if (type === 'success') {
    const delay = text.length > 40 ? 5000 : 2500
    importStatusTimer = setTimeout(() => {
      hideImportDictStatus()
      importStatusTimer = null
    }, delay)
  }
}

function hideImportDictStatus() {
  if (importStatusTimer) {
    clearTimeout(importStatusTimer)
    importStatusTimer = null
  }
  importDictStatus.textContent = ''
  importDictStatus.className = 'dict-empty-status is-hidden'
  toolbarImportStatus.textContent = ''
  toolbarImportStatus.className = 'dict-toolbar-status is-hidden'
  hideImportDictProgress()
}

function activeImportProgressEl() {
  return isLookupPanelActive() ? toolbarImportProgress : importDictProgress
}

function showImportDictProgress(pct) {
  const bar = activeImportProgressEl()
  if (!bar) return
  const value = Math.min(Math.max(Math.round(pct), 0), 100)
  bar.classList.remove('is-hidden')
  bar.setAttribute('aria-hidden', 'false')
  const fill = bar.querySelector('.dm-progress-fill')
  const text = bar.querySelector('.dm-progress-text')
  if (fill) fill.style.width = `${value}%`
  if (text) text.textContent = `${value}%`
}

function hideImportDictProgress() {
  for (const bar of [importDictProgress, toolbarImportProgress]) {
    if (!bar) continue
    bar.classList.add('is-hidden')
    bar.setAttribute('aria-hidden', 'true')
    const fill = bar.querySelector('.dm-progress-fill')
    const text = bar.querySelector('.dm-progress-text')
    if (fill) fill.style.width = '0%'
    if (text) text.textContent = '0%'
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
  hideImportDictProgress()
  showImportDictStatus(
    expectedSuffix === 'mdx' ? '正在转换 MDX 词典…' : '正在加载词典…',
    ''
  )
  if (expectedSuffix === 'mdx') {
    showImportDictProgress(0)
  }

  let unlistenProgress = null
  try {
    if (expectedSuffix === 'mdx') {
      unlistenProgress = window.wikit.onBuildProgress((pct) => {
        const value = Math.min(Math.max(Math.round((Number(pct) || 0) * 100), 0), 100)
        showImportDictProgress(value)
        showImportDictStatus(`正在转换 MDX 词典… ${value}%`, '')
      })
    }
    const dict = await window.wikit.loadLocalDict(filePath)
    await loadDictionaries()
    if (dict && dict.id) {
      dictSelect.value = dict.id
    }

    let missingStyle = false
    if (dict && dict.id) {
      try {
        const info = await window.wikit.getDictInfo(dict.id)
        missingStyle = !((info && info.style) || '').trim()
      } catch (_error) {
        missingStyle = false
      }
    }

    hideImportDictProgress()
    const dictName = dict && dict.name ? dict.name : '词典'
    if (missingStyle && expectedSuffix === 'mdx') {
      showImportDictStatus(
        `已导入：${dictName}（未找到样式，请将词条引用的 .css/.js 放到 MDX 同目录后重新导入）`,
        'success'
      )
    } else {
      showImportDictStatus(`已导入：${dictName}`, 'success')
    }
    searchInput.focus()
  } catch (error) {
    hideImportDictProgress()
    showImportDictStatus('导入失败：' + getErrorMessage(error), 'error')
  } finally {
    if (unlistenProgress) unlistenProgress()
    setImportControlsDisabled(false)
  }
}

async function removeCurrentDictionary() {
  closeAddDictMenu()
  const dictid = dictSelect.value
  if (!dictid) {
    showImportDictStatus('没有可删除的词典', 'error')
    return
  }
  const selected = dictSelect.options[dictSelect.selectedIndex]
  const dictName = selected ? selected.textContent : '当前词典'
  const confirmed = await dmConfirm(`确定从列表中删除「${dictName}」？\n（不会删除词典文件）`)
  if (!confirmed) return

  try {
    const removed = await window.wikit.removeLocalDict(dictid)
    const wasEditingSame =
      deCurrentDictId && (deCurrentDictId === dictid)

    resetLookupSurface()
    await loadDictionaries()
    await loadDeDictList()

    if (wasEditingSame) {
      deCurrentDictId = null
      deCurrentDictInfo = null
      deCurrentWord = null
      deCurrentDefinition = null
      deDictMeta = null
      deDictSelect.value = ''
      deDictSelect.dispatchEvent(new Event('change'))
    }

    if (removed) {
      if (dictSelect.options.length === 0) {
        showImportDictStatus(`已删除：${dictName}。请添加词典后开始查词。`, 'success')
      } else {
        showImportDictStatus(`已删除：${dictName}`, 'success')
        if (dictSelect.value) {
          lookupCurrentWord()
        }
      }
    } else {
      showImportDictStatus('未找到该词典配置，可能已删除', 'error')
    }
  } catch (error) {
    showImportDictStatus('删除失败：' + getErrorMessage(error), 'error')
  }
}

async function openEditCurrentDictionary() {
  closeAddDictMenu()
  const dictid = dictSelect.value
  if (!dictid) {
    showImportDictStatus('请先选择词典', 'error')
    return
  }

  const dictName = dictSelect.options[dictSelect.selectedIndex]?.textContent || dictid
  setActiveMode('tools')
  setActiveTool('dict-editor')
  await loadDeDictList()
  if (deDictSelect.value !== dictid) {
    addDeDictOption({ id: dictid, name: dictName })
    deDictSelect.value = dictid
  }
  deDictSelect.dispatchEvent(new Event('change'))
}

async function loadDictionaries() {
  try {
    const dictionaries = await window.wikit.getDictList()
    const previousId = dictSelect.value
    dictSelect.replaceChildren()

    for (const dictionary of dictionaries) {
      const option = document.createElement('option')
      option.value = dictionary.id
      option.textContent = dictionary.name
      dictSelect.appendChild(option)
    }

    if (dictionaries.length) {
      const stillThere = dictionaries.some((item) => item.id === previousId)
      dictSelect.value = stillThere ? previousId : dictionaries[0].id
      hide(noDictionary)
      show(wordContent)
      hideImportDictStatus()
      if (!searchInput.value.trim()) {
        showPlaceholder('Type a word to look up ...')
      }
    } else {
      hide(wordContent)
      show(noDictionary)
      resetLookupSurface()
    }
    updateDictMenuState()
  } catch (_error) {
    hide(wordContent)
    show(noDictionary)
    resetLookupSurface()
    updateDictMenuState()
  }
}

async function lookupCurrentWord(options = {}) {
  const dictid = dictSelect.value
  const preferredWord = String(options.preferredWord || searchInput.value || '').trim()
  const word = preferredWord.toLowerCase()
  const pushHistory = options.pushHistory !== false
  const hash = options.hash || ''
  const forceOpen = options.forceOpen === true

  if (!dictid || !word) {
    currentResponse = null
    showPlaceholder('Type a word to look up ...')
    return
  }

  const token = Symbol(word)
  latestLookup = token

  const fetchBodyHits = async (excludeWords = []) => {
    try {
      const searchHits = await window.wikit.searchDict(dictid, preferredWord)
      const exclude = new Set(
        (excludeWords || []).map((w) => String(w || '').toLowerCase()).filter(Boolean)
      )
      return (searchHits || []).filter((h) => {
        if (!h || h.kind !== 'body' || !h.word) return false
        return !exclude.has(String(h.word).toLowerCase())
      })
    } catch (_searchError) {
      return []
    }
  }

  try {
    const response = await window.wikit.lookup(dictid, word)

    if (latestLookup !== token) {
      return
    }

    const resolved = await resolveLookupEntry(dictid, preferredWord, response)
    if (latestLookup !== token) {
      return
    }

    currentResponse = resolved.response
    const words = sortHeadwordsByRelevance(
      Object.keys(resolved.response?.words || {}),
      preferredWord
    )

    // Clicking a result should open the entry, not re-show the fulltext list.
    if (forceOpen) {
      let openWord = resolved.resolved ? resolved.word : preferredWord
      let openHtml = resolved.resolved ? resolved.html : null
      if (!openHtml && words.length) {
        const exactKey = findExactWordKey(resolved.response?.words, preferredWord)
        openWord = exactKey || words[0]
        openHtml = resolved.response?.words?.[openWord] || null
      }
      if (openHtml) {
        if (searchInput.value.trim() !== openWord) {
          searchInput.value = openWord
        }
        if (pushHistory) {
          pushLookupHistory(openWord, dictid)
        }
        renderMeaning(openWord, resolved.response, { html: openHtml, hash })
        return
      }
      showPlaceholder('Word not found')
      return
    }

    // Always query fulltext so body hits are visible even when a headword also matches.
    const bodyHits = await fetchBodyHits(
      resolved.resolved ? [resolved.word, ...words] : words
    )
    if (latestLookup !== token) {
      return
    }

    if (resolved.resolved && resolved.html && bodyHits.length === 0) {
      if (searchInput.value.trim() !== resolved.word) {
        searchInput.value = resolved.word
      }
      if (pushHistory) {
        pushLookupHistory(resolved.word, dictid)
      }
      renderMeaning(resolved.word, resolved.response, { html: resolved.html, hash })
    } else if (resolved.resolved && resolved.html && bodyHits.length > 0) {
      // Exact / best headword first, then other headwords, then body hits.
      const headwords = sortHeadwordsByRelevance(
        [resolved.word, ...words.filter((w) => w !== resolved.word)],
        preferredWord
      )
      if (pushHistory) {
        pushLookupHistory(resolved.word, dictid)
      }
      renderCandidates(headwords, resolved.response, bodyHits, preferredWord)
    } else {
      renderCandidates(words, resolved.response, bodyHits, preferredWord)
    }
  } catch (_error) {
    currentResponse = null
    if (forceOpen) {
      showPlaceholder('Word not found')
      return
    }
    const bodyHits = await fetchBodyHits([])
    if (latestLookup !== token) {
      return
    }
    renderCandidates([], null, bodyHits, preferredWord)
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
}

function swapTranslationLanguages() {
  const previousSource = sourceLanguageSelect.value
  const previousTarget = targetLanguageSelect.value
  const nextSource = previousTarget
  const nextTarget = previousSource === 'auto'
    ? (previousTarget === 'en' ? 'zh' : 'en')
    : previousSource

  setCustomSelectValue(sourceLanguageSelect, nextSource)
  setCustomSelectValue(targetLanguageSelect, nextTarget)
  saveTranslationLanguages()
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

async function requestTranslation() {
  const text = translationInput.value.trim()
  if (!text) {
    latestTranslation = null
    renderTranslationPlaceholder('输入文本后按 Shift+Enter 翻译。')
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
  latestTranslation = null
  translateAction.disabled = false
  translateAction.textContent = '翻译'
  translationInput.value = ''
  renderTranslationPlaceholder('输入文本后按 Shift+Enter 翻译。')
  translationInput.focus()
}

function getPreviewBaseUrl() {
  if (!previewPort) {
    return null
  }
  return `http://127.0.0.1:${previewPort}`
}

function connectPreviewSocket() {
  if (previewSocket) {
    previewSocket.close()
  }

  if (!previewPort) {
    return
  }

  previewSocket = new WebSocket(`ws://127.0.0.1:${previewPort}/wss`)
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
  const previewBaseUrl = getPreviewBaseUrl()
  if (!previewBaseUrl) {
    return false
  }

  for (let i = 0; i < 30; i += 1) {
    try {
      const response = await fetch(previewBaseUrl)
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
    previewPort = await window.wikit.startPreviewServer(sourceDir)
  } else if (!previewPort) {
    previewPort = await window.wikit.startPreviewServer('')
  }

  if (previewPort) {
    started = await waitForPreviewServer()
  }

  setActiveMode('dictionary')

  if (started) {
    hide(meaningPlaceholder)
    hide(candidateList)
    hide(noCandidate)
    hide(meaningFrame)
    show(previewFrame)
    previewFrame.src = getPreviewBaseUrl()
  }
}

async function stopPreviewer() {
  await window.wikit.stopPreviewServer()
  if (previewSocket) {
    previewSocket.close()
    previewSocket = null
  }
  previewFrame.removeAttribute('src')
  previewPort = null
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
swapLanguagesAction.addEventListener('click', swapTranslationLanguages)
sourceLanguageSelect.addEventListener('change', handleTranslationLanguageChange)
targetLanguageSelect.addEventListener('change', handleTranslationLanguageChange)
translationInput.addEventListener('keydown', (event) => {
  if (event.key !== 'Enter') {
    return
  }
  if (event.metaKey || event.ctrlKey) {
    event.preventDefault()
    copyTranslationOutput().catch(() => {})
    return
  }
  if (event.shiftKey) {
    event.preventDefault()
    requestTranslation()
  }
})
document.addEventListener('keydown', (event) => {
  if (activeMode !== 'translate' || event.key !== 'Enter') {
    return
  }
  if (event.metaKey || event.ctrlKey || event.altKey || event.shiftKey) {
    return
  }
  if (event.target !== translationInput) {
    event.preventDefault()
    translationInput.focus()
  }
})
searchInput.addEventListener('input', scheduleLookup)
dictSelect.addEventListener('change', () => {
  updateDictMenuState()
  clearLookupHistory()
  lookupCurrentWord({ pushHistory: true })
})
if (lookupBackBtn) {
  lookupBackBtn.addEventListener('click', () => goLookupHistory(-1))
}
if (lookupForwardBtn) {
  lookupForwardBtn.addEventListener('click', () => goLookupHistory(1))
}
document.addEventListener('keydown', (event) => {
  if (activeMode !== 'dictionary' || !isLookupPanelActive()) return
  const isMeta = event.metaKey || event.ctrlKey
  if (isMeta && event.key === '[') {
    event.preventDefault()
    goLookupHistory(-1)
  } else if (isMeta && event.key === ']') {
    event.preventDefault()
    goLookupHistory(1)
  } else if (event.altKey && event.key === 'ArrowLeft') {
    event.preventDefault()
    goLookupHistory(-1)
  } else if (event.altKey && event.key === 'ArrowRight') {
    event.preventDefault()
    goLookupHistory(1)
  }
})
importWikitDictBtn.addEventListener('click', () => importDictionary('wikit'))
importMdxDictBtn.addEventListener('click', () => importDictionary('mdx'))
toolbarImportWikitBtn.addEventListener('click', () => importDictionary('wikit'))
toolbarImportMdxBtn.addEventListener('click', () => importDictionary('mdx'))
addDictMenuBtn.addEventListener('click', (event) => {
  event.stopPropagation()
  toggleAddDictMenu()
})
toolbarEditDictBtn.addEventListener('click', () => {
  openEditCurrentDictionary().catch(() => {})
})
toolbarRemoveDictBtn.addEventListener('click', () => {
  removeCurrentDictionary().catch(() => {})
})
openConfigDirBtn.addEventListener('click', () => {
  window.wikit.openConfigDir().catch(() => {})
})
document.addEventListener('click', (event) => {
  if (!dictMenu || dictMenu.classList.contains('is-hidden')) {
    return
  }
  if (event.target.closest('.dict-menu-control')) {
    return
  }
  closeAddDictMenu()
})
previewFrame.addEventListener('load', connectPreviewSocket)

window.wikit.onTranslationSettingsUpdated((settingsJson) => {
  updateActiveModelLabel(parseJson(settingsJson, null))
})
reportRendererLog('renderer.bridge', 'info', 'bridge ready', {
  hasWikit: typeof window.wikit !== 'undefined'
})
window.wikit.startPreviewer = startPreviewer
window.wikit.stopPreviewer = stopPreviewer
window.wikit.startStaticFileServer()
  .then((port) => {
    reportRendererLog('renderer.static-server', 'info', 'ready', { port })
  })
  .catch((error) => {
    reportRendererLog('renderer.static-server', 'error', 'failed', { error: String(error) })
  })

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
document.getElementById('importCssBtn').addEventListener('click', () => importDictAsset('css'))
document.getElementById('importJsBtn').addEventListener('click', () => importDictAsset('js'))
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

function getDictDirName() {
  if (!currentDictDir) return ''
  return currentDictDir.split(/[\\/]/).pop() || currentDictDir
}

function getDictTomlPath() {
  // LocalDictionary::create reads <stem>.toml next to entries.txt
  return `${currentDictDir}/entries.toml`
}

function getLegacyDictTomlPath() {
  const dirName = getDictDirName()
  return `${currentDictDir}/${dirName}.toml`
}

function resolveAssetRef(ref, fallbackName) {
  const value = (ref || '').trim()
  if (value.startsWith('@')) {
    return `${currentDictDir}/${value.slice(1)}`
  }
  if (!value) {
    return `${currentDictDir}/${fallbackName}`
  }
  return null
}

async function readDictMeta() {
  const dirName = getDictDirName()
  const data = {
    name: dirName,
    version: '1.0',
    authors: [''],
    distributors: [''],
    description: '',
    homepage: '',
    css: '',
    js: ''
  }
  const candidates = [getDictTomlPath(), getLegacyDictTomlPath()]
  for (const tomlPath of candidates) {
    try {
      const content = await window.wikit.readTextFile(tomlPath)
      if (!content) continue
      const parsed = parseToml(content)
      if (parsed.name) data.name = parsed.name
      if (parsed.version) data.version = parsed.version
      if (parsed.authors) data.authors = parsed.authors
      if (parsed.distributors) data.distributors = parsed.distributors
      if (parsed.description) data.description = parsed.description
      if (parsed.homepage) data.homepage = parsed.homepage
      if (parsed.css) data.css = parsed.css
      if (parsed.js) data.js = parsed.js
      break
    } catch (_e) {}
  }
  return data
}

async function loadDictAssets() {
  if (!currentDictDir) {
    cssEditor.textContent = ''
    jsEditor.textContent = ''
    dictMetaCssRef = ''
    dictMetaJsRef = ''
    return
  }

  const dirName = getDictDirName()
  const meta = await readDictMeta()
  dictMetaCssRef = meta.css || ''
  dictMetaJsRef = meta.js || ''

  let cssContent = ''
  let jsContent = ''

  const cssPath = resolveAssetRef(dictMetaCssRef, `${dirName}.css`)
  if (cssPath) {
    try {
      const content = await window.wikit.readTextFile(cssPath)
      if (content) cssContent = content
    } catch (_e) {}
  } else if (dictMetaCssRef && !dictMetaCssRef.trim().startsWith('@')) {
    cssContent = dictMetaCssRef
  }

  const jsPath = resolveAssetRef(dictMetaJsRef, `${dirName}.js`)
  if (jsPath) {
    try {
      const content = await window.wikit.readTextFile(jsPath)
      if (content) jsContent = content
    } catch (_e) {}
  } else if (dictMetaJsRef && !dictMetaJsRef.trim().startsWith('@')) {
    jsContent = dictMetaJsRef
  }

  cssEditor.textContent = cssContent
  jsEditor.textContent = jsContent
}

async function syncDictAssetsForBuild() {
  const dirName = getDictDirName()
  const cssContent = cssEditor.textContent || ''
  const jsContent = jsEditor.textContent || ''
  const cssFileName = `${dirName}.css`
  const jsFileName = `${dirName}.js`
  const cssPath = `${currentDictDir}/${cssFileName}`
  const jsPath = `${currentDictDir}/${jsFileName}`

  if (cssContent.trim()) {
    const ok = await window.wikit.writeTextFile(cssPath, cssContent)
    if (!ok) throw new Error('写入 CSS 文件失败')
    dictMetaCssRef = `@${cssFileName}`
  } else {
    dictMetaCssRef = ''
  }

  if (jsContent.trim()) {
    const ok = await window.wikit.writeTextFile(jsPath, jsContent)
    if (!ok) throw new Error('写入 JS 文件失败')
    dictMetaJsRef = `@${jsFileName}`
  } else {
    dictMetaJsRef = ''
  }

  const meta = await readDictMeta()
  meta.css = dictMetaCssRef
  meta.js = dictMetaJsRef
  if (!meta.authors.length) meta.authors = ['']
  if (!meta.distributors.length) meta.distributors = ['']
  const ok = await window.wikit.writeTextFile(getDictTomlPath(), buildToml(meta))
  if (!ok) throw new Error('写入词典元信息失败')
}

async function importDictAsset(kind) {
  if (!currentDictDir) {
    showSaveStatus('请先选择词典目录', 'error')
    return
  }
  const filters = kind === 'css'
    ? [
        { name: 'CSS', extensions: ['css'] },
        { name: 'All Files', extensions: ['*'] }
      ]
    : [
        { name: 'JavaScript', extensions: ['js'] },
        { name: 'All Files', extensions: ['*'] }
      ]
  const filePath = await window.wikit.openFile(filters)
  if (!filePath) return
  try {
    const content = await window.wikit.readTextFile(filePath)
    if (content == null) {
      showSaveStatus('读取文件失败', 'error')
      return
    }
    if (kind === 'css') {
      cssEditor.textContent = content
    } else {
      jsEditor.textContent = content
    }
    markModified()
    schedulePreview()
    const name = filePath.split(/[\\/]/).pop() || filePath
    showSaveStatus(`已导入 ${name}，构建词典时会打入 .wikit`, 'success')
  } catch (_e) {
    showSaveStatus('读取文件失败', 'error')
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
  const data = await readDictMeta()
  dictMetaCssRef = data.css || ''
  dictMetaJsRef = data.js || ''
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
  const existing = await readDictMeta()
  const data = {
    name: document.getElementById('metaName').value.trim(),
    version: document.getElementById('metaVersion').value.trim() || '1.0',
    authors: document.getElementById('metaAuthors').value.split(',').map(s => s.trim()).filter(Boolean),
    distributors: document.getElementById('metaDistributors').value.split(',').map(s => s.trim()).filter(Boolean),
    description: document.getElementById('metaDescription').value.trim(),
    homepage: document.getElementById('metaHomepage').value.trim(),
    css: existing.css || dictMetaCssRef || '',
    js: existing.js || dictMetaJsRef || '',
  }
  if (!data.authors.length) data.authors = ['']
  if (!data.distributors.length) data.distributors = ['']
  dictMetaCssRef = data.css
  dictMetaJsRef = data.js
  const ok = await window.wikit.writeTextFile(getDictTomlPath(), buildToml(data))
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
  await loadDictAssets()
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
    previewIframe.srcdoc = ''
    return
  }
  contentEditor.textContent = entry.content || ''
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
  const dirName = getDictDirName()
  const txtPath = `${currentDictDir}/entries.txt`
  const outPath = `${currentDictDir}/${dirName}.wikit`
  let txtExists
  try { await window.wikit.readTextFile(txtPath); txtExists = true } catch (_e) { txtExists = false }
  if (!txtExists) {
    showSaveStatus('未找到 entries.txt', 'error')
    return
  }

  const btn = document.getElementById('buildDictBtn')
  btn.disabled = true
  btn.textContent = '构建中...'
  showBuildProgress(0)
  let unlistenProgress = null
  try {
    await syncDictAssetsForBuild()
    const hasCss = !!(cssEditor.textContent || '').trim()
    const hasJs = !!(jsEditor.textContent || '').trim()
    unlistenProgress = window.wikit.onBuildProgress((pct) => {
      showBuildProgress(pct * 100)
    })
    const result = await window.wikit.buildWikit(txtPath, outPath)
    if (result.ok) {
      showBuildProgress(100)
      const packed = []
      if (hasCss) packed.push('CSS')
      if (hasJs) packed.push('JS')
      const suffix = packed.length ? `（已包含 ${packed.join('/')}）` : ''
      showSaveStatus('已构建: ' + dirName + '.wikit' + suffix, 'success')
    } else {
      showSaveStatus(result.error || '构建失败', 'error')
    }
  } catch (error) {
    showSaveStatus('构建失败: ' + getErrorMessage(error), 'error')
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
reportRendererLog('renderer.bootstrap', 'info', 'initialization completed', { activeMode })

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
    const entries = sortSearchEntriesByRelevance(
      await window.wikit.searchDict(deCurrentDictId, word),
      word
    )
    deEntryList.replaceChildren()
    if (entries.length === 0) {
      deEntryList.innerHTML = '<div class="de-empty">未找到词条</div>'
      return
    }
    for (const entry of entries) {
      const item = document.createElement('div')
      item.className = 'de-entry-item'
      item.dataset.word = entry.word
      const kindLabel = entry.kind === 'body' ? '（正文）' : ''
      item.textContent = `${entry.word}${kindLabel}`
      if (entry.snippet) {
        item.title = entry.snippet
      }
      item.addEventListener('click', async (event) => {
        event.preventDefault()
        event.stopPropagation()
        let definition = entry.definition
        if (!definition) {
          try {
            const resp = await window.wikit.lookup(deCurrentDictId, entry.word)
            definition = resp?.words?.[entry.word] || Object.values(resp?.words || {})[0] || ''
          } catch (_lookupError) {
            definition = ''
          }
        }
        deSelectEntry(entry.word, definition)
      })
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
    el.classList.toggle('is-selected', el.dataset.word === word)
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
  let doc = `<!DOCTYPE html><html><head><meta charset="UTF-8">${chromeStyle}${css ? '<style>' + css.replace(/<\//g, '<\\/') + '</style>' : ''}</head><body>${html}`
  if (js) doc += `<script>${js.replace(/<\//g, '<\\/')}<\/script>`
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

deEditMetaBtn.addEventListener('click', () => {
  if (!deCurrentDictInfo) {
    deShowSaveStatus('请先选择词典', 'error')
    return
  }
  document.getElementById('deMetaName').value = deCurrentDictInfo.name || ''
  document.getElementById('deMetaDesc').value = deCurrentDictInfo.desc || ''
  document.getElementById('deMetaCss').value = deCssEditor.textContent || deCurrentDictInfo.style || ''
  document.getElementById('deMetaJs').value = deJsEditor.textContent || deCurrentDictInfo.script || ''
  document.getElementById('deMetaOverlay').classList.remove('is-hidden')
  document.getElementById('deMetaName').focus()
})

function closeDeMetaOverlay() {
  document.getElementById('deMetaOverlay').classList.add('is-hidden')
}

document.getElementById('deMetaClose').addEventListener('click', closeDeMetaOverlay)
document.getElementById('deMetaCloseTop').addEventListener('click', closeDeMetaOverlay)

async function refreshDictionaryAfterUpdate(dict) {
  if (!dict || !dict.id) return

  deCurrentDictId = dict.id
  addDeDictOption(dict)
  await loadDeDictList()
  deDictSelect.value = dict.id

  deCurrentDictInfo = await window.wikit.getDictInfo(dict.id)
  deDictMeta = {
    css: deCurrentDictInfo.style || '',
    js: deCurrentDictInfo.script || ''
  }
  deCssEditor.textContent = deDictMeta.css
  deJsEditor.textContent = deDictMeta.js
  deRenderViewer()

  const previousLookupId = dictSelect.value
  await loadDictionaries()
  const stillAvailable = Array.from(dictSelect.options).some((opt) => opt.value === dict.id)
  if (stillAvailable) {
    dictSelect.value = dict.id
  } else if (previousLookupId) {
    dictSelect.value = previousLookupId
  }
  if (dictSelect.value) {
    lookupCurrentWord()
  }
}

document.getElementById('deMetaSave').addEventListener('click', async () => {
  if (!deCurrentDictId) {
    deShowSaveStatus('请先选择词典', 'error')
    return
  }
  const name = document.getElementById('deMetaName').value.trim()
  const desc = document.getElementById('deMetaDesc').value.trim()
  const style = document.getElementById('deMetaCss').value || ''
  const script = document.getElementById('deMetaJs').value || ''
  if (!name) {
    deShowSaveStatus('词典名称不能为空', 'error')
    return
  }

  const saveBtn = document.getElementById('deMetaSave')
  saveBtn.disabled = true
  deShowSaveStatus('正在保存元信息…', '')
  try {
    const dict = await window.wikit.republishLocalDict(
      deCurrentDictId,
      style,
      script,
      deCurrentDictId,
      name,
      desc
    )
    closeDeMetaOverlay()
    await refreshDictionaryAfterUpdate(dict)
    deShowSaveStatus('元信息已保存，词典已重新加载', 'success')
  } catch (error) {
    deShowSaveStatus('保存失败: ' + getErrorMessage(error), 'error')
  } finally {
    saveBtn.disabled = false
  }
})

deSaveBtn.addEventListener('click', () => {
  deShowSaveStatus('wikit/mdx 词典为只读格式，词条修改仅在本会话生效；样式请用「打包样式并导出」', 'error')
})

async function importDeAsset(kind) {
  if (!deCurrentDictId) {
    deShowSaveStatus('请先选择词典', 'error')
    return
  }
  const filters = kind === 'css'
    ? [
        { name: 'CSS', extensions: ['css'] },
        { name: 'All Files', extensions: ['*'] }
      ]
    : [
        { name: 'JavaScript', extensions: ['js'] },
        { name: 'All Files', extensions: ['*'] }
      ]
  const filePath = await window.wikit.openFile(filters)
  if (!filePath) return
  try {
    const content = await window.wikit.readTextFile(filePath)
    if (content == null) {
      deShowSaveStatus('读取文件失败', 'error')
      return
    }
    if (kind === 'css') {
      deCssEditor.textContent = content
      if (deDictMeta) deDictMeta.css = content
    } else {
      deJsEditor.textContent = content
      if (deDictMeta) deDictMeta.js = content
    }
    deRenderViewer()
    const name = filePath.split(/[\\/]/).pop() || filePath
    deShowSaveStatus(`已导入 ${name}，点击「打包样式并导出」写入 .wikit`, 'success')
  } catch (_e) {
    deShowSaveStatus('读取文件失败', 'error')
  }
}

document.getElementById('deImportCssBtn').addEventListener('click', () => importDeAsset('css'))
document.getElementById('deImportJsBtn').addEventListener('click', () => importDeAsset('js'))

deCssEditor.addEventListener('input', () => {
  if (deDictMeta) deDictMeta.css = deCssEditor.textContent || ''
  deRenderViewer()
})
deJsEditor.addEventListener('input', () => {
  if (deDictMeta) deDictMeta.js = deJsEditor.textContent || ''
  deRenderViewer()
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
  const style = deCssEditor.textContent || ''
  const script = deJsEditor.textContent || ''
  deExportBtn.disabled = true
  deShowSaveStatus('正在打包样式…', '')
  try {
    const dict = await window.wikit.republishLocalDict(
      deCurrentDictId,
      style,
      script,
      outPath,
      deCurrentDictInfo ? deCurrentDictInfo.name : null,
      deCurrentDictInfo ? deCurrentDictInfo.desc : null
    )
    await refreshDictionaryAfterUpdate(dict)
    const packed = []
    if (style.trim()) packed.push('CSS')
    if (script.trim()) packed.push('JS')
    const suffix = packed.length ? `（已包含 ${packed.join('/')}）` : ''
    deShowSaveStatus('已导出并重新加载: ' + dictName + '.wikit' + suffix, 'success')
  } catch (error) {
    deShowSaveStatus('导出失败: ' + getErrorMessage(error), 'error')
  } finally {
    deExportBtn.disabled = false
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
