import type { MouseEvent } from 'react'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { useIsMobile } from '../../app/viewport.js'
import { useTranslate } from '../../i18n/index.js'
import whale from '../../assets/deepseek-icon.svg'
import { ChatInput } from '../chatInput/ChatInput.js'
import { MobileHeader } from './MobileHeader.js'
import styles from './Welcome.module.css'

/** 新对话：问候语 + 输入框。桌面端居中；移动端输入框吸底 */
export const Welcome = () => {
  const t = useTranslate()
  const isMobile = useIsMobile()
  const startDrag = (event: MouseEvent) => {
    if (event.button === 0) {
      void getCurrentWindow().startDragging()
    }
  }
  return (
    <div className={styles.container}>
      {isMobile ? (
        <MobileHeader />
      ) : (
        <div className={styles.desktopDragBar} onMouseDown={startDrag} />
      )}
      <div className={styles.welcome}>
        <div className={styles.greeting}>
          <img className={styles.whale} src={whale} alt="" draggable={false} />
          <span>{t('welcomeMessageChat250808')}</span>
        </div>
        <ChatInput sessionId={null} />
      </div>
    </div>
  )
}
