// E2E smoke tests — require a production Tauri build to run.
// Run locally: npm run tauri build && npm run test:e2e
// CI: see .github/workflows/e2e.yml (triggered manually or on release tags)

describe('SkillScout smoke tests', () => {
  it('renders the app window', async () => {
    // Wait for the Vue app to mount before reading the document title
    await $('nav').waitForDisplayed({ timeout: 15000 })
    const title = await browser.getTitle()
    expect(title).toBe('SkillScout')
  })

  it('shows the Projects view by default', async () => {
    // The sidebar link for Projects should be visible on launch
    const projectsLink = await $('a[href="/"]')
    await expect(projectsLink).toBeDisplayed()
    await expect(projectsLink).toHaveAttribute('aria-current', 'page')
  })

  it('navigates to the Settings view', async () => {
    const settingsLink = await $('a[href="/settings"]')
    await settingsLink.click()
    const heading = await $('h1')
    await expect(heading).toHaveText('Settings')
  })

  it('navigates to the Agents view', async () => {
    const agentsLink = await $('a[href="/agents"]')
    await agentsLink.click()
    const heading = await $('h1')
    await expect(heading).toHaveText('Agents')
  })
})
