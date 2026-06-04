// E2E smoke tests — require a production Tauri build to run.
// Run locally: npm run tauri build && npm run test:e2e
// CI: see .github/workflows/e2e.yml (triggered manually or on release tags)

describe('SkillScout smoke tests', () => {
  it('renders the app window', async () => {
    // Use waitForExist rather than waitForDisplayed: in headless CI the
    // WebView2 window may report zero-size bounding boxes even though the
    // app has fully mounted and elements are in the DOM.
    await $('nav').waitForExist({ timeout: 15000 })
    const title = await browser.getTitle()
    expect(title).toBe('SkillScout')
  })

  it('shows the Projects view by default', async () => {
    const projectsLink = await $('a[href="/"]')
    await expect(projectsLink).toExist()
    await expect(projectsLink).toHaveAttribute('aria-current', 'page')
  })

  it('navigates to the Settings view', async () => {
    // Use JS click to bypass the interactability check that fails when
    // elements have zero-size bounding boxes in headless CI.
    const settingsLink = await $('a[href="/settings"]')
    await browser.execute((el: HTMLElement) => el.click(), settingsLink)
    const heading = await $('h1')
    await expect(heading).toHaveText('Settings')
  })

  it('navigates to the Agents view', async () => {
    const agentsLink = await $('a[href="/agents"]')
    await browser.execute((el: HTMLElement) => el.click(), agentsLink)
    const heading = await $('h1')
    await expect(heading).toHaveText('Agents')
  })
})
