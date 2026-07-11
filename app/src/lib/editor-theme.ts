import { EditorView } from '@codemirror/view'

export function baseEditorTheme() {
  return EditorView.theme({
    '&': { height: '100%', color: 'var(--np-text)', fontSize: '14px', position: 'relative' },
    '.cm-scroller': { lineHeight: '1.45' },
    '.cm-content': { padding: '10px 12px', minHeight: '100%' },
    '.cm-line': { color: 'var(--np-text)', textDecoration: 'none', fontWeight: '400' },
    '.cm-gutters': { backgroundColor: 'transparent', color: 'var(--np-muted)', border: '0' },
    '.cm-activeLine': { backgroundColor: 'transparent' },
    '.cm-focused': { outline: '0' },
    '.cm-fat-cursor': {
      backgroundColor: 'var(--np-vim-cursor) !important',
      color: 'var(--np-vim-cursor-text) !important',
    },
    '.cm-cursor, .cm-dropCursor': { borderLeftColor: 'var(--np-vim-cursor) !important' },
    '&:not(.cm-focused) .cm-fat-cursor': {
      backgroundColor: 'transparent !important',
      color: 'transparent !important',
      outline: '1px solid var(--np-vim-cursor)',
    },
    '.cm-panels': {
      color: 'var(--np-text)',
      backgroundColor: 'var(--np-chrome)',
      borderTop: '1px solid var(--np-border)',
    },
    '.cm-panels.cm-panels-top': {
      position: 'absolute', top: '8px', right: '10px', left: 'auto', zIndex: '8', border: '0', backgroundColor: 'transparent',
    },
    '.cm-panel.cm-search.np-find-panel': {
      display: 'grid', gap: '5px', boxSizing: 'content-box', width: 'max-content', maxWidth: 'calc(100% - 20px)',
      padding: '6px 8px', color: 'var(--np-text)', backgroundColor: 'var(--np-chrome)',
      border: '1px solid var(--np-border)', borderRadius: '6px', boxShadow: 'var(--np-shadow)',
    },
    '.np-find-row': {
      display: 'grid', gridTemplateColumns: '210px auto auto auto auto', columnGap: '8px', rowGap: '5px',
      alignItems: 'center', minWidth: '0', width: 'max-content',
    },
    '.np-replace-row': {
      display: 'none', gridTemplateColumns: 'minmax(150px, 1fr) auto auto', gap: '5px', alignItems: 'center', minWidth: '0',
    },
    '.np-find-panel.is-replace-open .np-replace-row': { display: 'grid' },
    '.cm-panel.cm-search.np-find-panel input[type="search"], .cm-panel.cm-search.np-find-panel input[type="text"]': {
      width: '100%', minWidth: '0', height: '28px', padding: '0 7px', color: 'var(--np-text)',
      backgroundColor: 'var(--np-surface)', border: '1px solid var(--np-border)', borderRadius: '3px', fontSize: '13px',
    },
    '.cm-panel.cm-search.np-find-panel input[name="search"]': { width: '210px', maxWidth: '210px' },
    '.cm-panel.cm-search.np-find-panel button': {
      height: '28px', minWidth: '28px', margin: '0', padding: '0 7px', color: 'var(--np-text)',
      backgroundColor: 'var(--np-control)', backgroundImage: 'none', border: '1px solid var(--np-border)',
      borderRadius: '3px', fontSize: '12px', cursor: 'pointer',
    },
    '.cm-panel.cm-search.np-find-panel button:hover, .cm-panel.cm-search.np-find-panel button:focus-visible': {
      backgroundColor: 'var(--np-control-active)', borderColor: 'var(--np-accent)',
    },
    '.np-find-toggle[aria-pressed="true"]': {
      color: '#ffffff', backgroundColor: 'var(--np-accent) !important', borderColor: 'var(--np-accent) !important',
    },
    '.np-find-count': {
      minWidth: '34px', color: 'var(--np-muted)', textAlign: 'center', fontSize: '12px', whiteSpace: 'nowrap',
    },
    '.np-find-count.is-empty': { display: 'none' },
    '.np-find-nav, .np-find-options, .np-find-actions': {
      display: 'flex', flex: '0 0 auto', gap: '4px', minWidth: '0',
    },
    '.np-find-options': { paddingLeft: '1px', borderLeft: '1px solid var(--np-border)' },
    '.np-find-replace-toggle': { minWidth: '48px !important' },
    '.np-find-action': { minWidth: '48px !important' },
    '.np-find-close': { padding: '0', fontSize: '16px' },
    '@media (max-width: 680px)': {
      '.cm-panels.cm-panels-top': { right: '8px', left: '8px' },
      '.cm-panel.cm-search.np-find-panel': { width: 'auto' },
      '.np-find-row': { gridTemplateColumns: 'minmax(150px, 1fr) auto auto' },
      '.cm-panel.cm-search.np-find-panel input[name="search"]': { width: '100%', maxWidth: 'none' },
      '.np-find-nav, .np-find-options, .np-find-actions': { gridColumn: 'auto' },
      '.np-find-options': {
        gridColumn: '1 / -1', paddingLeft: '0', borderLeft: '0', justifyContent: 'flex-start',
      },
      '.np-find-actions': { gridColumn: '1 / -1', justifyContent: 'flex-end' },
      '.np-replace-row': { gridTemplateColumns: 'minmax(120px, 1fr) auto' },
      '.np-replace-row .np-find-action:last-child': { gridColumn: '1 / -1' },
    },
  })
}

export function editorAppearance(backgroundColor: string, fontFamily: string, fontSize: number) {
  return EditorView.theme({
    '&': { backgroundColor },
    '.cm-scroller': { fontFamily, fontSize: `${fontSize}px` },
  })
}
