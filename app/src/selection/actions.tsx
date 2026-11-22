import {
  IconChecklistOutline16,
  IconCopyOutline16,
  IconGlobeLanguageOutline14,
  IconSearchOutline16,
  IconThinkOutline16,
} from '../components/base/icons.js'
import { useTranslate } from '../i18n/index.js'

export const selectionActions = [
  { id: 'explain', labelKey: 'selectionExplain', icon: <IconThinkOutline16 /> },
  { id: 'summarize', labelKey: 'selectionSummarize', icon: <IconChecklistOutline16 /> },
  { id: 'translate', labelKey: 'selectionTranslate', icon: <IconGlobeLanguageOutline14 /> },
  { id: 'copy', labelKey: 'selectionCopy', icon: <IconCopyOutline16 /> },
  { id: 'search', labelKey: 'selectionSearch', icon: <IconSearchOutline16 /> },
] as const

/** 文字选区和截图选区共用的动作条。 */
export const SelectionActions = ({
  onAction,
  className = 'bar',
}: {
  onAction: (action: string) => void
  className?: string
}) => {
  const t = useTranslate()
  return (
    <div className={className} role="toolbar" aria-label={t('selectionToolbar')}>
      {selectionActions.map((action) => (
        <button
          key={action.id}
          type="button"
          className="action"
          aria-label={t(action.labelKey)}
          onPointerDown={(event) => event.stopPropagation()}
          onClick={(event) => {
            event.stopPropagation()
            onAction(action.id)
          }}
        >
          {action.icon}
          <span>{t(action.labelKey)}</span>
        </button>
      ))}
    </div>
  )
}
