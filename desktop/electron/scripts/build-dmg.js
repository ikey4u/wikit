const { spawnSync } = require('node:child_process')
const { existsSync, rmSync } = require('node:fs')
const { join, resolve } = require('node:path')

const projectDir = resolve(__dirname, '..')
const distDir = join(projectDir, 'dist')

function run(command, args, cwd) {
  const result = spawnSync(command, args, {
    cwd,
    stdio: 'inherit',
    shell: process.platform === 'win32'
  })

  if (result.status !== 0) {
    process.exit(result.status || 1)
  }
}

if (process.platform !== 'darwin') {
  console.error('DMG packaging is only supported on macOS.')
  process.exit(1)
}

if (existsSync(distDir)) {
  rmSync(distDir, { recursive: true, force: true })
}

run('npm', ['run', 'build:native'], projectDir)
run('npx', ['electron-builder', '--mac', 'dmg', '--publish', 'never'], projectDir)
