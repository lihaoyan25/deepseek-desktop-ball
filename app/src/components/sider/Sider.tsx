import clsx from 'clsx'
import { useEffect, useMemo, useState, type MouseEvent } from 'react'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { useTranslate } from '../../i18n/index.js'
import brand from '../../assets/deepseek-brand.svg'
import {
  IconCloseOutline16,
  IconPanelLeftOutline16,
  IconPinOutline16,
  IconSearchOutline16,
  IconSettingsOutline16,
  IconTrashOutline16,
  NewChatIcon,
} from '../base/icons.js'
import { setSiderCollapsed } from '../../services/settings.js'
import { groupSessions } from '../../services/chat/sessionGroups.js'
import {
  newChat,
  removeSession,
  selectSession,
  setSessionsPinned,
  useChatStore,
} from '../../services/chat/store.js'
import { Button } from '../base/Button.js'
import { IconButton } from '../base/IconButton.js'
import { Modal } from '../base/Modal.js'
import { showToast } from '../base/toastState.js'
import { openSettings } from '../settings/settingsDialogState.js'
import { closeMobileSider, useMobileSider } from './mobileSiderState.js'
import { ConversationSearch } from './ConversationSearch.js'
import { closeSiderSearch, openSiderSearch, useSiderSearch } from './siderSearchState.js'
import { SessionItem } from './SessionItem.js'
import styles from './Sider.module.css'

