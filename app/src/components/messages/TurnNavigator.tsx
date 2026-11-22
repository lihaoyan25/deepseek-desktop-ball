import {
  useEffect,
  useId,
  useMemo,
  useRef,
  useState,
  type CSSProperties,
  type RefObject,
} from 'react'
import { useTranslate } from '../../i18n/index.js'
import type { TurnRailItem } from './turnRail.js'
import styles from './TurnNavigator.module.css'

const TURN_SPACING_PX = 10
const ACTIVE_LINE_OFFSET = 72

/** 从可见的用户消息生成右侧固定节距的轮次导航。 */
export const TurnNavigator = ({
  items,
  scrollRef,
  contentRef,
  onNavigate,
}: {
  items: readonly TurnRailItem[]
  scrollRef: RefObject<HTMLDivElement | null>
  contentRef: RefObject<HTMLDivElement | null>
  onNavigate: (item: TurnRailItem) => void
}) => {
  const t = useTranslate()
  const previewId = useId()
  const railRef = useRef<HTMLElement>(null)
  const scrollerRef = useRef<HTMLDivElement>(null)
  const [activeId, setActiveId] = useState<string | null>(null)
  const [preview, setPreview] = useState<TurnRailItem | null>(null)
  const [band, setBand] = useState({ top: 0, height: 0 })
  const [railScrollTop, setRailScrollTop] = useState(0)

  const indexById = useMemo(() => new Map(items.map((item, index) => [item.id, index])), [items])
  const previewIndex = preview ? indexById.get(preview.id) : undefined
  const previewTop =
    previewIndex === undefined
      ? 0
      : previewIndex * TURN_SPACING_PX + 6 - railScrollTop

  // 长会话中让当前 mark 始终落在轨道可视范围内。
  useEffect(() => {
    const scroller = scrollerRef.current
    if (!scroller || !activeId) {
      return
    }
    const index = indexById.get(activeId)
    if (index === undefined) {
      return
    }
    const top = index * TURN_SPACING_PX
    const viewTop = scroller.scrollTop
    const viewBottom = viewTop + scroller.clientHeight
    if (top < viewTop + 24) {
      scroller.scrollTo({ top: Math.max(0, top - 24), behavior: 'smooth' })
    } else if (top + 10 > viewBottom - 24) {
      scroller.scrollTo({
        top: top + 10 - scroller.clientHeight + 24,
        behavior: 'smooth',
      })
    }
  }, [activeId, indexById])

  useEffect(() => {
    if (items.length < 2) {
      setActiveId(null)
      return
    }
    const scroll = scrollRef.current
    if (!scroll) {
      return
    }
    let frame = 0

    const update = () => {
      frame = 0
      const anchors = scroll.querySelectorAll<HTMLElement>('[data-turn-id]')
      if (anchors.length === 0) {
        setActiveId(items[0]?.id ?? null)
        return
      }
      const line = scroll.getBoundingClientRect().top + ACTIVE_LINE_OFFSET
      let next = items[0]?.id ?? null
      for (const item of items) {
        const anchor = scroll.querySelector<HTMLElement>(
          `[data-turn-id="${CSS.escape(item.id)}"]`
        )
        if (anchor && anchor.getBoundingClientRect().top <= line) {
          next = item.id
        }
      }
      setActiveId(next)
    }

    const onScroll = () => {
      if (frame === 0) {
        frame = requestAnimationFrame(update)
      }
    }

    update()
    scroll.addEventListener('scroll', onScroll, { passive: true })
    return () => {
      if (frame !== 0) {
        cancelAnimationFrame(frame)
      }
      scroll.removeEventListener('scroll', onScroll)
    }
  }, [items, scrollRef])

  // 消息内容增长时更新时间线状态；不把 ResizeObserver 挂在每个消息上。
  useEffect(() => {
    const content = contentRef.current
    if (!content || items.length < 2) {
      return
    }
    const scroll = scrollRef.current
    if (!scroll) {
      return
    }
    const observer = new ResizeObserver(() => {
      const line = scroll.getBoundingClientRect().top + ACTIVE_LINE_OFFSET
      let next = items[0]?.id ?? null
      for (const item of items) {
        const anchor = scroll.querySelector<HTMLElement>(
          `[data-turn-id="${CSS.escape(item.id)}"]`
        )
        if (anchor && anchor.getBoundingClientRect().top <= line) {
          next = item.id
        }
      }
      setActiveId(next)
    })
    observer.observe(content)
    return () => observer.disconnect()
  }, [items, contentRef, scrollRef])

  // 轨道只覆盖消息阅读区，不压住底部输入框。
  useEffect(() => {
    const scroll = scrollRef.current
    if (!scroll) {
      return
    }
    const updateBand = () => {
      const rect = scroll.getBoundingClientRect()
      const footer = scroll.querySelector<HTMLElement>('[data-chat-footer]')
      const footerTop = footer?.getBoundingClientRect().top ?? rect.bottom
      const height = Math.max(0, Math.min(rect.bottom, footerTop) - rect.top)
      const parentTop = railRef.current?.parentElement?.getBoundingClientRect().top ?? rect.top
      setBand({ top: rect.top - parentTop + height / 2, height })
    }
    updateBand()
    const observer = new ResizeObserver(updateBand)
    observer.observe(scroll)
    const footer = scroll.querySelector<HTMLElement>('[data-chat-footer]')
    if (footer) {
      observer.observe(footer)
    }
    window.addEventListener('resize', updateBand)
    return () => {
      observer.disconnect()
      window.removeEventListener('resize', updateBand)
    }
  }, [scrollRef, items])

  useEffect(() => {
    const scroller = scrollerRef.current
    if (!scroller) {
      return
    }
    const onScroll = () => setRailScrollTop(scroller.scrollTop)
    scroller.addEventListener('scroll', onScroll, { passive: true })
    return () => scroller.removeEventListener('scroll', onScroll)
  }, [])

  if (items.length < 2) {
    return null
  }

  const bandHeight = Math.max(0, band.height - 64)
  const frameStyle: CSSProperties = {
    top: band.top,
    maxHeight: Math.min(420, bandHeight),
  }

  return (
    <nav
      ref={railRef}
      className={styles.frame}
      style={frameStyle}
      aria-label={t('turnNavigatorLabel')}
      onPointerLeave={() => setPreview(null)}
    >
      <div ref={scrollerRef} className={styles.scroller}>
        <div
          className={styles.marks}
          style={{ height: items.length * TURN_SPACING_PX + 12 }}
        >
          {items.map((item, index) => (
            <button
              key={item.id}
              type="button"
              className={`${styles.mark} ${item.id === activeId ? styles.markActive : ''}`}
              style={
                {
                  top: index * TURN_SPACING_PX + 6,
                  '--turn-mark-scale': String(
                    0.6 - (Math.min(items.length - 1 - index, 12) / 12) * 0.2
                  ),
                  '--turn-mark-opacity': String(
                    0.9 - (Math.min(items.length - 1 - index, 12) / 12) * 0.45
                  ),
                } as CSSProperties
              }
              aria-label={t('turnNavigatorJump', { turn: item.turn })}
              aria-current={item.id === activeId ? 'true' : undefined}
              aria-describedby={preview?.id === item.id ? previewId : undefined}
              onPointerEnter={() => setPreview(item)}
              onFocus={() => setPreview(item)}
              onBlur={() => setPreview(null)}
              onClick={() => {
                setActiveId(item.id)
                onNavigate(item)
              }}
            />
          ))}
        </div>
      </div>
      {preview ? (
        <div
          id={previewId}
          className={styles.preview}
          role="tooltip"
          style={{ top: Math.max(0, Math.min(previewTop, Math.max(0, bandHeight - 100))) }}
        >
          <div className={styles.previewPrompt}>
            {preview.prompt || t('turnNavigatorFallback', { turn: preview.turn })}
          </div>
          {preview.response ? <div className={styles.previewResponse}>{preview.response}</div> : null}
        </div>
      ) : null}
    </nav>
  )
}
