import { getCurrentWindow } from '@tauri-apps/api/window'
import styles from './WindowResizeHandles.module.css'

type ResizeDirection =
  | 'East'
  | 'North'
  | 'NorthEast'
  | 'NorthWest'
  | 'South'
  | 'SouthEast'
  | 'SouthWest'
  | 'West'

const directions: Array<[string, ResizeDirection]> = [
  ['north', 'North'],
  ['south', 'South'],
  ['west', 'West'],
  ['east', 'East'],
  ['northWest', 'NorthWest'],
  ['northEast', 'NorthEast'],
  ['southWest', 'SouthWest'],
  ['southEast', 'SouthEast'],
]

/** 无边框窗口的四边/四角 resize 命中区，不新增可见控件。 */
export const WindowResizeHandles = () => (
  <div className={styles.layer} aria-hidden="true">
    {directions.map(([className, direction]) => (
      <div
        key={direction}
        className={`${styles.handle} ${styles[className]}`}
        onPointerDown={(event) => {
          if (event.button !== 0) {
            return
          }
          event.preventDefault()
          event.stopPropagation()
          void getCurrentWindow()
            .startResizeDragging(direction)
            .catch(() => {})
        }}
      />
    ))}
  </div>
)
