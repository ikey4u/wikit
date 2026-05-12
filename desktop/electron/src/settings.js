const modelTab = document.getElementById('modelTab')
const shortcutTab = document.getElementById('shortcutTab')
const modelPanel = document.getElementById('modelPanel')
const shortcutPanel = document.getElementById('shortcutPanel')
const providerSelect = document.getElementById('providerSelect')
const endpointInput = document.getElementById('endpointInput')
const modelInput = document.getElementById('modelInput')
const modelSelect = document.getElementById('modelSelect')
const apiKeyInput = document.getElementById('apiKeyInput')
const temperatureInput = document.getElementById('temperatureInput')
const timeoutInput = document.getElementById('timeoutInput')
const testConnectionButton = document.getElementById('testConnectionButton')
const saveSettingsButton = document.getElementById('saveSettingsButton')
const settingsMessage = document.getElementById('settingsMessage')
const toggleShortcutInput = document.getElementById('toggleShortcutInput')
const recordShortcutButton = document.getElementById('recordShortcutButton')
const saveShortcutButton = document.getElementById('saveShortcutButton')
const shortcutMessage = document.getElementById('shortcutMessage')

const providerDefaults = {
  'openai-compatible': ['https://api.openai.com/v1', 'gpt-4o-mini'],
  deepseek: ['https://api.deepseek.com', 'deepseek-v4-pro'],
  ollama: ['http://127.0.0.1:11434', 'qwen2.5:7b'],
  'llama-cpp': ['http://127.0.0.1:8080/v1', 'local-model'],
  vllm: ['http://127.0.0.1:8000/v1', 'Qwen2.5-7B-Instruct']
}

const providerModels = {
  deepseek: ['deepseek-v4-flash', 'deepseek-v4-pro']
}

let recordingShortcut = false
let pressedShortcutModifiers = {
  meta: false,
  ctrl: false,
  alt: false,
  shift: false
}

function parseJson(value, fallback) {
  try {
    return JSON.parse(value)
  } catch (_error) {
    return fallback
  }
}

function setActiveTab(tab) {
  const modelActive = tab === 'model'
  modelTab.classList.toggle('is-active', modelActive)
  shortcutTab.classList.toggle('is-active', !modelActive)
  modelPanel.classList.toggle('is-hidden', !modelActive)
  shortcutPanel.classList.toggle('is-hidden', modelActive)
}

function collectSettings() {
  return {
    provider: providerSelect.value,
    endpoint: endpointInput.value.trim(),
    model: modelSelect.hidden ? modelInput.value.trim() : modelSelect.value,
    api_key: apiKeyInput.value.trim(),
    temperature: Number(temperatureInput.value || 0.2),
    timeout: Number(timeoutInput.value || 60)
  }
}

function updateModelControl(provider, selectedModel) {
  const models = providerModels[provider] || []
  if (!models.length) {
    modelSelect.hidden = true
    modelInput.hidden = false
    modelInput.value = selectedModel || providerDefaults[provider][1]
    return
  }

  modelInput.hidden = true
  modelSelect.hidden = false
  modelSelect.replaceChildren()
  for (const model of models) {
    const option = document.createElement('option')
    option.value = model
    option.textContent = model
    modelSelect.appendChild(option)
  }
  modelSelect.value = models.includes(selectedModel) ? selectedModel : providerDefaults[provider][1]
}

function applySettings(settings) {
  const provider = providerDefaults[settings.provider] ? settings.provider : 'ollama'
  providerSelect.value = provider
  endpointInput.value = settings.endpoint || providerDefaults[provider][0]
  updateModelControl(provider, settings.model || providerDefaults[provider][1])
  apiKeyInput.value = settings.api_key || ''
  temperatureInput.value = String(settings.temperature ?? 0.2)
  timeoutInput.value = String(settings.timeout ?? 60)
}

function normalizeShortcutKey(event) {
  if (event.key === 'Dead' || event.code === 'IntlBackslash') {
    return ''
  }
  if (event.code === 'Space') {
    return 'Space'
  }
  if (event.code.startsWith('Key')) {
    return event.code.slice(3)
  }
  if (event.code.startsWith('Digit')) {
    return event.code.slice(5)
  }
  const aliases = {
    ArrowUp: 'Up',
    ArrowDown: 'Down',
    ArrowLeft: 'Left',
    ArrowRight: 'Right',
    Escape: 'Esc'
  }
  return aliases[event.key] || event.key
}

function acceleratorFromEvent(event) {
  pressedShortcutModifiers = {
    meta: pressedShortcutModifiers.meta || event.metaKey || event.key === 'Meta',
    ctrl: pressedShortcutModifiers.ctrl || event.ctrlKey || event.key === 'Control',
    alt: pressedShortcutModifiers.alt || event.altKey || event.key === 'Alt',
    shift: pressedShortcutModifiers.shift || event.shiftKey || event.key === 'Shift'
  }

  const parts = []
  if (pressedShortcutModifiers.meta) {
    parts.push('Command')
  }
  if (pressedShortcutModifiers.ctrl) {
    parts.push('Control')
  }
  if (pressedShortcutModifiers.alt) {
    parts.push('Alt')
  }
  if (pressedShortcutModifiers.shift) {
    parts.push('Shift')
  }

  const key = normalizeShortcutKey(event)
  if (key && !['Meta', 'Control', 'Alt', 'Shift'].includes(key)) {
    parts.push(key.length === 1 ? key.toUpperCase() : key)
  }
  return parts.join('+')
}

