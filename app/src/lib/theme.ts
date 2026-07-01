export type ThemeName = 'light' | 'dark'

export function editorBackgroundForTheme(theme: ThemeName, configuredColor: string) {
  const lightColor = isLightColor(configuredColor)
  if (theme === 'dark' && lightColor) return '#1e2228'
  if (theme === 'light' && !lightColor) return '#ffffff'
  return configuredColor
}

export function isLightColor(color: string) {
  const match = /^#([0-9a-f]{6})$/i.exec(color.trim())
  if (!match) return true
  const value = Number.parseInt(match[1], 16)
  const red = (value >> 16) & 0xff
  const green = (value >> 8) & 0xff
  const blue = value & 0xff
  const luminance = (0.2126 * red + 0.7152 * green + 0.0722 * blue) / 255
  return luminance >= 0.5
}
