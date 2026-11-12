import { test, expect } from '@playwright/test';
import { readFileSync } from 'node:fs';
import { deflateSync } from 'node:zlib';

const SIG = '5VERv8NMvzbJMEkV8xnrLkEaWRtSz9CosKDYjCJjBRnbJLgp8uirBgmQpjKhoR4tjF3ZpRzrFmBV6UjKdiSZkQUW';
const OUTER = 'CLMM9tUoggJu2wagPkkqs9eFG4BWhVBZWkP1qv3Sp7tR';
const TOKEN = 'TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA';

const SUCCESS_LOGS = [
  `Program ${OUTER} invoke [1]`,
  'Program log: Instruction: OpenPosition',
  `Program ${TOKEN} invoke [2]`,
  'Program log: Instruction: Transfer',
  `Program ${TOKEN} consumed 4645 of 195000 compute units`,
  `Program ${TOKEN} success`,
  `Program ${OUTER} consumed 90232 of 200000 compute units`,
  `Program ${OUTER} success`,
];

const FAILED_LOGS = [
  `Program ${OUTER} invoke [1]`,
  'Program log: Instruction: OpenPosition',
  `Program ${OUTER} consumed 1200 of 200000 compute units`,
  `Program ${OUTER} failed: custom program error: 0x1`,
];

const CORS = {
  'access-control-allow-origin': '*',
  'access-control-allow-headers': '*',
  'access-control-allow-methods': 'POST, OPTIONS',
};

/**
 * Stubs everything off-origin. JSON-RPC getTransaction behaves like a real node: without
 * maxSupportedTransactionVersion >= 1, a v1 transaction is rejected with -32015.
 */
/**
 * Account data for a program's Anchor IDL account, as the program writes it: 8-byte
 * discriminator, 32-byte authority, u32 LE length, zlib-compressed JSON.
 */
function anchorIdlAccountBase64(idl) {
  const compressed = deflateSync(Buffer.from(JSON.stringify(idl)));
  const data = Buffer.alloc(8 + 32 + 4 + compressed.length);
  data.writeUInt32LE(compressed.length, 40);
  compressed.copy(data, 44);
  return data.toString('base64');
}

const CPMM = 'CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C';
// Where Anchor keeps CPMM's IDL (computed with @coral-xyz/anchor's idlAddress)
const CPMM_IDL_ADDRESS = '3HD1FNEKoNh5aYfvw3VrNWy6WwrEtS6JYx1RmFTE7DMC';
const CPMM_IDL = JSON.parse(readFileSync(new URL('../src/tests/CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C-idl.json', import.meta.url), 'utf8'));

/**
 * `accounts` maps an address to base64 account data that getAccountInfo returns; every
 * other account does not exist.
 */
async function mockNetwork(page, { logs = SUCCESS_LOGS, err = null, result, rpcError, accounts = {} } = {}) {
  const requests = [];
  await page.routeWebSocket(/^wss?:\/\/(?!localhost)/, () => {});
  await page.route(url => url.hostname !== 'localhost', async route => {
    const request = route.request();
    if (request.method() === 'OPTIONS') return route.fulfill({ status: 204, headers: CORS });
    if (request.method() !== 'POST') return route.fulfill({ status: 204 });

    const body = request.postDataJSON();
    if (body.method === 'getAccountInfo') {
      const data = accounts[body.params[0]];
      const value = data
        ? { data: [data, 'base64'], executable: false, lamports: 1, owner: CPMM, rentEpoch: 0, space: 0 }
        : null;
      return route.fulfill({ json: { jsonrpc: '2.0', id: body.id, result: { context: { slot: 1 }, value } }, headers: CORS });
    }
    if (body.method !== 'getTransaction') {
      return route.fulfill({ json: { jsonrpc: '2.0', id: body.id, result: null }, headers: CORS });
    }
    requests.push(body);
    const config = body.params[1] ?? {};
    let reply;
    if (rpcError) {
      reply = { error: rpcError };
    } else if (!(config.maxSupportedTransactionVersion >= 1)) {
      reply = { error: { code: -32015, message: 'Transaction version (1) is not supported by the requesting client.' } };
    } else {
      reply = { result: result !== undefined ? result : { slot: 123456, version: 1, meta: { err, logMessages: logs } } };
    }
    return route.fulfill({ json: { jsonrpc: '2.0', id: body.id, ...reply }, headers: CORS });
  });
  return requests;
}

