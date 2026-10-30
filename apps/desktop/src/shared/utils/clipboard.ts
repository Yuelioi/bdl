import { invoke, isTauri } from '@tauri-apps/api/core'

import { isAndroidPlatform } from '../platform/environment'
import { writeNativeClipboardText } from '../api/tauri'

export const readClipboardText = async (android = isAndroidPlatform()): Promise<string> => {
  if (android) return invoke<string>('mobile_read_clipboard_text')
  if (isTauri()) return invoke<string>('desktop_read_clipboard_text')
  return navigator.clipboard.readText()
}

export const writeClipboardText = async (text: string, android = isAndroidPlatform()): Promise<void> => {
  if (android || isTauri()) return writeNativeClipboardText(text, android)
  return navigator.clipboard.writeText(text)
}
