import { invoke } from '@tauri-apps/api/core'
import { useTranslate } from '../../i18n/index.js'
import { IconButton } from '../base/IconButton.js'
import { IconExitFullscreenOutline16, IconMinimizeOutline16 } from '../base/icons.js'
import styles from './WindowControls.module.css'

/** 放大模式顶栏控制：恢复迷你 / 隐藏到悬浮球 */
export const WindowControls = () => {
  const t = useTranslate()
  return (
    <div className={styles.controls}>
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
        icon={<IconMinimizeOutline16 />}
        aria-label={t('closePanel')}
        tooltip={t('closePanel')}
        tooltipPlacement="bottom"
        onClick={() => void invoke('close_panel')}
      />
    </div>
  )
}
