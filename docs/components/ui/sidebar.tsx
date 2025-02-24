'use client'

import { useRef, useEffect } from 'react'
import { useAppProvider } from '@/app/app-provider'
import { useSelectedLayoutSegments } from 'next/navigation'
import Link from 'next/link'
import SidebarLink from './sidebar-link'
import SidebarLinkGroup from './sidebar-link-group'
import SidebarLinkSubgroup from './sidebar-link-subgroup'

export default function SupportSidebar() {
  const sidebar = useRef<HTMLDivElement>(null)
  const { sidebarOpen, setSidebarOpen } = useAppProvider()
  const segments = useSelectedLayoutSegments()

  // close on click outside
  useEffect(() => {
    const clickHandler = ({ target }: { target: EventTarget | null }): void => {
      if (!sidebar.current) return;
      if (!sidebarOpen || sidebar.current.contains(target as Node)) return
      setSidebarOpen(false)
    }
    document.addEventListener('click', clickHandler)
    return () => document.removeEventListener('click', clickHandler)
  })

  // close if the esc key is pressed
  useEffect(() => {
    const keyHandler = ({ keyCode }: { keyCode: number }): void => {
      if (!sidebarOpen || keyCode !== 27) return
      setSidebarOpen(false)
    }
    document.addEventListener('keydown', keyHandler)
    return () => document.removeEventListener('keydown', keyHandler)
  })  

  return (
    <div>
        {/* Backdrop */}
        <div
          className={`md:hidden fixed inset-0 bg-slate-900/20 z-10 lg:hidden lg:z-auto transition-opacity duration-200 ${
            sidebarOpen ? "opacity-100" : "opacity-0 pointer-events-none"
          }`}
          aria-hidden="true"
        ></div>       

        {/* Sidebar */}
        <aside
          ref={sidebar}
          id="sidebar"
          className={`fixed left-0 top-0 bottom-0 w-64 h-screen border-r border-slate-200 md:left-auto md:shrink-0 z-10 dark:border-slate-800 dark:bg-slate-900 transform transition-transform ease-out duration-200 ${sidebarOpen ? "max-md:translate-x-0" : "max-md:-translate-x-full max-md:opacity-0"}`}
        >
          {/* Gradient bg displaying on light layout only */}
          <div
            className="absolute inset-0 -left-[9999px] bg-linear-to-b from-slate-50 to-white pointer-events-none -z-10 dark:hidden"
            aria-hidden="true"
          ></div>

          <div className="fixed top-0 bottom-0 w-64 px-4 sm:px-6 md:pl-0 md:pr-8 overflow-y-auto no-scrollbar">
            <div className="pt-24 md:pt-28 pb-8">
              {/* Docs nav */}
              <nav className="md:block">
                <ul className="text-sm">
                  {/* 1st level */}
                  <SidebarLinkGroup open={segments.includes('documentation')}>
                    {(handleClick, open) => {
                      return (
                        <>
                          <a
                            href="#0"
                            className={`relative flex items-center font-[650] text-slate-800 p-1 before:absolute before:inset-0 before:rounded-sm before:bg-linear-to-tr before:from-blue-400 before:to-purple-500 before:opacity-20 before:-z-10 before:pointer-events-none dark:text-slate-200 ${!segments.includes('documentation') && 'before:hidden'
                              }`}
                            onClick={(e) => {
                              e.preventDefault();
                              handleClick();
                            }}
                          >
                            <svg className="mr-3 shrink-0" width="24" height="24" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
                              <path
                                className="fill-blue-400"
                                d="M19.888 7.804a.88.88 0 0 0-.314-.328l-7.11-4.346a.889.889 0 0 0-.927 0L4.426 7.476a.88.88 0 0 0-.314.328L12 12.624l7.888-4.82Z"
                              />
                              <path
                                className="fill-white dark:fill-slate-800"
                                d="M4.112 7.804a.889.889 0 0 0-.112.43v7.892c0 .31.161.597.426.758l7.11 4.346c.14.085.3.13.464.13v-8.736l-7.888-4.82Z"
                              />
                              <path
                                className="fill-blue-600"
                                d="M19.888 7.804c.073.132.112.28.112.43v7.892c0 .31-.161.597-.426.758l-7.11 4.346c-.14.085-.3.13-.464.13v-8.736l7.888-4.82Z"
                              />
                            </svg>
                            <span>Documentation</span>
                          </a>
                          <ul className={`mb-3 ml-4 pl-6 border-l border-slate-200 dark:border-slate-800 ${!open && 'hidden'}`}>
                            <li className="mt-3">
                              <SidebarLink href="/documentation/fundamentals">
                                Fundamentals
                              </SidebarLink>
                            </li>
                            <li className="mt-3">
                              <a className="flex items-center space-x-3 font-medium text-slate-800 dark:text-slate-200" href="#0">
                                Methods and Parameters
                              </a>
                            </li>
                            <li className="mt-3">
                              <a className="flex items-center space-x-3 font-medium text-slate-800 dark:text-slate-200" href="#0">
                                Merge Fields
                              </a>
                            </li>
                            <SidebarLinkSubgroup title="Alternative Schemas" open={segments.includes('alternative-scheme')}>
                              <li className="mt-3">
                                <a className="flex items-center space-x-3 font-medium text-slate-800 dark:text-slate-200" href="#0">
                                  File system
                                </a>
                              </li>
                              <li className="mt-3">
                                <a className="flex items-center space-x-3 font-medium text-slate-800 dark:text-slate-200" href="#0">
                                  Describing responses
                                </a>
                              </li>
                            </SidebarLinkSubgroup>
                            <SidebarLinkSubgroup title="E-Commerce" open={segments.includes('ecommerce')}>
                              <li className="mt-3">
                                <a className="flex items-center space-x-3 font-medium text-slate-800 dark:text-slate-200" href="#0">
                                  Path parameters
                                </a>
                              </li>
                              <li className="mt-3">
                                <a className="flex items-center space-x-3 font-medium text-slate-800 dark:text-slate-200" href="#0">
                                  Query string parameters
                                </a>
                              </li>
                            </SidebarLinkSubgroup>
                            <li className="mt-3">
                              <a className="flex items-center space-x-3 font-medium text-slate-800 dark:text-slate-200" href="#0">
                                Account Exports
                              </a>
                            </li>
                            <li className="mt-3">
                              <a className="flex items-center space-x-3 font-medium text-slate-800 dark:text-slate-200" href="#0">
                                Integrations
                              </a>
                            </li>
                            <li className="mt-3">
                              <a className="flex items-center space-x-3 font-medium text-slate-800 dark:text-slate-200" href="#0">
                                Add a contact
                              </a>
                            </li>
                          </ul>
                        </>
                      )
                    }}
                  </SidebarLinkGroup>
                  {/* 1st level */}
                  <SidebarLinkGroup open={segments.includes('guides')}>
                    {(handleClick, open) => {
                      return (
                        <>
                          <a
                            href="#0"
                            className={`relative flex items-center font-[650] text-slate-800 p-1 before:absolute before:inset-0 before:rounded-sm before:bg-linear-to-tr before:from-blue-400 before:to-purple-500 before:opacity-20 before:-z-10 before:pointer-events-none dark:text-slate-200 ${!segments.includes('guides') && 'before:hidden'
                              }`}
                            onClick={(e) => {
                              e.preventDefault();
                              handleClick();
                            }}
                          >
                            <svg className="mr-3 shrink-0" width="24" height="24" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
                              <path
                                className="fill-purple-400"
                                d="M19.888 7.804a.88.88 0 0 0-.314-.328l-7.11-4.346a.889.889 0 0 0-.927 0L4.426 7.476a.88.88 0 0 0-.314.328L12 12.624l7.888-4.82Z"
                              />
                              <path
                                className="fill-white dark:fill-slate-800"
                                d="M4.112 7.804a.889.889 0 0 0-.112.43v7.892c0 .31.161.597.426.758l7.11 4.346c.14.085.3.13.464.13v-8.736l-7.888-4.82Z"
                              />
                              <path
                                className="fill-purple-600"
                                d="M19.888 7.804c.073.132.112.28.112.43v7.892c0 .31-.161.597-.426.758l-7.11 4.346c-.14.085-.3.13-.464.13v-8.736l7.888-4.82Z"
                              />
                            </svg>
                            <span>Guides / Tutorials</span>
                          </a>
                          <ul className={`mb-3 ml-4 pl-6 border-l border-slate-200 dark:border-slate-800 ${!open && 'hidden'}`}>
                            <li className="mt-3">
                              <SidebarLink href="/guides/marketing-api-quick-start">
                                Marketing API Quick Start
                              </SidebarLink>
                            </li>
                            <li className="mt-3">
                              <a className="flex items-center space-x-3 font-medium text-slate-800 dark:text-slate-200" href="#0">
                                Create an account
                              </a>
                            </li>
                            <li className="mt-3">
                              <a className="flex items-center space-x-3 font-medium text-slate-800 dark:text-slate-200" href="#0">
                                Generate your API key
                              </a>
                            </li>
                            <li className="mt-3">
                              <a className="flex items-center space-x-3 font-medium text-slate-800 dark:text-slate-200" href="#0">
                                Make your first API call
                              </a>
                            </li>
                            <li className="mt-3">
                              <a className="flex items-center space-x-3 font-medium text-slate-800 dark:text-slate-200" href="#0">
                                Next steps
                              </a>
                            </li>                            
                          </ul>
                        </>
                      )
                    }}
                  </SidebarLinkGroup>
                  {/* 1st level */}
                  <li className="mb-1">
                    <Link
                      href="/help/how-can-we-help"
                      className={`relative flex items-center font-[650] text-slate-800 p-1 before:absolute before:inset-0 before:rounded-sm before:bg-linear-to-tr before:from-blue-400 before:to-purple-500 before:opacity-20 before:-z-10 before:pointer-events-none dark:text-slate-200 ${!segments.includes('help') && 'before:hidden'}`}
                      onClick={() => setSidebarOpen(false)}
                    >
                      <svg className="mr-3 shrink-0" width="24" height="24" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
                        <path
                          className="fill-sky-400"
                          d="M19.888 7.804a.88.88 0 0 0-.314-.328l-7.11-4.346a.889.889 0 0 0-.927 0L4.426 7.476a.88.88 0 0 0-.314.328L12 12.624l7.888-4.82Z"
                        />
                        <path
                          className="fill-white dark:fill-slate-800"
                          d="M4.112 7.804a.889.889 0 0 0-.112.43v7.892c0 .31.161.597.426.758l7.11 4.346c.14.085.3.13.464.13v-8.736l-7.888-4.82Z"
                        />
                        <path
                          className="fill-sky-600"
                          d="M19.888 7.804c.073.132.112.28.112.43v7.892c0 .31-.161.597-.426.758l-7.11 4.346c-.14.085-.3.13-.464.13v-8.736l7.888-4.82Z"
                        />
                      </svg>
                      <span>Help / Support</span>
                    </Link>
                  </li>                  
                </ul>
              </nav>
            </div>
          </div>
        </aside>
      </div>
  )
}