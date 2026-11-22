import { create } from 'zustand'

type SiderSearchState = {
  open: boolean
}

export const useSiderSearch = create<SiderSearchState>(() => ({ open: false }))

export const openSiderSearch = () => useSiderSearch.setState({ open: true })
export const closeSiderSearch = () => useSiderSearch.setState({ open: false })
