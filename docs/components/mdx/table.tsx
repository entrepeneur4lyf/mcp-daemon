export function TableHead({ children }: {
  children: React.ReactNode
}) {
  return (
    <thead>{children}</thead>
  )
}

export function TableBody({ children }: {
  children: React.ReactNode
}) {
  return (
    <tbody>{children}</tbody>
  )
}

export function TableHeadRow({ children }: {
  children: React.ReactNode
}) {
  return (
    <tr className="text-left text-slate-800 dark:text-slate-200 whitespace-nowrap">{children}</tr>
  )
}

export function TableBodyRow({ children }: {
  children: React.ReactNode
}) {
  return (
    <tr className="border-t border-slate-200 dark:border-slate-800 whitespace-nowrap">{children}</tr>
  )
}

export function TableTh({ children }: {
  children: React.ReactNode
}) {
  return (
    <th className="font-medium px-2 first:pl-0 last:pr-0 py-3">{children}</th>
  )
}

export function TableTd({ children }: {
  children: React.ReactNode
}) {
  return (
    <td className="px-2 first:pl-0 last:pr-0 py-3">{children}</td>
  )
}

export default function PostTable({ children }: {
  children: React.ReactNode
}) {
  return (
    <div className="overflow-x-auto">
      <table className="text-base table-auto w-full my-2 border-b border-slate-200 dark:border-slate-800">
        {children}
      </table>
    </div>
  )
}