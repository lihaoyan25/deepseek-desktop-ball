import type { MouseEvent } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { getCurrentWindow } from '@tauri-apps/api/window'
import {
  IconMaximizeOutline16,
  IconMenuOutline20,
  IconMinimizeOutline16,
  IconNewChatOutline20,
} from '../base/icons.js'
import { useTranslate } from '../../i18n/index.js'
import { newChat } from '../../services/chat/store.js'
import { IconButton } from '../base/IconButton.js'
import { openMobileSider } from '../sider/mobileSiderState.js'
import styles from './MobileHeader.module.css'

/** 移动端顶栏：左侧菜单打开侧边栏；会话页再显示标题与开启新对话 */
export const MobileHeader = ({ title }: { title?: string }) => {
  const t = useTranslate()
  const startDrag = (event: MouseEvent) => {
    if (event.button === 0) {
      void getCurrentWindow().startDragging()
    }
  }
  const closePanel = () => {
    void invoke('close_panel')
  }
  const maximizePanel = () => {
    void invoke('maximize_panel')
  }
  return (
    <div className={styles.header}>
      <IconButton
        size="xl"
        tone="primary"
        icon={<IconMenuOutline20 />}
        aria-label={t('expandSiderbarButton')}
        onClick={openMobileSider}
      />
      {title === undefined ? (
        <div className={styles.dragSpacer} onMouseDown={startDrag} />
      ) : (
        <>
          <div className={styles.title} onMouseDown={startDrag}>
            {title}
          </div>
          <div className={styles.actions}>
            <IconButton
              size="xl"
              tone="primary"
              icon={<IconNewChatOutline20 />}
              aria-label={t('newChat')}
              onClick={newChat}
            />
            <IconButton
              size="m"
              tone="primary"
              icon={<IconMaximizeOutline16 />}
              aria-label={t('maximizePanel')}
              tooltip={t('maximizePanel')}
              tooltipPlacement="bottom"
              onClick={maximizePanel}
            />
            <IconButton
              size="m"
              tone="primary"
              icon={<IconMinimizeOutline16 />}
              aria-label={t('closePanel')}
              tooltip={t('closePanel')}
              tooltipPlacement="bottom"
              onClick={closePanel}
            />
          </div>
        </>
      )}
      {title === undefined ? (
        <div className={styles.actions}>
          <IconButton
            size="m"
            tone="primary"
            icon={<IconMaximizeOutline16 />}
            aria-label={t('maximizePanel')}
            tooltip={t('maximizePanel')}
            tooltipPlacement="bottom"
            onClick={maximizePanel}
          />
          <IconButton
            size="m"
            tone="primary"
            icon={<IconMinimizeOutline16 />}
            aria-label={t('closePanel')}
            tooltip={t('closePanel')}
            tooltipPlacement="bottom"
            onClick={closePanel}
          />
        </div>
      ) : null}
    </div>
  )
}
