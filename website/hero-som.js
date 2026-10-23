/* Hero SOM animation: raw DOM tags drift, a plasma sweep passes, they snap into a Semantic Object Model tree. Decorative. */
(function () {
  var box = document.getElementById('hero-som');
  if (!box) return;
  var mqWide = window.matchMedia('(min-width: 1000px)');
  var mqStill = window.matchMedia('(prefers-reduced-motion: reduce)');
  var ORANGE = '232,133,58', BG = '#111110';
  // [role, hint, depth, rawTag]
  var TREE = [
    ['page', 'plasmate.app', 0, '<html>'],
    ['nav', '', 1, '<div>'],
    ['link', 'GitHub', 2, '<a>'],
    ['link', 'Docs', 2, '<span>'],
    ['main', '', 1, '<div>'],
    ['heading', 'The browser engine', 2, '<h1>'],
    ['paragraph', 'Plasmate compiles', 2, '<p>'],
    ['button', 'Install', 2, '<div>'],
    ['form', '', 2, '<form>'],
    ['textbox', 'query', 3, '<input>'],
    ['footer', '', 1, '<div>'],
    ['link', 'Apache-2.0', 2, '<a>']
  ];
  var NOISE = ['<span>', '<br>', '<svg>', '<script>', '<i>', '<div>', '<style>', '<b>', '<li>', '<td>'];
  var PERIOD = 11, SWEEP0 = 3.2, SWEEP1 = 5.4, HOLD1 = 9.2;
  var canvas, ctx, W = 0, H = 0, dpr = 1, nodes = [], raf = 0, visible = false, started = false, t0 = 0, acc = 0, last = 0;
  var seed = 7; function rnd() { seed = (seed * 16807) % 2147483647; return seed / 2147483647; }
  function ease(x) { return x <= 0 ? 0 : x >= 1 ? 1 : x < 0.5 ? 4 * x * x * x : 1 - Math.pow(-2 * x + 2, 3) / 2; }
  function cl(x) { return x < 0 ? 0 : x > 1 ? 1 : x; }

  function build() {
    seed = 7; nodes = [];
    TREE.forEach(function (r, i) { nodes.push({ role: r[0], hint: r[1], d: r[2], tag: r[3], row: i, noise: false }); });
    NOISE.forEach(function (tag) { nodes.push({ tag: tag, noise: true }); });
    nodes.forEach(function (n) {
      n.ax = 0.08 + rnd() * 0.84; n.ay = 0.1 + rnd() * 0.8;
      n.fx = 0.4 + rnd() * 0.6; n.fy = 0.4 + rnd() * 0.6; n.ph = rnd() * 6.28; n.rot = (rnd() - 0.5) * 0.5;
    });
  }

  function size() {
    var r = box.getBoundingClientRect();
    W = Math.round(r.width); H = Math.round(r.height);
    dpr = Math.min(2, window.devicePixelRatio || 1);
    canvas.width = Math.max(1, W * dpr); canvas.height = Math.max(1, H * dpr);
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  }

  function pill(x, y, text, sub, k, alpha, rot) {
    ctx.save(); ctx.translate(x, y); ctx.rotate(rot); ctx.globalAlpha = alpha;
    ctx.font = '500 12px "IBM Plex Mono", ui-monospace, monospace';
    var w = ctx.measureText(text).width + 18, h = 20;
    ctx.fillStyle = k > 0.5 ? 'rgba(40,26,16,0.95)' : 'rgba(26,25,24,0.92)';
    ctx.strokeStyle = 'rgba(' + (k > 0.5 ? ORANGE + ',' + (0.55 + 0.45 * k) : '74,70,64,0.9') + ')';
    ctx.lineWidth = 1;
    ctx.beginPath(); if (ctx.roundRect) ctx.roundRect(0, -h / 2, w, h, 6); else ctx.rect(0, -h / 2, w, h);
    ctx.fill(); ctx.stroke();
    ctx.fillStyle = k > 0.5 ? '#F0EDE8' : '#B5AEA8'; ctx.textBaseline = 'middle';
    ctx.fillText(text, 9, 1);
    if (sub && k > 0.5) {
      ctx.globalAlpha = alpha * cl((k - 0.5) * 2) * 0.75;
      ctx.font = '400 11px "IBM Plex Mono", ui-monospace, monospace';
      ctx.fillStyle = '#8A8480'; ctx.fillText('"' + sub + '"', w + 8, 1);
    }
    ctx.restore();
    return w;
  }

  function frame(t) {
    var tt = t % PERIOD;
    ctx.clearRect(0, 0, W, H);
    var sweepP = (tt - SWEEP0) / (SWEEP1 - SWEEP0); // 0..1 while sweeping
    var sx = -0.15 + sweepP * 1.3; // normalized x of plasma front
    var back = ease((tt - HOLD1) / (PERIOD - HOLD1)); // un-snap
    var rowH = Math.min(24, (H - 24) / TREE.length), top = (H - rowH * (TREE.length - 1)) / 2, left = 18, ind = 22;
    var pos = [];
    nodes.forEach(function (n, i) {
      var dx = (n.ax + 0.035 * Math.sin(t * n.fx + n.ph)) * W, dy = (n.ay + 0.05 * Math.cos(t * n.fy + n.ph)) * H;
      var hit = tt >= SWEEP0 && sx > n.ax ? cl((tt - (SWEEP0 + (n.ax + 0.15) / 1.3 * (SWEEP1 - SWEEP0))) / 0.7) : 0;
      var k = ease(hit) * (1 - back);
      if (n.noise) {
        var a = 0.55 * (1 - k);
        if (a > 0.01) pill(dx, dy, n.tag, '', 0, a, n.rot * (1 - k));
        return;
      }
      var tx = left + n.d * ind, ty = top + n.row * rowH;
      var x = dx + (tx - dx) * k, y = dy + (ty - dy) * k;
      pos[n.row] = { x: x, y: y, k: k };
      n._x = x; n._y = y; n._k = k;
    });
    // connectors
    ctx.lineWidth = 1;
    TREE.forEach(function (r, i) {
      if (!r[2]) return;
      var p; for (var j = i - 1; j >= 0; j--) if (TREE[j][2] === r[2] - 1) { p = j; break; }
      var a = pos[p], b = pos[i], k = Math.min(a.k, b.k);
      if (k < 0.6) return;
      ctx.strokeStyle = 'rgba(' + ORANGE + ',' + (0.45 * cl((k - 0.6) / 0.4)) + ')';
      ctx.beginPath(); ctx.moveTo(a.x + 8, a.y + 10); ctx.lineTo(a.x + 8, b.y); ctx.lineTo(b.x, b.y); ctx.stroke();
    });
    nodes.forEach(function (n) {
      if (n.noise) return;
      var k = n._k;
      pill(n._x, n._y, k > 0.5 ? n.role : n.tag, n.hint, k, 0.6 + 0.4 * k, n.rot * (1 - k));
    });
    // plasma sweep
    if (sweepP > 0 && sweepP < 1) {
      var px = sx * W, g = ctx.createLinearGradient(px - 90, 0, px + 14, 0);
      g.addColorStop(0, 'rgba(' + ORANGE + ',0)'); g.addColorStop(0.85, 'rgba(' + ORANGE + ',0.22)'); g.addColorStop(1, 'rgba(' + ORANGE + ',0)');
      ctx.fillStyle = g; ctx.fillRect(px - 90, 0, 104, H);
      ctx.fillStyle = 'rgba(' + ORANGE + ',0.9)'; ctx.fillRect(px, 0, 1.5, H);
    }
  }

  function loop(now) {
    raf = 0;
    if (!visible || document.hidden) return;
    var dt = Math.min(0.05, (now - last) / 1000); last = now; acc += dt;
    frame(acc);
    raf = requestAnimationFrame(loop);
  }
  function play() { if (!raf && visible && !document.hidden && !mqStill.matches) { last = performance.now(); raf = requestAnimationFrame(loop); } }
  function stop() { if (raf) cancelAnimationFrame(raf); raf = 0; }
  function still() { stop(); frame(SWEEP1 + 1.5); }

  function init() {
    if (started || !mqWide.matches) return;
    started = true;
    canvas = document.createElement('canvas');
    canvas.setAttribute('aria-hidden', 'true');
    box.appendChild(canvas);
    ctx = canvas.getContext('2d');
    if (!ctx) return;
    build(); size();
    var go = function () { if (mqStill.matches) still(); else { frame(acc); play(); } };
    (document.fonts && document.fonts.load ? document.fonts.load('500 12px "IBM Plex Mono"').catch(function () {}) : Promise.resolve()).then(go);
    window.addEventListener('resize', function () { if (!mqWide.matches) { stop(); return; } size(); if (mqStill.matches) still(); else { frame(acc); play(); } });
    document.addEventListener('visibilitychange', function () { document.hidden ? stop() : play(); });
    if (mqStill.addEventListener) mqStill.addEventListener('change', function () { mqStill.matches ? still() : play(); });
  }

  if ('IntersectionObserver' in window) {
    new IntersectionObserver(function (es) {
      visible = es[0].isIntersecting;
      if (visible) { init(); play(); } else stop();
    }).observe(box);
  } else { visible = true; init(); }
  if (mqWide.addEventListener) mqWide.addEventListener('change', function () { if (mqWide.matches) { init(); if (canvas) { size(); play(); } } });
})();
