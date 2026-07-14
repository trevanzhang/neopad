import { describe, expect, it } from 'vitest'
import { aiSlashCommands, matchAiSlashCommandPrefix } from './ai-slash-commands'

describe('AI slash commands', () => {
  it('waits for two slashes without requiring a leading space', () => {
    expect(matchAiSlashCommandPrefix('/')).toBeNull()
    expect(matchAiSlashCommandPrefix('//')).not.toBeNull()
    expect(matchAiSlashCommandPrefix('paragraph//tra')?.[1]).toBe('//tra')
    expect(matchAiSlashCommandPrefix('正文//')?.[1]).toBe('//')
  })

  it('accepts the Chinese punctuation equivalent', () => {
    expect(matchAiSlashCommandPrefix('、')).toBeNull()
    expect(matchAiSlashCommandPrefix('、、')?.[1]).toBe('、、')
    expect(matchAiSlashCommandPrefix('正文、、')?.[1]).toBe('、、')
    expect(matchAiSlashCommandPrefix('、、、')).toBeNull()
  })

  it('does not activate inside URLs, paths, or longer slash runs', () => {
    expect(matchAiSlashCommandPrefix('https://')).toBeNull()
    expect(matchAiSlashCommandPrefix('file://')).toBeNull()
    expect(matchAiSlashCommandPrefix('path/to//')).toBeNull()
    expect(matchAiSlashCommandPrefix('path/to、、')).toBeNull()
    expect(matchAiSlashCommandPrefix('///')).toBeNull()
  })

  it('only exposes focused editing actions', () => {
    expect(aiSlashCommands).toEqual(['continue', 'polish', 'summarize', 'translate'])
  })
})
