import type { Options } from '@wdio/types'
import { spawn, type ChildProcess } from 'child_process'
import { resolve } from 'path'
import { homedir, platform } from 'os'

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

export const config: Options.Testrunner = {
  specs: ['./specs/**/*.ts'],
  maxInstances: 1,
  capabilities: [
    {
      maxInstances: 1,
      // Do not set browserName — tauri-driver handles browser selection itself.
      'tauri:options': {
        application: getTauriAppPath(),
        // webviewOptions is required on Windows for WebView2 to attach correctly.
        webviewOptions: {},
      },
    },
  ],

  logLevel: 'warn',
  bail: 1,
  waitforTimeout: 10000,
  connectionRetryTimeout: 120000,
  connectionRetryCount: 3,

  hostname: '127.0.0.1',
  port: 4444,

  framework: 'mocha',
  reporters: ['spec'],
  mochaOpts: {
    ui: 'bdd',
    timeout: 60000,
  },

  // beforeSession/afterSession (not onPrepare/onComplete) is the correct hook
  // pair for tauri-driver — it runs after the wdio runner is ready but before
  // the WebDriver session is created, matching when tauri-driver is expected.
  beforeSession: () => {
    tauriDriverProcess = spawn(
      resolve(homedir(), '.cargo', 'bin', 'tauri-driver'),
      [],
      { stdio: [null, process.stdout, process.stderr] },
    )
  },

  afterSession: () => {
    tauriDriverProcess?.kill()
  },
}
