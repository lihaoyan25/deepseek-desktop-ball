import { useChatStore } from './store.js'
import { loadSession } from './db.js'
import type { Message, SessionSummary } from './types.js'

export type ConversationSearchResult = {
  id: string
  title: string
  updatedAt: number
  snippet: string
}

type SearchEntry = {
  title: string
  updatedAt: number
  text: string
}

const cache = new Map<string, SearchEntry>()

const normalize = (value: string) => value.toLocaleLowerCase()

const messageText = (message: Message): string => {
  if (message.role === 'user') {
    return message.content
  }
  return message.blocks
    .filter((block): block is Extract<(typeof message.blocks)[number], { type: 'text' }> =>
      block.type === 'text'
    )
    .map((block) => block.text)
    .join('\n')
}

const loadEntry = async (summary: SessionSummary): Promise<SearchEntry> => {
  const cached = cache.get(summary.id)
  if (cached && cached.updatedAt === summary.updatedAt && cached.title === summary.title) {
    return cached
  }
  const session = await loadSession(summary.id)
  const entry: SearchEntry = {
    title: summary.title,
    updatedAt: summary.updatedAt,
    text: session?.messages.map(messageText).join('\n') ?? '',
  }
  cache.set(summary.id, entry)
  return entry
}

const snippetFor = (entry: SearchEntry, query: string): string => {
  const text = entry.text.trim()
  if (!text) {
    return entry.title
  }
  const index = normalize(text).indexOf(normalize(query))
  if (index < 0) {
    const titleIndex = normalize(entry.title).indexOf(normalize(query))
    if (titleIndex >= 0) {
      return text.slice(0, 150)
    }
    return text.slice(0, 150)
  }
  const start = Math.max(0, index - 48)
  const end = Math.min(text.length, index + query.length + 110)
  return `${start > 0 ? '...' : ''}${text.slice(start, end)}${end < text.length ? '...' : ''}`
}

/** 在本地会话标题和所有消息文本中查找；首次搜索会从 IndexedDB 建立内存缓存。 */
export const searchConversationHistory = async (
  rawQuery: string
): Promise<ConversationSearchResult[]> => {
  const query = rawQuery.trim()
  if (!query) {
    return []
  }
  const normalizedQuery = normalize(query)
  const summaries = useChatStore.getState().sessions
  const entries = await Promise.all(
    summaries.map(async (summary) => ({
      summary,
      entry: await loadEntry(summary),
    }))
  )
  const results: ConversationSearchResult[] = []
  for (const { summary, entry } of entries) {
    if (
      !normalize(entry.title).includes(normalizedQuery) &&
      !normalize(entry.text).includes(normalizedQuery)
    ) {
      continue
    }
    results.push({
      id: summary.id,
      title: entry.title,
      updatedAt: summary.updatedAt,
      snippet: snippetFor(entry, query),
    })
  }
  results.sort((a, b) => b.updatedAt - a.updatedAt)
  return results.slice(0, 50)
}
