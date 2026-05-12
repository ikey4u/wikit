const providerSelect = document.getElementById('providerSelect')
const endpointInput = document.getElementById('endpointInput')
const modelInput = document.getElementById('modelInput')
const apiKeyInput = document.getElementById('apiKeyInput')
const temperatureInput = document.getElementById('temperatureInput')
const timeoutInput = document.getElementById('timeoutInput')
const testConnectionButton = document.getElementById('testConnectionButton')
const saveSettingsButton = document.getElementById('saveSettingsButton')
const settingsMessage = document.getElementById('settingsMessage')

const providerDefaults = {
  'openai-compatible': ['https://api.openai.com/v1', 'gpt-4o-mini'],
  ollama: ['http://127.0.0.1:11434', 'qwen2.5:7b'],
  'llama-cpp': ['http://127.0.0.1:8080/v1', 'local-model'],
  vllm: ['http://127.0.0.1:8000/v1', 'Qwen2.5-7B-Instruct']
}

function parseJson(value, fallback) {
  try {
    return JSON.parse(value)
  } catch (_error) {
    return fallback
  }
}

function collectSettings() {
  return {
    provider: providerSelect.value,
    endpoint: endpointInput.value.trim(),
    model: modelInput.value.trim(),
    api_key: apiKeyInput.value.trim(),
    temperature: Number(temperatureInput.value || 0.2),
    timeout: Number(timeoutInput.value || 60)
  }
}

function applySettings(settings) {
  providerSelect.value = settings.provider || 'ollama'
  endpointInput.value = settings.endpoint || providerDefaults[providerSelect.value][0]
  modelInput.value = settings.model || providerDefaults[providerSelect.value][1]
  apiKeyInput.value = settings.api_key || ''
  temperatureInput.value = String(settings.temperature ?? 0.2)
  timeoutInput.value = String(settings.timeout ?? 60)
}

async function loadSettings() {
  try {
    const settingsJson = await window.wikit.getTranslationSettings()
    applySettings(parseJson(settingsJson, {}))
    settingsMessage.textContent = '翻译设置已加载。'
  } catch (error) {
    settingsMessage.textContent = String(error && error.message ? error.message : error)
  }
}

providerSelect.addEventListener('change', () => {
  const defaults = providerDefaults[providerSelect.value]
  endpointInput.value = defaults[0]
  modelInput.value = defaults[1]
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
    settingsMessage.textContent = '设置已保存。'
  } catch (error) {
    settingsMessage.textContent = String(error && error.message ? error.message : error)
  } finally {
    saveSettingsButton.disabled = false
  }
})

loadSettings()
