// The parts.demo page: the typed `parts` collection (generated from crates/parts-schema) live in a browser.
// The link fragment carries the state.store cap (`#cap=...`) and, in production, the wire.ws-door grant (`g=`, `w=`).
import { open, DbError, type Change, type Link, type Watch } from '@ce-net/db/web';
import { parts, type Part, type PartCollection } from '../../sdk/ts-web/parts-demo';

const LOW = 20;
const $ = <T extends HTMLElement>(id: string) => document.getElementById(id) as T;

const all = new Map<string, Part>();
let shown: Part[] = [];
let bin: string | null = null;
let synced = false;
let feedN = 0;
const flash = new Set<string>();
let col: PartCollection;
let binWatch: Watch<Part> | null = null;
const binRows = new Map<string, Part>();

function say(e: unknown): void {
  $('err').textContent = e instanceof DbError ? e.message : e instanceof Error ? e.message : String(e);
}

function tick(el: HTMLElement, to: number, suffix = ''): void {
  const from = Number(el.dataset.v ?? '0');
  el.dataset.v = String(to);
  const t0 = performance.now();
  const step = (t: number) => {
    const k = Math.min(1, (t - t0) / 700);
    const v = Math.round(from + (to - from) * (1 - Math.pow(1 - k, 3)));
    el.innerHTML = v.toLocaleString('en-US') + suffix;
    if (k < 1) requestAnimationFrame(step);
  };
  requestAnimationFrame(step);
}

function renderStats(): void {
  const rows = [...all.values()];
  const bins = new Set(rows.map((r) => r.bin));
  const low = rows.filter((r) => r.qty < LOW).length;
  tick($('n-parts'), rows.length);
  $('s-parts').textContent = `in ${bins.size} bin${bins.size === 1 ? '' : 's'}`;
  tick($('n-units'), rows.reduce((s, r) => s + r.qty, 0));
  tick($('n-low'), low, `<small>/ ${rows.length}</small>`);
  $('ring').setAttribute('stroke-dashoffset', String(94.25 * (1 - (rows.length ? low / rows.length : 0))));
  const pills = $('bins');
  const names = [...bins].sort();
  if (bin !== null && !bins.has(bin)) names.push(bin), names.sort();
  pills.replaceChildren(
    ...[null, ...names].map((b) => {
      const el = document.createElement('button');
      el.type = 'button';
      el.textContent = b ?? 'All';
      el.setAttribute('aria-pressed', String(b === bin));
      el.onclick = () => void pick(b);
      return el;
    }),
  );
}

function renderRows(): void {
  const src = bin === null ? [...all.values()] : [...binRows.values()];
  shown = src.sort((a, b) => (a.bin < b.bin ? -1 : a.bin > b.bin ? 1 : a.name.localeCompare(b.name)));
  $('list-h').textContent = bin === null ? 'All bins' : `Bin ${bin} (byBin query)`;
  $('list-n').textContent = `${shown.length} row${shown.length === 1 ? '' : 's'}`;
  const ul = $('rows');
  if (!shown.length) {
    const li = document.createElement('li');
    li.className = 'empty';
    li.textContent = synced ? 'No parts yet. Add one below, or run: parts add "M3x8 screw" A4 500' : '';
    ul.replaceChildren(li);
    return;
  }
  ul.replaceChildren(
    ...shown.map((p) => {
      const li = document.createElement('li');
      li.className = 'row' + (flash.has(p.id) ? ' flash' : '');
      li.dataset.id = p.id;
      const who = p.by ? ` · by ${p.by}` : '';
      li.innerHTML = `<div><div class="nm"></div><div class="meta"></div></div><div class="bin"></div><div class="qty"><button class="step" type="button" aria-label="One less">-</button><output></output><button class="step" type="button" aria-label="One more">+</button></div>`;
      li.querySelector('.nm')!.textContent = p.name;
      li.querySelector('.meta')!.textContent = (p.supplier ? p.supplier + who : who.slice(3)) || p.id.slice(-6);
      li.querySelector('.bin')!.textContent = p.bin;
      const out = li.querySelector('output')!;
      out.textContent = p.qty.toLocaleString('en-US');
      out.className = p.qty < LOW ? 'low' : '';
      const [minus, plus] = li.querySelectorAll('button');
      minus.onclick = () => void bump(p.id, -1);
      plus.onclick = () => void bump(p.id, 1);
      return li;
    }),
  );
  if (flash.size) {
    const ids = [...flash];
    flash.clear();
    requestAnimationFrame(() =>
      setTimeout(() => {
        for (const id of ids) document.querySelector(`.row[data-id="${id}"]`)?.classList.remove('flash');
      }, 1200),
    );
  }
}

