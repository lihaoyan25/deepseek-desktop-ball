import { useEffect, useRef, useState } from 'react'
import { useTranslate } from '../../i18n/index.js'
import { IconChatBubbleOutline16, IconCloseOutline16, IconSearchOutline16 } from '../base/icons.js'
import { IconButton } from '../base/IconButton.js'
import { searchConversationHistory, type ConversationSearchResult } from '../../services/chat/search.js'
import styles from './ConversationSearch.module.css'

const formatTime = (timestamp: number) => {
  const date = new Date(timestamp)
  const now = new Date()
  if (
    date.getFullYear() === now.getFullYear() &&
    date.getMonth() === now.getMonth() &&
    date.getDate() === now.getDate()
  ) {
    return `${String(date.getHours()).padStart(2, '0')}:${String(date.getMinutes()).padStart(2, '0')}`
  }
  return new Intl.DateTimeFormat(undefined, { month: 'long', day: 'numeric' }).format(date)
}

export const ConversationSearch = ({
  onClose,
  onSelect,
}: {
  onClose: () => void
  onSelect: (id: string) => void
}) => {
  const t = useTranslate()
  const [query, setQuery] = useState('')
  const [results, setResults] = useState<ConversationSearchResult[]>([])
  const [loading, setLoading] = useState(false)
  const inputRef = useRef<HTMLInputElement>(null)

  useEffect(() => {
    inputRef.current?.focus()
  }, [])

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        onClose()
      }
    }
    document.addEventListener('keydown', onKeyDown)
    return () => document.removeEventListener('keydown', onKeyDown)
  }, [onClose])

  useEffect(() => {
    let alive = true
    if (!query.trim()) {
      setResults([])
      setLoading(false)
      return
    }
    setLoading(true)
    const timer = setTimeout(async () => {
      try {
        const found = await searchConversationHistory(query)
        if (alive) {
          setResults(found)
        }
      } finally {
        if (alive) {
          setLoading(false)
        }
      }
    }, 160)
    return () => {
      alive = false
      clearTimeout(timer)
    }
  }, [query])

  return (
    <div className={styles.shell}>
      <div className={styles.searchRow}>
        <span className={styles.searchIcon}>
          <IconSearchOutline16 />
        </span>
        <input
          ref={inputRef}
          className={styles.input}
          value={query}
          onChange={(event) => setQuery(event.target.value)}
          placeholder={t('searchConversationsPlaceholder')}
          aria-label={t('searchSessions')}
          spellCheck={false}
        />
        <IconButton
          icon={<IconCloseOutline16 />}
          aria-label={t('close')}
          onClick={onClose}
        />
      </div>
      {query.trim() ? (
        <div className={styles.results}>
          {!loading && results.length === 0 ? (
            <div className={styles.empty}>{t('searchNoResults')}</div>
          ) : null}
          {results.map((result) => (
            <button
              key={result.id}
              type="button"
              className={styles.result}
              onClick={() => onSelect(result.id)}
            >
              <span className={styles.resultIcon}>
                <IconChatBubbleOutline16 />
              </span>
              <span className={styles.resultText}>
                <span className={styles.resultHeading}>
                  <span className={styles.resultTitle}>{result.title || t('untitledChat')}</span>
                  <span className={styles.resultTime}>{formatTime(result.updatedAt)}</span>
                </span>
                <span className={styles.snippet}>{result.snippet}</span>
              </span>
            </button>
          ))}
        </div>
      ) : null}
    </div>
  )
}
