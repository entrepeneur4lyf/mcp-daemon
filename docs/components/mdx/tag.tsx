interface PostTagProps {
  color: string
  children: React.ReactNode
}

export default function PostTag({ color, ...props }: PostTagProps) {

  const tagColor = (color: string) => {
    switch (color) {
      case 'teal':
        return 'bg-teal-500/25 text-teal-600'
      case 'rose':
        return 'bg-rose-500/25 text-rose-500'
      case 'purple':
        return 'bg-purple-500/25 text-purple-600'
      default:
        return 'bg-blue-500/25 text-blue-500'
    }
  }

  return (
    <div className={`text-sm inline-flex font-medium rounded-sm text-center px-1 ${tagColor(color)}`}>
      {props.children}
    </div>
  )
}