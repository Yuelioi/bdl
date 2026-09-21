import { invoke } from '@tauri-apps/api/core'

import { isAndroidPlatform } from './platform'

export const readClipboardText = async (android = isAndroidPlatform()): Promise<string> => {
  if (android) return invoke<string>('mobile_read_clipboard_text')
  return navigator.clipboard.readText()
}
