/* Tiny runtime that renders the design mockups (exported from the design canvas) as plain HTML.
   Template: {{a.b}} lookups, <sc-if value>, <sc-for list as>, onX="{{fn}}" events.
   Logic: class Component extends DCLogic { state; renderVals(); componentDidMount() }.
   For reference only; not part of the app. */
(function () {
  class DCLogic {
    constructor(props) { this.props = props || {}; this.state = {}; }
    setState(patch) { this.state = Object.assign({}, this.state, typeof patch === 'function' ? patch(this.state) : patch); if (this.__render) this.__schedule(); }
  }
  window.DCLogic = DCLogic;
  const EVENTS = { onclick: 'click', ondoubleclick: 'dblclick', onpointerdown: 'pointerdown', onpointermove: 'pointermove', onpointerup: 'pointerup', onmousedown: 'mousedown', onmouseenter: 'mouseenter', onmouseleave: 'mouseleave', oninput: 'input', onchange: 'change', onkeydown: 'keydown', onfocus: 'focus', onblur: 'blur', oncontextmenu: 'contextmenu', ondragover: 'dragover', ondragleave: 'dragleave', ondrop: 'drop' };
  const get = (scope, path) => path.trim().split('.').reduce((o, k) => (o == null ? undefined : o[k]), scope);
  const ONE = /^\{\{\s*([\w.$]+)\s*\}\}$/;
  const interp = (str, scope) => str.replace(/\{\{\s*([\w.$]+)\s*\}\}/g, (_, p) => { const v = get(scope, p); return v == null ? '' : String(v); });
  function render(nodes, scope, parent) {
    nodes.forEach((n) => {
      if (n.nodeType === 3) { parent.appendChild(document.createTextNode(interp(n.nodeValue, scope))); return; }
      if (n.nodeType !== 1) return;
      const tag = n.tagName.toLowerCase();
      if (tag === 'sc-if') { const m = (n.getAttribute('value') || '').match(ONE); if (m && get(scope, m[1])) render(Array.from(n.childNodes), scope, parent); return; }
      if (tag === 'sc-for') {
        const m = (n.getAttribute('list') || '').match(ONE); const as = n.getAttribute('as');
        const list = m ? get(scope, m[1]) : null;
        (list || []).forEach((item, i) => { const sc = Object.create(scope); sc[as] = item; sc.$index = i; render(Array.from(n.childNodes), sc, parent); });
        return;
      }
      const el = n.namespaceURI === 'http://www.w3.org/2000/svg' ? document.createElementNS(n.namespaceURI, n.tagName) : document.createElement(tag);
      Array.from(n.attributes).forEach((a) => {
        const name = a.name;
        if (name.indexOf('hint-placeholder') === 0) return;
        const ev = EVENTS[name.toLowerCase()];
        const m = a.value.match(ONE);
        if (ev) { const fn = m ? get(scope, m[1]) : null; if (typeof fn === 'function') el.addEventListener(ev, fn); return; }
        if (m && (name === 'disabled' || name === 'checked')) { if (get(scope, m[1])) el.setAttribute(name, ''); return; }
        if (name === 'value' && (tag === 'input' || tag === 'textarea')) { el.__value = interp(a.value, scope); return; }
        el.setAttribute(name, interp(a.value, scope));
      });
      render(Array.from(tag === 'template' ? n.content.childNodes : n.childNodes), scope, el);
      if (el.__value !== undefined) { if (tag === 'textarea') el.textContent = el.__value; el.value = el.__value; }
      parent.appendChild(el);
    });
  }
  const pathOf = (el, root) => { const p = []; while (el && el !== root) { const par = el.parentNode; if (!par) return null; p.unshift(Array.prototype.indexOf.call(par.childNodes, el)); el = par; } return p; };
  const byPath = (root, p) => p.reduce((el, i) => (el ? el.childNodes[i] : null), root);
  window.addEventListener('DOMContentLoaded', () => {
    const host = document.querySelector('x-dc');
    const tpl = document.createElement('template');
    const helmet = host.querySelector('helmet');
    if (helmet) { Array.from(helmet.children).forEach((c) => document.head.appendChild(c)); helmet.remove(); }
    tpl.innerHTML = host.innerHTML;
    host.innerHTML = '';
    const root = document.createElement('div'); host.replaceWith(root);
    const src = document.querySelector('script[type="text/x-dc"]').textContent;
    const Component = new Function('DCLogic', src + '\nreturn Component;')(DCLogic);
    const c = new Component({});
    let queued = false;
    c.__render = () => {
      queued = false;
      const act = document.activeElement; const ap = act && root.contains(act) ? pathOf(act, root) : null;
      const sel = act && 'selectionStart' in act ? [act.selectionStart, act.selectionEnd] : null;
      const frag = document.createDocumentFragment();
      render(Array.from(tpl.content.childNodes), c.renderVals(), frag);
      root.replaceChildren(frag);
      if (ap) { const el = byPath(root, ap); if (el && el.focus) { el.focus(); if (sel && 'setSelectionRange' in el) { try { el.setSelectionRange(sel[0], sel[1]); } catch (e) {} } } }
    };
    c.__schedule = () => { if (!queued) { queued = true; queueMicrotask(c.__render); } };
    c.__render();
    if (c.componentDidMount) c.componentDidMount();
  });
})();
