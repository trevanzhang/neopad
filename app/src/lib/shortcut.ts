export function normalizeShortcutInput(value: string) {
  const normalized = value.trim().toUpperCase()
  if (/^F(?:[1-9]|1[0-2])?$/.test(normalized)) return normalized
  return Array.from(normalized).find((character) => /^[A-Z0-9]$/.test(character)) ?? ''
}

export function normalizeStoredShortcutKey(value: string, fallback: string) {
  const normalized = value.trim().toUpperCase()
  if (/^[A-Z0-9]$/.test(normalized) || /^F(?:[1-9]|1[0-2])$/.test(normalized)) return normalized
  return Array.from(normalized).find((character) => /^[A-Z0-9]$/.test(character)) ?? fallback
}

export function formatShortcutLabel(baseKey: string, modifiers: string[]) {
  return [...modifiers, normalizeStoredShortcutKey(baseKey, '')].filter(Boolean).join('+')
}