function clock(): string {
  const d = new Date();
  return d.toTimeString().slice(0, 8) + '.' + String(d.getMilliseconds()).padStart(3, '0');
}

function feed(c: Change<Part>, old: Part | undefined): void {
  if (!synced || c.kind === 'synced') return;
  const li = document.createElement('li');
  const t = document.createElement('time');
  t.textContent = clock();
  const d = document.createElement('span');
  if (c.kind === 'removed') d.innerHTML = `removed <b></b>`;
  else if (c.kind === 'added') d.innerHTML = `added <b></b> ${c.doc.bin} · qty ${c.doc.qty}`;
  else d.innerHTML = `<b></b> ${c.doc.bin} · qty ${c.old.qty} → ${c.doc.qty}`;
  d.querySelector('b')!.textContent = c.kind === 'removed' ? (old?.name ?? c.key) : c.doc.name;
  const by = c.kind !== 'removed' ? c.doc.by : undefined;
  if (by) {
    const s = document.createElement('span');
    s.className = 'by';
    s.textContent = ` · ${by}`;
    d.append(s);
  }
  li.append(t, d);
  $('feed').prepend(li);
  while ($('feed').children.length > 60) $('feed').lastElementChild!.remove();
  $('feed-n').textContent = String(++feedN);
}

async function bump(id: string, d: number): Promise<void> {
  try {
    await col.update(id, (x) => {
      x.qty = Math.max(0, x.qty + d);
      x.by = 'web';
      x.at = Date.now();
    });
  } catch (e) {
    say(e);
  }
}

async function pick(b: string | null): Promise<void> {
  bin = b;
  binWatch?.close();
  binWatch = null;
  binRows.clear();
  renderStats();
  renderRows();
  if (b === null) return;
  // the bin view is the typed index query, live: rows entering, changing in and leaving bin `b`
  const w = await col.byBin(b).watch();
  binWatch = w;
  for await (const c of w) {
    if (c.kind === 'added' || c.kind === 'changed') binRows.set(c.doc.id, c.doc);
    if (c.kind === 'removed') binRows.delete(c.key);
    if (c.kind === 'changed') flash.add(c.doc.id);
    if (bin === b) renderRows();
  }
}

function setLink(l: Link): void {
  const el = $('link');
  el.dataset.s = l;
  el.querySelector('span')!.textContent = l === 'live' ? 'Live' : l === 'offline' ? 'Offline' : l === 'resyncing' ? 'Resyncing' : 'Connecting';
}

function theme(): void {
  const b = $('theme');
  const set = (light: boolean) => {
    if (light) document.documentElement.dataset.theme = 'light';
    else delete document.documentElement.dataset.theme;
    b.textContent = light ? 'Dark' : 'Light';
    try {
      localStorage.setItem('ce-theme', light ? 'light' : 'dark');
    } catch {
      /* private window: the switch still works for this page */
    }
  };
  set(document.documentElement.dataset.theme === 'light');
  b.onclick = () => set(document.documentElement.dataset.theme !== 'light');
}

async function main(): Promise<void> {
  theme();
  renderRows();
  const db = await open('parts-demo');
  col = parts(db);
  void col.onLink(setLink);
  $('add').addEventListener('submit', async (ev) => {
    ev.preventDefault();
    const f = new FormData(ev.target as HTMLFormElement);
    try {
      await col.insert({ name: String(f.get('name')).trim(), bin: String(f.get('bin')).trim().toUpperCase(), qty: Number(f.get('qty')), by: 'web', at: Date.now() });
      (ev.target as HTMLFormElement).reset();
      say('');
    } catch (e) {
      say(e);
    }
  });
  const w = await col.watch();
  for await (const c of w) {
    const old = c.kind === 'removed' ? all.get(c.key) : undefined;
    if (c.kind === 'added' || c.kind === 'changed') all.set(c.doc.id, c.doc);
    if (c.kind === 'removed') all.delete(c.key);
    if (c.kind === 'changed' || (c.kind === 'added' && synced)) flash.add(c.doc.id);
    if (c.kind === 'synced') synced = true;
    feed(c, old);
    if (synced) {
      renderStats();
      if (bin === null) renderRows();
    }
  }
}

main().catch(say);