function collectErrors(page) {
  const errors = [];
  page.on('pageerror', e => errors.push(`pageerror: ${e.message}`));
  page.on('console', m => { if (m.type() === 'error') errors.push(`console: ${m.text()}`); });
  return errors;
}

async function openLookup(page) {
  // On-chain IDL discovery is unrelated to these checks and needs real account data
  await page.addInitScript(() => localStorage.setItem('sologger_lookupAutoIdl', 'false'));
  await page.goto('lookup');
  await expect(page.getByRole('heading', { name: 'Transaction Lookup' })).toBeVisible();
}

function summaryValue(page, label) {
  return page.locator('.card', { has: page.getByRole('heading', { name: label, exact: true }) }).locator('p');
}

for (const [path, heading] of [
  ['', 'Solana Log Explorer'],
  ['convert', 'Convert Solana Logs'],
  ['lookup', 'Transaction Lookup'],
  ['about', 'About Sologger'],
]) {
  test(`/${path} renders without browser errors`, async ({ page }) => {
    const errors = collectErrors(page);
    await mockNetwork(page);
    await page.goto(path);
    await expect(page.getByRole('heading', { name: heading })).toBeVisible();
    await page.waitForLoadState('networkidle');
    expect(errors).toEqual([]);
  });
}

test('lookup requests a v1-capable transaction and renders the result', async ({ page }) => {
  const errors = collectErrors(page);
  const requests = await mockNetwork(page);
  await openLookup(page);

  await page.getByPlaceholder('Transaction signature...').fill(SIG);
  await page.getByRole('button', { name: 'Lookup' }).click();

  await expect(summaryValue(page, 'Status')).toHaveText('Success');
  expect(requests).toHaveLength(1);
  expect(requests[0].params).toEqual([SIG, { encoding: 'json', maxSupportedTransactionVersion: 1 }]);
  await expect(summaryValue(page, 'Slot')).toHaveText('123456');
  await expect(summaryValue(page, 'Invocations')).toHaveText('2');
  await expect(summaryValue(page, 'Top-level CU')).toHaveText('90,232');
  await expect(page.getByRole('heading', { name: /Compute Unit Breakdown/ })).toBeVisible();
  await expect(page.locator('.slick-row').first()).toBeVisible();
  expect(page.url()).toContain(`sig=${SIG}`);
  expect(errors).toEqual([]);
});

test('lookup deep link fetches automatically', async ({ page }) => {
  const requests = await mockNetwork(page);
  await page.addInitScript(() => localStorage.setItem('sologger_lookupAutoIdl', 'false'));
  await page.goto(`lookup?sig=${SIG}`);
  await expect(summaryValue(page, 'Invocations')).toHaveText('2');
  expect(requests).toHaveLength(1);
});

test('lookup marks a failed transaction', async ({ page }) => {
  await mockNetwork(page, { logs: FAILED_LOGS, err: { InstructionError: [0, { Custom: 1 }] } });
  await openLookup(page);
  await page.getByPlaceholder('Transaction signature...').fill(SIG);
  await page.keyboard.press('Enter');
  await expect(summaryValue(page, 'Status')).toHaveText('Failed');
});

test('lookup surfaces an RPC error', async ({ page }) => {
  await mockNetwork(page, { rpcError: { code: -32602, message: 'Invalid param: WrongSize' } });
  await openLookup(page);
  await page.getByPlaceholder('Transaction signature...').fill(SIG);
  await page.getByRole('button', { name: 'Lookup' }).click();
  await expect(page.getByText('Invalid param: WrongSize')).toBeVisible();
  await expect(page.getByText('Parsed transaction logs will appear here')).toBeVisible();
});

