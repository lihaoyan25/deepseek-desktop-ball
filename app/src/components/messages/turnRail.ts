import type { AssistantMessage, Message } from '../../services/chat/types.js'

/** 一条导航记录对应一条用户消息及其后的助手回复。 */
export type TurnRailItem = {
  id: string
  turn: number
  prompt: string
  response: string
}

const PROMPT_LIMIT = 50
const RESPONSE_LIMIT = 120

const normalize = (text: string) => text.replace(/\s+/g, ' ').trim()

const clip = (text: string, limit: number) => {
  if (text.length <= limit) {
    return text
  }
  return `${text.slice(0, limit - 1).trimEnd()}…`
}

const assistantText = (message: AssistantMessage) =>
  normalize(
    message.blocks
      .filter((block) => block.type === 'text')
      .map((block) => block.text)
      .join(' ')
  )

/** 从当前可见分支生成右侧轮次导航；不依赖数据库或额外索引。 */
export const buildTurnRailItems = (messages: readonly Message[]): TurnRailItem[] => {
  const items: TurnRailItem[] = []
  for (let index = 0; index < messages.length; index += 1) {
    const message = messages[index]
    if (message.role !== 'user') {
      continue
    }
    let response = ''
    for (let next = index + 1; next < messages.length; next += 1) {
      const candidate = messages[next]
      if (candidate.role !== 'assistant') {
        break
      }
      const text = assistantText(candidate)
      if (text) {
        response = text
      }
    }
    items.push({
      id: message.id,
      turn: items.length + 1,
      prompt: clip(normalize(message.content), PROMPT_LIMIT),
      response: clip(response, RESPONSE_LIMIT),
    })
  }
  return items
}
