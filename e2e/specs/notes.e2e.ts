describe('NeoPad desktop note workflow', () => {
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

  it('organizes archive notes and prompts into folders from the library', async () => {
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
    await expect($('.tab-item.active')).toHaveText('E2E Renamed')
    await expect($("//button[contains(@class, 'archived') and .//span[normalize-space()='E2E Renamed']]"))
      .toBeDisplayed()

    await click('.note-library-prompt-root .note-library-create-folder')
    await (await $('.input-dialog input')).setValue('Writing')
    await browser.keys('Enter')
    await expect($("//section[contains(@class, 'prompt-library-group')]//button[contains(@class, 'note-library-directory') and .//span[normalize-space()='Writing']]"))
      .toBeDisplayed()

    await click('.note-library-create-prompt')
    await (await $('.input-dialog input')).setValue('Review prompt')
    await browser.keys('Enter')
    await expect($('.note-library')).toBeDisplayed()
    await dragAndDrop(
      "//section[contains(@class, 'prompt-library-group')]//button[contains(@class, 'note-library-entry') and .//span[normalize-space()='Review prompt']]",
      "//section[contains(@class, 'prompt-library-group')]//button[contains(@class, 'note-library-directory') and .//span[normalize-space()='Writing']]",
    )
    await click("//section[contains(@class, 'prompt-library-group')]//button[contains(@class, 'note-library-directory') and .//span[normalize-space()='Writing']]")
    await expect($("//section[contains(@class, 'prompt-library-group')]//button[contains(@class, 'note-library-entry') and .//span[normalize-space()='Review prompt']]"))
      .toBeDisplayed()

    await click('.note-library-prompt-root .note-library-create-folder')
    await (await $('.input-dialog input')).setValue('Reference')
    await browser.keys('Enter')
    await browser.pause(250)
    await dragAndDrop(
      "//section[contains(@class, 'prompt-library-group')]//button[contains(@class, 'note-library-directory') and .//span[normalize-space()='Writing']]",
      "//section[contains(@class, 'prompt-library-group')]//button[contains(@class, 'note-library-directory') and .//span[normalize-space()='Reference']]",
    )

    await click("//section[contains(@class, 'prompt-library-group')]//button[contains(@class, 'note-library-directory') and .//span[normalize-space()='Reference']]")
    const movedDirectory = "//section[contains(@class, 'prompt-library-group')]//button[contains(@class, 'note-library-directory') and .//span[normalize-space()='Writing']]"
    await expect($(movedDirectory)).toBeDisplayed()
    await click(movedDirectory)
    await expect($("//section[contains(@class, 'prompt-library-group')]//button[contains(@class, 'note-library-entry') and .//span[normalize-space()='Review prompt']]"))
      .toBeDisplayed()

    await contextClick(movedDirectory)
    await click("//div[contains(@class, 'note-library-context-menu')]//button[normalize-space()='Rename']")
    await (await $('.input-dialog input')).setValue('Drafts')
    await browser.keys('Enter')
    const renamedDirectory = "//section[contains(@class, 'prompt-library-group')]//button[contains(@class, 'note-library-directory') and .//span[normalize-space()='Drafts']]"
    await expect($(renamedDirectory)).toBeDisplayed()

    await contextClick(renamedDirectory)
    await click("//div[contains(@class, 'note-library-context-menu')]//button[normalize-space()='Delete']")
    await expect($('.confirmation-dialog-message')).toHaveText(expect.stringContaining('Drafts'))
    await click('.input-dialog-actions .danger')
    await expect($(renamedDirectory)).not.toExist()
    const trashedPrompt = $("//button[contains(@class, 'trashed') and .//span[normalize-space()='Review prompt']]")
    if (!(await trashedPrompt.isDisplayed().catch(() => false))) {
      await click('.note-library-trash-root .note-library-root-toggle')
    }
    await expect(trashedPrompt)
      .toBeDisplayed()
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
