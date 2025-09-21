import React from 'react'
import { NextPage } from 'next'
import Head from 'next/head'
import PerformanceDashboard from '../components/PerformanceDashboard'

const DashboardPage: NextPage = () => {
  return (
    <>
      <Head>
        <title>Performance Dashboard - AI Orchestrator Hub</title>
        <meta name="description" content="Real-time performance monitoring dashboard for AI Orchestrator Hub optimizations" />
        <meta name="viewport" content="width=device-width, initial-scale=1" />
        <link rel="icon" href="/favicon.ico" />
      </Head>
      <main>
        <PerformanceDashboard />
      </main>
    </>
  )
}

export default DashboardPage