test('lookup explains a missing transaction', async ({ page }) => {
  await mockNetwork(page, { result: null });
  await openLookup(page);
  await page.getByPlaceholder('Transaction signature...').fill(SIG);
  await page.getByRole('button', { name: 'Lookup' }).click();
  await expect(page.getByText(/Transaction not found/)).toBeVisible();
});

test('home log replay requests a v1-capable transaction and adds rows', async ({ page }) => {
  const errors = collectErrors(page);
  const requests = await mockNetwork(page);
  await page.goto('');
  await expect(page.getByRole('heading', { name: 'Solana Log Explorer' })).toBeVisible();

  const input = page.getByPlaceholder(/Paste a transaction signature to fetch/);
  if (!(await input.isVisible())) await page.getByText('Log Replay', { exact: true }).click();
  await input.fill(SIG);
  await page.getByRole('button', { name: 'Replay', exact: true }).click();

  await expect(input).toHaveValue('');
  expect(requests).toHaveLength(1);
  expect(requests[0].params[1].maxSupportedTransactionVersion).toBe(1);
  await expect(page.locator('.slick-row').first()).toBeVisible();
  expect(errors).toEqual([]);
});

test('home popular-program picker fills the Program ID and adds it', async ({ page }) => {
  const errors = collectErrors(page);
  await mockNetwork(page);
  await page.goto('');
  await expect(page.getByRole('heading', { name: 'Solana Log Explorer' })).toBeVisible();

  const picker = page.getByLabel('Popular programs');
  const input = page.getByPlaceholder('Enter Program ID');
  await picker.selectOption({ label: 'SPL Token' });
  await expect(input).toHaveValue(TOKEN);
  await expect(picker).toHaveValue('', { message: 'picker resets so the same entry can be picked again' });

  await page.getByRole('button', { name: 'Add Program' }).click();
  await expect(page.getByRole('button', { name: `Stop monitoring ${TOKEN}` })).toBeVisible();
  await expect(input).toHaveValue('');
  expect(errors).toEqual([]);
});

const ATA = 'ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL';
const TOKEN_LOGS = [
  `Program ${TOKEN} invoke [1]`,
  'Program log: Instruction: Transfer',
  `Program ${TOKEN} consumed 4645 of 200000 compute units`,
  `Program ${TOKEN} success`,
];
const ATA_LOGS = [
  `Program ${ATA} invoke [1]`,
  `Program ${TOKEN} invoke [2]`,
  'Program log: Instruction: InitializeAccount3',
  `Program ${TOKEN} consumed 1000 of 190000 compute units`,
  `Program ${TOKEN} success`,
  `Program ${ATA} consumed 9000 of 200000 compute units`,
  `Program ${ATA} success`,
];

/**
 * A stand-in RPC for logsSubscribe: confirms each subscription and records the socket
 * with the program it subscribed to, so a test can push notifications or drop it.
 */
async function mockLogStream(page) {
  await mockNetwork(page);
  const subscriptions = [];
  await page.routeWebSocket(/^wss?:\/\/(?!localhost)/, ws => {
    ws.onMessage(message => {
      const request = JSON.parse(message);
      if (request.method !== 'logsSubscribe') return;
      const id = subscriptions.length + 1;
      subscriptions.push({ ws, id, programId: request.params[0].mentions[0] });
      ws.send(JSON.stringify({ jsonrpc: '2.0', id: request.id, result: id }));
    });
  });
  return subscriptions;
}

function notify({ ws, id }, signature, logs) {
  ws.send(JSON.stringify({
    jsonrpc: '2.0',
    method: 'logsNotification',
    params: { subscription: id, result: { context: { slot: 42 }, value: { signature, err: null, logs } } },
  }));
}

async function addProgram(page, programId) {
  await page.getByPlaceholder('Enter Program ID').fill(programId);
  await page.getByRole('button', { name: 'Add Program' }).click();
}

function programChip(page, programId) {
  return page.locator('div', { has: page.getByRole('button', { name: `Stop monitoring ${programId}` }) }).last();
}

