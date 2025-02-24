'use client'

import { useState, useEffect } from 'react'

export default function SecondaryNav() {

  const [targets, setTargets] = useState<HTMLElement[]>([])
  const [links, setLinks] = useState<HTMLElement[]>([])

  const scrollSpy = () => {
    const links = document.querySelectorAll('[data-scrollspy-link]') as NodeListOf<HTMLElement>
    if (links.length < 1) return
    const addActive = (i: number) => {
      const link = links[i] ? links[i] : links[0]
      link.classList.add('scrollspy-active')
    }
    const removeActive = (i: number) => {
      links[i].classList.remove('scrollspy-active')
    }
    const removeAllActive = () => [...Array(targets.length).keys()].forEach((link) => removeActive(link))
    const targetMargin = 100
    let currentActive = 0
    addActive(0)
    // listen for scroll events
    window.addEventListener('scroll', () => {
      const current = targets.length - [...targets].reverse().findIndex((target) => window.scrollY >= target.offsetTop - targetMargin) - 1
      if (current !== currentActive) {
        removeAllActive()
        currentActive = current
        addActive(current)
      }
    })
  }
  
  // select targets
  useEffect(() => {
    const targets = document.querySelectorAll('h2') as NodeListOf<HTMLElement>
    setTargets(Array.from(targets))
  }, [])  

  // populate the right sidebar
  useEffect(() => {
    let linksArray: HTMLElement[]  = []
    targets.map((target) => {
      linksArray.push(target)
    })
    setLinks(linksArray)
  }, [targets])

  // init scrollspy
  useEffect(() => {
    scrollSpy()
  }, [links])

  return (
    <div className="hidden xl:block w-48 shrink-0">
      {links.length > 0 &&
        <nav>
          <div className="fixed bottom-0 h-[calc(100vh-5rem)] w-48 overflow-y-auto pt-32 pb-8 no-scrollbar">
            <div className="border-l border-slate-200 dark:border-slate-800">
              <div className="text-xs font-[650] text-slate-400 uppercase pl-4 py-1.5 dark:text-slate-200">On this page</div>
              <ul className="text-sm">
                {links.map((link, linkIndex) => (
                  <li key={linkIndex}>
                    <a
                      data-scrollspy-link
                      className="relative block font-normal text-slate-600 pl-4 py-1.5 before:absolute before:-left-px before:top-2 before:bottom-2 before:w-0.5"
                      href={`#${link.id}`}
                    >
                      {link.textContent}
                    </a>
                  </li>
                ))}
              </ul>
            </div>
          </div>
        </nav>
      }
    </div>
  )
}