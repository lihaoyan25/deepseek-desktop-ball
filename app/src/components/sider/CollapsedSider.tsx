import { IconPanelLeftOutline16, IconSearchOutline16, NewChatIcon } from '../base/icons.js'
import { useTranslate } from '../../i18n/index.js'
import { setSiderCollapsed } from '../../services/settings.js'
import { newChat } from '../../services/chat/store.js'
import { IconButton } from '../base/IconButton.js'
import { openSiderSearch } from './siderSearchState.js'
import styles from './Sider.module.css'

/** 收起后左上角的按钮组 */
export const CollapsedSider = ({ visible }: { visible: boolean }) => {
  const t = useTranslate()
  return (
    <div
      className={`${styles.collapsed} ${visible ? '' : styles.collapsedHidden}`}
      aria-hidden={!visible}
    >
      <div className={styles.collapsedGroup}>
        <IconButton
          size="m"
          tone="primary"
          icon={<IconPanelLeftOutline16 />}
          tooltip={t('expandSiderbarButton')}
          onClick={() => setSiderCollapsed(false)}
        />
        <IconButton
          size="m"
          tone="primary"
          icon={<IconSearchOutline16 />}
          tooltip={t('searchSessions')}
          onClick={() => {
            setSiderCollapsed(false)
            openSiderSearch()
          }}
        />
        <IconButton
          size="m"
          tone="primary"
          icon={<NewChatIcon />}
          tooltip={t('newChat')}
          onClick={newChat}
        />
      </div>
    </div>
  )
}
