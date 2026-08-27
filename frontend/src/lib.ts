export type DiffPart = { value: string; type: 'same' | 'added' | 'removed' };

export function compact(value: string, limit = 180): string {
  const clean = value.replace(/\s+/g, ' ').trim();
  return clean.length > limit ? `${clean.slice(0, limit).trim()}…` : clean;
}

export function diffWords(oldText: string, newText: string): { old: DiffPart[]; next: DiffPart[] } {
  const a = oldText.split(/(\s+)/).filter(Boolean).slice(0, 500);
  const b = newText.split(/(\s+)/).filter(Boolean).slice(0, 500);
  const rows = a.length + 1;
  const cols = b.length + 1;
  const table = new Uint16Array(rows * cols);
  for (let i = a.length - 1; i >= 0; i--) {
    for (let j = b.length - 1; j >= 0; j--) {
      table[i * cols + j] = a[i] === b[j]
        ? table[(i + 1) * cols + j + 1] + 1
        : Math.max(table[(i + 1) * cols + j], table[i * cols + j + 1]);
    }
  }
  const old: DiffPart[] = [], next: DiffPart[] = [];
  let i = 0, j = 0;
  while (i < a.length || j < b.length) {
    if (i < a.length && j < b.length && a[i] === b[j]) {
      old.push({ value: a[i], type: 'same' }); next.push({ value: b[j], type: 'same' }); i++; j++;
    } else if (j < b.length && (i === a.length || table[i * cols + j + 1] >= table[(i + 1) * cols + j])) {
      next.push({ value: b[j++], type: 'added' });
    } else if (i < a.length) {
      old.push({ value: a[i++], type: 'removed' });
    }
  }
  if (oldText.split(/(\s+)/).filter(Boolean).length > 500) old.push({ value: ' …', type: 'same' });
  if (newText.split(/(\s+)/).filter(Boolean).length > 500) next.push({ value: ' …', type: 'same' });
  return { old, next };
}

export function csvEscape(value: unknown): string {
  return `"${String(value ?? '').replaceAll('"', '""')}"`;
}
