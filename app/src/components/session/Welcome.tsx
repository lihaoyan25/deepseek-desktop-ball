import { invoke } from '@tauri-apps/api/core'
import { useIsMobile } from '../../app/viewport.js'
import { useWindowDrag } from '../../app/useWindowDrag.js'
import { useTranslate } from '../../i18n/index.js'
import whale from '../../assets/deepseek-icon.svg'
import { ChatInput } from '../chatInput/ChatInput.js'
import { MobileHeader } from './MobileHeader.js'
import type { PanelMode } from '../../app/panelMode.js'
import styles from './Welcome.module.css'

/** 新对话：问候语 + 输入框。桌面端居中；移动端输入框吸底 */
export const Welcome = ({ panelMode }: { panelMode: PanelMode }) => {
  const t = useTranslate()
  const isMobile = useIsMobile()
  const toggleMode = () => {
    void invoke(panelMode === 'max' ? 'restore_panel' : 'maximize_panel')
  }
  const drag = useWindowDrag(toggleMode)
  return (
    <div className={styles.container}>
      {isMobile ? (
        <MobileHeader mode={panelMode} />
      ) : (
        <div className={styles.desktopDragBar} {...drag} />
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
