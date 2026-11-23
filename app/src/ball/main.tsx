import { useEffect, useRef } from 'react'
import { createRoot } from 'react-dom/client'
import { invoke } from '@tauri-apps/api/core'
import { getCurrentWindow } from '@tauri-apps/api/window'
import icon from '../assets/deepseek-icon.svg'
import tail from '../assets/deepseek-tail.svg'
import './ball.css'

const Ball = () => {
  const drag = useRef({
    active: false,
    moved: false,
    dragging: false,
    startX: 0,
    startY: 0,
  })
  const suppressClickUntil = useRef(0)
  const finishTimer = useRef<number | null>(null)

  const clearFinishTimer = () => {
    if (finishTimer.current !== null) {
      window.clearTimeout(finishTimer.current)
      finishTimer.current = null
    }
  }

  const finishDrag = () => {
    if (!drag.current.dragging) {
      return
    }
    drag.current.dragging = false
    drag.current.active = false
    void invoke('save_ball_position')
  }

  const finishDragSoon = (delay = 300) => {
    clearFinishTimer()
    finishTimer.current = window.setTimeout(finishDrag, delay)
  }

  useEffect(() => {
    let unlisten: (() => void) | undefined
    void getCurrentWindow()
      .onMoved(() => {
        // 只保存用户拖动造成的移动；tuck/restore 不是拖动，不能覆盖 rest。
        if (drag.current.dragging) {
          finishDragSoon()
        }
      })
      .then((stop) => {
        unlisten = stop
      })
    return () => {
      unlisten?.()
      clearFinishTimer()
    }
  }, [])

  return (
    <button
      className="ball"
      type="button"
      aria-label="打开 DeepSeek"
      onPointerDown={(event) => {
        if (event.button !== 0) {
          return
        }
        clearFinishTimer()
        drag.current = {
          active: true,
          moved: false,
          dragging: false,
          startX: event.screenX,
          startY: event.screenY,
        }
      }}
      onPointerMove={(event) => {
        if (!drag.current.active || drag.current.moved) {
          return
        }
        const distance = Math.hypot(
          event.screenX - drag.current.startX,
          event.screenY - drag.current.startY
        )
        if (distance < 5) {
          return
        }
        drag.current.moved = true
        drag.current.dragging = true
        suppressClickUntil.current = Date.now() + 700
        void getCurrentWindow().startDragging()
        // startDragging 后不一定还有 pointerup；等窗口停稳后保存。
        finishDragSoon(600)
      }}
      onPointerUp={() => {
        drag.current.active = false
        if (drag.current.moved) {
          finishDragSoon(250)
        } else {
          drag.current.dragging = false
        }
      }}
      onPointerCancel={() => {
        drag.current.active = false
        if (drag.current.moved) {
          finishDragSoon(250)
        } else {
          drag.current.dragging = false
        }
      }}
      onClick={() => {
        if (drag.current.moved || Date.now() < suppressClickUntil.current) {
          drag.current.moved = false
          drag.current.active = false
          return
        }
        clearFinishTimer()
        document.body.classList.add('ball-leaving')
        window.setTimeout(() => {
          void invoke('toggle_from_ball')
        }, 70)
        window.setTimeout(() => {
          document.body.classList.remove('ball-leaving')
        }, 360)
      }}
    >
      <img className="whale" src={icon} alt="" draggable={false} />
      <img className="tail" src={tail} alt="" draggable={false} />
    </button>
  )
}

createRoot(document.getElementById('root') as HTMLElement).render(<Ball />)