test('home counts a transaction once when two monitored programs report it', async ({ page }) => {
  const errors = collectErrors(page);
  const subscriptions = await mockLogStream(page);
  await page.goto('');
  await addProgram(page, TOKEN);
  await addProgram(page, ATA);
  await expect.poll(() => subscriptions.length).toBe(2);

  for (const subscription of subscriptions) notify(subscription, 'SIG_SHARED', ATA_LOGS);
  notify(subscriptions[0], 'SIG_TOKEN_ONLY', TOKEN_LOGS);

  // ATA_LOGS parse into 2 records (ATA + its Token CPI), the second transaction into 1
  await expect(summaryValue(page, 'Total Logs')).toHaveText('3');
  await expect(summaryValue(page, 'Unique TXs')).toHaveText('2');
  expect(errors).toEqual([]);
});

test('home reconnects a dropped connection and keeps streaming', async ({ page }) => {
  const subscriptions = await mockLogStream(page);
  await page.goto('');
  await addProgram(page, TOKEN);
  await expect.poll(() => subscriptions.length).toBe(1);
  await expect(programChip(page, TOKEN)).toContainText('Awaiting logs');

  await subscriptions[0].ws.close();
  await expect(programChip(page, TOKEN)).toContainText('Reconnecting');
  await expect.poll(() => subscriptions.length, { timeout: 10_000 }).toBe(2);
  expect(subscriptions[1].programId).toBe(TOKEN);
  await expect(programChip(page, TOKEN)).toContainText('Awaiting logs');

  notify(subscriptions[1], 'SIG_AFTER_RECONNECT', TOKEN_LOGS);
  await expect(summaryValue(page, 'Total Logs')).toHaveText('1');
  await expect(programChip(page, TOKEN)).toContainText('Connected');
});

/**
 * Relative luminance of a success toast's background (its translucent color blended over
 * the page) and the contrast ratio of its detail text against it, per WCAG.
 */
function successToastColors(page) {
  return page.evaluate(() => {
    // Computed colors come back as rgb()/rgba() (0-255) or color(srgb ...) (0-1)
    const parse = css => {
      const srgb = css.match(/color\(srgb ([^)]+)\)/);
      const parts = (srgb ?? css.match(/rgba?\(([^)]+)\)/))[1].split(/[\s,/]+/).filter(Boolean).map(Number);
      const [r, g, b] = parts.slice(0, 3).map(v => (srgb ? v * 255 : v));
      return { r, g, b, a: parts[3] ?? 1 };
    };
    const luminance = ({ r, g, b }) => {
      const channel = v => (v /= 255) <= 0.03928 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4;
      return 0.2126 * channel(r) + 0.7152 * channel(g) + 0.0722 * channel(b);
    };
    const message = document.querySelector('.p-toast-message-success');
    const toast = parse(getComputedStyle(message).backgroundColor);
    const pageBg = parse(getComputedStyle(document.body).backgroundColor);
    const blended = Object.fromEntries(['r', 'g', 'b'].map(k => [k, toast[k] * toast.a + pageBg[k] * (1 - toast.a)]));
    const text = parse(getComputedStyle(message.querySelector('.p-toast-detail')).color);
    const [bg, fg] = [luminance(blended), luminance(text)];
    return { background: bg, contrast: (Math.max(bg, fg) + 0.05) / (Math.min(bg, fg) + 0.05) };
  });
}

for (const os of ['light', 'dark']) {
  for (const theme of ['light', 'dark']) {
    test(`toasts follow the app's ${theme} theme and stay readable (OS ${os})`, async ({ page }) => {
      await page.emulateMedia({ colorScheme: os });
      await page.addInitScript(t => localStorage.setItem('theme', t), theme);
      await mockLogStream(page);
      await page.goto('');
      await addProgram(page, TOKEN);
      await expect(page.locator('.p-toast-message-success')).toBeVisible();

      const { background, contrast } = await successToastColors(page);
      if (theme === 'dark') expect(background, 'toast background should be dark').toBeLessThan(0.2);
      else expect(background, 'toast background should be light').toBeGreaterThan(0.6);
      expect(contrast, 'detail text contrast (WCAG AA needs 4.5)').toBeGreaterThanOrEqual(4.5);
    });
  }
}

