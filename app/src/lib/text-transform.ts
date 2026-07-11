import { simplifiedToTraditionalMap, traditionalToSimplifiedMap } from './chinese-maps'
import { convertChinese, digestText, toFullWidth, toHalfWidth } from './document-utils'

const md5Shifts = [
  7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22,
  5, 9, 14, 20, 5, 9, 14, 20, 5, 9, 14, 20, 5, 9, 14, 20,
  4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23,
  6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21,
]
const md5Constants = Array.from({ length: 64 }, (_, index) =>
  Math.floor(Math.abs(Math.sin(index + 1)) * 0x100000000) | 0)

function md5(text: string) {
  const input = new TextEncoder().encode(text)
  const paddedLength = Math.ceil((input.length + 9) / 64) * 64
  const bytes = new Uint8Array(paddedLength)
  bytes.set(input)
  bytes[input.length] = 0x80
  const view = new DataView(bytes.buffer)
  const bitLength = input.length * 8
  view.setUint32(paddedLength - 8, bitLength >>> 0, true)
  view.setUint32(paddedLength - 4, Math.floor(bitLength / 0x100000000), true)
  let a0 = 0x67452301, b0 = 0xefcdab89, c0 = 0x98badcfe, d0 = 0x10325476

  for (let offset = 0; offset < bytes.length; offset += 64) {
    let a = a0, b = b0, c = c0, d = d0
    for (let i = 0; i < 64; i += 1) {
      let f: number, g: number
      if (i < 16) { f = (b & c) | (~b & d); g = i }
      else if (i < 32) { f = (d & b) | (~d & c); g = (5 * i + 1) % 16 }
      else if (i < 48) { f = b ^ c ^ d; g = (3 * i + 5) % 16 }
      else { f = c ^ (b | ~d); g = (7 * i) % 16 }
      const nextD = d
      d = c
      c = b
      const sum = (a + f + md5Constants[i] + view.getInt32(offset + g * 4, true)) | 0
      b = (b + ((sum << md5Shifts[i]) | (sum >>> (32 - md5Shifts[i])))) | 0
      a = nextD
    }
    a0 = (a0 + a) | 0; b0 = (b0 + b) | 0; c0 = (c0 + c) | 0; d0 = (d0 + d) | 0
  }
  return [a0, b0, c0, d0]
    .flatMap((value) => [0, 8, 16, 24].map((shift) => ((value >>> shift) & 0xff).toString(16).padStart(2, '0')))
    .join('')
}

export async function transformText(action: string, text: string, unsupportedHashMessage: string) {
  switch (action) {
    case 'uppercase': return text.toUpperCase()
    case 'lowercase': return text.toLowerCase()
    case 'capitalize': return text.replace(/\b\p{L}/gu, (character) => character.toUpperCase())
    case 'trimLines': return text.split('\n').map((line) => line.trim()).join('\n')
    case 'removeBlankLines': return text.split('\n').filter((line) => line.trim()).join('\n')
    case 'sortLines': return text.split('\n').sort((left, right) => left.localeCompare(right)).join('\n')
    case 'deduplicateLines': return [...new Set(text.split('\n'))].join('\n')
    case 'toSimplified': return convertChinese(text, traditionalToSimplifiedMap)
    case 'toTraditional': return convertChinese(text, simplifiedToTraditionalMap)
    case 'toHalfWidth': return toHalfWidth(text)
    case 'toFullWidth': return toFullWidth(text)
    case 'addLineNumbers': return text.split('\n').map((line, index) => `${index + 1}. ${line}`).join('\n')
    case 'removeLineNumbers': return text.replace(/^\s*\d+[\).\u3001]\s*/gm, '')
    case 'urlEncode': return encodeURIComponent(text)
    case 'urlDecode': return decodeURIComponent(text)
    case 'base64Encode': return btoa(unescape(encodeURIComponent(text)))
    case 'base64Decode': return decodeURIComponent(escape(atob(text)))
    case 'md5Hash': return md5(text)
    case 'sha1Hash': return digestText('SHA-1', text, unsupportedHashMessage)
    case 'sha256Hash': return digestText('SHA-256', text, unsupportedHashMessage)
    default: return text
  }
}
