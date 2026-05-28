import type { Options } from '@wdio/types'
import { spawn, type ChildProcess } from 'child_process'
import { resolve } from 'path'
import { platform } from 'os'

// Platform-specific Tauri release binary path.
// On macOS, Tauri produces a .app bundle; the executable is inside it.
function getTauriAppPath(): string {
  const releaseDir = resolve(process.cwd(), 'src-tauri/target/release')
  if (platform() === 'darwin') {
    return resolve(releaseDir, 'bundle/macos/SkillScout.app/Contents/MacOS/skillscout')
  }
  if (platform() === 'win32') {
    return resolve(releaseDir, 'skillscout.exe')
  }
  return resolve(releaseDir, 'skillscout')
}

let tauriDriverProcess: ChildProcess | undefined

function spawnTauriDriver(): Promise<ChildProcess> {
  return new Promise((resolve, reject) => {
    const proc = spawn('tauri-driver', [], {
      stdio: [null, process.stdout, process.stderr],
    })

    proc.on('error', (err) => {
      reject(new Error(`Failed to start tauri-driver: ${err.message}. Install with: cargo install tauri-driver`))
    })

    // Give tauri-driver 1 second to start and detect early exits
    const startTimeout = setTimeout(() => resolve(proc), 1000)

    proc.on('exit', (code) => {
      clearTimeout(startTimeout)
      reject(new Error(`tauri-driver exited early with code ${code}`))
    })
  })
}

export const config: Options.Testrunner = {
  specs: ['./specs/**/*.ts'],
  maxInstances: 1,
  capabilities: [
    {
      maxInstances: 1,
      browserName: 'chrome',
      'tauri:options': {
        application: getTauriAppPath(),
      },
    },
  ],

  logLevel: 'warn',
  bail: 1,
  waitforTimeout: 10000,
  connectionRetryTimeout: 120000,
  connectionRetryCount: 3,

  // tauri-driver acts as the WebDriver server; install it with:
  // cargo install tauri-driver
  hostname: '127.0.0.1',
  port: 4444,

  framework: 'mocha',
  reporters: ['spec'],
  mochaOpts: {
    ui: 'bdd',
    timeout: 60000,
  },

  onPrepare: async () => {
    tauriDriverProcess = await spawnTauriDriver()
  },

  onComplete: () => {
    tauriDriverProcess?.kill()
  },
}
