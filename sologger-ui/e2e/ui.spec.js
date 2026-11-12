import { test, expect } from '@playwright/test';

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
async function mockNetwork(page, { logs = SUCCESS_LOGS, err = null, result, rpcError } = {}) {
  const requests = [];
  await page.routeWebSocket(/^wss?:\/\/(?!localhost)/, () => {});
  await page.route(url => url.hostname !== 'localhost', async route => {
    const request = route.request();
    if (request.method() === 'OPTIONS') return route.fulfill({ status: 204, headers: CORS });
    if (request.method() !== 'POST') return route.fulfill({ status: 204 });

    const body = request.postDataJSON();
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
