import fs from 'node:fs'
import net from 'node:net'
import path from 'node:path'
import { spawn, type ChildProcess } from 'node:child_process'

const root = path.resolve(import.meta.dirname, '..')
const workspace = path.join(root, 'target', 'e2e-workspace')
const application = path.join(root, 'target', 'release', 'neopad-app.exe')
const edgeDriver = path.join(root, 'target', 'tools', 'msedgedriver.exe')
process.env.NEOPAD_WORKSPACE = workspace
process.env.NEOPAD_E2E = '1'
let tauriDriver: ChildProcess | null = null

function waitForPort(port: number, timeoutMs: number) {
  const startedAt = Date.now()
  return new Promise<void>((resolve, reject) => {
    const attempt = () => {
      const socket = net.createConnection({ host: '127.0.0.1', port })
      socket.once('connect', () => {
        socket.destroy()
        resolve()
      })
      socket.once('error', () => {
        socket.destroy()
        if (Date.now() - startedAt >= timeoutMs) {
          reject(new Error(`tauri-driver did not listen on port ${port}`))
        } else {
          setTimeout(attempt, 200)
        }
      })
    }
    attempt()
  })
}

export const config: WebdriverIO.Config = {
  runner: 'local',
  specs: ['./specs/**/*.e2e.ts'],
  maxInstances: 1,
  hostname: '127.0.0.1',
  port: 4444,
  capabilities: [
    {
      'tauri:options': {
        application,
      },
    },
  ],
  logLevel: 'warn',
  waitforTimeout: 10_000,
  connectionRetryTimeout: 90_000,
  connectionRetryCount: 2,
  framework: 'mocha',
  reporters: ['spec'],
  mochaOpts: {
    ui: 'bdd',
    timeout: 60_000,
  },
  async onPrepare() {
    fs.rmSync(workspace, { recursive: true, force: true })
    fs.mkdirSync(workspace, { recursive: true })
    if (!fs.existsSync(edgeDriver)) {
      throw new Error(`Missing Edge WebDriver: ${edgeDriver}. Run msedgedriver-tool first.`)
    }
    tauriDriver = spawn(
      path.join(process.env.USERPROFILE ?? '', '.cargo', 'bin', 'tauri-driver.exe'),
      ['--native-driver', edgeDriver],
      { env: process.env, stdio: ['ignore', 'pipe', 'pipe'] },
    )
    tauriDriver.stdout?.on('data', (data) => process.stdout.write(`[tauri-driver] ${data}`))
    tauriDriver.stderr?.on('data', (data) => process.stderr.write(`[tauri-driver] ${data}`))
    await waitForPort(4444, 30_000)
  },
  onComplete() {
    tauriDriver?.kill()
    tauriDriver = null
  },
}
