import clsx from 'clsx'
import styles from './Switch.module.css'

type SwitchProps = {
  checked: boolean
  onChange: (checked: boolean) => void
  ariaLabel: string
  className?: string
}

/** 与应用主题一致的开关：浅色时黑色开启轨道，深色时白色开启轨道。 */
export const Switch = ({ checked, onChange, ariaLabel, className }: SwitchProps) => (
  <button
    type="button"
    role="switch"
    aria-checked={checked}
    aria-label={ariaLabel}
    className={clsx(styles.switch, className)}
    onClick={() => onChange(!checked)}
  >
    <span className={styles.track}>
      <span className={styles.thumb} />
    </span>
  </button>
)
