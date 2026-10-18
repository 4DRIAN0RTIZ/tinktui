// The page is the TUI. Queries are fixtures except Changelog, which is read
// from changelog.json (git-cliff --context, newest first), same as NeoComposer.
// Commit links appear only when CHANGELOG_REPO is set and the commit has an id.

const CHANGELOG_REPO = '';
const SPINNER = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];

const GROUP_CLASS = {
  Features: 'feat',
  'Bug Fixes': 'fix',
  Documentation: 'docs',
  Refactor: 'refactor',
  Performance: 'perf',
  Testing: 'docs',
};

const GROUP_LABEL = {
  Features: 'feat',
  'Bug Fixes': 'fix',
  Documentation: 'docs',
  Refactor: 'refactor',
  Performance: 'perf',
  Testing: 'test',
  Chore: 'chore',
  CI: 'ci',
  Build: 'build',
  Style: 'style',
};

const queries = [
  {
    id: 'users',
    code: "// $hidden no entra en la tabla. t muestra el dump.\nUser::latest()->take(5)->get()",
    headers: ['id', 'username', 'email', 'enabled', 'created_at'],
    hidden: [['password', '"&lt;oculto&gt;"'], ['remember_token', 'null']],
    elapsed: 0.67,
    rows: [
      row(['18', 'ldare', 'gina20@example.org', 'true', '2026-10-17 09:14:02'], ['number', 'string', 'string', 'bool', 'string']),
      row(['19', 'isabel', 'ms@example.net', 'true', '2026-10-16 18:02:41'], ['number', 'string', 'string', 'bool', 'string']),
      row(['16', 'keebler', 'zk@example.net', 'false', '2026-10-12 11:47:09'], ['number', 'string', 'string', 'bool', 'string']),
    ],
  },
  {
    id: 'about',
    code: "config('tinker-tui')",
    headers: ['campo', 'valor'],
    rows: [
      row(['nombre', 'tinker-tui'], ['string', 'string']),
      row(['version', 'v0.1.0'], ['string', 'string']),
      row(['qué', 'escribes, ejecutas, y lo ves como tabla'], ['string', 'string']),
      row(['docker', 'tinker-tui --container mi-app-laravel'], ['string', 'string']),
      row(['local', 'tinker-tui, en la carpeta del proyecto'], ['string', 'string']),
      row(['editor', 'Ctrl+E abre $EDITOR y vuelves con el código'], ['string', 'string']),
      row(['historial', 'lo que ejecutas queda, también al cerrar'], ['string', 'string']),
      row(['ocultos', '$hidden no sale en la tabla'], ['string', 'string']),
      row(['estado', 'cada ejecución empieza de cero'], ['string', 'string']),
      row(['cuidado', 'delete() y update() pegan en la base, ya'], ['string', 'string']),
      row(['autor', { text: '4DRIAN0RTIZ', href: 'https://github.com/4DRIAN0RTIZ' }], ['string', 'string']),
      row(['sitio', { text: 'cuevaneander.tech', href: 'https://cuevaneander.tech' }], ['string', 'string']),
    ],
  },
  {
    id: 'connect',
    code: "// Sin argumentos: si TINKER_TUI_CONTAINER tiene nombre, usa ese contenedor.\n// Si no, php artisan tinker en el directorio actual.\nhelp('conectar')",
    headers: ['caso', 'cómo'],
    wrap: 1,
    rows: [
      row(['compilar', 'cargo install --path . — deja el comando tinker-tui en el PATH'], ['string', 'string']),
      row(['docker', 'tinker-tui --container mi-app-laravel'], ['string', 'string']),
      row(['atajo', 'tinker-tui -c mi-app-laravel'], ['string', 'string']),
      row(['local', 'entra a la carpeta del proyecto (donde está artisan) y corre tinker-tui'], ['string', 'string']),
      row(['otra carpeta', 'tinker-tui --local /ruta/al/proyecto'], ['string', 'string']),
      row(['no repetir', 'export TINKER_TUI_CONTAINER=mi-app-laravel en .bashrc o .zshrc'], ['string', 'string']),
      row(['sin argumentos', 'si esa variable existe y no está vacía, entra al contenedor; si no, modo local'], ['string', 'string']),
      row(['por dentro', 'docker exec -i NOMBRE php artisan tinker, o php artisan tinker si es local'], ['string', 'string']),
      row(['el contenedor', 'tiene que estar en marcha, y el nombre es el del contenedor, no el del servicio'], ['string', 'string']),
    ],
  },
  {
    id: 'requirements',
    code: "help('requisitos')",
    headers: ['qué', 'para'],
    wrap: 1,
    rows: [
      row(['Rust', 'compilarlo. rustup basta'], ['string', 'string']),
      row(['Laravel', 'un proyecto con Tinker, que viene incluido'], ['string', 'string']),
      row(['Docker', 'solo si la app corre en un contenedor'], ['string', 'string']),
      row(['PHP', 'solo si corre en tu máquina, y tiene que estar en el PATH'], ['string', 'string']),
    ],
  },
  {
    id: 'examples',
    code: "help('ejemplos')",
    headers: ['consulta', 'para qué'],
    wrap: 0,
    rows: [
      row(['User::count()', 'cuántos hay'], ['string', 'string']),
      row(["User::latest()->take(5)->get(['id', 'email', 'created_at'])", 'los últimos, solo con esas columnas'], ['string', 'string']),
      row(["User::pluck('email')", 'una lista, no modelos enteros'], ['string', 'string']),
      row(["$admins = User::where('role', 'admin')->get();\n$admins->count()", 'varias líneas: se muestra el valor de la última'], ['string', 'string']),
    ],
  },
  {
    id: 'tips',
    code: "help('consejos')",
    headers: ['tema', 'detalle'],
    wrap: 1,
    rows: [
      row(['punto y coma', 'cada línea lleva ; menos la última, que la app lo agrega si falta'], ['string', 'string']),
      row(['estado', 'cada ejecución empieza de cero. Si necesitas una variable, escríbela en el mismo bloque'], ['string', 'string']),
      row(['columnas', 'pide solo las que vas a mirar: ->get([\'id\', \'name\'])'], ['string', 'string']),
      row(['editor', 'Ctrl+E abre $VISUAL, si no $EDITOR, y si no hay ninguno, Neovim'], ['string', 'string']),
      row(['cuidado', 'delete() y update() pegan en la base al momento. Sobre todo en producción'], ['string', 'string']),
    ],
  },
  {
    id: 'keys',
    code: "config('tinker-tui.shortcuts')",
    headers: ['tecla', 'acción'],
    wrap: 1,
    rows: [
      row(['Ctrl+R / F5', 'Ejecutar'], ['string', 'string']),
      row(['Ctrl+E', 'Abrir el código en $EDITOR'], ['string', 'string']),
      row(['Ctrl+L', 'Limpiar el resultado'], ['string', 'string']),
      row(['Ctrl+C / q', 'Salir de la app'], ['string', 'string']),
      row(['Esc / i', 'Salir del editor / volver a él'], ['string', 'string']),
      row(['Tab', 'Historial ↔ resultado'], ['string', 'string']),
      row(['j / k', 'Moverse'], ['string', 'string']),
      row(['Enter', 'Cargar la consulta del historial'], ['string', 'string']),
      row(['d', 'Borrar la consulta del historial'], ['string', 'string']),
      row(['t', 'Tabla ↔ vista clásica'], ['string', 'string']),
      row(['h / l', 'Columnas'], ['string', 'string']),
      row(['0 / $', 'Primera / última columna'], ['string', 'string']),
      row(['g / G', 'Inicio / final'], ['string', 'string']),
    ],
  },
  {
    id: 'changelog',
    code: "Changelog::latest()->get(['version', 'type', 'change'])",
    headers: ['version', 'tipo', 'cambio'],
    wrap: 2,
    rows: [],
    pending: true,
  },
];