test('home loads a program\'s on-chain IDL when it is added', async ({ page }) => {
  const errors = collectErrors(page);
  await mockNetwork(page, { accounts: { [CPMM_IDL_ADDRESS]: anchorIdlAccountBase64(CPMM_IDL) } });
  await page.goto('');
  await addProgram(page, CPMM);
  await expect(page.getByText(`On-chain IDL loaded for ${CPMM.substring(0, 8)}...`)).toBeVisible();
  expect(errors).toEqual([]);
});

test('home says when a program has no on-chain IDL', async ({ page }) => {
  await mockNetwork(page);
  await page.goto('');
  await addProgram(page, CPMM);
  await expect(page.getByText(`No on-chain IDL found for ${CPMM.substring(0, 8)}...`)).toBeVisible();
});

test('lookup discovers the on-chain IDL of a program in the transaction', async ({ page }) => {
  const errors = collectErrors(page);
  await mockNetwork(page, {
    logs: [
      `Program ${CPMM} invoke [1]`,
      'Program log: Instruction: SwapBaseInput',
      `Program ${CPMM} consumed 30000 of 200000 compute units`,
      `Program ${CPMM} success`,
    ],
    accounts: { [CPMM_IDL_ADDRESS]: anchorIdlAccountBase64(CPMM_IDL) },
  });
  await page.goto('lookup');
  await page.getByPlaceholder('Transaction signature...').fill(SIG);
  await page.getByRole('button', { name: 'Lookup' }).click();
  await expect(page.getByText(`On-chain IDL loaded for ${CPMM.substring(0, 8)}…`)).toBeVisible();
  expect(errors).toEqual([]);
});

/** Whether keyboard focus is inside an open dialog. */
function focusIsInDialog(page) {
  return page.evaluate(() => Boolean(document.activeElement?.closest('[role="dialog"]')));
}

/** Tabs forwards and backwards past the dialog's controls; focus must never leave it. */
async function expectFocusTrapped(page) {
  expect(await focusIsInDialog(page), 'focus moves into the dialog when it opens').toBe(true);
  for (const key of ['Tab', 'Shift+Tab']) {
    for (let i = 0; i < 8; i++) {
      await page.keyboard.press(key);
      expect(await focusIsInDialog(page), `focus stays in the dialog after ${key} x${i + 1}`).toBe(true);
    }
  }
}

test('the mainnet warning is an accessible dialog: focus moves in and stays, Escape cancels', async ({ page }) => {
  await mockNetwork(page);
  await page.goto('');
  const network = page.locator('select').filter({ has: page.locator('option', { hasText: 'Mainnet' }) });
  await network.focus();
  await network.selectOption({ label: 'Mainnet' });

  const dialog = page.getByRole('dialog', { name: 'Mainnet Warning' });
  await expect(dialog).toBeVisible();
  await expect(dialog).toHaveAttribute('aria-modal', 'true');
  await expectFocusTrapped(page);

  await page.keyboard.press('Escape');
  await expect(dialog).toBeHidden();
  await expect(network).toHaveValue('wss://api.devnet.solana.com');
  await expect(network, 'focus returns to what opened the dialog').toBeFocused();
});

