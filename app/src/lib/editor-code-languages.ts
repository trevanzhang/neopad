import { LanguageDescription } from '@codemirror/language'

export const editorCodeLanguages = [
  LanguageDescription.of({
    name: 'JavaScript',
    alias: ['js', 'jsx'],
    extensions: ['js', 'mjs', 'cjs', 'jsx'],
    load: () => import('@codemirror/lang-javascript').then(({ javascript }) => javascript({ jsx: true })),
  }),
  LanguageDescription.of({
    name: 'TypeScript',
    alias: ['ts', 'tsx'],
    extensions: ['ts', 'mts', 'cts', 'tsx'],
    load: () => import('@codemirror/lang-javascript').then(({ javascript }) => javascript({ jsx: true, typescript: true })),
  }),
  LanguageDescription.of({
    name: 'JSON',
    alias: ['jsonc'],
    extensions: ['json', 'jsonc'],
    load: () => import('@codemirror/lang-json').then(({ json }) => json()),
  }),
  LanguageDescription.of({
    name: 'HTML',
    alias: ['htm'],
    extensions: ['html', 'htm'],
    load: () => import('@codemirror/lang-html').then(({ html }) => html()),
  }),
  LanguageDescription.of({
    name: 'CSS',
    extensions: ['css'],
    load: () => import('@codemirror/lang-css').then(({ css }) => css()),
  }),
  LanguageDescription.of({
    name: 'Python',
    alias: ['py'],
    extensions: ['py'],
    load: () => import('@codemirror/lang-python').then(({ python }) => python()),
  }),
  LanguageDescription.of({
    name: 'Rust',
    alias: ['rs'],
    extensions: ['rs'],
    load: () => import('@codemirror/lang-rust').then(({ rust }) => rust()),
  }),
  LanguageDescription.of({
    name: 'SQL',
    extensions: ['sql'],
    load: () => import('@codemirror/lang-sql').then(({ sql }) => sql()),
  }),
]
