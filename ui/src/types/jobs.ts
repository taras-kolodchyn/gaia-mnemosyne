export type JobDTO = {
  id: string
  job_type: string
  status: 'pending' | 'running' | 'success' | 'failed'
  created_at: string
  updated_at: string
  progress?: number
}