test('lookup row detail and IDL decode dialogs are accessible and reopen', async ({ page }) => {
  const errors = collectErrors(page);
  await mockNetwork(page, {
    logs: [
      `Program ${CPMM} invoke [1]`,
      'Program log: Instruction: SwapBaseInput',
      `Program ${CPMM} consumed 30000 of 200000 compute units`,
      `Program ${CPMM} success`,
    ],
    accounts: { [CPMM_IDL_ADDRESS]: anchorIdlAccountBase64(CPMM_IDL) },
  });
  await page.goto('lookup');
  await page.getByPlaceholder('Transaction signature...').fill(SIG);
  await page.getByRole('button', { name: 'Lookup' }).click();
  await expect(page.getByText(`On-chain IDL loaded for ${CPMM.substring(0, 8)}…`)).toBeVisible();

  const row = page.locator('.slick-row').first().locator('.slick-cell').nth(1);
  const detail = page.getByRole('dialog', { name: 'Log Detail' });
  await row.click();
  await expect(detail).toBeVisible();
  await expectFocusTrapped(page);
  await page.keyboard.press('Escape');
  await expect(detail).toBeHidden();

  // The same row opens again after closing
  await row.click();
  await expect(detail).toBeVisible();
  await detail.getByRole('button', { name: 'Close' }).click();
  await expect(detail).toBeHidden();

  await row.click();
  await detail.getByRole('button', { name: /Decode with IDL/ }).click();
  const decode = page.getByRole('dialog', { name: 'IDL Decode' });
  await expect(decode).toBeVisible();
  await expectFocusTrapped(page);
  await page.keyboard.press('Escape');
  await expect(decode).toBeHidden();
  expect(errors).toEqual([]);
});

async function lookupSuccessLogs(page) {
  await mockNetwork(page);
  await openLookup(page);
  await page.getByPlaceholder('Transaction signature...').fill(SIG);
  await page.getByRole('button', { name: 'Lookup' }).click();
  await expect(page.locator('.slick-row').first()).toBeVisible();
}

/** A short description of the focused element, for comparing Tab sequences. */
function focused(page) {
  return page.evaluate(() => {
    const el = document.activeElement;
    const grid = document.querySelector('.slick-grid-container');
    if (grid && el === grid.lastElementChild) return 'rows';
    if (el.getAttribute('role') === 'columnheader') return `header:${el.textContent.trim()}`;
    if (grid?.contains(el)) return `inside-grid:${el.className}`;
    return `outside:${el.tagName.toLowerCase()}`;
  });
}

const dialogOpen = page => page.getByRole('dialog').count().then(n => n > 0);

/** Whether the scrolling header strip and the rows below it are scrolled to the same place. */
function headersAligned(page) {
  return page.evaluate(() => {
    const header = document.querySelector('.slick-pane-header.slick-pane-right .slick-header');
    const rows = document.querySelector('.slick-pane-top.slick-pane-right .slick-viewport');
    return Math.abs(header.scrollLeft - rows.scrollLeft) <= 1;
  });
}

/** WCAG contrast ratio between a link in the highlighted row and its cell's background. */
function highlightedLinkContrast(page) {
  return page.evaluate(() => {
    const link = document.querySelector('.slick-cell.selected a');
    const toRgb = css => css.match(/[\d.]+/g).slice(0, 3).map(Number);
    const luminance = ([r, g, b]) => {
      const channel = v => (v /= 255) <= 0.03928 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4;
      return 0.2126 * channel(r) + 0.7152 * channel(g) + 0.0722 * channel(b);
    };
    const fg = luminance(toRgb(getComputedStyle(link).color));
    const bg = luminance(toRgb(getComputedStyle(link.closest('.slick-cell')).backgroundColor));
    return (Math.max(fg, bg) + 0.05) / (Math.min(fg, bg) + 0.05);
  });
}

test('log grid Tab order: sortable headers, one stop for the rows, then out; nothing opens', async ({ page }) => {
  const errors = collectErrors(page);
  await lookupSuccessLogs(page);
  await page.getByRole('button', { name: 'Columns' }).focus();

  const stops = [];
  for (let i = 0; i < 10; i++) {
    await page.keyboard.press('Tab');
    stops.push(await focused(page));
    expect(await dialogOpen(page), `no dialog after Tab ${i + 1}`).toBe(false);
    // Realigned within a frame of the header taking focus
    await expect.poll(() => headersAligned(page), { message: `headers line up with their columns after Tab ${i + 1}`, timeout: 2000 }).toBe(true);
  }
  expect(stops).toEqual([
    'header:Time', 'header:Level', 'header:Slot', 'header:Depth', 'header:Idx',
    'header:Instruction', 'header:CU Used', 'header:Error Code', 'rows', stops[9],
  ]);
  expect(stops[9]).toMatch(/^outside:/);

  await page.keyboard.press('Shift+Tab');
  expect(await focused(page)).toBe('rows');
  await page.keyboard.press('Shift+Tab');
  expect(await focused(page)).toBe('header:Error Code');
  expect(errors).toEqual([]);
});

