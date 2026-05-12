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
  updateModelControl(providerSelect.value, defaults[1])
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
