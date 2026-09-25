const BASE = Deno.env.get('OSEDE_BASE') || 'http://localhost:8080';
const LOG = Deno.env.get('OSEDE_ADMINVISIT_LOG') || '/tmp/adminvisit.log';
const ADMIN_USER = 'admin';
const ADMIN_PASS = 'Aragorn2025!';

let cookie = '';

function log(msg) {
  const line = new Date().toISOString() + ' ' + msg + '\n';
  try {
    Deno.stdout.writeSync(new TextEncoder().encode(line));
  } catch (e) {}
  try {
    const f = Deno.openSync(LOG, { append: true, write: true, create: true });
    f.writeSync(new TextEncoder().encode(line));
    f.close();
  } catch (e) {}
}

async function login() {
  log('adminvisit: login as ' + ADMIN_USER);
  const r = await fetch(BASE + '/login.php', {
    method: 'POST',
    headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
    body: 'username=' + encodeURIComponent(ADMIN_USER) + '&password=' + encodeURIComponent(ADMIN_PASS),
    redirect: 'manual',
  });
  const sc = r.headers.get('set-cookie');
  if (sc) cookie = sc.split(';')[0];
  log('adminvisit: login http ' + r.status + ' cookie=' + (cookie || 'NONE'));
}

async function get(url) {
  const r = await fetch(BASE + url, {
    headers: { 'Cookie': cookie },
    redirect: 'follow',
  });
  const html = await r.text();
  return { status: r.status, html: html };
}

function resolve(u) {
  try {
    return new URL(u, BASE).href;
  } catch (e) {
    return u;
  }
}

function makeSandbox(url) {
  const fetchShim = (u, opts) => {
    const full = resolve(u);
    log('adminvisit: outgoing ' + full + ' (script on ' + url + ')');
    const headers = {};
    if (opts && opts.headers) {
      for (const k of Object.keys(opts.headers)) headers[k] = opts.headers[k];
    }
    headers['Cookie'] = cookie;
    const o = Object.assign({}, opts || {}, { headers: headers });
    return fetch(full, o).catch(e => {
      log('adminvisit: fetch error ' + e);
      return null;
    });
  };

  class XMLHttpRequest {
    constructor() {
      this._headers = {};
      this.responseText = '';
      this.status = 0;
      this.onload = null;
    }
    open(method, url) {
      this._method = method;
      this._url = url;
    }
    setRequestHeader(k, v) {
      this._headers[k] = v;
    }
    send(body) {
      const full = resolve(this._url);
      log('adminvisit: xhr ' + this._method + ' ' + full + ' (from ' + url + ')');
      const headers = Object.assign({}, this._headers);
      headers['Cookie'] = cookie;
      fetch(full, { method: this._method, headers: headers, body: body || undefined })
        .then(r => {
          this.status = r.status;
          return r.text();
        })
        .then(t => {
          this.responseText = t;
          if (this.onload) this.onload();
        })
        .catch(e => log('adminvisit: xhr error ' + e));
    }
  }

  class Image {
    constructor() {
      this._src = '';
    }
    set src(v) {
      this._src = v;
      const full = resolve(v);
      log('adminvisit: img ' + full + ' (from ' + url + ')');
      fetch(full, { headers: { 'Cookie': cookie } }).catch(e => log('adminvisit: img error ' + e));
    }
    get src() {
      return this._src;
    }
  }

  const stubEl = () => ({
    style: {},
    setAttribute() {},
    getAttribute() {
      return null;
    },
    appendChild() {},
    addEventListener() {},
    innerHTML: '',
    textContent: '',
    value: '',
  });

  const document = {
    get cookie() {
      return cookie;
    },
    set cookie(v) {
      log('adminvisit: script sets cookie ' + v);
    },
    createElement: stubEl,
    createTextNode: () => ({}),
    getElementById: () => stubEl(),
    querySelector: () => null,
    querySelectorAll: () => [],
    getElementsByName: () => [stubEl()],
    head: stubEl(),
    body: stubEl(),
    location: { href: BASE + url },
    write() {},
    writeln() {},
    title: '',
  };

  const location = { href: BASE + url, origin: BASE };
  const window = {
    location: location,
    document: document,
    fetch: fetchShim,
    XMLHttpRequest: XMLHttpRequest,
    Image: Image,
    setTimeout: setTimeout,
    clearTimeout: clearTimeout,
    setInterval: setInterval,
    clearInterval: clearInterval,
    alert: m => log('adminvisit: alert ' + m),
    addEventListener() {},
    removeEventListener() {},
    navigator: { userAgent: 'OsedeAdminVisit/1.0' },
    localStorage: { getItem: () => null, setItem() {}, removeItem() {} },
    history: { pushState() {} },
    open() {},
  };
  window.window = window;
  return { window: window, document: document, fetch: fetchShim, XHR: XMLHttpRequest, Image: Image, location: location };
}

function runScripts(html, url) {
  const re = /<script(?:\s[^>]*)?>([\s\S]*?)<\/script>/gi;
  let m;
  while ((m = re.exec(html)) !== null) {
    const code = m[1].trim();
    if (!code) continue;
    log('adminvisit: executing script on ' + url + ':\n' + code);
    try {
      const s = makeSandbox(url);
      const fn = new Function('window', 'document', 'fetch', 'XMLHttpRequest', 'Image', 'location', code);
      fn(s.window, s.document, s.fetch, s.XHR, s.Image, s.location);
    } catch (e) {
      log('adminvisit: script error: ' + e);
    }
  }
}

async function tick() {
  let r = await get('/');
  runScripts(r.html, '/');
  let r2 = await get('/panel/messages.php');
  if (r2.html.indexOf('Inkorg') === -1) {
    log('adminvisit: session lost, re-login');
    await login();
    r2 = await get('/panel/messages.php');
  }
  runScripts(r2.html, '/panel/messages.php');
  const ids = [];
  const re = /\?read=(\d+)"[^>]*>.*?<\/a> <b>\(oläst\)/g;
  let m;
  while ((m = re.exec(r2.html)) !== null) ids.push(m[1]);
  for (const id of ids) {
    const u = '/panel/messages.php?read=' + id;
    const r3 = await get(u);
    runScripts(r3.html, u);
  }
}

async function main() {
  log('adminvisit: starting, base=' + BASE);
  await login();
  for (;;) {
    try {
      await tick();
    } catch (e) {
      log('adminvisit: tick error: ' + e);
    }
    await new Promise(res => setTimeout(res, 60000));
  }
}

main();
