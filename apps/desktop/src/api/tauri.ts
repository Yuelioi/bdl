import { invoke } from '@tauri-apps/api/core'

import type { DownloadTask, NormalizedSourceTree } from './dto'

export interface CommandErrorShape {
  code: string
  message: string
}

export class BdlCommandError extends Error {
  readonly code: string

  constructor(error: CommandErrorShape) {
    super(error.message)
    this.name = 'BdlCommandError'
    this.code = error.code
  }
}

export interface ParseCreateSourceRequest {
  input: string
  fetch_streams?: boolean
}

export interface ParseCloseSourceResponse {
  removed: boolean
}

export interface SelectionCreateTasksRequest {
  source_id: string
  part_ids: string[]
  output_dir?: string
  archive_mode?: 'fast' | 'complete_archive'
  output_extension?: string
}

export const parseCreateSource = (request: ParseCreateSourceRequest) =>
  invokeCommand<NormalizedSourceTree>('parse_create_source', { request })

export const parseLoadMore = () => invokeCommand<void>('parse_load_more')

export const parseLoadAll = () => invokeCommand<void>('parse_load_all')

export const parseCloseSource = (sourceId: string) =>
  invokeCommand<ParseCloseSourceResponse>('parse_close_source', { sourceId })

export const parseRefreshSource = () => invokeCommand<void>('parse_refresh_source')

export const selectionCreateTasks = (request: SelectionCreateTasksRequest) =>
  invokeCommand<DownloadTask[]>('selection_create_tasks', { request })

const invokeCommand = async <T>(command: string, args?: Record<string, unknown>): Promise<T> => {
  try {
    return await invoke<T>(command, args)
  } catch (error) {
    throw normalizeCommandError(error)
  }
}

const normalizeCommandError = (error: unknown): BdlCommandError => {
  if (isCommandErrorShape(error)) {
    return new BdlCommandError(error)
  }

  if (error instanceof Error) {
    return new BdlCommandError({ code: 'frontend_error', message: error.message })
  }

  return new BdlCommandError({ code: 'frontend_error', message: String(error) })
}

const isCommandErrorShape = (value: unknown): value is CommandErrorShape => {
  if (!value || typeof value !== 'object') {
    return false
  }

  const candidate = value as Record<string, unknown>
  return typeof candidate.code === 'string' && typeof candidate.message === 'string'
}
