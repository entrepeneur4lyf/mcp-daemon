import { useState } from 'react'

interface SidebarLinkGroupProps {
  children: (handleClick: () => void, openGroup: boolean) => React.ReactNode
  open: boolean
}

export default function SidebarLinkGroup({
  children,
  open
}: SidebarLinkGroupProps) {
  const [openGroup, setOpenGroup] = useState<boolean>(open)

  const handleClick = () => {
    setOpenGroup(!openGroup);
  }  

  return (
    <li className="mb-1">
      {children(handleClick, openGroup)}
    </li>
  )
}
