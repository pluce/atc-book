import { test, expect } from '@playwright/test';

test.describe('ATC BOOK E2E Capabilities', () => {

  const mockCharts = {
    charts: [
      {
        url: 'http://example.com/lfpg_iac.pdf',
        filename: 'AD 2 LFPG IAC RWY 08R',
        category: 'Instrument Approach',
        subtitle: 'ILS RWY 08R',
        tags: ['Pistes', '08R'],
        page: 'AD 2 LFPG 01',
      },
      {
        url: 'http://example.com/lfpg_arr.pdf',
        filename: 'AD 2 LFPG ARR',
        category: 'Arrival',
        subtitle: 'STAR PISTE 08',
        tags: ['Arrivee'],
        page: 'AD 2 LFPG 02',
      },
      {
        url: 'http://example.com/lfpg_vac.pdf',
        filename: 'AD-2.LFPG.pdf',
        category: 'VAC',
        subtitle: 'Carte VAC',
        tags: ['VAC'],
      }
    ]
  };

  test.beforeEach(async ({ page }) => {
    // Mock the API call to avoid external dependency and ensure speed
    await page.route('**/api/charts?icao=LFPG*', async route => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify(mockCharts),
      });
    });

    await page.goto('/');
  });

  test('Capability 1: Search and Navigation', async ({ page }) => {
    // Verify title
    await expect(page).toHaveTitle(/ATC BOOK/);
    await expect(page.getByRole('heading', { name: 'ATC BOOK', level: 1 })).toBeVisible();

    // Perform search
    const searchInput = page.getByTestId('search-input'); 
    await searchInput.fill('LFPG');
    await page.getByTestId('search-submit').click();

    // Verify results using testid
    await expect(page.getByTestId('results-title')).toBeVisible();
    
    // Verify SIA chart using testid
    await expect(page.getByTestId('chart-title').filter({ hasText: 'ILS RWY 08R' })).toBeVisible();

    // Verify ATLAS VAC chart is present
    await expect(page.getByTestId('chart-title').filter({ hasText: 'Carte VAC' })).toBeVisible();
  });

  test('Capability 2 & 3: Filtering', async ({ page }) => {
    // Search first
    await page.getByTestId('search-input').fill('LFPG');
    await page.getByTestId('search-submit').click();
    await expect(page.getByText('Instrument Approach')).toBeVisible();

    // Text Filter - using testid
    const filterInput = page.getByTestId('filter-input');
    await filterInput.fill('ILS');
    // ARR chart should disappear, ILS chart should remain
    await expect(page.getByText('STAR PISTE 08')).not.toBeVisible();
    await expect(page.getByText('ILS RWY 08R')).toBeVisible();
  });

  test('Capability 4 & 5: Selection and Dock', async ({ page }) => {
    await page.getByTestId('search-input').fill('LFPG');
    await page.getByTestId('search-submit').click();
    await expect(page.getByTestId('results-title')).toBeVisible();

    // Select a chart by clicking its card
    await page.getByTestId('chart-title').filter({ hasText: 'ILS RWY 08R' }).click();
    
    // Check if the "Pin" button in filters is enabled
    const pinButton = page.getByTestId('btn-pin');
    await expect(pinButton).toBeEnabled();

    // Pin it
    await pinButton.click();

    // Verify Dock appears
    const dockContainer = page.getByTestId('dock-container');
    await expect(dockContainer).toBeVisible({ timeout: 5000 });
    
    // Verify item in dock
    const dockItem = page.getByTestId('dock-item').filter({ hasText: 'AD 2 LFPG' });
    await expect(dockItem).toBeVisible(); 
  });

  test('Capability 6: Viewer', async ({ page }) => {
    await page.getByTestId('search-input').fill('LFPG');
    await page.getByTestId('search-submit').click();
    await expect(page.getByText('Instrument Approach')).toBeVisible();

    // Find the viewer button
    // It's inside the card for ILS RWY 08R
    // Note: The card contains multiple buttons (pin, checkbox, download/viewer)
    // We added data-testid="btn-viewer-open" to the viewer button
    // But since it's repeated for each card, we need to scope it to the specific card
    const card = page.locator('.group').filter({ hasText: 'ILS RWY 08R' });
    await card.getByTestId('btn-viewer-open').click();
    
    await expect(page.locator('.fixed.z-\\[50\\] iframe')).toBeVisible();
  });

  test('Capability 7: Theme Toggle', async ({ page }) => {
    const html = page.locator('html');
    const toggleBtn = page.getByTestId('theme-toggle');

    await expect(toggleBtn).toBeVisible();
    const initialClass = await html.getAttribute('class');
    
    await toggleBtn.click();
    
    await expect(async () => {
      const newClass = await html.getAttribute('class');
      expect(newClass).not.toEqual(initialClass);
    }).toPass({ timeout: 5000 });
  });

  test('Capability 8: Language Switch', async ({ page }) => {
    // Switch to English
    await page.getByTestId('lang-en').click();
    
    // Check for "ICAO Code" label which changes from "Code ICAO"
    await expect(page.getByText('ICAO Code')).toBeVisible(); 
    
    // Switch back to French
    await page.getByTestId('lang-fr').click();
    await expect(page.getByText('Code ICAO')).toBeVisible();
  });

});
