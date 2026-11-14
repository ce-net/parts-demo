// Live-update proof: page <-> CLI on the dev store. Writes screenshots and a timing log to this folder.
const puppeteer = require('puppeteer-core');
const { execFileSync } = require('child_process');
const fs = require('fs');
const T = process.env.T;
const env = { ...process.env, CE_DB_STORE: '127.0.0.1:7811', CE_DB_CAP: 'demo-cap' };
const cli = (...a) => execFileSync(T + '/parts', a, { env }).toString().trim();
const log = (s) => { console.log(s); fs.appendFileSync('proof.log', s + '\n'); };
const wait = (ms) => new Promise((r) => setTimeout(r, ms));
(async () => {
  const b = await puppeteer.launch({ executablePath: '/usr/bin/chromium', headless: true, args: ['--no-sandbox', '--disable-gpu'] });
  const p = await b.newPage();
  p.on('console', (m) => log('page console: ' + m.text()));
  p.on('response', (r) => { if (r.status() >= 400) log('page fetch ' + r.status() + ' ' + r.url()); });
  p.on('pageerror', (e) => log('page error: ' + e.message));
  for (const l of cli('list', '--bin', 'A4').split('\n')) if (l.includes('M3x8')) cli('set', l.split(/\s+/)[0], '500');
  await p.setViewport({ width: 1440, height: 900 });
  await p.goto('http://127.0.0.1:8790/#cap=demo-cap', { waitUntil: 'load' });
  await p.waitForFunction(() => document.querySelectorAll('.row').length >= 7, { timeout: 15000 });
  await wait(900);
  log('page: ' + (await p.$eval('#link', (e) => e.textContent.trim())) + ', rows ' + (await p.$$eval('.row', (r) => r.length)));
  await p.screenshot({ path: '1-page-loaded.png' });

  // 1. the CLI changes a row: the page shows it
  const line = cli('list', '--bin', 'A4').split('\n').find((l) => l.includes('M3x8'));
  const screw = line.split(/\s+/)[0];
  const q0 = Number(line.split(/\s+/)[2]);
  const q1 = q0 - 37;
  const t0 = Date.now();
  log('cli: parts take ' + screw + ' 37 -> ' + cli('take', screw, '37'));
  await p.waitForFunction((id, q) => document.querySelector(`.row[data-id="${id}"] output`)?.textContent === String(q), { timeout: 5000, polling: 'raf' }, screw, q1);
  log(`page saw the CLI's take (${q0} -> ${q1}) ${Date.now() - t0} ms after the CLI call started`);
  await wait(800);
  await p.screenshot({ path: '2-cli-change-live-on-page.png' });

  // 2. the CLI adds a part: it appears
  const t1 = Date.now();
  const nid = cli('add', 'M5x20 bolt ' + (Date.now() % 1000), 'A4', '75');
  await p.waitForFunction((id) => !!document.querySelector(`.row[data-id="${id}"]`), { timeout: 5000, polling: 'raf' }, nid);
  log(`page saw the CLI's add ${Date.now() - t1} ms after the CLI call started`);

  // 3. the page changes a row: the CLI's watch prints it
  const before = fs.readFileSync('watch.log', 'utf8').length;
  const t2 = Date.now();
  await p.click(`.row[data-id="${screw}"] button[aria-label="One more"]`);
  let got = '';
  while (Date.now() - t2 < 5000) {
    got = fs.readFileSync('watch.log', 'utf8').slice(before);
    if (got.includes(`qty ${q1} -> ${q1 + 1}`)) break;
    await wait(10);
  }
  log(`parts watch printed the page's + (${q1} -> ${q1 + 1}) ${Date.now() - t2} ms after the click: ${got.trim()}`);
  await wait(800);
  await p.screenshot({ path: '3-page-change-by-web.png' });

  // 4. the bin view is the typed byBin query, live
  await p.evaluate(() => [...document.querySelectorAll('#bins button')].find((b) => b.textContent === 'A4').click());
  await p.waitForFunction(() => document.querySelector('#list-h').textContent.includes('A4'), { timeout: 5000 });
  const t3 = Date.now();
  log('cli: parts set ' + screw + ' 18 -> ' + cli('set', screw, '18'));
  await p.waitForFunction((id) => document.querySelector(`.row[data-id="${id}"] output`)?.textContent === '18', { timeout: 5000, polling: 'raf' }, screw);
  log(`bin A4 view (byBin query watch) saw the CLI's set to 18 ${Date.now() - t3} ms after the CLI call started`);
  await wait(800);
  await p.screenshot({ path: '4-bin-a4-query-live.png' });

  await p.setViewport({ width: 390, height: 844, isMobile: true, deviceScaleFactor: 2 });
  await p.waitForFunction(() => document.querySelectorAll('.row').length >= 8, { timeout: 15000 });
  await wait(900);
  await p.screenshot({ path: '5-phone-390.png', fullPage: true });
  log('horizontal scroll at 390: ' + (await p.evaluate(() => document.documentElement.scrollWidth > window.innerWidth)));
  await b.close();
})().catch((e) => { log('FAIL ' + e.stack); process.exit(1); });