const state = {
  focus: 'history',
  index: 0,
  row: 0,
  views: {},
  cleared: false,
  running: false,
  spin: 0,
  elapsed: 0.67,
  version: 'v0.1.0',
  updated: null,
  flash: null,
};

let runToken = 0;
let flashTimer = 0;

function row(cells, kinds) {
  return { cells, kinds };
}

function escapeHtml(str) {
  return String(str)
    .replaceAll('&', '&amp;')
    .replaceAll('<', '&lt;')
    .replaceAll('>', '&gt;')
    .replaceAll('"', '&quot;');
}

function cellText(cell) {
  return typeof cell === 'object' && cell ? cell.text : String(cell ?? '');
}

function current() {
  return queries[state.index];
}

function viewOf(q) {
  return state.views[q.id] || 'table';
}

function formatDate(unix) {
  return new Date(unix * 1000).toLocaleDateString('es', {
    day: 'numeric',
    month: 'short',
    year: 'numeric',
    timeZone: 'UTC',
  });
}

function taggedVersion(releases) {
  return (releases || []).find((release) => release.version)?.version || null;
}

function latestStamp(releases) {
  const stamps = [];
  for (const release of releases || []) {
    if (typeof release.timestamp === 'number') stamps.push(release.timestamp);
    for (const commit of release.commits || []) {
      const stamp = commit?.author?.timestamp ?? commit?.timestamp;
      if (typeof stamp === 'number') stamps.push(stamp);
    }
  }
  return stamps.length ? Math.max(...stamps) : null;
}

