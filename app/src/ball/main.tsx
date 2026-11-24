import { useRef } from 'react'
import { createRoot } from 'react-dom/client'
import { invoke } from '@tauri-apps/api/core'
import icon from '../assets/deepseek-icon.svg'
import tail from '../assets/deepseek-tail.svg'
import './ball.css'

const Ball = () => {
  const drag = useRef({
    active: false,
    moved: false,
    startX: 0,
    startY: 0,
  })
  const suppressClickUntil = useRef(0)

  /** 贴边状态下显示的是尾巴，不是悬浮球；尾巴只负责点击恢复，不能拖动窗口。 */
  const isTucked = () =>
    document.body.classList.contains('tucked-left') ||
    document.body.classList.contains('tucked-right')

  return (
    <button
      className="ball"
      type="button"
      aria-label="打开 DeepSeek"
      onPointerDown={(event) => {
        if (event.button !== 0) {
          return
        }
        event.currentTarget.setPointerCapture(event.pointerId)
        drag.current = {
          active: true,
          moved: false,
          startX: event.screenX,
          startY: event.screenY,
        }
      }}
      onPointerMove={(event) => {
        if (isTucked() || !drag.current.active || drag.current.moved) {
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
        suppressClickUntil.current = Date.now() + 700
        // 自定义拖动：由 Rust 跟踪鼠标并直接移动窗口，绕开 Windows 的系统拖拽线框。
        void invoke('start_ball_drag')
      }}
      onPointerUp={() => {
        drag.current.active = false
      }}
      onPointerCancel={() => {
        drag.current.active = false
      }}
      onClick={() => {
        if (drag.current.moved || Date.now() < suppressClickUntil.current) {
          drag.current.moved = false
          drag.current.active = false
          return
        }
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
