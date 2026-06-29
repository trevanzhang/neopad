describe('NeoPad desktop note workflow', () => {
  it('starts with the pinned default pages', async () => {
    await $('[data-ready="true"]').waitForExist()
    await expect($('.tab-item=Inbox')).toBeDisplayed()
    await expect($('.tab-item=Clipboard')).toBeDisplayed()
    await expect($('.code-editor')).toBeDisplayed()
  })

  it('saves edits before switching tabs', async () => {
    await $('.tab-add').click()
    await browser.waitUntil(async () => (await $('.tab-item.active').getText()) === 'Untitled', {
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

    await $('.tab-item=Inbox').click()
    await browser.waitUntil(async () => (await $('.tab-item.active').getText()) === 'Inbox')
    await $('.tab-item=Untitled').click()
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
})
