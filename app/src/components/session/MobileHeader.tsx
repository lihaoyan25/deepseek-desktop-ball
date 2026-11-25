import { invoke } from '@tauri-apps/api/core'
import {
  IconExitFullscreenOutline16,
  IconMaximizeOutline16,
  IconMenuOutline20,
  IconMinimizeOutline16,
  IconNewChatOutline20,
  IconWindowMaximizeOutline16,
  IconWindowRestoreOutline16,
} from '../base/icons.js'
import { useTranslate } from '../../i18n/index.js'
import { newChat } from '../../services/chat/store.js'
import { IconButton } from '../base/IconButton.js'
import { openMobileSider } from '../sider/mobileSiderState.js'
import { useWindowDrag } from '../../app/useWindowDrag.js'
import type { PanelMode } from '../../app/panelMode.js'
import styles from './MobileHeader.module.css'

/** 移动端顶栏：左侧菜单打开侧边栏；会话页再显示标题与开启新对话 */
export const MobileHeader = ({ title, mode }: { title?: string; mode: PanelMode }) => {
  const t = useTranslate()
  const isMini = mode === 'mini'
  const isFull = mode === 'full'
  const toggleMode = () => {
    void invoke(isMini ? 'maximize_panel' : 'restore_panel')
  }
  const drag = useWindowDrag(toggleMode)
  const closePanel = () => {
    void invoke('close_panel')
  }
  const maximizePanel = () => {
    void invoke('maximize_panel')
  }
  const toggleFullPanel = () => {
    void invoke('toggle_full_panel')
  }
  const restorePanel = () => {
    void invoke('restore_panel')
  }
  const modeIcon = isFull ? (
    <IconWindowRestoreOutline16 />
  ) : isMini ? (
    <IconMaximizeOutline16 />
  ) : (
    <IconWindowMaximizeOutline16 />
  )
  const modeLabel = isMini
    ? t('maximizePanel')
    : isFull
      ? t('restoreWindow')
      : t('maximizeWindow')
  const modeAction = isMini ? maximizePanel : toggleFullPanel
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
        <div className={styles.dragSpacer} {...drag} />
      ) : (
        <>
          <div className={styles.title} {...drag}>
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
            {isMini ? null : (
              <IconButton
                size="m"
                tone="primary"
                icon={<IconExitFullscreenOutline16 />}
                aria-label={t('restorePanel')}
                tooltip={t('restorePanel')}
                tooltipPlacement="bottom"
                onClick={restorePanel}
              />
            )}
            <IconButton
              size="m"
              tone="primary"
              icon={modeIcon}
              aria-label={modeLabel}
              tooltip={modeLabel}
              tooltipPlacement="bottom"
              onClick={modeAction}
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
          {isMini ? null : (
            <IconButton
              size="m"
              tone="primary"
              icon={<IconExitFullscreenOutline16 />}
              aria-label={t('restorePanel')}
              tooltip={t('restorePanel')}
              tooltipPlacement="bottom"
              onClick={restorePanel}
            />
          )}
          <IconButton
            size="m"
            tone="primary"
            icon={modeIcon}
            aria-label={modeLabel}
            tooltip={modeLabel}
            tooltipPlacement="bottom"
            onClick={modeAction}
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
