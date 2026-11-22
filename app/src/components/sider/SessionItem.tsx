import clsx from 'clsx'
import { memo, useState } from 'react'
import { useTranslate } from '../../i18n/index.js'
import {
  IconCheckOutline16,
  IconChecklistOutline16,
  IconEditOutline16,
  IconEllipsisOutline16,
  IconPinOutline16,
  IconTrashOutline16,
} from '../base/icons.js'
import { renameSession, selectSession } from '../../services/chat/store.js'
import type { SessionSummary } from '../../services/chat/types.js'
import { IconButton } from '../base/IconButton.js'
import { Menu } from '../base/Menu.js'
import { closeMobileSider } from './mobileSiderState.js'
import styles from './SessionItem.module.css'

export const SessionItem = memo(function SessionItem({
  session,
  active,
  selectMode = false,
  selected = false,
  onToggleSelect,
  onPin,
  onMultiSelect,
  onDelete,
}: {
  session: SessionSummary
  active: boolean
  selectMode?: boolean
  selected?: boolean
  onToggleSelect?: () => void
  onPin?: (pinned: boolean) => void
  onMultiSelect?: () => void
  onDelete: () => void
}) {
  const t = useTranslate()
  const [editing, setEditing] = useState(false)
  const displayTitle = session.title || t('untitledChat')
  const pinned = Boolean(session.pinnedAt)

  if (editing) {
    const finish = (title: string) => {
      setEditing(false)
      if (title.trim() && title.trim() !== displayTitle) {
        renameSession(session.id, title.trim())
      }
    }
    return (
      <input
        className={styles.renameInput}
        defaultValue={displayTitle}
        autoFocus
        onFocus={(event) => event.target.select()}
        onBlur={(event) => finish(event.target.value)}
        onKeyDown={(event) => {
          if (event.key === 'Enter' && !event.nativeEvent.isComposing) {
            finish(event.currentTarget.value)
          } else if (event.key === 'Escape') {
            setEditing(false)
          }
        }}
      />
    )
  }

  if (selectMode) {
    return (
      <div
        className={clsx(styles.item, styles.selectItem, selected && styles.selected)}
        role="checkbox"
        aria-checked={selected}
        tabIndex={0}
        onClick={onToggleSelect}
        onKeyDown={(event) => {
          if (event.key === 'Enter' || event.key === ' ') {
            event.preventDefault()
            onToggleSelect?.()
          }
        }}
      >
        <span className={clsx(styles.checkbox, selected && styles.checkboxChecked)}>
          {selected ? <IconCheckOutline16 /> : null}
        </span>
        <span className={styles.title}>{displayTitle}</span>
      </div>
    )
  }

  return (
    <div
      className={clsx(styles.item, active && styles.active)}
      onClick={() => {
        selectSession(session.id)
        closeMobileSider()
      }}
    >
      <div className={styles.title}>{displayTitle}</div>
      <div className={styles.action} onClick={(event) => event.stopPropagation()}>
        <Menu
          options={[
            {
              key: 'rename',
              label: t('sessionItemMenuRenameOption'),
              icon: <IconEditOutline16 />,
              onSelect: () => setEditing(true),
            },
            {
              key: 'pin',
              label: pinned ? t('sessionItemMenuUnpinOption') : t('sessionItemMenuPinOption'),
              icon: <IconPinOutline16 />,
              onSelect: () => onPin?.(!pinned),
            },
            {
              key: 'multi',
              label: t('sessionItemMenuMultiSelectOption'),
              icon: <IconChecklistOutline16 />,
              onSelect: () => onMultiSelect?.(),
            },
            {
              key: 'delete',
              label: t('delete'),
              icon: <IconTrashOutline16 />,
              danger: true,
              onSelect: onDelete,
            },
          ]}
        >
          {({ toggle }) => (
            <IconButton icon={<IconEllipsisOutline16 />} aria-label={t('more')} onClick={toggle} />
          )}
        </Menu>
      </div>
    </div>
  )
})
