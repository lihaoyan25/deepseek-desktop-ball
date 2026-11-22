import { useEffect, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { openUrl } from '@tauri-apps/plugin-opener'
import { useTranslate } from '../i18n/index.js'
import { useSettings } from '../services/settings.js'
import { useIsMobile, useMobileViewport, usePreventPageScroll } from './viewport.js'
import type { PanelMode } from './panelMode.js'
import {
  bindChatPersistence,
  initChat,
  newChat,
  retrySaveSessions,
  selectSession,
  sendMessage,
  useChatStore,
} from '../services/chat/store.js'
import { setDraftText, addDraftFiles, getDraft } from '../services/chat/drafts.js'
import { useUploads } from '../services/attachments.js'
import { SearchPanel } from '../components/search/SearchPanel.js'
import { closeSearchPanel } from '../components/search/searchPanelState.js'
import { useSearchPanel } from '../components/search/searchPanelState.js'
import { SessionView } from '../components/session/SessionView.js'
import { MobileHeader } from '../components/session/MobileHeader.js'
import { WindowControls } from '../components/session/WindowControls.js'
import { WindowResizeHandles } from '../components/session/WindowResizeHandles.js'
import { Welcome } from '../components/session/Welcome.js'
import { ensureApiKey } from '../components/settings/settingsDialogState.js'
import { SettingsDialog } from '../components/settings/SettingsDialog.js'
import { Toast } from '../components/base/Toast.js'
import { showToast } from '../components/base/toastState.js'
import { onUploadNotice } from '../services/attachments.js'
import { Button } from '../components/base/Button.js'
import { stopReadAloud } from '../services/speech/readAloud.js'
import { CollapsedSider } from '../components/sider/CollapsedSider.js'
import { closeMobileSider } from '../components/sider/mobileSiderState.js'
import { MobileSider, Sider } from '../components/sider/Sider.js'
import styles from './App.module.css'

export const App = () => {
  const t = useTranslate()
  const isMobile = useIsMobile()
  const locale = useSettings((state) => state.locale)
  const selectionAssistant = useSettings((state) => state.selectionAssistant)
  // 移动端侧边栏以独立抽屉管理开合状态
  const siderCollapsed = useSettings((state) => state.siderCollapsed) && !isMobile
  const ready = useChatStore((state) => state.ready)
  const initError = useChatStore((state) => state.initError)
  const saveError = useChatStore((state) => Object.values(state.sessionSaveErrors).some(Boolean))
  const importingHistory = useChatStore((state) => state.importingHistory)
  const session = useChatStore((state) =>
    state.activeId !== null && Object.hasOwn(state.sessionCache, state.activeId)
      ? state.sessionCache[state.activeId]
      : undefined
  )
  const sessionLoadError = useChatStore((state) =>
    state.activeId === null ? false : state.sessionLoadErrors[state.activeId] === true
  )
  const frameRef = useMobileViewport(isMobile)
  const streaming = useChatStore((state) =>
    state.activeId === null ? false : Object.hasOwn(state.streaming, state.activeId)
  )
  const activeId = useChatStore((state) => state.activeId)
  const searchEntry = useSearchPanel((state) => state.entry)
  const [panelMode, setPanelMode] = useState<PanelMode>('mini')

  usePreventPageScroll(isMobile)

  useEffect(() => {
    document.documentElement.lang = locale
  }, [locale])

  // 文件数、存储空间达到上限时，附件上只显示「上传失败」，原因用 toast 说明
  useEffect(() => onUploadNotice(showToast), [])
  useEffect(bindChatPersistence, [])

  // 搜索结果面板只属于当前会话
  useEffect(closeSearchPanel, [activeId])
  // 切换会话时停止朗读
  useEffect(stopReadAloud, [activeId])
  // 移动端选中会话、切换布局后收起抽屉
  useEffect(closeMobileSider, [activeId, isMobile])

  useEffect(() => {
    void initChat()
    // 首次打开先在设置里输入 API Key
    ensureApiKey()
  }, [])

  useEffect(() => {
    void invoke('set_selection_assistant_enabled', { enabled: selectionAssistant })
  }, [selectionAssistant])

  useEffect(() => {
    let alive = true
    let unlisten: (() => void) | undefined
    void listen<{ action: string; dataUrl: string }>('selection-image-action', (event) => {
      const { action, dataUrl } = event.payload
      const prompt =
        action === 'summarize'
          ? '请总结这张截图。'
          : action === 'translate'
            ? '请翻译截图中的文字。'
            : action === 'search'
              ? '请搜索并总结这张截图的内容。'
              : '请解释这张截图。'
      void (async () => {
        const apiKey = useSettings.getState().apiKey
        if (!apiKey) {
          ensureApiKey()
          return
        }
        const response = await fetch(dataUrl)
        const blob = await response.blob()
        const file = new File([blob], 'screenshot.png', { type: 'image/png' })
        newChat()
        const error = addDraftFiles(null, [file], apiKey)
        if (error) {
          showToast(error)
          return
        }
        setDraftText(null, prompt)
        let sent = false
        const deadline = Date.now() + 30_000
        const waitForUpload = () => {
          if (sent || !alive) {
            return
          }
          const draft = getDraft(null)
          const tasks = useUploads.getState().tasks
          if (draft.attachments.some((attachment) => tasks[attachment.id]?.status === 'error')) {
            showToast(t('uploadFileStatusFailed'))
            return
          }
          if (
            draft.attachments.length > 0 &&
            draft.attachments.every((attachment) => tasks[attachment.id]?.status === 'done')
          ) {
            sent = true
            const previousSearch = useSettings.getState().search
            const useSearch = action === 'search'
            if (useSearch) {
              useSettings.setState({ search: true })
            }
            sendMessage(null)
            if (useSearch) {
              useSettings.setState({ search: previousSearch })
            }
            return
          }
          if (Date.now() > deadline) {
            showToast(t('uploadFileStatusFailed'))
            return
          }
          window.setTimeout(waitForUpload, 100)
        }
        waitForUpload()
      })().catch(() => {
        showToast(t('uploadFileStatusFailed'))
      })
    }).then((stop) => {
      if (alive) {
        unlisten = stop
      } else {
        stop()
      }
    })
    return () => {
      alive = false
      unlisten?.()
    }
  }, [t])

  useEffect(() => {
    let alive = true
    let unlisten: (() => void) | undefined
    void listen<{ action: string; text: string }>('selection-action', (event) => {
      const { action, text } = event.payload
      if (!text) {
        return
      }
      const target = /[\u3400-\u9fff]/.test(text) ? '英文' : '中文'
      const prompt =
        action === 'summarize'
          ? `总结一下：\n\n“${text}”`
          : action === 'translate'
            ? `把下面内容翻译成${target}：\n\n“${text}”`
            : action === 'search'
              ? `搜索并总结：\n\n“${text}”`
              : `解释一下：\n\n“${text}”`
      const previousSearch = useSettings.getState().search
      const useSearch = action === 'search'
      if (useSearch) {
        useSettings.setState({ search: true })
      }
      newChat()
      setDraftText(null, prompt)
      sendMessage(null)
      if (useSearch) {
        useSettings.setState({ search: previousSearch })
      }
    }).then((stop) => {
      if (alive) {
        unlisten = stop
      } else {
        stop()
      }
    })
    return () => {
      alive = false
      unlisten?.()
    }
  }, [])

  useEffect(() => {
    let alive = true
    let unlisten: (() => void) | undefined
    const update = (mode: unknown) => {
      if (mode === 'mini' || mode === 'max') {
        setPanelMode(mode)
      }
    }
    void invoke<string>('panel_mode')
      .then(update)
      .catch(() => {})
    void listen<string>('panel-mode', (event) => update(event.payload)).then((stop) => {
      if (alive) {
        unlisten = stop
      } else {
        stop()
      }
    })
    return () => {
      alive = false
      unlisten?.()
    }
  }, [])

  useEffect(() => {
    const isExternal = (url: string) => /^https?:\/\//i.test(url)
    const openExternal = (url: string) => {
      if (isExternal(url)) {
        void openUrl(url).catch(() => {})
      }
    }
    const onClick = (event: MouseEvent) => {
      const target = event.target as HTMLElement | null
      const anchor = target?.closest('a[href]') as HTMLAnchorElement | null
      if (!anchor || !isExternal(anchor.href)) {
        return
      }
      event.preventDefault()
      event.stopPropagation()
      openExternal(anchor.href)
    }
    const originalOpen = window.open
    window.open = ((url?: string | URL, target?: string, features?: string) => {
      const href = typeof url === 'string' ? url : (url?.toString() ?? '')
      if (isExternal(href)) {
        openExternal(href)
        return null
      }
      return originalOpen.call(window, url as string, target, features)
    }) as typeof window.open
    document.addEventListener('click', onClick, true)
    return () => {
      document.removeEventListener('click', onClick, true)
      window.open = originalOpen
    }
  }, [])

  return (
    <>
      <div ref={frameRef} className={styles.frame}>
        {isMobile ? <MobileSider /> : <Sider collapsed={siderCollapsed} />}
        <main className={styles.main}>
          {saveError ? (
            <div className={styles.saveError} role="alert">
              <span>{t('sessionSaveFailed')}</span>
              <Button
                variant="outlined"
                size="s"
                disabled={importingHistory}
                onClick={retrySaveSessions}
              >
                {t('retry')}
              </Button>
            </div>
          ) : null}
          <div className={styles.content}>
            {!isMobile ? <CollapsedSider visible={siderCollapsed} /> : null}
            {!ready || (activeId !== null && !session) ? (
              <div className={styles.loadingView}>
                {isMobile ? <MobileHeader mode={panelMode} /> : null}
                <div className={styles.loadingState} role="status" aria-live="polite">
                  <span>
                    {(!ready ? initError : sessionLoadError)
                      ? t(!ready ? 'sessionListLoadFailed' : 'sessionLoadFailed')
                      : t('sessionLoading')}
                  </span>
                  {(!ready ? initError : sessionLoadError) ? (
                    <Button
                      variant="outlined"
                      size="m"
                      onClick={() => {
                        if (!ready) {
                          void initChat()
                        } else if (activeId !== null) {
                          selectSession(activeId)
                        }
                      }}
                    >
                      {t('retry')}
                    </Button>
                  ) : null}
                </div>
              </div>
            ) : session ? (
              <SessionView
                session={session}
                streaming={streaming}
                siderCollapsed={siderCollapsed}
                panelMode={panelMode}
              />
            ) : (
              <Welcome panelMode={panelMode} />
            )}
          </div>
        </main>
        <SearchPanel />
        {!isMobile && searchEntry === null ? <WindowControls mode={panelMode} /> : null}
        <WindowResizeHandles />
      </div>
      <SettingsDialog />
      <Toast />
      <div className="window-border" />
    </>
  )
}