/** 展开的侧边栏：名称与收起按钮、开启新对话、按时间分组的会话列表、底部设置；移动端放在抽屉里。 */
export const Sider = ({
  mobile = false,
  collapsed = false,
}: {
  mobile?: boolean
  collapsed?: boolean
}) => {
  const t = useTranslate()
  const startDrag = (event: MouseEvent) => {
    if (event.button === 0) {
      void getCurrentWindow().startDragging()
    }
  }
  const sessions = useChatStore((state) => state.sessions)
  const activeId = useChatStore((state) => state.activeId)
  const [now, setNow] = useState(Date.now)
  const groups = useMemo(() => groupSessions(sessions, now, t), [sessions, now, t])
  const [selectMode, setSelectMode] = useState(false)
  const [selectedIds, setSelectedIds] = useState<Set<string>>(() => new Set())
  const searchOpen = useSiderSearch((state) => state.open)
  const [deletingIds, setDeletingIds] = useState<string[]>([])
  const [deletePending, setDeletePending] = useState(false)

  useEffect(() => {
    let timer: ReturnType<typeof setTimeout>
    const refresh = () => {
      clearTimeout(timer)
      const time = Date.now()
      setNow(time)
      const midnight = new Date(time).setHours(24, 0, 0, 0)
      timer = setTimeout(refresh, midnight - time)
    }
    const onVisibilityChange = () => {
      if (document.visibilityState === 'visible') {
        refresh()
      }
    }
    refresh()
    document.addEventListener('visibilitychange', onVisibilityChange)
    window.addEventListener('focus', refresh)
    return () => {
      clearTimeout(timer)
      document.removeEventListener('visibilitychange', onVisibilityChange)
      window.removeEventListener('focus', refresh)
    }
  }, [])

  const selectedSessions = useMemo(
    () => sessions.filter((session) => selectedIds.has(session.id)),
    [sessions, selectedIds]
  )
  const allSelectedPinned =
    selectedSessions.length > 0 && selectedSessions.every((session) => Boolean(session.pinnedAt))

  const clearSelection = () => {
    setSelectMode(false)
    setSelectedIds(new Set())
  }

  const enterSelectMode = (id?: string) => {
    setSelectMode(true)
    setSelectedIds(new Set(id ? [id] : []))
  }

  const toggleSelected = (id: string) => {
    setSelectedIds((current) => {
      const next = new Set(current)
      if (next.has(id)) {
        next.delete(id)
      } else {
        next.add(id)
      }
      return next
    })
  }

  const handlePin = async (id: string, pinned: boolean) => {
    if (!(await setSessionsPinned([id], pinned))) {
      showToast({ key: 'sessionPinFailed', type: 'error' })
    }
  }

  const handleBatchPin = async () => {
    if (selectedSessions.length === 0) {
      return
    }
    const pin = !allSelectedPinned
    if (await setSessionsPinned(selectedSessions.map((session) => session.id), pin)) {
      clearSelection()
    } else {
      showToast({ key: 'sessionPinFailed', type: 'error' })
    }
  }

  const confirmDelete = async () => {
    setDeletePending(true)
    let failed = 0
    for (const id of deletingIds) {
      if (!(await removeSession(id))) {
        failed += 1
      }
    }
    setDeletePending(false)
    setDeletingIds([])
    if (failed === 0) {
      clearSelection()
    } else {
      showToast({ key: 'sessionDeleteFailed', type: 'error' })
    }
  }

  const closeSearch = () => closeSiderSearch()

  const handleSearchSelect = (id: string) => {
    selectSession(id)
    closeSiderSearch()
    closeMobileSider()
  }

  if (searchOpen) {
    return (
      <div
        className={clsx(
          styles.sider,
          mobile && styles.mobile,
          collapsed && styles.collapsedSider,
          styles.searching
        )}
        aria-hidden={collapsed || undefined}
      >
        <ConversationSearch onClose={closeSearch} onSelect={handleSearchSelect} />
      </div>
    )
  }

  return (
    <div
      className={clsx(
        styles.sider,
        mobile && styles.mobile,
        collapsed && styles.collapsedSider
      )}
      aria-hidden={collapsed || undefined}
    >
      <div className={styles.top}>
        <div className={styles.appName} onMouseDown={startDrag}>
          <img className={styles.brand} src={brand} alt="DeepSeek" draggable={false} />
        </div>
        <div className={styles.dragSpacer} onMouseDown={startDrag} />
        <div className={styles.topActions}>
          {!selectMode ? (
            <IconButton
              size="m"
              icon={<IconSearchOutline16 />}
              tooltip={t('searchSessions')}
              tooltipPlacement="bottom"
              aria-label={t('searchSessions')}
              onClick={openSiderSearch}
            />
          ) : null}
          <IconButton
            size="m"
            icon={<IconPanelLeftOutline16 />}
            tooltip={t('collapseSiderbarButton')}
            tooltipPlacement="bottom"
            onClick={() => (mobile ? closeMobileSider() : setSiderCollapsed(true))}
          />
        </div>
      </div>
      {selectMode ? (
        <div className={styles.selectionHeader}>
          <span className={styles.selectionCount}>
            {t('selectedConversations', { count: selectedIds.size })}
          </span>
          <IconButton
            size="m"
            icon={<IconCloseOutline16 />}
            aria-label={t('cancel')}
            onClick={clearSelection}
          />
        </div>
      ) : (
        <button
          type="button"
          className={clsx(styles.settings, styles.newChat)}
          onClick={() => {
            newChat()
            closeMobileSider()
          }}
        >
          <span className={styles.settingsIcon}>
            <NewChatIcon />
          </span>
          <span className={styles.settingsLabel}>{t('newChat')}</span>
        </button>
      )}
      <div className={styles.listWrap}>
        <div className={styles.list}>
          {groups.map((group) => (
            <div key={group.key} className={styles.section}>
              <div className={styles.sectionHeader}>{group.label}</div>
              {group.sessions.map((session) => (
                <SessionItem
                  key={session.id}
                  session={session}
                  active={session.id === activeId}
                  selectMode={selectMode}
                  selected={selectedIds.has(session.id)}
                  onToggleSelect={() => toggleSelected(session.id)}
                  onPin={(pinned) => void handlePin(session.id, pinned)}
                  onMultiSelect={() => enterSelectMode(session.id)}
                  onDelete={() => setDeletingIds([session.id])}
                />
              ))}
            </div>
          ))}
        </div>
      </div>
      {selectMode ? (
        <div className={styles.selectionFooter}>
          <button
            type="button"
            className={styles.selectionAction}
            disabled={selectedSessions.length === 0}
            onClick={() => void handleBatchPin()}
          >
            <IconPinOutline16 />
            <span>{allSelectedPinned ? t('unpinSelected') : t('pinSelected')}</span>
          </button>
          <span className={styles.selectionDivider} />
          <button
            type="button"
            className={clsx(styles.selectionAction, styles.selectionDelete)}
            disabled={selectedSessions.length === 0}
            onClick={() => setDeletingIds(selectedSessions.map((session) => session.id))}
          >
            <IconTrashOutline16 />
            <span>{t('delete')}</span>
          </button>
        </div>
      ) : (
        <div className={styles.bottom}>
          <button type="button" className={styles.settings} onClick={() => openSettings('general')}>
            <span className={styles.settingsIcon}>
              <IconSettingsOutline16 />
            </span>
            <span className={styles.settingsLabel}>{t('settings')}</span>
          </button>
        </div>
      )}
      {deletingIds.length > 0 ? (
        <Modal
          title={
            deletingIds.length > 1
              ? t('deleteSelectedSessionsModalTitle', { count: deletingIds.length })
              : t('deleteSessionModalContentTitle')
          }
          onClose={() => {
            if (!deletePending) {
              setDeletingIds([])
            }
          }}
          footer={
            <>
              <Button
                variant="outlined"
                disabled={deletePending}
                onClick={() => setDeletingIds([])}
              >
                {t('cancel')}
              </Button>
              <Button variant="danger" disabled={deletePending} onClick={() => void confirmDelete()}>
                {t('delete')}
              </Button>
            </>
          }
        >
          {deletingIds.length > 1
            ? t('deleteSelectedSessionsDescription')
            : t('deleteSessionWithFilesDescription')}
        </Modal>
      ) : null}
    </div>
  )
}

/** 移动端抽屉：浮在页面上方，点空白区域关闭。 */
export const MobileSider = () => {
  const open = useMobileSider((state) => state.open)
  return (
    <>
      <div
        className={clsx(styles.drawerMask, !open && styles.drawerMaskHidden)}
        onClick={open ? closeMobileSider : undefined}
      />
      <div
        className={clsx(styles.drawer, !open && styles.drawerHidden)}
        aria-hidden={!open}
      >
        <Sider mobile />
      </div>
    </>
  )
}