function highlightPhp(line) {
  let html = '';
  let i = 0;
  while (i < line.length) {
    if (line.startsWith('//', i)) {
      html += `<span class="dim">${escapeHtml(line.slice(i))}</span>`;
      break;
    }
    const ch = line[i];
    if (ch === '"' || ch === "'") {
      let j = i + 1;
      while (j < line.length && line[j] !== ch) {
        if (line[j] === '\\') j += 1;
        j += 1;
      }
      j = Math.min(line.length, j + 1);
      html += `<span class="str">${escapeHtml(line.slice(i, j))}</span>`;
      i = j;
      continue;
    }
    if (line.startsWith('::', i) || line.startsWith('->', i) || line.startsWith('=>', i)) {
      const op = line.startsWith('->', i) ? '->' : line.slice(i, i + 2);
      html += `<span class="dim">${op}</span>`;
      i += op.length;
      continue;
    }
    if (/\d/.test(ch) && (i === 0 || /\W/.test(line[i - 1]))) {
      let j = i;
      while (j < line.length && /\d/.test(line[j])) j += 1;
      html += `<span class="num">${line.slice(i, j)}</span>`;
      i = j;
      continue;
    }
    if (/[A-Z]/.test(ch)) {
      let j = i + 1;
      while (j < line.length && /\w/.test(line[j])) j += 1;
      html += `<span class="id">${escapeHtml(line.slice(i, j))}</span>`;
      i = j;
      continue;
    }
    if (ch === '$') {
      let j = i + 1;
      while (j < line.length && /\w/.test(line[j])) j += 1;
      html += `<span class="var">${escapeHtml(line.slice(i, j))}</span>`;
      i = j;
      continue;
    }
    html += escapeHtml(ch);
    i += 1;
  }
  return html;
}

function rawValue(text, kind) {
  if (kind === 'number' || kind === 'bool') return escapeHtml(text);
  if (kind === 'null' || text === 'null') return '<span class="dim">null</span>';
  return `<span class="str">"${escapeHtml(text)}"</span>`;
}

function rawDump(q) {
  const blocks = q.rows.map((item, index) => {
    const lines = [`<span class="dim">=</span> <span class="id">Model</span> <span class="dim">{#${index + 1}</span>`];
    q.headers.forEach((header, col) => {
      const kind = item.kinds[col] || 'string';
      const text = cellText(item.cells[col]);
      lines.push(`    <span class="id">${escapeHtml(header)}</span><span class="dim">:</span> ${rawValue(text, kind)}`);
    });
    for (const [key, value] of q.hidden || []) {
      lines.push(`    <span class="dim">#${escapeHtml(key)}: ${value}</span>`);
    }
    lines.push('<span class="dim">}</span>');
    return lines.join('\n');
  });
  return `<pre class="raw">${blocks.join('\n\n')}</pre>`;
}

function renderCell(cell, kind, wrap) {
  const text = cellText(cell);
  const cls = [kind && kind !== 'string' ? kind : '', wrap ? 'wrap' : ''].filter(Boolean).join(' ');
  const inner = cell && cell.href
    ? `<a href="${escapeHtml(cell.href)}" target="_blank" rel="noopener">${escapeHtml(text)}</a>`
    : escapeHtml(text);
  return `<td class="${cls}">${inner}</td>`;
}

