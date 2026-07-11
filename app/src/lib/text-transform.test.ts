import { describe, expect, it } from 'vitest'
import { transformText } from './text-transform'

describe('text transforms', () => {
  it('preserves standard MD5 and UTF-8 behavior', async () => {
    expect(await transformText('md5Hash', '', 'unsupported')).toBe('d41d8cd98f00b204e9800998ecf8427e')
    expect(await transformText('md5Hash', 'abc', 'unsupported')).toBe('900150983cd24fb0d6963f7d28e17f72')
    expect(await transformText('md5Hash', 'NeoPad 中文', 'unsupported')).toBe('345569363954b92528df89ddb789be2a')
  })

  it('keeps line and width transformations stable', async () => {
    expect(await transformText('deduplicateLines', 'b\na\nb', '')).toBe('b\na')
    expect(await transformText('toHalfWidth', 'ＡＢ　１', '')).toBe('AB 1')
    expect(await transformText('addLineNumbers', 'a\nb', '')).toBe('1. a\n2. b')
  })
})
