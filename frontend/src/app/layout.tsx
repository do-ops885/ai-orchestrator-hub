import { Inter } from 'next/font/google'
import './globals.css'
import { ClientErrorBoundary } from '@/components/ClientErrorBoundary'

const inter = Inter({ subsets: ['latin'] })

// eslint-disable-next-line react-refresh/only-export-components
export const metadata = {
  title: 'Multiagent Hive System',
  description:
    'A sophisticated multiagent system implementing hive/swarm intelligence with NLP self-learning capabilities.',
}

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en">
      <body className={inter.className}>
        <ClientErrorBoundary>
          {children}
        </ClientErrorBoundary>
      </body>
    </html>
  )
}
