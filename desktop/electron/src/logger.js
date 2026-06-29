const fs = require('node:fs')
const path = require('node:path')

let logFilePath = null

function formatError(error) {
  if (!error) {
    return ''
  }
  if (error instanceof Error) {
    return error.stack || error.message
  }
  return String(error)
}

function formatMeta(meta) {
  if (meta === undefined || meta === null) {
    return ''
  }
  try {
    return ` ${JSON.stringify(meta)}`
  } catch (_error) {
    return ` ${String(meta)}`
  }
}

function writeLine(level, scope, message, meta) {
  const line = `[${new Date().toISOString()}] [${level}] [${scope}] ${message}${formatMeta(meta)}\n`
  process.stdout.write(line)
  if (!logFilePath) {
    return
  }
  try {
    fs.mkdirSync(path.dirname(logFilePath), { recursive: true })
    fs.appendFileSync(logFilePath, line)
  } catch (error) {
    process.stderr.write(`[logger] failed to write log file: ${formatError(error)}\n`)
  }
}

function initLogger(userDataPath) {
  logFilePath = path.join(userDataPath, 'logs', 'main.log')
  writeLine('INFO', 'logger', 'log file initialized', { logFilePath })
  return logFilePath
}

function createLogger(scope) {
  return {
    info(message, meta) {
      writeLine('INFO', scope, message, meta)
    },
    warn(message, meta) {
      writeLine('WARN', scope, message, meta)
    },
    error(message, meta) {
      writeLine('ERROR', scope, message, meta)
    }
  }
}

module.exports = {
  initLogger,
  createLogger,
  getLogFilePath: () => logFilePath
}