test('log grid: arrows move between cells, Enter opens details with explorer links', async ({ page }) => {
  await lookupSuccessLogs(page);
  const rows = page.locator('.slick-grid-container > :last-child');
  await page.getByRole('columnheader', { name: 'Error Code' }).focus();
  await page.keyboard.press('Tab');
  expect(await focused(page)).toBe('rows');
  const live = page.locator('.logs-table-root [aria-live="polite"]');
  await expect(live).toContainText('Enter opens details');

  await page.keyboard.press('ArrowDown');
  await page.keyboard.press('ArrowRight');
  expect(await dialogOpen(page)).toBe(false);
  await expect(live).toContainText(/Row \d of 2/);
  expect(await highlightedLinkContrast(page), 'links stay readable in the highlighted row').toBeGreaterThanOrEqual(3);

  await page.keyboard.press('Enter');
  const detail = page.getByRole('dialog', { name: 'Log Detail' });
  await expect(detail).toBeVisible();
  await expect(detail.getByRole('link', { name: SIG })).toHaveAttribute('href', new RegExp(`/tx/${SIG}`));
  await expect(detail.getByRole('link', { name: TOKEN }).or(detail.getByRole('link', { name: OUTER })).first())
    .toHaveAttribute('href', /\/(account|address)\//);

  await page.keyboard.press('Escape');
  await expect(detail).toBeHidden();
  await expect(rows).toBeFocused();
  await page.keyboard.press('ArrowUp');
  expect(await dialogOpen(page)).toBe(false);
});

test('log grid: a click opens details, a click on a link does not', async ({ page }) => {
  await lookupSuccessLogs(page);
  const popup = page.waitForEvent('popup');
  await page.locator('.slick-cell a').first().click();
  await (await popup).close();
  expect(await dialogOpen(page)).toBe(false);

  await page.locator('.slick-row').first().locator('.slick-cell').nth(1).click();
  await expect(page.getByRole('dialog', { name: 'Log Detail' })).toBeVisible();
});

test('log grid: sortable headers sort from the keyboard and announce their order', async ({ page }) => {
  await lookupSuccessLogs(page);
  const header = page.getByRole('columnheader', { name: 'CU Used' });
  const firstRowCu = () => page.evaluate(() => {
    const rows = [...document.querySelectorAll('.grid-canvas-right .slick-row')].sort((a, b) => a.offsetTop - b.offsetTop);
    return rows[0].textContent;
  });
  await header.focus();
  await expect(header).toHaveAttribute('aria-sort', 'none');
  await page.keyboard.press('Enter');
  await expect(header).toHaveAttribute('aria-sort', 'ascending');
  expect(await firstRowCu()).toContain('4,645');
  await page.keyboard.press(' ');
  await expect(header).toHaveAttribute('aria-sort', 'descending');
  expect(await firstRowCu()).toContain('90,232');
  await expect(header).toBeFocused();
});

test('phone-width log cards open details from the keyboard', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await mockNetwork(page);
  await openLookup(page);
  await page.getByPlaceholder('Transaction signature...').fill(SIG);
  await page.getByRole('button', { name: 'Lookup' }).click();
  const card = page.getByRole('button', { name: /Program:/ }).first();
  await expect(card).toBeVisible();

  await card.focus();
  await page.keyboard.press('Enter');
  const detail = page.getByRole('dialog', { name: 'Log Detail' });
  await expect(detail).toBeVisible();
  await page.keyboard.press('Escape');
  await expect(card).toBeFocused();
  await page.keyboard.press(' ');
  await expect(detail).toBeVisible();
});