function renderTable(q) {
  const head = ['<th class="mark"></th>', ...q.headers.map((header) => `<th>${escapeHtml(header)}</th>`)].join('');
  const body = q.rows.map((item, index) => {
    const selected = index === state.row ? ' class="selected"' : '';
    const mark = index === state.row ? '▶' : '';
    const cells = item.cells.map((cell, col) => renderCell(cell, item.kinds[col], q.wrap === col)).join('');
    return `<tr${selected} data-row="${index}"><td class="mark">${mark}</td>${cells}</tr>`;
  }).join('');
  return `<table><thead><tr>${head}</tr></thead><tbody>${body}</tbody></table>`;
}

function historyLabel(q) {
  const first = q.code.split('\n').find((line) => line.trim() && !line.trim().startsWith('//')) || q.code;
  return first.trim();
}

function renderHistory() {
  const list = document.getElementById('history');
  list.innerHTML = queries.map((q, index) => {
    const selected = index === state.index;
    return `<li><button type="button" role="option" data-index="${index}" aria-selected="${selected}">${selected ? '▶ ' : ''}${escapeHtml(historyLabel(q))}</button></li>`;
  }).join('');
  const selected = list.querySelector('[aria-selected="true"]');
  if (selected) reveal(selected, list);
}

function renderEditor() {
  const lines = current().code.split('\n');
  document.getElementById('editor').innerHTML = lines.map((line, index) => {
    return `<div><span class="ln">${index + 1}</span>${highlightPhp(line) || ' '}</div>`;
  }).join('');
}

function outputTitle() {
  if (state.cleared) return 'Resultado';
  const q = current();
  if (q.pending) return 'Resultado <span class="dim">cargando…</span>';
  const elapsed = state.elapsed.toFixed(2);
  let status = '<span class="ok">✓</span>';
  if (state.running) status = `<span class="run">${SPINNER[state.spin]} ejecutando ${elapsed}s</span>`;
  else if (q.error) status = `<span class="bad">✗ ${elapsed}s</span>`;
  else if (typeof q.elapsed === 'number') status = `<span class="ok">✓ ${q.elapsed.toFixed(2)}s</span>`;
  const view = viewOf(q) === 'raw' ? 'raw' : 'tabla';
  return `Resultado ${status} <span class="dim">· ${q.rows.length} filas · ${q.headers.length} cols · ${view} (t)</span>`;
}

function renderOutput() {
  document.getElementById('output-title').innerHTML = outputTitle();
  const body = document.getElementById('output');
  const q = current();
  if (state.cleared) {
    body.innerHTML = '<p class="placeholder">Ctrl+R o F5 para ejecutar</p>';
    return;
  }
  if (state.running) {
    body.innerHTML = '<p class="placeholder">ejecutando…</p>';
    return;
  }
  if (q.error) {
    body.innerHTML = `<p class="bad">${escapeHtml(q.error)}</p>`;
    return;
  }
  if (q.pending) {
    body.innerHTML = '<p class="placeholder">cargando changelog…</p>';
    return;
  }
  if (!q.rows.length) {
    body.innerHTML = '<p class="placeholder">sin filas</p>';
    return;
  }
  body.innerHTML = viewOf(q) === 'raw' ? rawDump(q) : renderTable(q);
  const selected = body.querySelector('tr.selected');
  if (selected) reveal(selected, body);
}

function keysFor() {
  if (state.flash) return state.flash;
  if (state.focus === 'history') {
    return 'j/k mover · Enter cargar · Tab resultado · Ctrl+R ejecutar · t vista';
  }
  if (state.cleared || viewOf(current()) === 'raw') {
    return 'j/k scroll · g/G inicio/fin · t tabla · Tab historial · Ctrl+R ejecutar';
  }
  return 'j/k fila · h/l columna · g/G inicio/fin · t raw · Tab historial · Ctrl+R ejecutar';
}

