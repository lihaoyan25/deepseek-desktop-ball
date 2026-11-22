import { createRoot } from 'react-dom/client'
import { invoke } from '@tauri-apps/api/core'
import { setupTheme } from '../app/theme.js'
import { SelectionActions } from './actions.js'
import '../app/theme.css'
import '../app/global.css'
import './selection.css'

setupTheme()

const SelectionToolbar = () => (
  <SelectionActions onAction={(action) => void invoke('selection_action', { action }).catch(() => {})} />
)

createRoot(document.getElementById('root') as HTMLElement).render(<SelectionToolbar />)
