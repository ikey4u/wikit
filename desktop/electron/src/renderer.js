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

function show(element) {
  element.classList.remove('is-hidden')
}

function hide(element) {
  element.classList.add('is-hidden')
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
  if (lookupTimer) {
    clearTimeout(lookupTimer)
  }
  lookupTimer = setTimeout(lookupCurrentWord, 50)
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

searchInput.addEventListener('input', scheduleLookup)
dictSelect.addEventListener('change', lookupCurrentWord)
previewFrame.addEventListener('load', connectPreviewSocket)

window.wikit.startPreviewer = startPreviewer
window.wikit.stopPreviewer = stopPreviewer
window.wikit.startStaticFileServer().catch(() => {})
loadDictionaries()