function renderChrome() {
  document.getElementById('keys').textContent = keysFor();
  document.querySelectorAll('[data-version]').forEach((el) => {
    el.textContent = state.version;
  });
  const footer = document.getElementById('footer-updated');
  footer.textContent = state.updated
    ? `Última actualización: ${formatDate(state.updated)}`
    : 'Última actualización: —';
  document.getElementById('history-panel').classList.toggle('focus', state.focus === 'history');
  document.getElementById('output-panel').classList.toggle('focus', state.focus === 'output');
}

function render() {
  renderHistory();
  renderEditor();
  renderOutput();
  renderChrome();
}

function reveal(el, scroller) {
  const elBox = el.getBoundingClientRect();
  const box = scroller.getBoundingClientRect();
  if (elBox.top < box.top) scroller.scrollTop -= box.top - elBox.top;
  else if (elBox.bottom > box.bottom) scroller.scrollTop += elBox.bottom - box.bottom;
}

function selectQuery(index) {
  runToken += 1;
  state.index = (index + queries.length) % queries.length;
  state.row = 0;
  state.cleared = false;
  state.running = false;
  const output = document.getElementById('output');
  output.scrollTop = 0;
  output.scrollLeft = 0;
  render();
}

function moveRow(dir) {
  const q = current();
  if (state.cleared || viewOf(q) === 'raw') {
    document.getElementById('output').scrollTop += dir * 28;
    return;
  }
  if (!q.rows.length) return;
  state.row = Math.max(0, Math.min(q.rows.length - 1, state.row + dir));
  renderOutput();
}

function flash(message) {
  state.flash = message;
  renderChrome();
  clearTimeout(flashTimer);
  flashTimer = setTimeout(() => {
    state.flash = null;
    renderChrome();
  }, 1700);
}

function wait(ms) {
  if (window.matchMedia('(prefers-reduced-motion: reduce)').matches) return Promise.resolve();
  return new Promise((resolve) => setTimeout(resolve, ms));
}

async function execute() {
  const token = ++runToken;
  const q = current();
  state.cleared = false;
  state.running = true;
  state.spin = 0;
  state.elapsed = 0;
  render();
  const started = performance.now();
  const timer = setInterval(() => {
    if (token !== runToken) return;
    state.spin = (state.spin + 1) % SPINNER.length;
    state.elapsed = (performance.now() - started) / 1000;
    document.getElementById('output-title').innerHTML = outputTitle();
  }, 80);
  await wait(280);
  clearInterval(timer);
  if (token !== runToken) return;
  state.running = false;
  state.elapsed = (performance.now() - started) / 1000;
  q.elapsed = state.elapsed;
  render();
}

