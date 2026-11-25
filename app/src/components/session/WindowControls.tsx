import { invoke } from '@tauri-apps/api/core'
import { useTranslate } from '../../i18n/index.js'
import { IconButton } from '../base/IconButton.js'
import {
  IconExitFullscreenOutline16,
  IconMaximizeOutline16,
  IconMinimizeOutline16,
  IconWindowMaximizeOutline16,
  IconWindowRestoreOutline16,
} from '../base/icons.js'
import type { PanelMode } from '../../app/panelMode.js'
import styles from './WindowControls.module.css'

/** 桌面顶栏控制：迷你模式切换放大，放大模式恢复迷你，然后隐藏到悬浮球。 */
export const WindowControls = ({ mode }: { mode: PanelMode }) => {
  const t = useTranslate()
  const isMini = mode === 'mini'
  const isFull = mode === 'full'
  const windowLabel = isFull ? t('restoreWindow') : t('maximizeWindow')
  return (
    <div className={styles.controls}>
      {isMini ? (
        <IconButton
          size="m"
          tone="primary"
          icon={<IconMaximizeOutline16 />}
          aria-label={t('maximizePanel')}
          tooltip={t('maximizePanel')}
          tooltipPlacement="bottom"
          onClick={() => void invoke('maximize_panel')}
        />
      ) : (
        <>
          <IconButton
            size="m"
            tone="primary"
            icon={<IconExitFullscreenOutline16 />}
            aria-label={t('restorePanel')}
            tooltip={t('restorePanel')}
            tooltipPlacement="bottom"
            onClick={() => void invoke('restore_panel')}
          />
          <IconButton
            size="m"
            tone="primary"
            icon={
              isFull ? <IconWindowRestoreOutline16 /> : <IconWindowMaximizeOutline16 />
            }
            aria-label={windowLabel}
            tooltip={windowLabel}
            tooltipPlacement="bottom"
            onClick={() => void invoke('toggle_full_panel')}
          />
        </>
      )}
      <IconButton
        size="m"
        tone="primary"
        icon={<IconMinimizeOutline16 />}
        aria-label={t('closePanel')}
        tooltip={t('closePanel')}
        tooltipPlacement="bottom"
        onClick={() => void invoke('close_panel')}
      />
    </div>
  )
}
