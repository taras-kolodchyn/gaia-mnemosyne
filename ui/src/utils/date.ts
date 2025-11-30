import dayjs from 'dayjs'

export function formatDate(ts: string) {
  return dayjs(ts).format('YYYY-MM-DD HH:mm:ss')
}

export function formatTimestamp(ts: string) {
  return dayjs(ts).format('YYYY-MM-DD HH:mm:ss')
}
