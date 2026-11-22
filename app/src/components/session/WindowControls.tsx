import { invoke } from '@tauri-apps/api/core'
import { useTranslate } from '../../i18n/index.js'
import { IconButton } from '../base/IconButton.js'
import {
  IconExitFullscreenOutline16,
  IconMaximizeOutline16,
  IconMinimizeOutline16,
} from '../base/icons.js'
import type { PanelMode } from '../../app/panelMode.js'
import styles from './WindowControls.module.css'

/** 桌面顶栏控制：迷你模式切换放大，放大模式恢复迷你，然后隐藏到悬浮球。 */
export const WindowControls = ({ mode }: { mode: PanelMode }) => {
  const t = useTranslate()
  const isMax = mode === 'max'
  const modeLabel = isMax ? t('restorePanel') : t('maximizePanel')
  return (
    <div className={styles.controls}>
      <IconButton
        size="m"
        tone="primary"
        icon={isMax ? <IconExitFullscreenOutline16 /> : <IconMaximizeOutline16 />}
        aria-label={modeLabel}
        tooltip={modeLabel}
        tooltipPlacement="bottom"
        onClick={() => void invoke(isMax ? 'restore_panel' : 'maximize_panel')}
      />
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
