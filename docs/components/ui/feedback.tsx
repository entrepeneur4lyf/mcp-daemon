import Image from 'next/image'
import Feedback01 from '@/public/images/feedback-01.svg'
import Feedback02 from '@/public/images/feedback-02.svg'
import Feedback03 from '@/public/images/feedback-03.svg'
import Feedback04 from '@/public/images/feedback-04.svg'

export default function Feedback() {
  return (
    <div className="flex items-center justify-between py-8 border-b border-slate-200 space-x-6 dark:border-slate-800">
      <div className="text-lg font-bold text-slate-800 dark:text-slate-200">Was this helpful?</div>
      <div className="flex items-center space-x-4">
        <button className="opacity-30">
          <span className="sr-only">No, it didn't help</span>
          <Image src={Feedback01} width={21} height={21} alt="No, it didn't help" />
        </button>
        <button className="opacity-30">
          <span className="sr-only">Still feel confused</span>
          <Image src={Feedback02} width={21} height={21} alt="Still feel confused" />
        </button>
        <button>
          <span className="sr-only">Sounds good!</span>
          <Image src={Feedback03} width={21} height={21} alt="Sounds good!" />
        </button>
        <button className="opacity-30">
          <span className="sr-only">Excellent article</span>
          <Image src={Feedback04} width={21} height={21} alt="Excellent article" />
        </button>
      </div>
    </div>
  )
}
