import { useEffect, useState } from 'react'
import { createRoot } from 'react-dom/client'
import { invoke } from '@tauri-apps/api/core'
import { setupTheme } from '../app/theme.js'
import { SelectionActions } from '../selection/actions.js'
import '../app/theme.css'
import '../app/global.css'
import '../selection/selection.css'
import './capture.css'

setupTheme()

type CaptureImage = {
  dataUrl: string
  width: number
  height: number
}

type Rect = { x: number; y: number; width: number; height: number }
type Point = { x: number; y: number }

const Capture = () => {
  const [image, setImage] = useState<CaptureImage | null>(null)
  const [start, setStart] = useState<Point | null>(null)
  const [current, setCurrent] = useState<Point | null>(null)
  const [selected, setSelected] = useState<Rect | null>(null)

  const rect = (() => {
    if (!start || !current) {
      return null
    }
    return {
      x: Math.min(start.x, current.x) / window.innerWidth,
      y: Math.min(start.y, current.y) / window.innerHeight,
      width: Math.abs(current.x - start.x) / window.innerWidth,
      height: Math.abs(current.y - start.y) / window.innerHeight,
    }
  })()

  useEffect(() => {
    invoke<CaptureImage>('capture_image')
      .then((value) => setImage(value))
      .catch(() => {
        void invoke('capture_cancel')
      })
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        void invoke('capture_cancel')
      }
    }
    window.addEventListener('keydown', onKeyDown)
    return () => window.removeEventListener('keydown', onKeyDown)
  }, [])

  const finishSelection = () => {
    if (!rect || rect.width * window.innerWidth < 5 || rect.height * window.innerHeight < 5) {
      setStart(null)
      setCurrent(null)
      return
    }
    setSelected(rect)
    setStart(null)
    setCurrent(null)
  }

  const local = selected
    ? {
        left: selected.x * window.innerWidth,
        top: selected.y * window.innerHeight,
        width: selected.width * window.innerWidth,
        height: selected.height * window.innerHeight,
      }
    : null

  const barWidth = 360
  const barHeight = 48
  const barPosition = (() => {
    if (!local) {
      return null
    }
    const margin = 10
    let left = local.left + local.width + margin
    if (left + barWidth > window.innerWidth - margin) {
      left = local.left - barWidth - margin
    }
    left = Math.max(margin, Math.min(left, window.innerWidth - barWidth - margin))
    let top = local.top + margin
    if (top + barHeight > window.innerHeight - margin) {
      top = local.top - barHeight - margin
    }
    top = Math.max(margin, Math.min(top, window.innerHeight - barHeight - margin))
    return { left, top }
  })()

  const action = (kind: string) => {
    if (!selected) {
      return
    }
    void invoke('capture_action', { action: kind, rect: selected }).catch(() => {})
  }

  return (
    <div
      className="capture"
      onContextMenu={(event) => {
        event.preventDefault()
        void invoke('capture_cancel')
      }}
      onPointerDown={(event) => {
        if (selected || event.button !== 0) {
          return
        }
        setStart({ x: event.clientX, y: event.clientY })
        setCurrent({ x: event.clientX, y: event.clientY })
      }}
      onPointerMove={(event) => {
        if (start) {
          setCurrent({ x: event.clientX, y: event.clientY })
        }
      }}
      onPointerUp={finishSelection}
    >
      {image ? <img className="screen" src={image.dataUrl} alt="" draggable={false} /> : null}
      <div className="dim" />
      {rect ? (
        <div
          className="selection"
          style={{
            left: `${rect.x * window.innerWidth}px`,
            top: `${rect.y * window.innerHeight}px`,
            width: `${rect.width * window.innerWidth}px`,
            height: `${rect.height * window.innerHeight}px`,
          }}
        />
      ) : null}
      {!selected ? <div className="hint">拖动选择截图区域 · Esc 取消</div> : null}
      {selected && barPosition ? (
        <div
          className="capture-actions"
          style={{ left: barPosition.left, top: barPosition.top, width: barWidth }}
        >
          <SelectionActions onAction={action} />
        </div>
      ) : null}
    </div>
  )
}

createRoot(document.getElementById('root') as HTMLElement).render(<Capture />)
