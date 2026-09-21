import { spawnSync } from 'node:child_process'
import fs from 'node:fs'
import os from 'node:os'
import path from 'node:path'
import process from 'node:process'
import { fileURLToPath } from 'node:url'

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..')
const desktopDir = path.join(repoRoot, 'apps', 'desktop')
const androidDir = path.join(desktopDir, 'src-tauri', 'gen', 'android')
const androidAppDir = path.join(androidDir, 'app')
const buildFile = path.join(androidAppDir, 'build.gradle.kts')
const keystorePropertiesPath = path.join(androidDir, 'keystore.properties')
const pnpm = process.platform === 'win32' ? 'pnpm.cmd' : 'pnpm'

const run = (command, args, options = {}) => {
  const result = spawnSync(command, args, {
    cwd: options.cwd ?? repoRoot,
    env: options.env ?? process.env,
    stdio: 'inherit',
  })
  if (result.error) throw result.error
  if (result.status !== 0) {
    throw new Error(`${command} ${args.join(' ')} failed with exit code ${result.status ?? 'unknown'}.`)
  }
}

const requireEnv = (name) => {
  const value = process.env[name]?.trim()
  if (!value) throw new Error(`Missing required Android release signing variable: ${name}`)
  if (/\r|\n/.test(value)) throw new Error(`${name} must not contain line breaks.`)
  return value
}

const propertyValue = (value) => value.replaceAll('\\', '\\\\')

const resolveApkSigner = () => {
  const androidHome = process.env.ANDROID_HOME ?? process.env.ANDROID_SDK_ROOT
  if (!androidHome) throw new Error('ANDROID_HOME or ANDROID_SDK_ROOT must be set for Android release builds.')
  const buildToolsDir = path.join(androidHome, 'build-tools')
  if (!fs.existsSync(buildToolsDir)) throw new Error(`Android build-tools directory not found: ${buildToolsDir}`)

  const versions = fs
    .readdirSync(buildToolsDir, { withFileTypes: true })
    .filter((entry) => entry.isDirectory())
    .map((entry) => entry.name)
    .sort((left, right) => left.localeCompare(right, undefined, { numeric: true }))
    .reverse()
  for (const version of versions) {
    const candidate = path.join(buildToolsDir, version, process.platform === 'win32' ? 'apksigner.bat' : 'apksigner')
    if (fs.existsSync(candidate)) return candidate
  }
  throw new Error(`apksigner was not found under ${buildToolsDir}.`)
}

const packageVersion = JSON.parse(fs.readFileSync(path.join(desktopDir, 'package.json'), 'utf8')).version
const keyAlias = requireEnv('ANDROID_KEY_ALIAS')
const keyPassword = requireEnv('ANDROID_KEY_PASSWORD')
const storePassword = process.env.ANDROID_KEYSTORE_PASSWORD?.trim() || keyPassword
const encodedKeystore = process.env.ANDROID_KEY_BASE64?.trim()
const configuredKeystorePath = process.env.ANDROID_KEYSTORE_PATH?.trim()

if (Boolean(encodedKeystore) === Boolean(configuredKeystorePath)) {
  throw new Error('Set exactly one of ANDROID_KEY_BASE64 or ANDROID_KEYSTORE_PATH for Android release signing.')
}

if (!fs.existsSync(buildFile)) {
  run(pnpm, ['tauri', 'android', 'init', '--ci', '--skip-targets-install'], { cwd: desktopDir })
}

let temporaryDirectory
let keystorePath
try {
  if (encodedKeystore) {
    temporaryDirectory = fs.mkdtempSync(path.join(os.tmpdir(), 'bdl-android-signing-'))
    keystorePath = path.join(temporaryDirectory, 'release-keystore.jks')
    fs.writeFileSync(keystorePath, Buffer.from(encodedKeystore, 'base64'), { mode: 0o600 })
  } else {
    keystorePath = path.resolve(configuredKeystorePath)
    if (!fs.existsSync(keystorePath)) throw new Error(`Android release keystore not found: ${keystorePath}`)
  }

  const properties = [
    `keyAlias=${propertyValue(keyAlias)}`,
    `keyPassword=${propertyValue(keyPassword)}`,
    `storeFile=${propertyValue(keystorePath.replaceAll('\\', '/'))}`,
    `storePassword=${propertyValue(storePassword)}`,
    '',
  ].join('\n')
  fs.writeFileSync(keystorePropertiesPath, properties, { mode: 0o600 })

  run(pnpm, ['tauri', 'android', 'build', '--target', 'aarch64', '--ci'], {
    cwd: desktopDir,
    env: { ...process.env, BDL_ANDROID_RELEASE_SIGNING: '1' },
  })

  const apkSource = path.join(androidAppDir, 'build', 'outputs', 'apk', 'universal', 'release', 'app-universal-release.apk')
  const aabSource = path.join(androidAppDir, 'build', 'outputs', 'bundle', 'universalRelease', 'app-universal-release.aab')
  if (!fs.existsSync(apkSource)) {
    const unsignedApk = path.join(
      androidAppDir,
      'build',
      'outputs',
      'apk',
      'universal',
      'release',
      'app-universal-release-unsigned.apk',
    )
    if (fs.existsSync(unsignedApk)) {
      throw new Error('Android release build produced only an unsigned APK; release signing was not applied.')
    }
    throw new Error(`Android release APK not found: ${apkSource}`)
  }
  if (!fs.existsSync(aabSource)) throw new Error(`Android release AAB not found: ${aabSource}`)

  const outputDir = path.join(repoRoot, 'dist', 'android')
  fs.rmSync(outputDir, { recursive: true, force: true })
  fs.mkdirSync(outputDir, { recursive: true })
  const apkOutput = path.join(outputDir, `BDL-v${packageVersion}-android-arm64.apk`)
  const aabOutput = path.join(outputDir, `BDL-v${packageVersion}-android-arm64.aab`)
  fs.copyFileSync(apkSource, apkOutput)
  fs.copyFileSync(aabSource, aabOutput)

  run(resolveApkSigner(), ['verify', '--verbose', '--print-certs', apkOutput])
  console.log(`Android release artifacts:\n- ${apkOutput}\n- ${aabOutput}`)
} finally {
  fs.rmSync(keystorePropertiesPath, { force: true })
  if (temporaryDirectory) fs.rmSync(temporaryDirectory, { recursive: true, force: true })
}
