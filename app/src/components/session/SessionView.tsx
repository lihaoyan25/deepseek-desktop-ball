import clsx from 'clsx'
import { invoke } from '@tauri-apps/api/core'
import { useIsMobile } from '../../app/viewport.js'
import { useWindowDrag } from '../../app/useWindowDrag.js'
import { useTranslate } from '../../i18n/index.js'
import type { Session } from '../../services/chat/types.js'
import { ChatInput } from '../chatInput/ChatInput.js'
import { MessageList } from '../messages/MessageList.js'
import { MobileHeader } from './MobileHeader.js'
import type { PanelMode } from '../../app/panelMode.js'
import styles from './SessionView.module.css'

/** 会话页：标题栏（移动端为顶栏）+ 消息列表，输入框吸底 */
export const SessionView = ({
  session,
  streaming,
  siderCollapsed,
  panelMode,
}: {
  session: Session
  streaming: boolean
  siderCollapsed: boolean
  panelMode: PanelMode
}) => {
  const t = useTranslate()
  const isMobile = useIsMobile()
  const title = session.title || t('untitledChat')
  const toggleMode = () => {
    void invoke(panelMode === 'max' ? 'restore_panel' : 'maximize_panel')
  }
  const drag = useWindowDrag(toggleMode)
  return (
    <div className={styles.view}>
      {isMobile ? (
        <MobileHeader title={title} mode={panelMode} />
      ) : (
        <div className={styles.header} {...drag}>
          <div className={clsx(styles.title, siderCollapsed && styles.withCollapsedSider)}>
            {title}
          </div>
        </div>
      )}
      <MessageList
        session={session}
        streaming={streaming}
        footer={
          <div className={styles.inputContainer} data-chat-footer>
            <ChatInput sessionId={session.id} />
            {isMobile ? null : <div className={styles.caveat}>{t('contentCaveat')}</div>}
          </div>
        }
      />
    </div>
  )
}
