export function fmtDur(ms: number): string {
  const s = Math.max(0, Math.floor(ms / 1000))
  const m = Math.floor(s / 60)
  const h = Math.floor(m / 60)
  if (h > 0) return `${h}:${String(m % 60).padStart(2, '0')}:${String(s % 60).padStart(2, '0')}`
  return `${m}:${String(s % 60).padStart(2, '0')}`
}

export function fmtTime(ms: number | null): string {
  if (!ms) return '—'
  const d = new Date(ms)
  const p = (n: number) => String(n).padStart(2, '0')
  const hm = `${p(d.getHours())}:${p(d.getMinutes())}`
  const dayOf = (x: Date) => new Date(x.getFullYear(), x.getMonth(), x.getDate()).getTime()
  const diffDays = Math.round((dayOf(new Date()) - dayOf(d)) / 86400000)
  if (diffDays === 0) return `今天 ${hm}`
  if (diffDays === 1) return `昨天 ${hm}`
  if (diffDays === 2) return `前天 ${hm}`
  if (diffDays > 365) return `${Math.max(1, Math.round(diffDays / 365))} 年前`
  if (diffDays > 30) return `${Math.max(1, Math.round(diffDays / 30))} 个月前`
  return `${p(d.getMonth() + 1)}-${p(d.getDate())} ${hm}`
}

export function isoToday(): string {
  const d = new Date()
  const p = (n: number) => String(n).padStart(2, '0')
  return `${d.getFullYear()}-${p(d.getMonth() + 1)}-${p(d.getDate())}`
}

export function dateZh(iso: string): string {
  const m = iso?.match(/^(\d{4})-(\d{1,2})-(\d{1,2})/)
  if (!m) return iso
  return `${Number(m[1])}/${Number(m[2])}/${Number(m[3])}`
}
