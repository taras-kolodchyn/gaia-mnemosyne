import { mnemoGet } from './client'
import type { JobDTO } from '../types/jobs'

export async function fetchJobs(includeHistory = false): Promise<JobDTO[]> {
  const res = await mnemoGet(includeHistory ? '/v1/jobs?include_history=true' : '/v1/jobs')
  return res as JobDTO[]
}
