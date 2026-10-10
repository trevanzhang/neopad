import fs from 'node:fs'
import path from 'node:path'
import { execFileSync } from 'node:child_process'

describe('NeoPad desktop note workflow', () => {
  afterEach(async function () {
    if (this.currentTest?.state !== 'failed') return
    const state = await browser.execute(() => ({
      tabs: [...document.querySelectorAll('.tab-item')].map((tab) => ({
        title: tab.textContent, active: tab.classList.contains('active'),
      })),
      status: document.querySelector('.status-bar')?.textContent,
      editor: document.querySelector('.cm-content')?.textContent,
    }))
    console.error('Desktop failure state:', JSON.stringify(state))
  })

  async function click(selector: string) {
    const element = await $(selector)
    await element.waitForExist()
    await browser.execute((target) => (target as HTMLElement).click(), element)
  }

  async function dragAndDrop(sourceSelector: string, targetSelector: string) {
    let lastError: unknown
    for (let attempt = 0; attempt < 3; attempt += 1) {
      const source = await $(sourceSelector)
      const target = await $(targetSelector)
      await source.waitForDisplayed()
      await target.waitForDisplayed()
      try {
        await source.dragAndDrop(target)
        return
      } catch (error) {
        lastError = error
        await browser.pause(150)
      }
    }
    throw lastError
  }

  async function contextClick(selector: string) {
    const element = await $(selector)
    await element.waitForDisplayed()
    await element.click({ button: 'right' })
  }

  async function useEnglishLocale() {
    await browser.keys(['Control', ','])
    await expect($('.settings-panel')).toBeDisplayed()
    const language = await $("//select[option[@value='en'] and option[@value='zh']]")
    await language.selectByAttribute('value', 'en')
    await click('.settings-close')
    await expect($('.settings-panel')).not.toBeDisplayed()
  }

  it('starts with the pinned default pages', async () => {
    await $('[data-ready="true"]').waitForExist()
    await useEnglishLocale()
    await expect($('.tab-item=Inbox')).toBeDisplayed()
    await expect($('.tab-item=Clipboard')).toBeDisplayed()
    await expect($('.code-editor')).toBeDisplayed()
  })

  it('deploys the bundled CLI and repairs a modified installation from Settings', async () => {
    const workspace = process.env.NEOPAD_WORKSPACE!
    const installed = path.join(workspace, '.agents', 'bin', 'neopad-cli.exe')
    const bundled = path.resolve(import.meta.dirname, '../../target/release/neopad-cli.exe')
    expect(fs.readFileSync(installed).equals(fs.readFileSync(bundled))).toBe(true)
    fs.writeFileSync(installed, 'damaged CLI')
    try {
      await browser.keys(['Control', ','])
      await click("//nav[contains(@class, 'settings-tabs')]//button[.//span[normalize-space()='Agent tools']]")
      await expect($('.agent-tools-actions strong')).toHaveText(expect.stringContaining('Needs repair'))
      await click("//div[contains(@class, 'agent-tools-actions')]//button[normalize-space()='Install / repair CLI']")
      await expect($('.agent-tools-actions strong')).toHaveText(expect.stringContaining('Ready'))
      expect(fs.readFileSync(installed).equals(fs.readFileSync(bundled))).toBe(true)
    } finally {
      fs.copyFileSync(bundled, installed)
      if (await $('.settings-close').isDisplayed()) await click('.settings-close')
    }
  })

  it('opens and edits the root Agent guide without importing it or replacing custom instructions', async () => {
    const workspace = process.env.NEOPAD_WORKSPACE!
    const guide = path.join(workspace, 'AGENTS.md')
    const cli = path.join(workspace, '.agents', 'bin', 'neopad-cli.exe')
    expect(fs.readFileSync(guide, 'utf8')).toContain('NeoPad 工作指南')
    await click('.tab-item=Inbox')
    const editor = await $('.cm-content')
    await browser.execute((element) => (element as HTMLElement).focus(), editor)
    await browser.keys(['Control', 'End'])
    await browser.keys('Pending edit before opening Agent guide')
    await browser.keys(['Control', ','])
    await click("//nav[contains(@class, 'settings-tabs')]//button[.//span[normalize-space()='Agent tools']]")
    await click('.agent-guide-open')
    await expect($('.settings-panel')).not.toBeDisplayed()
    await expect($('.tab-item.active')).toHaveText('AGENTS')
    expect(fs.readFileSync(path.join(workspace, 'notes', 'inbox.md'), 'utf8'))
      .toContain('Pending edit before opening Agent guide')
    await browser.execute((element) => (element as HTMLElement).focus(), editor)
    await browser.keys(['Control', 'a'])
    await browser.keys('# 用户定制 Agent 指南\n\n请保留这个约定。')
    await browser.waitUntil(() => fs.readFileSync(guide, 'utf8').includes('请保留这个约定。'))
    const customized = fs.readFileSync(guide, 'utf8')
    const listed = JSON.parse(execFileSync(cli, ['--workspace', workspace, 'list'], {
      encoding: 'utf8', windowsHide: true,
    }))
    expect(listed.ok).toBe(true)
    expect(listed.data.pages.some((note: { fileName: string }) => note.fileName === 'AGENTS.md')).toBe(false)
    await browser.keys(['Control', ','])
    await click("//nav[contains(@class, 'settings-tabs')]//button[.//span[normalize-space()='Agent tools']]")
    await click("//div[contains(@class, 'agent-tools-actions')]//button[normalize-space()='Install / repair CLI']")
    await expect($('.agent-tools-actions strong')).toHaveText(expect.stringContaining('Ready'))
    expect(fs.readFileSync(guide, 'utf8')).toBe(customized)
    await click('.settings-close')
    await contextClick('.tab-item=AGENTS')
    await click("//div[contains(@class, 'tab-context-menu')]//button[.//span[normalize-space()='Close']]")
    await expect($('.tab-item=AGENTS')).not.toExist()
    await click('.tab-item=Inbox')
  })

  it('refreshes saved editor content after the deployed CLI appends a note', async () => {
    const workspace = process.env.NEOPAD_WORKSPACE!
    const cli = path.join(workspace, '.agents', 'bin', 'neopad-cli.exe')
    const input = path.join(workspace, 'agent-content.txt')
    fs.writeFileSync(input, 'Agent refresh: CLI 与桌面共享笔记', 'utf8')
    await click('.tab-item=Inbox')
    const result = JSON.parse(execFileSync(cli, ['append-inbox', '--content-file', input], {
      encoding: 'utf8', windowsHide: true,
    }))
    expect(result.ok).toBe(true)
    await browser.waitUntil(async () => (await $('.cm-content').getText()).includes('Agent refresh:'), {
      timeoutMsg: 'saved editor did not refresh the CLI append',
    })
    await expect($('.status-bar')).toHaveText(expect.stringContaining('Saved'))
  })

  it('saves edits before switching tabs', async () => {
    await click('.tab-add')
    await browser.waitUntil(async () => (await $('.tab-item.active').getText()) === 'Untitled', {
      timeout: 30_000,
      timeoutMsg: 'new note did not become active',
    })

    const editor = await $('.cm-content')
    await editor.waitForDisplayed()
    await browser.execute((element) => (element as HTMLElement).focus(), editor)
    await browser.keys(['Control', 'a'])
    await browser.keys('E2E autosave content')
    await browser.waitUntil(async () => (await $('.status-bar').getText()).includes('Saved'), {
      timeoutMsg: 'note was not saved',
    })

    await click('.tab-item=Inbox')
    await browser.waitUntil(async () => (await $('.tab-item.active').getText()) === 'Inbox')
    await click('.tab-item=Untitled')
    await browser.waitUntil(async () => (await $('.cm-content').getText()).includes('E2E autosave content'), {
      timeoutMsg: 'saved content was not restored after switching tabs',
    })
  })

  it('renames a tab with the themed input dialog', async () => {
    const tab = await $('.tab-item=Untitled')
    await tab.doubleClick()
    await expect($('.input-dialog')).toBeDisplayed()

    const input = await $('.input-dialog input')
    expect(await input.getValue()).toBe('Untitled')
    await input.setValue('E2E Renamed')
    await browser.keys('Enter')
    await expect($('.input-dialog')).not.toBeDisplayed()
    await expect($('.tab-item=E2E Renamed')).toBeDisplayed()

    await (await $('.tab-item=E2E Renamed')).doubleClick()
    await browser.keys('Escape')
    await expect($('.input-dialog')).not.toBeDisplayed()
    await expect($('.app-shell')).toBeDisplayed()
  })

  it('opens native-backed settings without losing the active note', async () => {
    await browser.keys(['Control', ','])
    await expect($('.settings-panel')).toBeDisplayed()
    await click('.settings-close')
    await expect($('.settings-panel')).not.toBeDisplayed()
    expect(await $('.cm-content').getText()).toContain('E2E autosave content')
  })

  it('uses Escape to close settings and menus before hiding the window', async () => {
    await browser.keys(['Control', ','])
    await expect($('.settings-panel')).toBeDisplayed()
    await browser.keys('Escape')
    await expect($('.settings-panel')).not.toBeDisplayed()
    await expect($('.app-shell')).toBeDisplayed()

    const menuTitle = await $('.menu-title')
    await menuTitle.click()
    await expect($('.menu-root:focus-within .menu-popover')).toBeDisplayed()
    await browser.keys('Escape')
    await browser.waitUntil(async () => !(await $('.menu-root:focus-within').isExisting()), {
      timeoutMsg: 'Escape did not close the open menu',
    })
    await expect($('.app-shell')).toBeDisplayed()
  })

  it('toggles the theme with F9 and immersive fullscreen with F11', async () => {
    const shell = await $('.app-shell')
    const initialTheme = await shell.getAttribute('class')
    await browser.keys('F9')
    await browser.waitUntil(async () => (await shell.getAttribute('class')) !== initialTheme, {
      timeoutMsg: 'F9 did not toggle the theme',
    })

    await browser.keys('F11')
    await browser.waitUntil(async () => (await shell.getAttribute('class')).includes('immersive'), {
      timeoutMsg: 'F11 did not enter immersive mode',
    })
    await expect($('.window-chrome')).not.toBeDisplayed()
    await expect($('.tab-bar')).not.toBeDisplayed()
    await expect($('.status-bar')).not.toBeDisplayed()
    await expect($('.code-editor')).toBeDisplayed()

    await browser.keys('F11')
    await browser.waitUntil(async () => !(await shell.getAttribute('class')).includes('immersive'), {
      timeoutMsg: 'F11 did not leave immersive mode',
    })

    await browser.keys('F11')
    await browser.waitUntil(async () => (await shell.getAttribute('class')).includes('immersive'))
    await browser.keys('Escape')
    await browser.waitUntil(async () => !(await shell.getAttribute('class')).includes('immersive'), {
      timeoutMsg: 'Escape did not leave immersive mode',
    })
    await expect($('.app-shell')).toBeDisplayed()
  })

  it('cycles tabs with keyboard shortcuts and tab bar arrows', async () => {
    await click('.tab-item=Inbox')
    await browser.keys(['Control', 'Tab'])
    await browser.waitUntil(async () => (await $('.tab-item.active').getText()) === 'Clipboard')

    await browser.keys(['Control', 'Shift', 'Tab'])
    await browser.waitUntil(async () => (await $('.tab-item.active').getText()) === 'Inbox')

    await click('button[aria-label="Next tab"]')
    await browser.waitUntil(async () => (await $('.tab-item.active').getText()) === 'Clipboard')
    await click('button[aria-label="Previous tab"]')
    await browser.waitUntil(async () => (await $('.tab-item.active').getText()) === 'Inbox')
  })

  it('reorders tabs by dragging them', async () => {
    await dragAndDrop('.tab-item=E2E Renamed', '.tab-item=Inbox')
    await browser.waitUntil(async () => {
      const tabs = await $$('.tab-item')
      return (await tabs[0]?.getText()) === 'E2E Renamed'
    }, { timeoutMsg: 'dragging a tab did not update its position' })
  })

  it('orders projects before archive and remembers the draggable file browser width', async () => {
    await browser.keys('F4')
    await expect($('.note-library')).toBeDisplayed()
    const groups = await browser.execute(() => [...document.querySelectorAll('.note-library-group')]
      .map(group => group.querySelector('.note-library-root span:last-of-type')?.textContent))
    expect(groups).toEqual(['Notes', 'Projects', 'Archive', 'Trash'])
    expect(await browser.execute(() => {
      const icon = document.querySelector('.note-library-create-project svg')!.getBoundingClientRect()
      const button = document.querySelector('.note-library-create-project')!.getBoundingClientRect()
      return icon.width <= button.width && icon.height <= button.height
    })).toBe(true)
    const width = () => browser.execute(() => document.querySelector('.note-library')!.getBoundingClientRect().width)
    const originalWidth = await width()
    const resizer = await $('.note-library-resizer')
    const location = await resizer.getLocation()
    const x = Math.round(location.x + 3)
    const y = Math.round(location.y + 80)
    await browser.performActions([{ type: 'pointer', id: 'library-resize', parameters: { pointerType: 'mouse' }, actions: [
      { type: 'pointerMove', duration: 0, origin: 'viewport', x, y },
      { type: 'pointerDown', button: 0 },
      { type: 'pointerMove', duration: 300, origin: 'viewport', x: x + 100, y },
      { type: 'pointerUp', button: 0 },
    ] }])
    await browser.releaseActions()
    await browser.waitUntil(async () => (await width()) >= originalWidth + 80, {
      timeoutMsg: 'dragging the divider did not widen the file browser',
    })
    const draggedWidth = await width()
    expect(await browser.execute(() => Number(localStorage.getItem('neopad.libraryWidth')))).toBe(draggedWidth)
    await expect($('.app-shell.library-resizing')).not.toExist()

    await browser.keys('F4')
    await expect($('.note-library-resizer')).not.toExist()
    await browser.keys('F4')
    expect(await width()).toBe(draggedWidth)
    await browser.refresh()
    await $('[data-ready="true"]').waitForExist()
    await browser.keys('F4')
    await expect($('.note-library')).toBeDisplayed()
    expect(await width()).toBe(draggedWidth)

    await browser.execute(() => (document.querySelector('.note-library-resizer') as HTMLElement).focus())
    await browser.keys('ArrowRight')
    expect(await width()).toBe(draggedWidth + 16)
    const widenedWidth = await width()
    const windowSize = await browser.getWindowSize()
    await browser.setWindowSize(420, 360)
    await browser.waitUntil(async () => (await width()) < widenedWidth)
    expect(await browser.execute(() => document.querySelector('.workspace-pane')!.getBoundingClientRect().width))
      .toBeGreaterThanOrEqual(240)
    await browser.setWindowSize(windowSize.width, windowSize.height)
    await browser.waitUntil(async () => (await width()) === widenedWidth)

    await browser.keys('F11')
    await expect($('.note-library-resizer')).not.toExist()
    expect(await browser.execute(() => document.querySelector('.workspace-pane')!.getBoundingClientRect().width
      >= document.querySelector('.app-shell')!.clientWidth - 1)).toBe(true)
    await browser.keys('F11')
    await expect($('.note-library')).toBeDisplayed()
    expect(await width()).toBe(widenedWidth)
    await browser.saveScreenshot(path.resolve(import.meta.dirname, '../../target/library-resize.png'))
    await (await $('.note-library-resizer')).doubleClick()
    await browser.waitUntil(async () => (await width()) === 168)
    await browser.keys('F4')
  })

  it('organizes archive notes and projects into folders from the library', async () => {
    await browser.keys('F4')
    await expect($('.note-library')).toBeDisplayed()

    await click('.note-library-archive-root .note-library-create-folder')
    await (await $('.input-dialog input')).setValue('Projects')
    await browser.keys('Enter')
    await expect($('.note-library-directory=Projects')).toBeDisplayed()

    await dragAndDrop(
      "//section[1]//button[contains(@class, 'note-library-entry') and .//span[normalize-space()='E2E Renamed']]",
      '.note-library-directory=Projects',
    )
    await click('.note-library-directory=Projects')
    await expect($("//button[contains(@class, 'archived') and .//span[normalize-space()='E2E Renamed']]"))
      .toBeDisplayed()
    await click("//button[contains(@class, 'archived') and .//span[normalize-space()='E2E Renamed']]")
    await browser.waitUntil(async () => (await $('.tab-item.active').getText()) === 'E2E Renamed', {
      timeoutMsg: 'archived note did not become active',
    })
    await expect($("//button[contains(@class, 'archived') and .//span[normalize-space()='E2E Renamed']]"))
      .toBeDisplayed()

    await click('.note-library-project-root .note-library-create-folder')
    await (await $('.input-dialog input')).setValue('Writing')
    await browser.keys('Enter')
    await expect($("//section[contains(@class, 'project-library-group')]//button[contains(@class, 'note-library-directory') and .//span[normalize-space()='Writing']]"))
      .toBeDisplayed()

    const writingFolder = "//section[contains(@class, 'project-library-group')]//button[contains(@class, 'note-library-directory') and .//span[normalize-space()='Writing']]"
    await contextClick(writingFolder)
    await click("//div[contains(@class, 'note-library-context-menu')]//button[normalize-space()='New project note']")
    await (await $('.input-dialog input')).setValue('Review project')
    await browser.keys('Enter')
    await click(writingFolder)
    await expect($("//section[contains(@class, 'project-library-group')]//button[contains(@class, 'note-library-entry') and .//span[normalize-space()='Review project']]"))
      .toBeDisplayed()

    await click('.note-library-project-root .note-library-create-folder')
    await (await $('.input-dialog input')).setValue('Reference')
    await browser.keys('Enter')
    await browser.pause(250)
    await dragAndDrop(
      "//section[contains(@class, 'project-library-group')]//button[contains(@class, 'note-library-directory') and .//span[normalize-space()='Writing']]",
      "//section[contains(@class, 'project-library-group')]//button[contains(@class, 'note-library-directory') and .//span[normalize-space()='Reference']]",
    )

    await click("//section[contains(@class, 'project-library-group')]//button[contains(@class, 'note-library-directory') and .//span[normalize-space()='Reference']]")
    const movedDirectory = "//section[contains(@class, 'project-library-group')]//button[contains(@class, 'note-library-directory') and .//span[normalize-space()='Writing']]"
    await expect($(movedDirectory)).toBeDisplayed()
    await click(movedDirectory)
    await expect($("//section[contains(@class, 'project-library-group')]//button[contains(@class, 'note-library-entry') and .//span[normalize-space()='Review project']]"))
      .toBeDisplayed()

    await contextClick(movedDirectory)
    await click("//div[contains(@class, 'note-library-context-menu')]//button[normalize-space()='Rename']")
    await (await $('.input-dialog input')).setValue('Drafts')
    await browser.keys('Enter')
    const renamedDirectory = "//section[contains(@class, 'project-library-group')]//button[contains(@class, 'note-library-directory') and .//span[normalize-space()='Drafts']]"
    await expect($(renamedDirectory)).toBeDisplayed()

    await contextClick(renamedDirectory)
    await click("//div[contains(@class, 'note-library-context-menu')]//button[normalize-space()='Delete']")
    await expect($('.confirmation-dialog-message')).toHaveText(expect.stringContaining('Drafts'))
    await click('.input-dialog-actions .danger')
    await expect($(renamedDirectory)).not.toExist()
    const trashedProject = $("//button[contains(@class, 'trashed') and .//span[normalize-space()='Review project']]")
    if (!(await trashedProject.isDisplayed().catch(() => false))) {
      await click('.note-library-trash-root .note-library-root-toggle')
    }
    await expect(trashedProject)
      .toBeDisplayed()
  })

  it('moves the current note into freshly created projects from the tab menu', async () => {
    if (await $('.note-library').isDisplayed()) await click('.tab-menu')
    const workspace = process.env.NEOPAD_WORKSPACE!
    const cli = path.join(workspace, '.agents', 'bin', 'neopad-cli.exe')
    const metadata = () => JSON.parse(fs.readFileSync(path.join(workspace, 'meta', 'tabs.json'), 'utf8'))
    const moveRoot = '.tab-project-subroot > button'
    const moveTarget = (directory: string) => `.tab-project-submenu button[title="${directory}"]`

    for (const title of ['Inbox', 'Clipboard']) {
      await contextClick(`.tab-item=${title}`)
      await expect($(moveRoot)).toBeDisabled()
      await browser.keys('Escape')
    }

    await click('.tab-add')
    await browser.waitUntil(async () => (await $('.tab-item.active').getText()) === 'Untitled')
    await (await $('.tab-item=Untitled')).doubleClick()
    await (await $('.input-dialog input')).setValue('Quick move note')
    await browser.keys('Enter')
    await expect($('.tab-item=Quick move note')).toBeDisplayed()
    const original = metadata().tabs.find((tab: { title: string }) => tab.title === 'Quick move note')

    // Create nested folders via the deployed CLI while the file browser is closed.
    for (const directory of ['Quick move/Ideas', 'Quick move/Final']) {
      expect(JSON.parse(execFileSync(cli, ['project-create', directory], {
        encoding: 'utf8', windowsHide: true,
      })).ok).toBe(true)
    }
    const editor = await $('.cm-content')
    await browser.execute((element) => (element as HTMLElement).focus(), editor)
    await browser.keys(['Control', 'a'])
    await browser.keys('Unsaved edit before moving to a project')
    await contextClick('.tab-item=Quick move note')
    await (await $(moveRoot)).click()
    await expect($(moveTarget('Quick move/Ideas'))).toBeDisplayed()
    await click(moveTarget('Quick move/Ideas'))
    await browser.waitUntil(() => metadata().tabs.find((tab: { id: string }) => tab.id === original.id)
      ?.projectRelativePath === `Quick move/Ideas/${original.fileName}`)
    expect(fs.existsSync(path.join(workspace, 'notes', original.fileName))).toBe(false)
    expect(fs.readFileSync(path.join(workspace, 'projects', 'Quick move', 'Ideas', original.fileName), 'utf8'))
      .toContain('Unsaved edit before moving to a project')
    await expect($('.tab-item.active')).toHaveText('Quick move note')

    await contextClick('.tab-item=Quick move note')
    await (await $(moveRoot)).click()
    await expect($(moveTarget('Quick move/Ideas'))).toBeDisabled()
    await browser.saveScreenshot(path.resolve(import.meta.dirname, '../../target/project-move-menu.png'))
    await click(moveTarget('Quick move/Final'))
    await browser.waitUntil(() => metadata().tabs.find((tab: { id: string }) => tab.id === original.id)
      ?.projectRelativePath === `Quick move/Final/${original.fileName}`)

    // Continue editing after the move: the tab must carry the new save revision.
    await browser.execute((element) => (element as HTMLElement).focus(), editor)
    await browser.keys(['Control', 'End'])
    await browser.keys(' - edited after move')
    await browser.waitUntil(() => fs.readFileSync(path.join(workspace, 'projects', 'Quick move', 'Final', original.fileName), 'utf8')
      .includes('edited after move'))
    await expect($('.status-bar')).toHaveText(expect.stringContaining('Saved'))
    await expect($('.tab-item.active')).toHaveText('Quick move note')

    const windowSize = await browser.getWindowSize()
    try {
      await browser.setWindowSize(420, 360)
      for (const edge of ['left', 'right']) {
        await browser.execute((side) => document.querySelector('.tab-item.active')!.dispatchEvent(
          new MouseEvent('contextmenu', { bubbles: true, cancelable: true,
            clientX: side === 'left' ? 8 : innerWidth - 8, clientY: 50 }),
        ), edge)
        await (await $(moveRoot)).click()
        await expect($(moveTarget('Quick move/Final'))).toBeDisabled()
        const bounds = await browser.execute(() => {
          const menu = document.querySelector('.tab-context-menu')!.getBoundingClientRect()
          const submenu = document.querySelector('.tab-project-submenu')!.getBoundingClientRect()
          return { left: submenu.left, right: submenu.right, bottom: submenu.bottom,
            menuBottom: menu.bottom, width: innerWidth, height: innerHeight }
        })
        expect(bounds.left).toBeGreaterThanOrEqual(0)
        expect(bounds.right).toBeLessThanOrEqual(bounds.width)
        expect(bounds.bottom).toBeLessThanOrEqual(bounds.height)
        expect(bounds.menuBottom).toBeLessThanOrEqual(bounds.height)
        await browser.keys('Escape')
      }
    } finally { await browser.setWindowSize(windowSize.width, windowSize.height) }
  })

  it('reveals the exact project file in Explorer for Chinese and punctuated folders', async () => {
    const workspace = process.env.NEOPAD_WORKSPACE!
    const cli = path.join(workspace, '.agents', 'bin', 'neopad-cli.exe')
    const note = JSON.parse(fs.readFileSync(path.join(workspace, 'meta', 'tabs.json'), 'utf8'))
      .tabs.find((tab: { title: string }) => tab.title === 'Quick move note')
    const script = path.resolve(import.meta.dirname, '../scripts/explorer-selection.ps1')
    // Library actions must also resolve notes that are no longer open as tabs.
    await contextClick('.tab-item=Quick move note')
    await click("//div[contains(@class, 'tab-context-menu')]//button[.//span[normalize-space()='Close']]")
    await expect($('.tab-item=Quick move note')).not.toExist()
    for (const directory of ['论文相关', '研究 文件, #']) {
      const moved = JSON.parse(execFileSync(cli, ['move', note.id, '--project', directory], {
        encoding: 'utf8', windowsHide: true,
      }))
      expect(moved.ok).toBe(true)
      if (!(await $('.note-library').isDisplayed())) await click('.tab-menu')
      const folder = `//section[contains(@class, 'project-library-group')]//button[contains(@class, 'note-library-directory') and .//span[normalize-space()='${directory}']]`
      await expect($(folder)).toBeDisplayed()
      await click(folder)
      await contextClick("//section[contains(@class, 'project-library-group')]//button[contains(@class, 'note-library-entry') and .//span[normalize-space()='Quick move note']]")
      const expectedFile = path.join(workspace, 'projects', directory, note.fileName)
      const selection = (close = false) => JSON.parse(execFileSync('powershell.exe', [
        '-NoProfile', '-NonInteractive', '-ExecutionPolicy', 'Bypass', '-File', script,
        '-Path', expectedFile, ...(close ? ['-Close'] : []),
      ], { encoding: 'utf8', windowsHide: true }))
      try {
        await click("//div[contains(@class, 'note-library-context-menu')]//button[normalize-space()='Show in Explorer']")
        await browser.waitUntil(() => !!selection(), {
          timeout: 20_000, timeoutMsg: `Explorer did not select ${expectedFile}`,
        })
        const opened = selection()
        expect(opened.folder.toLowerCase()).toBe(path.dirname(expectedFile).toLowerCase())
        expect(opened.selected.map((selected: string) => selected.toLowerCase())).toContain(expectedFile.toLowerCase())
      } finally {
        selection(true)
        if (await $('.note-library').isDisplayed()) await click('.tab-menu')
      }
    }
  })

  it('maximizes and restores with Alt+Enter while edge snapping is enabled', async () => {
    await browser.setWindowSize(720, 520)
    await browser.keys(['Control', ','])
    const snapRow = await $('.settings-check-row*=Snap main window to screen edges')
    const snapCheckbox = await snapRow.$('input')
    await snapCheckbox.waitForExist()
    await browser.execute((element) => (element as HTMLInputElement).click(), snapCheckbox)
    await click('.settings-close')
    await expect($('.settings-panel')).not.toBeDisplayed()

    const restored = await browser.getWindowSize()
    await browser.keys(['Alt', 'Enter'])
    await browser.waitUntil(async () => {
      const maximized = await browser.getWindowSize()
      return maximized.width > restored.width && maximized.height > restored.height
    }, { timeoutMsg: 'Alt+Enter did not maximize the window' })

    await browser.keys(['Alt', 'Enter'])
    await browser.waitUntil(async () => {
      const current = await browser.getWindowSize()
      return current.width === restored.width && current.height === restored.height
    }, { timeoutMsg: 'Alt+Enter did not restore the original window size' })
  })
})
