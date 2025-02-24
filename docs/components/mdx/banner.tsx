interface PostBannerProps {
  type: string
  children: React.ReactNode
}

export default function PostBanner({ type, ...props }: PostBannerProps) {

  const typeIcon = (type: string) => {
    switch (type) {
      case 'important':
        return (
          <svg className="fill-purple-500 shrink-0 mr-4" width="16" height="16" viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg">
            <path d="M8 0C3.6 0 0 3.6 0 8s3.6 8 8 8 8-3.6 8-8-3.6-8-8-8Zm0 12c-.6 0-1-.4-1-1s.4-1 1-1 1 .4 1 1-.4 1-1 1Zm1-3H7V4h2v5Z" />
          </svg>
        );
      default:
        return (
          <svg className="fill-teal-500 shrink-0 mr-4" width="16" height="16" viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg">
            <path d="M8 0C3.6 0 0 3.6 0 8s3.6 8 8 8 8-3.6 8-8-3.6-8-8-8ZM7 11.4 3.6 8 5 6.6l2 2 4-4L12.4 6 7 11.4Z" />
          </svg>
        )
    }
  }

  return (
    <div className="text-sm p-4 bg-slate-50 border border-slate-200 rounded-sm dark:bg-slate-800 dark:border-slate-700">
      <div className="flex items-center">
        {typeIcon(type)}
        <div>
          {props.children}
        </div>
      </div>
    </div>
  )
}