describe('NeoPad desktop note workflow', () => {
  async function click(selector: string) {
    const element = await $(selector)
    await element.waitForExist()
    await browser.execute((target) => (target as HTMLElement).click(), element)
  }

  it('starts with the pinned default pages', async () => {
    await $('[data-ready="true"]').waitForExist()
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

  it('opens native-backed settings without losing the active note', async () => {
    await browser.keys('F8')
    await expect($('.settings-panel')).toBeDisplayed()
    const closeButton = await $('.settings-footer button')
    await browser.execute((element) => (element as HTMLElement).click(), closeButton)
    expect(await $('.cm-content').getText()).toContain('E2E autosave content')
  })

  it('uses Escape to close settings and menus before hiding the window', async () => {
    await browser.keys('F8')
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

  it('maximizes and restores with Alt+Enter while edge snapping is enabled', async () => {
    await browser.setWindowSize(720, 520)
    await browser.keys('F8')
    const snapRow = await $('.settings-check-row*=Snap main window to screen edges')
    const snapCheckbox = await snapRow.$('input')
    await snapCheckbox.waitForExist()
    await browser.execute((element) => (element as HTMLInputElement).click(), snapCheckbox)
    await click('.settings-footer button')

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
