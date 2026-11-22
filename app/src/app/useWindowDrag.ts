import { useEffect, useRef, type MouseEvent } from 'react'
import { getCurrentWindow } from '@tauri-apps/api/window'

const DRAG_THRESHOLD = 4

/**
 * 标题栏拖拽：先等待鼠标真正移动，再交给原生窗口拖动。
 * 这样单击不会立刻进入 Windows 移动循环，双击事件仍能正常触发。
 */
export const useWindowDrag = (onDoubleClick?: () => void) => {
  const pending = useRef<{ x: number; y: number } | null>(null)
  const cleanup = useRef<(() => void) | null>(null)

  useEffect(() => () => cleanup.current?.(), [])

  return {
    onMouseDown: (event: MouseEvent<HTMLElement>) => {
      if (event.button !== 0) {
        return
      }
      // 双击的第二次 mousedown 只记录；等 mouseup 后再切换模式，
      // 避免窗口布局先变化，第二次点击落到新页面的文字上造成整段选中。
      if (event.detail === 2 && onDoubleClick) {
        event.preventDefault()
        cleanup.current?.()
        pending.current = null
        const clearSelection = () => window.getSelection()?.removeAllRanges()
        const finish = () => {
          document.removeEventListener('mouseup', finish)
          onDoubleClick()
          clearSelection()
          requestAnimationFrame(clearSelection)
        }
        document.addEventListener('mouseup', finish)
        return
      }
      cleanup.current?.()
      const origin = { x: event.clientX, y: event.clientY }
      pending.current = origin

      const onMove = (moveEvent: globalThis.MouseEvent) => {
        if (
          pending.current &&
          Math.hypot(moveEvent.clientX - origin.x, moveEvent.clientY - origin.y) > DRAG_THRESHOLD
        ) {
          pending.current = null
          cleanup.current?.()
          void getCurrentWindow().startDragging()
        }
      }
      const onUp = () => {
        pending.current = null
        cleanup.current?.()
      }
      cleanup.current = () => {
        document.removeEventListener('mousemove', onMove)
        document.removeEventListener('mouseup', onUp)
        cleanup.current = null
      }
      document.addEventListener('mousemove', onMove)
      document.addEventListener('mouseup', onUp)
    },
  }
}