function onKey(event) {
  const key = event.key;
  if (event.target.closest('a') && (key === 'Enter' || key === ' ')) return;

  if (key === 'F5' || ((event.ctrlKey || event.metaKey) && key.toLowerCase() === 'r')) {
    event.preventDefault();
    execute();
    return;
  }
  if ((event.ctrlKey || event.metaKey) && key.toLowerCase() === 'l') {
    event.preventDefault();
    state.cleared = true;
    render();
    return;
  }
  if ((event.ctrlKey || event.metaKey) && key.toLowerCase() === 'e') {
    event.preventDefault();
    flash('en la app, Ctrl+E abre $EDITOR · aquí el editor es de solo lectura');
    return;
  }
  if (event.metaKey || event.ctrlKey || event.altKey) return;

  if (key === 'Tab') {
    event.preventDefault();
    state.focus = state.focus === 'history' ? 'output' : 'history';
    renderChrome();
    return;
  }
  if (key === 'q') {
    flash('q cierra la app, no el navegador');
    return;
  }
  if (key === 't') {
    event.preventDefault();
    const q = current();
    state.views[q.id] = viewOf(q) === 'raw' ? 'table' : 'raw';
    state.cleared = false;
    render();
    return;
  }
  if (key === 'i') {
    flash('el editor de esta página no se edita');
    return;
  }
  if (key === 'd' && state.focus === 'history') {
    flash('este historial no se borra');
    return;
  }

  const historyKeys = state.focus === 'history';
  if (historyKeys && (key === 'j' || key === 'ArrowDown')) {
    event.preventDefault();
    selectQuery(state.index + 1);
    return;
  }
  if (historyKeys && (key === 'k' || key === 'ArrowUp')) {
    event.preventDefault();
    selectQuery(state.index - 1);
    return;
  }
  if (historyKeys && key === 'Enter') {
    event.preventDefault();
    execute();
    return;
  }
  if (!historyKeys && (key === 'j' || key === 'ArrowDown')) {
    event.preventDefault();
    moveRow(1);
    return;
  }
  if (!historyKeys && key === 'Enter') {
    const link = current().rows[state.row]?.cells.find((cell) => cell && cell.href);
    if (link) window.open(link.href, '_blank', 'noopener');
    return;
  }
  if (!historyKeys && (key === 'k' || key === 'ArrowUp')) {
    event.preventDefault();
    moveRow(-1);
    return;
  }
  if (!historyKeys && (key === 'h' || key === 'ArrowLeft')) {
    event.preventDefault();
    document.getElementById('output').scrollLeft -= 72;
    return;
  }
  if (!historyKeys && (key === 'l' || key === 'ArrowRight')) {
    event.preventDefault();
    document.getElementById('output').scrollLeft += 72;
    return;
  }
  if (!historyKeys && (key === '0' || key === 'Home')) {
    event.preventDefault();
    document.getElementById('output').scrollLeft = 0;
    return;
  }
  if (!historyKeys && (key === '$' || key === 'End')) {
    event.preventDefault();
    const output = document.getElementById('output');
    output.scrollLeft = output.scrollWidth;
    return;
  }
  if (key === 'g' || key === 'Home') {
    event.preventDefault();
    if (historyKeys) selectQuery(0);
    else {
      state.row = 0;
      document.getElementById('output').scrollTop = 0;
      renderOutput();
    }
    return;
  }
  if (key === 'G') {
    event.preventDefault();
    if (historyKeys) selectQuery(queries.length - 1);
    else {
      const q = current();
      state.row = Math.max(0, q.rows.length - 1);
      const output = document.getElementById('output');
      output.scrollTop = output.scrollHeight;
      renderOutput();
    }
  }
}

function rowsFromReleases(releases) {
  const visible = (releases || []).filter((release) => release.version || (release.commits || []).length);
  const rows = [];
  for (const release of visible) {
    const version = release.version || 'Unreleased';
    for (const commit of release.commits || []) {
      const group = commit.group || 'Other';
      const label = GROUP_LABEL[group] || group.toLowerCase();
      const kind = GROUP_CLASS[group] || 'string';
      const sha = (commit.id || '').slice(0, 7);
      const message = commit.message || '';
      const cell = commit.id && CHANGELOG_REPO
        ? { text: `${message} ${sha}`, href: `${CHANGELOG_REPO}/commit/${commit.id}` }
        : message;
      rows.push(row([version, label, cell], ['string', kind, 'string']));
    }
  }
  return rows;
}

async function loadChangelog() {
  const q = queries.find((item) => item.id === 'changelog');
  try {
    const res = await fetch('changelog.json');
    if (!res.ok) throw new Error('changelog.json not found');
    const releases = await res.json();
    if (!Array.isArray(releases)) throw new Error('changelog.json must be an array');
    state.version = taggedVersion(releases) || state.version;
    state.updated = latestStamp(releases);
    q.rows = rowsFromReleases(releases);
    q.pending = false;
    q.error = null;
    const about = queries.find((item) => item.id === 'about');
    const versionRow = about.rows.find((item) => item.cells[0] === 'version');
    if (versionRow) versionRow.cells[1] = state.version;
  } catch (error) {
    console.error(error);
    q.pending = false;
    q.error = 'No se pudo leer changelog.json. Sírvela por HTTP; en file:// el navegador bloquea el fetch.';
  }
  if (!state.running) render();
}

document.getElementById('history').addEventListener('click', (event) => {
  const button = event.target.closest('button');
  if (!button) return;
  state.focus = 'history';
  selectQuery(Number(button.dataset.index));
});

document.getElementById('output').addEventListener('click', (event) => {
  const tr = event.target.closest('tr[data-row]');
  if (!tr) return;
  state.focus = 'output';
  state.row = Number(tr.dataset.row);
  render();
});

document.addEventListener('keydown', onKey);

render();
loadChangelog();
