'use client'

import { useAppProvider } from '@/app/app-provider'

export default function Hamburger() {

  const { sidebarOpen, setSidebarOpen } = useAppProvider()

  return (
    <button
      className="hamburger"
      aria-controls="sidebar"
      aria-expanded={sidebarOpen}
      onClick={() => setSidebarOpen(!sidebarOpen)}
    >
      <span className="sr-only">Menu</span>
      <svg className="w-6 h-6 fill-slate-600 dark:fill-slate-400" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
        <rect x="4" y="5" width="16" height="2"></rect>
        <rect x="4" y="11" width="16" height="2"></rect>
        <rect x="4" y="17" width="16" height="2"></rect>
      </svg>
    </button>
  )
}