test('the Columns menu reports its state and closes with Escape', async ({ page }) => {
  await lookupSuccessLogs(page);
  const button = page.getByRole('button', { name: 'Columns' });
  await expect(button).toHaveAttribute('aria-expanded', 'false');
  await button.focus();
  await page.keyboard.press('Enter');
  await expect(button).toHaveAttribute('aria-expanded', 'true');
  await expect(page.getByRole('checkbox', { name: 'Raw Logs' })).toBeVisible();
  await page.keyboard.press('Escape');
  await expect(button).toHaveAttribute('aria-expanded', 'false');
  await expect(page.getByRole('checkbox', { name: 'Raw Logs' })).toBeHidden();
  await expect(button).toBeFocused();
});

test('home program ID can still be typed by hand', async ({ page }) => {
  await mockNetwork(page);
  await page.goto('');
  await page.getByPlaceholder('Enter Program ID').fill(OUTER);
  await page.getByRole('button', { name: 'Add Program' }).click();
  await expect(page.getByRole('button', { name: `Stop monitoring ${OUTER}` })).toBeVisible();
});

async function convertAndAssert(page) {
  await page.locator('textarea').fill(SUCCESS_LOGS.join('\n'));
  await page.getByRole('button', { name: 'Convert Logs' }).click({ timeout: 5_000 });
  const output = page.locator('pre');
  await expect(output).toContainText('"totalLogs": 2');
  await expect(output).toContainText(OUTER);
  await expect(output).toContainText(TOKEN);
}

test('convert parses raw logs when reached from Home', async ({ page }) => {
  const errors = collectErrors(page);
  await mockNetwork(page);
  await page.goto('');
  await expect(page.getByRole('heading', { name: 'Solana Log Explorer' })).toBeVisible();
  await page.waitForLoadState('networkidle');
  await page.locator('a[href="/sologger/convert"]:visible').first().click();
  await expect(page.getByRole('heading', { name: 'Convert Solana Logs' })).toBeVisible();
  await convertAndAssert(page);
  expect(errors).toEqual([]);
});

test('convert parses raw logs on a direct page load', async ({ page }) => {
  await mockNetwork(page);
  await page.goto('convert');
  await expect(page.getByRole('heading', { name: 'Convert Solana Logs' })).toBeVisible();
  await convertAndAssert(page);
});

test('convert keeps its button disabled until the parser has loaded', async ({ page }) => {
  await mockNetwork(page);
  let releaseWasm;
  const wasmHeld = new Promise(resolve => { releaseWasm = resolve; });
  await page.route('**/*.wasm', async route => {
    await wasmHeld;
    await route.continue();
  });

  await page.goto('convert');
  await page.locator('textarea').fill(SUCCESS_LOGS.join('\n'));
  const button = page.getByRole('button', { name: 'Convert Logs' });
  await expect(button).toBeDisabled();

  releaseWasm();
  await expect(button).toBeEnabled();
});

test('a view whose chunk fails to load (stale tab after a redeploy) loads by full navigation', async ({ page }) => {
  await mockNetwork(page);
  let blocked = 0;
  // In the dev server the view is served as its own module, like a hashed chunk in a build
  await page.route(/\/src\/views\/AboutView\.vue/, route => {
    if (blocked++ === 0) return route.abort();
    return route.continue();
  });
  await page.goto('');
  await expect(page.getByRole('heading', { name: 'Solana Log Explorer' })).toBeVisible();

  await page.locator('a[href="/sologger/about"]:visible').first().click();
  await expect(page.getByRole('heading', { name: 'About Sologger' })).toBeVisible();
  await expect(page).toHaveURL(/\/sologger\/about$/);
  expect(blocked).toBeGreaterThanOrEqual(2);
});

test('navigation links reach every view', async ({ page }) => {
  await mockNetwork(page);
  await page.goto('');
  for (const [href, heading] of [
    ['/sologger/convert', 'Convert Solana Logs'],
    ['/sologger/lookup', 'Transaction Lookup'],
    ['/sologger/about', 'About Sologger'],
    ['/sologger/', 'Solana Log Explorer'],
  ]) {
    await page.locator(`a[href="${href}"]:visible`).first().click();
    await expect(page.getByRole('heading', { name: heading })).toBeVisible();
  }
});