function applyAppSettings(settings) {
  const accelerator = settings.shortcuts?.toggleWindow || 'Alt+1'
  toggleShortcutInput.value = accelerator
  const message = settings.shortcutState?.message || '快捷键设置已加载。'
  shortcutMessage.textContent = message
}

async function loadSettings() {
  try {
    const settingsJson = await window.wikit.getTranslationSettings()
    applySettings(parseJson(settingsJson, {}))
    settingsMessage.textContent = '模型设置已加载。'
  } catch (error) {
    settingsMessage.textContent = String(error && error.message ? error.message : error)
  }
}

async function loadAppSettings() {
  try {
    const settingsJson = await window.wikit.getAppSettings()
    applyAppSettings(parseJson(settingsJson, {}))
  } catch (error) {
    shortcutMessage.textContent = String(error && error.message ? error.message : error)
  }
}

providerSelect.addEventListener('change', () => {
  const defaults = providerDefaults[providerSelect.value]
  endpointInput.value = defaults[0]
  updateModelControl(providerSelect.value, defaults[1])
})

modelTab.addEventListener('click', () => setActiveTab('model'))
shortcutTab.addEventListener('click', () => setActiveTab('shortcut'))

recordShortcutButton.addEventListener('click', () => {
  recordingShortcut = true
  pressedShortcutModifiers = {
    meta: false,
    ctrl: false,
    alt: false,
    shift: false
  }
  toggleShortcutInput.focus()
  toggleShortcutInput.select()
  recordShortcutButton.textContent = '按键中...'
  shortcutMessage.textContent = '请按住修饰键，然后按一个普通按键。'
})

toggleShortcutInput.addEventListener('keydown', (event) => {
  if (!recordingShortcut) {
    return
  }
  event.preventDefault()
  event.stopPropagation()
  const accelerator = acceleratorFromEvent(event)
  const key = normalizeShortcutKey(event)
  const isModifierOnly = !key || ['Meta', 'Control', 'Alt', 'Shift'].includes(key)
  if (isModifierOnly) {
    shortcutMessage.textContent = '继续按一个普通按键完成录制。'
    return
  }
  if (!accelerator || !accelerator.includes('+')) {
    shortcutMessage.textContent = '快捷键需要至少一个修饰键和一个普通按键。'
    return
  }
  toggleShortcutInput.value = accelerator
  recordingShortcut = false
  recordShortcutButton.textContent = '录制'
  shortcutMessage.textContent = '快捷键已录制，点击保存后生效。'
})

toggleShortcutInput.addEventListener('blur', () => {
  if (recordingShortcut) {
    recordingShortcut = false
    pressedShortcutModifiers = {
      meta: false,
      ctrl: false,
      alt: false,
      shift: false
    }
    recordShortcutButton.textContent = '录制'
  }
})

testConnectionButton.addEventListener('click', async () => {
  testConnectionButton.disabled = true
  settingsMessage.textContent = '正在测试连接...'
  try {
    const responseJson = await window.wikit.testTranslationConnection(JSON.stringify(collectSettings()))
    const response = parseJson(responseJson, null)
    settingsMessage.textContent = response ? response.message : '连接测试没有返回结果。'
  } catch (error) {
    settingsMessage.textContent = String(error && error.message ? error.message : error)
  } finally {
    testConnectionButton.disabled = false
  }
})

saveSettingsButton.addEventListener('click', async () => {
  saveSettingsButton.disabled = true
  settingsMessage.textContent = '正在保存设置...'
  try {
    const settingsJson = await window.wikit.saveTranslationSettings(JSON.stringify(collectSettings()))
    applySettings(parseJson(settingsJson, collectSettings()))
    settingsMessage.textContent = '模型设置已保存。'
  } catch (error) {
    settingsMessage.textContent = String(error && error.message ? error.message : error)
  } finally {
    saveSettingsButton.disabled = false
  }
})

saveShortcutButton.addEventListener('click', async () => {
  saveShortcutButton.disabled = true
  shortcutMessage.textContent = '正在保存快捷键...'
  try {
    const settingsJson = await window.wikit.saveAppSettings(JSON.stringify({
      shortcuts: {
        toggleWindow: toggleShortcutInput.value.trim()
      }
    }))
    applyAppSettings(parseJson(settingsJson, {}))
  } catch (error) {
    shortcutMessage.textContent = String(error && error.message ? error.message : error)
  } finally {
    saveShortcutButton.disabled = false
  }
})

loadSettings()
loadAppSettings()
setActiveTab('model')
