import fs from 'node:fs'
import path from 'node:path'
import process from 'node:process'
import { fileURLToPath } from 'node:url'
import { resolveCargoTargetDir } from '../../../scripts/cargo-target.mjs'

const scriptDir = path.dirname(fileURLToPath(import.meta.url))
const desktopDir = path.resolve(scriptDir, '..')
const repoRoot = path.resolve(desktopDir, '..', '..')
process.env.CARGO_TARGET_DIR = resolveCargoTargetDir(repoRoot)

const javaMajorVersion = (javaHome) => {
  if (!javaHome) return null
  try {
    const release = fs.readFileSync(path.join(javaHome, 'release'), 'utf8')
    const match = release.match(/^JAVA_VERSION="(\d+)/m)
    return match ? Number.parseInt(match[1], 10) : null
  } catch {
    return null
  }
}

const androidStudioJavaHomes = () => {
  if (process.platform === 'win32') {
    return [path.join(process.env.ProgramFiles ?? 'C:\\Program Files', 'Android', 'Android Studio', 'jbr')]
  }
  if (process.platform === 'darwin') {
    return ['/Applications/Android Studio.app/Contents/jbr/Contents/Home']
  }
  return ['/opt/android-studio/jbr']
}

const configureAndroidJava = () => {
  if (process.argv[2] !== 'android') return

  const configuredHome = process.env.BDL_ANDROID_JAVA_HOME
  const currentMajor = javaMajorVersion(process.env.JAVA_HOME)
  if (!configuredHome && currentMajor !== null && currentMajor <= 21) return

  const javaHome = [configuredHome, ...androidStudioJavaHomes()].find(
    (candidate) => candidate && fs.existsSync(path.join(candidate, 'bin', process.platform === 'win32' ? 'java.exe' : 'java')),
  )
  if (!javaHome) return

  process.env.JAVA_HOME = javaHome
  process.env.PATH = `${path.join(javaHome, 'bin')}${path.delimiter}${process.env.PATH ?? ''}`
}

const configureAndroidPackaging = () => {
  if (process.argv[2] !== 'android') return

  const buildFile = path.join(desktopDir, 'src-tauri', 'gen', 'android', 'app', 'build.gradle.kts')
  if (!fs.existsSync(buildFile)) return

  const resourceRule = 'resources.excludes.add("META-INF/native-image/**")'
  const bundledCliResourceRules = [
    'resources.excludes.add("lib/**/ffmpeg")',
    'resources.excludes.add("lib/**/ffprobe")',
  ]
  const bundledCliRule = 'jniLibs.excludes.add("**/ffmpeg")'
  let source = fs.readFileSync(buildFile, 'utf8')
  const requiredRules = [resourceRule, ...bundledCliResourceRules, bundledCliRule]
  if (requiredRules.every((rule) => source.includes(rule))) return

  if (source.includes(resourceRule)) {
    const missingRules = requiredRules.slice(1).filter((rule) => !source.includes(rule))
    if (missingRules.length === 0) return
    source = source.replace(resourceRule, `${resourceRule}\n        ${missingRules.join('\n        ')}`)
    fs.writeFileSync(buildFile, source)
    return
  }

  const anchor = '    buildFeatures {'
  if (!source.includes(anchor)) {
    throw new Error(`Unable to configure Android packaging: missing buildFeatures block in ${buildFile}`)
  }

  const packaging = [
    '    packaging {',
    `        ${resourceRule}`,
    ...bundledCliResourceRules.map((rule) => `        ${rule}`),
    `        ${bundledCliRule}`,
    '    }',
    '',
  ].join('\n')

  fs.writeFileSync(buildFile, source.replace(anchor, `${packaging}${anchor}`))
}

const configureAndroidReleaseProguard = () => {
  if (process.argv[2] !== 'android') return

  const rulesFile = path.join(desktopDir, 'src-tauri', 'gen', 'android', 'app', 'proguard-rules.pro')
  if (!fs.existsSync(rulesFile)) return

  const marker = '# BDL: JavaCPP Android release warnings'
  const source = fs.readFileSync(rulesFile, 'utf8')
  if (source.includes(marker)) return

  const rules = [
    '',
    marker,
    '# JavaCPP ships optional desktop/tooling integrations that are not used by BDL on Android.',
    '-dontwarn java.lang.management.BufferPoolMXBean',
    '-dontwarn javax.management.MalformedObjectNameException',
    '-dontwarn javax.management.ObjectName',
    '-dontwarn org.osgi.annotation.versioning.ConsumerType',
    '-dontwarn org.slf4j.Logger',
    '-dontwarn org.slf4j.LoggerFactory',
    '',
  ].join('\n')

  fs.writeFileSync(rulesFile, `${source.trimEnd()}\n${rules}`)
}

const configureAndroidReleaseSigning = () => {
  if (process.argv[2] !== 'android' || process.env.BDL_ANDROID_RELEASE_SIGNING !== '1') return

  const buildFile = path.join(desktopDir, 'src-tauri', 'gen', 'android', 'app', 'build.gradle.kts')
  if (!fs.existsSync(buildFile)) {
    throw new Error(`Unable to configure Android release signing: missing ${buildFile}`)
  }

  let source = fs.readFileSync(buildFile, 'utf8')
  const fileInputStreamImport = 'import java.io.FileInputStream'
  if (!source.includes(fileInputStreamImport)) {
    const importAnchor = 'import java.util.Properties'
    if (!source.includes(importAnchor)) {
      throw new Error(`Unable to configure Android release signing: missing Properties import in ${buildFile}`)
    }
    source = source.replace(importAnchor, `${fileInputStreamImport}\n${importAnchor}`)
  }

  const signingMarker = '    // BDL: Android release signing'
  if (!source.includes(signingMarker)) {
    const buildTypesAnchor = '    buildTypes {'
    if (!source.includes(buildTypesAnchor)) {
      throw new Error(`Unable to configure Android release signing: missing buildTypes block in ${buildFile}`)
    }

    const signingBlock = [
      signingMarker,
      '    signingConfigs {',
      '        create("release") {',
      '            val keystorePropertiesFile = rootProject.file("keystore.properties")',
      '            require(keystorePropertiesFile.exists()) {',
      '                "Missing Android release signing properties: ${keystorePropertiesFile.path}"',
      '            }',
      '            val keystoreProperties = Properties().apply {',
      '                FileInputStream(keystorePropertiesFile).use { load(it) }',
      '            }',
      '            keyAlias = keystoreProperties.getProperty("keyAlias")',
      '            keyPassword = keystoreProperties.getProperty("keyPassword")',
      '            storeFile = file(keystoreProperties.getProperty("storeFile"))',
      '            storePassword = keystoreProperties.getProperty("storePassword")',
      '        }',
      '    }',
      '',
    ].join('\n')
    source = source.replace(buildTypesAnchor, `${signingBlock}${buildTypesAnchor}`)
  }

  const signingConfigRule = '            signingConfig = signingConfigs.getByName("release")'
  if (!source.includes(signingConfigRule)) {
    const releaseAnchor = '        getByName("release") {'
    if (!source.includes(releaseAnchor)) {
      throw new Error(`Unable to configure Android release signing: missing release build type in ${buildFile}`)
    }
    source = source.replace(releaseAnchor, `${releaseAnchor}\n${signingConfigRule}`)
  }

  fs.writeFileSync(buildFile, source)
}

const syncAndroidIcons = () => {
  if (process.argv[2] !== 'android') return

  const sourceRoot = path.join(desktopDir, 'src-tauri', 'icons', 'android')
  const targetRoot = path.join(desktopDir, 'src-tauri', 'gen', 'android', 'app', 'src', 'main', 'res')
  if (!fs.existsSync(sourceRoot) || !fs.existsSync(targetRoot)) return

  for (const sourcePath of fs.readdirSync(sourceRoot, { recursive: true, withFileTypes: true })) {
    if (!sourcePath.isFile()) continue
    const sourceFile = path.join(sourcePath.parentPath, sourcePath.name)
    const relativePath = path.relative(sourceRoot, sourceFile)
    const targetFile = path.join(targetRoot, relativePath)
    fs.mkdirSync(path.dirname(targetFile), { recursive: true })
    fs.copyFileSync(sourceFile, targetFile)
  }
}

configureAndroidJava()
configureAndroidPackaging()
configureAndroidReleaseProguard()
configureAndroidReleaseSigning()
syncAndroidIcons()

await import('@tauri-apps/cli/tauri.js')
