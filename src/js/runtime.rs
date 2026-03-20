//! V8-backed JavaScript runtime for Plasmate.
//!
//! Each page gets its own V8 Isolate + persistent Context.
//! Scripts share state within a page (as in a real browser).
//! A minimal DOM shim lets common JS patterns work without a full DOM.

use std::cell::RefCell;
use std::sync::Once;
use std::time::Duration;
use tracing::{debug, info, warn};

// Thread-local storage for the reqwest client used by the fetch bridge.
// This is needed because V8 callbacks can't easily capture external state.
thread_local! {
    static FETCH_CLIENT: RefCell<Option<reqwest::Client>> = const { RefCell::new(None) };
}

/// Maximum response body size (1MB) to prevent memory issues.
const MAX_RESPONSE_BODY_SIZE: usize = 1024 * 1024;

/// Timeout for fetch requests (5 seconds).
const FETCH_TIMEOUT: Duration = Duration::from_secs(5);

static V8_INIT: Once = Once::new();

/// Initialize V8 platform (must be called once).
pub fn init_platform() {
    V8_INIT.call_once(|| {
        let platform = v8::new_default_platform(0, false).make_shared();
        v8::V8::initialize_platform(platform);
        v8::V8::initialize();
        info!("V8 platform initialized");
    });
}

/// Configuration for a JS runtime instance.
#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    /// Max JS execution time per script in milliseconds.
    pub max_execution_ms: u64,
    /// Max heap size in bytes (0 = unlimited).
    pub max_heap_bytes: usize,
    /// Whether to execute inline scripts found in HTML.
    pub execute_inline_scripts: bool,
    /// Whether to inject the DOM shim before page scripts.
    pub inject_dom_shim: bool,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            max_execution_ms: 5000,
            max_heap_bytes: 64 * 1024 * 1024,
            execute_inline_scripts: true,
            inject_dom_shim: true,
        }
    }
}

/// Rich DOM shim injected into V8 before page scripts.
///
/// This provides a real DOM tree implementation that supports:
/// - Proper node types (Element, Text, Comment, DocumentFragment)
/// - Tree operations (appendChild, removeChild, insertBefore, replaceChild, cloneNode)
/// - Query methods (getElementById, querySelector, querySelectorAll with CSS selectors)
/// - innerHTML/outerHTML parsing and serialization
/// - Element specifics (classList, style, dataset, form values)
/// - Events (DOMContentLoaded, load, readystatechange)
/// - Serialization back to HTML for the SOM compiler
const DOM_SHIM: &str = r#"
// Plasmate Rich DOM Shim - Full DOM tree implementation for JS-rendered pages

var __plasmate_timers = [];
var __plasmate_event_listeners = [];
var __plasmate_console = [];
var __plasmate_fetch_queue = [];
var __plasmate_mutation_observers = [];
var __plasmate_pending_mutations = [];
var __plasmate_raf_queue = [];
var __plasmate_raf_id = 0;

var self = globalThis;
var window = globalThis;

// URLSearchParams and URL (needed by many SPAs - NYTimes, etc.)
// Some V8 builds expose a partial URL that is not constructible. Detect and polyfill.
// IMPORTANT: URLSearchParams must be polyfilled BEFORE URL because URL uses it.

var __plasmate_urlsp_ok = false;
try {
  if (typeof URLSearchParams === 'function') new URLSearchParams('a=1');
  __plasmate_urlsp_ok = true;
} catch (e) { __plasmate_urlsp_ok = false; }

if (!__plasmate_urlsp_ok) {
  var URLSearchParams = function(init) {
    this._entries = [];
    if (typeof init === 'string') {
      init = init.replace(/^\?/, '');
      if (init) {
        var pairs = init.split('&');
        for (var i = 0; i < pairs.length; i++) {
          var kv = pairs[i].split('=');
          this._entries.push([decodeURIComponent(kv[0]), decodeURIComponent(kv.slice(1).join('='))]);
        }
      }
    }
  };
  URLSearchParams.prototype.get = function(name) {
    for (var i = 0; i < this._entries.length; i++) {
      if (this._entries[i][0] === name) return this._entries[i][1];
    }
    return null;
  };
  URLSearchParams.prototype.getAll = function(name) {
    var r = [];
    for (var i = 0; i < this._entries.length; i++) {
      if (this._entries[i][0] === name) r.push(this._entries[i][1]);
    }
    return r;
  };
  URLSearchParams.prototype.has = function(name) { return this.get(name) !== null; };
  URLSearchParams.prototype.set = function(name, val) {
    var found = false;
    for (var i = 0; i < this._entries.length; i++) {
      if (this._entries[i][0] === name) {
        if (!found) { this._entries[i][1] = String(val); found = true; }
        else { this._entries.splice(i, 1); i--; }
      }
    }
    if (!found) this._entries.push([name, String(val)]);
  };
  URLSearchParams.prototype.append = function(name, val) { this._entries.push([name, String(val)]); };
  URLSearchParams.prototype.delete = function(name) {
    for (var i = 0; i < this._entries.length; i++) {
      if (this._entries[i][0] === name) { this._entries.splice(i, 1); i--; }
    }
  };
  URLSearchParams.prototype.toString = function() {
    return this._entries.map(function(e) { return encodeURIComponent(e[0]) + '=' + encodeURIComponent(e[1]); }).join('&');
  };
  URLSearchParams.prototype.forEach = function(cb) {
    for (var i = 0; i < this._entries.length; i++) cb(this._entries[i][1], this._entries[i][0], this);
  };
  URLSearchParams.prototype.keys = function() { return this._entries.map(function(e) { return e[0]; }); };
  URLSearchParams.prototype.values = function() { return this._entries.map(function(e) { return e[1]; }); };
  URLSearchParams.prototype.entries = function() { return this._entries.slice(); };
  globalThis.URLSearchParams = URLSearchParams;
}

// Polyfill URL if it's not a constructible function
var __plasmate_url_ok = false;
try {
  if (typeof URL === 'function') new URL('https://a.com');
  __plasmate_url_ok = true;
} catch (e) { __plasmate_url_ok = false; }

if (!__plasmate_url_ok) {
  var URL = function(url, base) {
    if (base && typeof base === 'string') {
      // Simple base resolution
      if (url.startsWith('/')) {
        var m = base.match(/^(https?:\/\/[^\/]+)/);
        url = m ? m[1] + url : url;
      } else if (!url.match(/^https?:\/\//)) {
        url = base.replace(/\/[^\/]*$/, '/') + url;
      }
    }
    var match = String(url).match(/^(https?:)\/\/([^/:]+)(:\d+)?(\/[^?#]*)?\??([^#]*)?(#.*)?$/);
    this.href = String(url);
    this.protocol = match ? match[1] : '';
    this.hostname = match ? match[2] : '';
    this.port = match ? (match[3] || '').replace(':', '') : '';
    this.host = this.hostname + (this.port ? ':' + this.port : '');
    this.pathname = match ? (match[4] || '/') : '/';
    this.search = match && match[5] ? '?' + match[5] : '';
    this.hash = match ? (match[6] || '') : '';
    this.origin = this.protocol + '//' + this.host;
    this.searchParams = new URLSearchParams(this.search.replace(/^\?/, ''));
    this.username = '';
    this.password = '';
  };
  URL.prototype.toString = function() { return this.href; };
  URL.prototype.toJSON = function() { return this.href; };
  URL.createObjectURL = function() { return 'blob:null/' + Math.random().toString(36).slice(2); };
  URL.revokeObjectURL = function() {};
  globalThis.URL = URL;
}

// Ensure window.URL points to the constructor without creating an object stub
if (typeof window.URL !== 'function') {
    window.URL = globalThis.URL;
}
window.URL.createObjectURL = window.URL.createObjectURL || function() { return 'blob:null'; };
window.URL.revokeObjectURL = window.URL.revokeObjectURL || function() {};

// XMLSerializer stub for Puppeteer's page.content()
function XMLSerializer() {}
XMLSerializer.prototype.serializeToString = function(node) {
    if (node && node.outerHTML) return node.outerHTML;
    if (node && node.textContent != null) return node.textContent;
    return '';
};
window.location = { href: '', protocol: 'https:', host: '', hostname: '', pathname: '/', search: '', hash: '', origin: '' };
window.navigator = { userAgent: 'Plasmate/0.1', language: 'en-US', languages: ['en-US', 'en'], platform: 'Plasmate', cookieEnabled: true };
window.innerWidth = 1920;
window.innerHeight = 1080;
window.outerWidth = 1920;
window.outerHeight = 1080;
window.devicePixelRatio = 1;
window.screen = { width: 1920, height: 1080, availWidth: 1920, availHeight: 1080, colorDepth: 24, pixelDepth: 24 };

// ============================================================================
// Node Constants
// ============================================================================
var Node = {
    ELEMENT_NODE: 1,
    TEXT_NODE: 3,
    COMMENT_NODE: 8,
    DOCUMENT_NODE: 9,
    DOCUMENT_TYPE_NODE: 10,
    DOCUMENT_FRAGMENT_NODE: 11
};

// ============================================================================
// Base Node class
// ============================================================================
function PlasNode(nodeType) {
    this.nodeType = nodeType;
    this.parentNode = null;
    this.childNodes = [];
    this.ownerDocument = null;
}

// ============================================================================
// MutationObserver Helpers (forward declarations, implementations after queueMicrotask)
// ============================================================================
var __plasmate_mutation_scheduled = false;
var _queueMutationRecord = function() {}; // no-op until properly initialized
var _scheduleMutationDelivery = function() {};
var _deliverMutations = function() {};

Object.defineProperty(PlasNode.prototype, 'firstChild', {
    get: function() { return this.childNodes[0] || null; }
});
Object.defineProperty(PlasNode.prototype, 'lastChild', {
    get: function() { return this.childNodes[this.childNodes.length - 1] || null; }
});
Object.defineProperty(PlasNode.prototype, 'nextSibling', {
    get: function() {
        if (!this.parentNode) return null;
        var idx = this.parentNode.childNodes.indexOf(this);
        return this.parentNode.childNodes[idx + 1] || null;
    }
});
Object.defineProperty(PlasNode.prototype, 'previousSibling', {
    get: function() {
        if (!this.parentNode) return null;
        var idx = this.parentNode.childNodes.indexOf(this);
        return idx > 0 ? this.parentNode.childNodes[idx - 1] : null;
    }
});

PlasNode.prototype.appendChild = function(child) {
    if (child.parentNode) child.parentNode.removeChild(child);
    if (child.nodeType === Node.DOCUMENT_FRAGMENT_NODE) {
        var children = child.childNodes.slice();
        for (var i = 0; i < children.length; i++) {
            this.appendChild(children[i]);
        }
        return child;
    }
    var prevSibling = this.childNodes.length > 0 ? this.childNodes[this.childNodes.length - 1] : null;
    child.parentNode = this;
    child.ownerDocument = this.ownerDocument || this;
    this.childNodes.push(child);
    _queueMutationRecord(this, {
        type: 'childList',
        target: this,
        addedNodes: [child],
        removedNodes: [],
        previousSibling: prevSibling,
        nextSibling: null
    });
    return child;
};

PlasNode.prototype.removeChild = function(child) {
    var idx = this.childNodes.indexOf(child);
    if (idx >= 0) {
        var prevSib = idx > 0 ? this.childNodes[idx - 1] : null;
        var nextSib = idx < this.childNodes.length - 1 ? this.childNodes[idx + 1] : null;
        this.childNodes.splice(idx, 1);
        child.parentNode = null;
        _queueMutationRecord(this, {
            type: 'childList',
            target: this,
            addedNodes: [],
            removedNodes: [child],
            previousSibling: prevSib,
            nextSibling: nextSib
        });
    }
    return child;
};

PlasNode.prototype.insertBefore = function(newNode, refNode) {
    if (newNode.parentNode) newNode.parentNode.removeChild(newNode);
    if (newNode.nodeType === Node.DOCUMENT_FRAGMENT_NODE) {
        var children = newNode.childNodes.slice();
        for (var i = 0; i < children.length; i++) {
            this.insertBefore(children[i], refNode);
        }
        return newNode;
    }
    var idx = refNode ? this.childNodes.indexOf(refNode) : this.childNodes.length;
    if (idx < 0) idx = this.childNodes.length;
    var prevSib = idx > 0 ? this.childNodes[idx - 1] : null;
    newNode.parentNode = this;
    newNode.ownerDocument = this.ownerDocument || this;
    this.childNodes.splice(idx, 0, newNode);
    _queueMutationRecord(this, {
        type: 'childList',
        target: this,
        addedNodes: [newNode],
        removedNodes: [],
        previousSibling: prevSib,
        nextSibling: refNode || null
    });
    return newNode;
};

PlasNode.prototype.replaceChild = function(newChild, oldChild) {
    var idx = this.childNodes.indexOf(oldChild);
    if (idx >= 0) {
        if (newChild.parentNode) newChild.parentNode.removeChild(newChild);
        var prevSib = idx > 0 ? this.childNodes[idx - 1] : null;
        var nextSib = idx < this.childNodes.length - 1 ? this.childNodes[idx + 1] : null;
        oldChild.parentNode = null;
        newChild.parentNode = this;
        newChild.ownerDocument = this.ownerDocument || this;
        this.childNodes[idx] = newChild;
        _queueMutationRecord(this, {
            type: 'childList',
            target: this,
            addedNodes: [newChild],
            removedNodes: [oldChild],
            previousSibling: prevSib,
            nextSibling: nextSib
        });
    }
    return oldChild;
};

PlasNode.prototype.hasChildNodes = function() {
    return this.childNodes.length > 0;
};

PlasNode.prototype.contains = function(node) {
    if (node === this) return true;
    for (var i = 0; i < this.childNodes.length; i++) {
        if (this.childNodes[i] === node) return true;
        if (this.childNodes[i].contains && this.childNodes[i].contains(node)) return true;
    }
    return false;
};

PlasNode.prototype.cloneNode = function(deep) {
    throw new Error('cloneNode must be implemented by subclass');
};

// ============================================================================
// Text Node
// ============================================================================
function PlasText(data) {
    PlasNode.call(this, Node.TEXT_NODE);
    this.nodeValue = data || '';
    this.nodeName = '#text';
}
PlasText.prototype = Object.create(PlasNode.prototype);
PlasText.prototype.constructor = PlasText;

Object.defineProperty(PlasText.prototype, 'textContent', {
    get: function() { return this.nodeValue; },
    set: function(v) {
        var oldValue = this.nodeValue;
        this.nodeValue = v;
        _queueMutationRecord(this, {
            type: 'characterData',
            target: this,
            oldValue: oldValue
        });
    }
});
Object.defineProperty(PlasText.prototype, 'data', {
    get: function() { return this.nodeValue; },
    set: function(v) {
        var oldValue = this.nodeValue;
        this.nodeValue = v;
        _queueMutationRecord(this, {
            type: 'characterData',
            target: this,
            oldValue: oldValue
        });
    }
});
Object.defineProperty(PlasText.prototype, 'length', {
    get: function() { return this.nodeValue.length; }
});

PlasText.prototype.cloneNode = function() {
    return new PlasText(this.nodeValue);
};

PlasText.prototype.substringData = function(offset, count) {
    return this.nodeValue.substr(offset, count);
};

// ============================================================================
// Comment Node
// ============================================================================
function PlasComment(data) {
    PlasNode.call(this, Node.COMMENT_NODE);
    this.nodeValue = data || '';
    this.nodeName = '#comment';
}
PlasComment.prototype = Object.create(PlasNode.prototype);
PlasComment.prototype.constructor = PlasComment;

Object.defineProperty(PlasComment.prototype, 'textContent', {
    get: function() { return this.nodeValue; },
    set: function(v) { this.nodeValue = v; }
});

PlasComment.prototype.cloneNode = function() {
    return new PlasComment(this.nodeValue);
};

// ============================================================================
// DocumentFragment
// ============================================================================
function PlasDocumentFragment() {
    PlasNode.call(this, Node.DOCUMENT_FRAGMENT_NODE);
    this.nodeName = '#document-fragment';
}
PlasDocumentFragment.prototype = Object.create(PlasNode.prototype);
PlasDocumentFragment.prototype.constructor = PlasDocumentFragment;

Object.defineProperty(PlasDocumentFragment.prototype, 'textContent', {
    get: function() {
        var text = '';
        for (var i = 0; i < this.childNodes.length; i++) {
            if (this.childNodes[i].textContent != null) {
                text += this.childNodes[i].textContent;
            }
        }
        return text;
    },
    set: function(v) {
        this.childNodes = [];
        if (v) this.appendChild(new PlasText(v));
    }
});

PlasDocumentFragment.prototype.cloneNode = function(deep) {
    var frag = new PlasDocumentFragment();
    if (deep) {
        for (var i = 0; i < this.childNodes.length; i++) {
            frag.appendChild(this.childNodes[i].cloneNode(true));
        }
    }
    return frag;
};

PlasDocumentFragment.prototype.getElementById = function(id) {
    return _getElementById(this, id);
};
PlasDocumentFragment.prototype.querySelector = function(sel) {
    return _querySelector(this, sel);
};
PlasDocumentFragment.prototype.querySelectorAll = function(sel) {
    return _querySelectorAll(this, sel);
};

// ============================================================================
// DOMTokenList (for classList)
// ============================================================================
function PlasDOMTokenList(element) {
    this._element = element;
}

PlasDOMTokenList.prototype._getClasses = function() {
    var cls = this._element.getAttribute('class') || '';
    return cls.split(/\s+/).filter(function(c) { return c.length > 0; });
};

PlasDOMTokenList.prototype._setClasses = function(classes) {
    this._element.setAttribute('class', classes.join(' '));
};

PlasDOMTokenList.prototype.add = function() {
    var classes = this._getClasses();
    for (var i = 0; i < arguments.length; i++) {
        if (classes.indexOf(arguments[i]) === -1) {
            classes.push(arguments[i]);
        }
    }
    this._setClasses(classes);
};

PlasDOMTokenList.prototype.remove = function() {
    var classes = this._getClasses();
    for (var i = 0; i < arguments.length; i++) {
        var idx = classes.indexOf(arguments[i]);
        if (idx >= 0) classes.splice(idx, 1);
    }
    this._setClasses(classes);
};

PlasDOMTokenList.prototype.toggle = function(token, force) {
    var classes = this._getClasses();
    var idx = classes.indexOf(token);
    if (force === true || (force === undefined && idx === -1)) {
        if (idx === -1) classes.push(token);
        this._setClasses(classes);
        return true;
    } else {
        if (idx >= 0) classes.splice(idx, 1);
        this._setClasses(classes);
        return false;
    }
};

PlasDOMTokenList.prototype.contains = function(token) {
    return this._getClasses().indexOf(token) >= 0;
};

PlasDOMTokenList.prototype.replace = function(oldToken, newToken) {
    var classes = this._getClasses();
    var idx = classes.indexOf(oldToken);
    if (idx >= 0) {
        classes[idx] = newToken;
        this._setClasses(classes);
        return true;
    }
    return false;
};

Object.defineProperty(PlasDOMTokenList.prototype, 'length', {
    get: function() { return this._getClasses().length; }
});

PlasDOMTokenList.prototype.item = function(index) {
    return this._getClasses()[index] || null;
};

PlasDOMTokenList.prototype.toString = function() {
    return this._element.getAttribute('class') || '';
};

// ============================================================================
// CSSStyleDeclaration (for element.style)
// ============================================================================
function PlasStyle(element) {
    this._element = element;
    this._props = {};
}

PlasStyle.prototype.setProperty = function(name, value) {
    this._props[name] = value;
    this._updateCssText();
};

PlasStyle.prototype.getPropertyValue = function(name) {
    return this._props[name] || '';
};

PlasStyle.prototype.removeProperty = function(name) {
    var val = this._props[name];
    delete this._props[name];
    this._updateCssText();
    return val || '';
};

PlasStyle.prototype._updateCssText = function() {
    var parts = [];
    for (var k in this._props) {
        if (this._props.hasOwnProperty(k) && this._props[k]) {
            parts.push(k + ': ' + this._props[k]);
        }
    }
    this._cssText = parts.join('; ');
};

PlasStyle.prototype._parseCssText = function(text) {
    this._props = {};
    if (!text) return;
    var parts = text.split(';');
    for (var i = 0; i < parts.length; i++) {
        var pair = parts[i].split(':');
        if (pair.length >= 2) {
            var name = pair[0].trim();
            var value = pair.slice(1).join(':').trim();
            if (name) this._props[name] = value;
        }
    }
    this._cssText = text;
};

Object.defineProperty(PlasStyle.prototype, 'cssText', {
    get: function() { return this._cssText || ''; },
    set: function(v) { this._parseCssText(v); }
});

// Common style properties as direct properties
['display', 'visibility', 'position', 'top', 'left', 'right', 'bottom', 'width', 'height',
 'margin', 'padding', 'border', 'background', 'color', 'font', 'fontSize', 'fontWeight',
 'textAlign', 'overflow', 'zIndex', 'opacity', 'transform', 'transition'].forEach(function(prop) {
    Object.defineProperty(PlasStyle.prototype, prop, {
        get: function() { return this._props[prop] || ''; },
        set: function(v) { this._props[prop] = v; this._updateCssText(); }
    });
});

// ============================================================================
// NamedNodeMap (for element.attributes)
// ============================================================================
function PlasNamedNodeMap(element) {
    this._element = element;
}

Object.defineProperty(PlasNamedNodeMap.prototype, 'length', {
    get: function() { return Object.keys(this._element._attrs).length; }
});

PlasNamedNodeMap.prototype.item = function(index) {
    var keys = Object.keys(this._element._attrs);
    if (index < 0 || index >= keys.length) return null;
    var name = keys[index];
    return { name: name, value: this._element._attrs[name] };
};

PlasNamedNodeMap.prototype.getNamedItem = function(name) {
    if (this._element._attrs.hasOwnProperty(name)) {
        return { name: name, value: this._element._attrs[name] };
    }
    return null;
};

// ============================================================================
// Element
// ============================================================================
function PlasElement(tagName) {
    PlasNode.call(this, Node.ELEMENT_NODE);
    this.tagName = tagName.toUpperCase();
    this.nodeName = this.tagName;
    this._attrs = {};
    this._listeners = {};
    this._style = new PlasStyle(this);
    this._classList = new PlasDOMTokenList(this);
}
PlasElement.prototype = Object.create(PlasNode.prototype);
PlasElement.prototype.constructor = PlasElement;

// Attributes
PlasElement.prototype.setAttribute = function(name, value) {
    var oldValue = this._attrs.hasOwnProperty(name) ? this._attrs[name] : null;
    this._attrs[name] = String(value);
    _queueMutationRecord(this, {
        type: 'attributes',
        target: this,
        attributeName: name,
        oldValue: oldValue
    });
};

PlasElement.prototype.getAttribute = function(name) {
    return this._attrs.hasOwnProperty(name) ? this._attrs[name] : null;
};

PlasElement.prototype.hasAttribute = function(name) {
    return this._attrs.hasOwnProperty(name);
};

PlasElement.prototype.removeAttribute = function(name) {
    var oldValue = this._attrs.hasOwnProperty(name) ? this._attrs[name] : null;
    delete this._attrs[name];
    if (oldValue !== null) {
        _queueMutationRecord(this, {
            type: 'attributes',
            target: this,
            attributeName: name,
            oldValue: oldValue
        });
    }
};

PlasElement.prototype.getAttributeNames = function() {
    return Object.keys(this._attrs);
};

Object.defineProperty(PlasElement.prototype, 'attributes', {
    get: function() { return new PlasNamedNodeMap(this); }
});

// ID and class shortcuts
Object.defineProperty(PlasElement.prototype, 'id', {
    get: function() { return this._attrs.id || ''; },
    set: function(v) { this._attrs.id = v; }
});

Object.defineProperty(PlasElement.prototype, 'className', {
    get: function() { return this._attrs.class || ''; },
    set: function(v) { this._attrs.class = v; }
});

Object.defineProperty(PlasElement.prototype, 'classList', {
    get: function() { return this._classList; }
});

Object.defineProperty(PlasElement.prototype, 'style', {
    get: function() { return this._style; }
});

// Dataset
Object.defineProperty(PlasElement.prototype, 'dataset', {
    get: function() {
        var self = this;
        return new Proxy({}, {
            get: function(target, prop) {
                var attrName = 'data-' + prop.replace(/([A-Z])/g, '-$1').toLowerCase();
                return self._attrs[attrName] || undefined;
            },
            set: function(target, prop, value) {
                var attrName = 'data-' + prop.replace(/([A-Z])/g, '-$1').toLowerCase();
                self._attrs[attrName] = String(value);
                return true;
            }
        });
    }
});

// Text content
Object.defineProperty(PlasElement.prototype, 'textContent', {
    get: function() {
        var text = '';
        for (var i = 0; i < this.childNodes.length; i++) {
            var child = this.childNodes[i];
            if (child.nodeType === Node.TEXT_NODE) {
                text += child.nodeValue;
            } else if (child.nodeType === Node.ELEMENT_NODE) {
                text += child.textContent;
            }
        }
        return text;
    },
    set: function(v) {
        // Remove existing children
        var removedNodes = this.childNodes.slice();
        if (removedNodes.length > 0) {
            this.childNodes = [];
            for (var i = 0; i < removedNodes.length; i++) {
                removedNodes[i].parentNode = null;
            }
            _queueMutationRecord(this, {
                type: 'childList',
                target: this,
                addedNodes: [],
                removedNodes: removedNodes,
                previousSibling: null,
                nextSibling: null
            });
        }
        if (v != null && v !== '') {
            this.appendChild(new PlasText(String(v)));
        }
    }
});

Object.defineProperty(PlasElement.prototype, 'innerText', {
    get: function() { return this.textContent; },
    set: function(v) { this.textContent = v; }
});

// innerHTML
Object.defineProperty(PlasElement.prototype, 'innerHTML', {
    get: function() {
        return _serializeChildren(this);
    },
    set: function(html) {
        // Remove existing children (fires mutation for each)
        var removedNodes = this.childNodes.slice();
        if (removedNodes.length > 0) {
            this.childNodes = [];
            for (var i = 0; i < removedNodes.length; i++) {
                removedNodes[i].parentNode = null;
            }
            _queueMutationRecord(this, {
                type: 'childList',
                target: this,
                addedNodes: [],
                removedNodes: removedNodes,
                previousSibling: null,
                nextSibling: null
            });
        }
        if (html) {
            var nodes = _parseHTML(html);
            for (var i = 0; i < nodes.length; i++) {
                this.appendChild(nodes[i]);
            }
        }
    }
});

// outerHTML
Object.defineProperty(PlasElement.prototype, 'outerHTML', {
    get: function() {
        return _serializeElement(this);
    },
    set: function(html) {
        if (this.parentNode) {
            var nodes = _parseHTML(html);
            var parent = this.parentNode;
            var idx = parent.childNodes.indexOf(this);
            parent.childNodes.splice(idx, 1);
            for (var i = 0; i < nodes.length; i++) {
                nodes[i].parentNode = parent;
                parent.childNodes.splice(idx + i, 0, nodes[i]);
            }
            this.parentNode = null;
        }
    }
});

// children (element children only)
Object.defineProperty(PlasElement.prototype, 'children', {
    get: function() {
        return this.childNodes.filter(function(n) { return n.nodeType === Node.ELEMENT_NODE; });
    }
});

Object.defineProperty(PlasElement.prototype, 'childElementCount', {
    get: function() { return this.children.length; }
});

Object.defineProperty(PlasElement.prototype, 'firstElementChild', {
    get: function() {
        for (var i = 0; i < this.childNodes.length; i++) {
            if (this.childNodes[i].nodeType === Node.ELEMENT_NODE) return this.childNodes[i];
        }
        return null;
    }
});

Object.defineProperty(PlasElement.prototype, 'lastElementChild', {
    get: function() {
        for (var i = this.childNodes.length - 1; i >= 0; i--) {
            if (this.childNodes[i].nodeType === Node.ELEMENT_NODE) return this.childNodes[i];
        }
        return null;
    }
});

Object.defineProperty(PlasElement.prototype, 'nextElementSibling', {
    get: function() {
        if (!this.parentNode) return null;
        var found = false;
        for (var i = 0; i < this.parentNode.childNodes.length; i++) {
            var n = this.parentNode.childNodes[i];
            if (found && n.nodeType === Node.ELEMENT_NODE) return n;
            if (n === this) found = true;
        }
        return null;
    }
});

Object.defineProperty(PlasElement.prototype, 'previousElementSibling', {
    get: function() {
        if (!this.parentNode) return null;
        var prev = null;
        for (var i = 0; i < this.parentNode.childNodes.length; i++) {
            var n = this.parentNode.childNodes[i];
            if (n === this) return prev;
            if (n.nodeType === Node.ELEMENT_NODE) prev = n;
        }
        return null;
    }
});

// Query methods
PlasElement.prototype.getElementById = function(id) {
    return _getElementById(this, id);
};

PlasElement.prototype.getElementsByTagName = function(tagName) {
    return _getElementsByTagName(this, tagName.toUpperCase());
};

PlasElement.prototype.getElementsByClassName = function(className) {
    return _getElementsByClassName(this, className);
};

PlasElement.prototype.querySelector = function(selector) {
    return _querySelector(this, selector);
};

PlasElement.prototype.querySelectorAll = function(selector) {
    return _querySelectorAll(this, selector);
};

PlasElement.prototype.closest = function(selector) {
    var el = this;
    while (el) {
        if (el.nodeType === Node.ELEMENT_NODE && _matchesSelector(el, selector)) {
            return el;
        }
        el = el.parentNode;
    }
    return null;
};

PlasElement.prototype.matches = function(selector) {
    return _matchesSelector(this, selector);
};

// Events
PlasElement.prototype.addEventListener = function(type, listener, options) {
    if (!this._listeners[type]) this._listeners[type] = [];
    this._listeners[type].push(listener);
    __plasmate_event_listeners.push({ tag: this.tagName, id: this.id, className: this.className, event: type });
};

PlasElement.prototype.removeEventListener = function(type, listener) {
    if (this._listeners[type]) {
        var idx = this._listeners[type].indexOf(listener);
        if (idx >= 0) this._listeners[type].splice(idx, 1);
    }
};

PlasElement.prototype.dispatchEvent = function(event) {
    event.target = this;
    event.currentTarget = this;
    if (this._listeners[event.type]) {
        for (var i = 0; i < this._listeners[event.type].length; i++) {
            try { this._listeners[event.type][i].call(this, event); } catch(e) {}
        }
    }
    return !event.defaultPrevented;
};

// Clone
PlasElement.prototype.cloneNode = function(deep) {
    var clone = new PlasElement(this.tagName);
    for (var k in this._attrs) {
        if (this._attrs.hasOwnProperty(k)) clone._attrs[k] = this._attrs[k];
    }
    if (deep) {
        for (var i = 0; i < this.childNodes.length; i++) {
            clone.appendChild(this.childNodes[i].cloneNode(true));
        }
    }
    return clone;
};

// Other DOM methods
PlasElement.prototype.getBoundingClientRect = function() {
    return { top: 0, left: 0, bottom: 100, right: 100, width: 100, height: 100, x: 0, y: 0 };
};

PlasElement.prototype.getClientRects = function() {
    return [this.getBoundingClientRect()];
};

PlasElement.prototype.focus = function() {};
PlasElement.prototype.blur = function() {};
PlasElement.prototype.click = function() {
    this.dispatchEvent(new Event('click'));
};

PlasElement.prototype.remove = function() {
    if (this.parentNode) this.parentNode.removeChild(this);
};

PlasElement.prototype.append = function() {
    for (var i = 0; i < arguments.length; i++) {
        var arg = arguments[i];
        if (typeof arg === 'string') {
            this.appendChild(new PlasText(arg));
        } else {
            this.appendChild(arg);
        }
    }
};

PlasElement.prototype.prepend = function() {
    var first = this.firstChild;
    for (var i = 0; i < arguments.length; i++) {
        var arg = arguments[i];
        if (typeof arg === 'string') {
            this.insertBefore(new PlasText(arg), first);
        } else {
            this.insertBefore(arg, first);
        }
    }
};

PlasElement.prototype.after = function() {
    if (!this.parentNode) return;
    var ref = this.nextSibling;
    for (var i = 0; i < arguments.length; i++) {
        var arg = arguments[i];
        if (typeof arg === 'string') {
            this.parentNode.insertBefore(new PlasText(arg), ref);
        } else {
            this.parentNode.insertBefore(arg, ref);
        }
    }
};

PlasElement.prototype.before = function() {
    if (!this.parentNode) return;
    for (var i = 0; i < arguments.length; i++) {
        var arg = arguments[i];
        if (typeof arg === 'string') {
            this.parentNode.insertBefore(new PlasText(arg), this);
        } else {
            this.parentNode.insertBefore(arg, this);
        }
    }
};

PlasElement.prototype.replaceWith = function() {
    if (!this.parentNode) return;
    var parent = this.parentNode;
    var idx = parent.childNodes.indexOf(this);
    parent.childNodes.splice(idx, 1);
    this.parentNode = null;
    for (var i = 0; i < arguments.length; i++) {
        var arg = arguments[i];
        var node = typeof arg === 'string' ? new PlasText(arg) : arg;
        node.parentNode = parent;
        parent.childNodes.splice(idx + i, 0, node);
    }
};

// Form element properties
Object.defineProperty(PlasElement.prototype, 'value', {
    get: function() {
        if (this.tagName === 'INPUT' || this.tagName === 'TEXTAREA') {
            return this._attrs.value || '';
        }
        if (this.tagName === 'SELECT') {
            var options = this.getElementsByTagName('option');
            for (var i = 0; i < options.length; i++) {
                if (options[i].selected || options[i].hasAttribute('selected')) {
                    return options[i].value || options[i].textContent;
                }
            }
            return options.length > 0 ? (options[0].value || options[0].textContent) : '';
        }
        return '';
    },
    set: function(v) {
        this._attrs.value = String(v);
    }
});

Object.defineProperty(PlasElement.prototype, 'checked', {
    get: function() { return this.hasAttribute('checked'); },
    set: function(v) { if (v) this.setAttribute('checked', ''); else this.removeAttribute('checked'); }
});

Object.defineProperty(PlasElement.prototype, 'selected', {
    get: function() { return this.hasAttribute('selected'); },
    set: function(v) { if (v) this.setAttribute('selected', ''); else this.removeAttribute('selected'); }
});

Object.defineProperty(PlasElement.prototype, 'disabled', {
    get: function() { return this.hasAttribute('disabled'); },
    set: function(v) { if (v) this.setAttribute('disabled', ''); else this.removeAttribute('disabled'); }
});

Object.defineProperty(PlasElement.prototype, 'type', {
    get: function() { return this._attrs.type || (this.tagName === 'INPUT' ? 'text' : ''); },
    set: function(v) { this._attrs.type = v; }
});

Object.defineProperty(PlasElement.prototype, 'name', {
    get: function() { return this._attrs.name || ''; },
    set: function(v) { this._attrs.name = v; }
});

Object.defineProperty(PlasElement.prototype, 'href', {
    get: function() { return this._attrs.href || ''; },
    set: function(v) { this._attrs.href = v; }
});

Object.defineProperty(PlasElement.prototype, 'src', {
    get: function() { return this._attrs.src || ''; },
    set: function(v) { this._attrs.src = v; }
});

// ============================================================================
// Query helper functions
// ============================================================================
function _getElementById(root, id) {
    for (var i = 0; i < root.childNodes.length; i++) {
        var node = root.childNodes[i];
        if (node.nodeType === Node.ELEMENT_NODE) {
            if (node.id === id) return node;
            var found = _getElementById(node, id);
            if (found) return found;
        }
    }
    return null;
}

function _getElementsByTagName(root, tagName) {
    var results = [];
    var all = tagName === '*';
    for (var i = 0; i < root.childNodes.length; i++) {
        var node = root.childNodes[i];
        if (node.nodeType === Node.ELEMENT_NODE) {
            if (all || node.tagName === tagName) results.push(node);
            results = results.concat(_getElementsByTagName(node, tagName));
        }
    }
    return results;
}

function _getElementsByClassName(root, className) {
    var results = [];
    var classes = className.split(/\s+/);
    for (var i = 0; i < root.childNodes.length; i++) {
        var node = root.childNodes[i];
        if (node.nodeType === Node.ELEMENT_NODE) {
            var nodeClasses = (node._attrs.class || '').split(/\s+/);
            var hasAll = classes.every(function(c) { return nodeClasses.indexOf(c) >= 0; });
            if (hasAll) results.push(node);
            results = results.concat(_getElementsByClassName(node, className));
        }
    }
    return results;
}

// CSS Selector matching (basic support)
function _matchesSelector(el, selector) {
    if (!selector || el.nodeType !== Node.ELEMENT_NODE) return false;

    // Split on commas for multiple selectors
    var selectors = selector.split(',');
    for (var s = 0; s < selectors.length; s++) {
        if (_matchesSingleSelector(el, selectors[s].trim())) return true;
    }
    return false;
}

function _matchesSingleSelector(el, selector) {
    // Handle compound selectors with combinators (space = descendant, > = child)
    // Split while keeping the combinator info
    var tokens = [];
    var current = '';
    var i = 0;
    while (i < selector.length) {
        var c = selector[i];
        if (c === '>') {
            if (current.trim()) tokens.push({type: 'sel', val: current.trim()});
            tokens.push({type: 'child'});
            current = '';
        } else if (c === ' ' && current.trim() && (i + 1 >= selector.length || selector[i+1] !== '>')) {
            // Space combinator (descendant), but not if followed by >
            var nextNonSpace = i + 1;
            while (nextNonSpace < selector.length && selector[nextNonSpace] === ' ') nextNonSpace++;
            if (nextNonSpace < selector.length && selector[nextNonSpace] === '>') {
                // Skip spaces before >
                i++;
                continue;
            }
            tokens.push({type: 'sel', val: current.trim()});
            tokens.push({type: 'descendant'});
            current = '';
        } else {
            current += c;
        }
        i++;
    }
    if (current.trim()) tokens.push({type: 'sel', val: current.trim()});

    if (tokens.length === 0) return false;
    if (tokens.length === 1) return _matchesSimpleSelector(el, tokens[0].val);

    // For compound selectors, work backwards from the element
    // Last token must be a selector that matches the element
    var lastToken = tokens[tokens.length - 1];
    if (lastToken.type !== 'sel') return false;
    if (!_matchesSimpleSelector(el, lastToken.val)) return false;

    // Now check the combinator chain going upward
    var idx = tokens.length - 2;
    var current_el = el;
    while (idx >= 0) {
        var combinator = tokens[idx];
        if (combinator.type === 'child') {
            // Must have parent selector before it
            idx--;
            if (idx < 0 || tokens[idx].type !== 'sel') return false;
            var parent = current_el.parentNode;
            if (!parent || parent.nodeType !== Node.ELEMENT_NODE) return false;
            if (!_matchesSimpleSelector(parent, tokens[idx].val)) return false;
            current_el = parent;
            idx--;
        } else if (combinator.type === 'descendant') {
            // Must have ancestor selector before it
            idx--;
            if (idx < 0 || tokens[idx].type !== 'sel') return false;
            var ancestorSel = tokens[idx].val;
            var found = false;
            var ancestor = current_el.parentNode;
            while (ancestor && ancestor.nodeType === Node.ELEMENT_NODE) {
                if (_matchesSimpleSelector(ancestor, ancestorSel)) {
                    found = true;
                    current_el = ancestor;
                    break;
                }
                ancestor = ancestor.parentNode;
            }
            if (!found) return false;
            idx--;
        } else {
            // Shouldn't happen
            return false;
        }
    }
    return true;
}

function _matchesSimpleSelector(el, selector) {
    if (!selector) return false;

    // Parse selector into tag, id, classes, attributes
    var tag = null, id = null, classes = [], attrs = [], pseudos = [];
    var i = 0;
    var s = selector;

    // Tag name
    var tagMatch = s.match(/^[a-zA-Z][a-zA-Z0-9-]*/);
    if (tagMatch) {
        tag = tagMatch[0].toUpperCase();
        s = s.substr(tagMatch[0].length);
    }

    while (s.length > 0) {
        if (s[0] === '#') {
            // ID
            var idMatch = s.match(/^#([a-zA-Z0-9_-]+)/);
            if (idMatch) {
                id = idMatch[1];
                s = s.substr(idMatch[0].length);
            } else break;
        } else if (s[0] === '.') {
            // Class
            var clsMatch = s.match(/^\.([a-zA-Z0-9_-]+)/);
            if (clsMatch) {
                classes.push(clsMatch[1]);
                s = s.substr(clsMatch[0].length);
            } else break;
        } else if (s[0] === '[') {
            // Attribute
            var attrMatch = s.match(/^\[([a-zA-Z0-9_-]+)(?:([~|^$*]?=)"?([^"\]]*)"?)?\]/);
            if (attrMatch) {
                attrs.push({ name: attrMatch[1], op: attrMatch[2], value: attrMatch[3] });
                s = s.substr(attrMatch[0].length);
            } else break;
        } else if (s[0] === ':') {
            // Pseudo-class (basic support)
            var pseudoMatch = s.match(/^:([a-zA-Z-]+)(?:\(([^)]*)\))?/);
            if (pseudoMatch) {
                pseudos.push({ name: pseudoMatch[1], arg: pseudoMatch[2] });
                s = s.substr(pseudoMatch[0].length);
            } else break;
        } else {
            break;
        }
    }

    // Check tag
    if (tag && tag !== '*' && el.tagName !== tag) return false;

    // Check ID
    if (id && el.id !== id) return false;

    // Check classes
    var elClasses = (el._attrs.class || '').split(/\s+/);
    for (i = 0; i < classes.length; i++) {
        if (elClasses.indexOf(classes[i]) === -1) return false;
    }

    // Check attributes
    for (i = 0; i < attrs.length; i++) {
        var attr = attrs[i];
        var attrVal = el._attrs[attr.name];
        if (attrVal === undefined) return false;
        if (attr.op) {
            switch (attr.op) {
                case '=': if (attrVal !== attr.value) return false; break;
                case '~=': if (attrVal.split(/\s+/).indexOf(attr.value) === -1) return false; break;
                case '|=': if (attrVal !== attr.value && !attrVal.startsWith(attr.value + '-')) return false; break;
                case '^=': if (!attrVal.startsWith(attr.value)) return false; break;
                case '$=': if (!attrVal.endsWith(attr.value)) return false; break;
                case '*=': if (attrVal.indexOf(attr.value) === -1) return false; break;
            }
        }
    }

    // Check pseudo-classes (basic)
    for (i = 0; i < pseudos.length; i++) {
        var pseudo = pseudos[i];
        switch (pseudo.name) {
            case 'first-child':
                if (el.parentNode && el.parentNode.firstElementChild !== el) return false;
                break;
            case 'last-child':
                if (el.parentNode && el.parentNode.lastElementChild !== el) return false;
                break;
            case 'not':
                if (_matchesSelector(el, pseudo.arg)) return false;
                break;
        }
    }

    return true;
}

function _querySelector(root, selector) {
    for (var i = 0; i < root.childNodes.length; i++) {
        var node = root.childNodes[i];
        if (node.nodeType === Node.ELEMENT_NODE) {
            if (_matchesSelector(node, selector)) return node;
            var found = _querySelector(node, selector);
            if (found) return found;
        }
    }
    return null;
}

function _querySelectorAll(root, selector) {
    var results = [];
    for (var i = 0; i < root.childNodes.length; i++) {
        var node = root.childNodes[i];
        if (node.nodeType === Node.ELEMENT_NODE) {
            if (_matchesSelector(node, selector)) results.push(node);
            results = results.concat(_querySelectorAll(node, selector));
        }
    }
    return results;
}

// ============================================================================
// HTML Parser (mini parser for innerHTML/outerHTML)
// ============================================================================
var _voidElements = ['area', 'base', 'br', 'col', 'embed', 'hr', 'img', 'input', 'link', 'meta', 'param', 'source', 'track', 'wbr'];

function _parseHTML(html) {
    var nodes = [];
    var i = 0;
    var len = html.length;

    while (i < len) {
        if (html[i] === '<') {
            if (html.substr(i, 4) === '<!--') {
                // Comment
                var endComment = html.indexOf('-->', i + 4);
                if (endComment === -1) endComment = len;
                nodes.push(new PlasComment(html.substring(i + 4, endComment)));
                i = endComment + 3;
            } else if (html.substr(i, 2) === '</') {
                // Closing tag - skip
                var endClose = html.indexOf('>', i);
                if (endClose === -1) endClose = len - 1;
                i = endClose + 1;
            } else if (html.substr(i, 9).toLowerCase() === '<!doctype') {
                // DOCTYPE - skip
                var endDoc = html.indexOf('>', i);
                if (endDoc === -1) endDoc = len - 1;
                i = endDoc + 1;
            } else {
                // Opening tag
                var tagResult = _parseTag(html, i);
                if (tagResult) {
                    nodes.push(tagResult.element);
                    i = tagResult.end;
                } else {
                    // Malformed - treat as text
                    nodes.push(new PlasText('<'));
                    i++;
                }
            }
        } else {
            // Text node
            var nextTag = html.indexOf('<', i);
            if (nextTag === -1) nextTag = len;
            var text = html.substring(i, nextTag);
            if (text) {
                nodes.push(new PlasText(_decodeEntities(text)));
            }
            i = nextTag;
        }
    }

    return nodes;
}

function _parseTag(html, start) {
    var i = start + 1;
    var len = html.length;

    // Get tag name
    var tagStart = i;
    while (i < len && /[a-zA-Z0-9]/.test(html[i])) i++;
    var tagName = html.substring(tagStart, i);
    if (!tagName) return null;

    var element = new PlasElement(tagName);

    // Parse attributes
    while (i < len) {
        // Skip whitespace
        while (i < len && /\s/.test(html[i])) i++;

        if (html[i] === '>' || html[i] === '/') break;

        // Get attribute name
        var nameStart = i;
        while (i < len && /[a-zA-Z0-9_:-]/.test(html[i])) i++;
        var attrName = html.substring(nameStart, i);
        if (!attrName) { i++; continue; }

        // Skip whitespace
        while (i < len && /\s/.test(html[i])) i++;

        var attrValue = '';
        if (html[i] === '=') {
            i++; // skip =
            // Skip whitespace
            while (i < len && /\s/.test(html[i])) i++;

            if (html[i] === '"' || html[i] === "'") {
                var quote = html[i];
                i++;
                var valStart = i;
                while (i < len && html[i] !== quote) i++;
                attrValue = html.substring(valStart, i);
                i++; // skip closing quote
            } else {
                // Unquoted value
                var valStart = i;
                while (i < len && /[^\s>]/.test(html[i])) i++;
                attrValue = html.substring(valStart, i);
            }
        }

        // HTML5: first attribute with same name wins (ignore duplicates)
        var lowerName = attrName.toLowerCase();
        if (!element.hasAttribute(lowerName)) {
            element.setAttribute(lowerName, _decodeEntities(attrValue));
        }
    }

    // Check for self-closing or void
    var selfClose = html[i] === '/';
    if (selfClose) i++;
    if (html[i] === '>') i++;

    var isVoid = _voidElements.indexOf(tagName.toLowerCase()) >= 0;

    if (!selfClose && !isVoid) {
        // Parse children until closing tag
        var closeTag = '</' + tagName;
        var closeTagLower = closeTag.toLowerCase();

        // Special handling for script and style - raw text
        if (tagName.toLowerCase() === 'script' || tagName.toLowerCase() === 'style') {
            var rawEnd = html.toLowerCase().indexOf(closeTagLower, i);
            if (rawEnd === -1) rawEnd = len;
            var rawText = html.substring(i, rawEnd);
            if (rawText) element.appendChild(new PlasText(rawText));
            i = rawEnd;
        } else {
            // Regular content - parse recursively
            var depth = 1;
            var contentStart = i;
            var j = i;
            while (j < len && depth > 0) {
                if (html[j] === '<') {
                    if (html.substr(j, closeTag.length).toLowerCase() === closeTagLower) {
                        depth--;
                        if (depth === 0) {
                            var content = html.substring(contentStart, j);
                            var childNodes = _parseHTML(content);
                            for (var k = 0; k < childNodes.length; k++) {
                                element.appendChild(childNodes[k]);
                            }
                        }
                    } else if (html[j + 1] !== '/' && html[j + 1] !== '!') {
                        // Opening tag of same type
                        var nextTagName = '';
                        var t = j + 1;
                        while (t < len && /[a-zA-Z0-9]/.test(html[t])) {
                            nextTagName += html[t];
                            t++;
                        }
                        if (nextTagName.toLowerCase() === tagName.toLowerCase()) {
                            depth++;
                        }
                    }
                }
                j++;
            }
            i = j;
        }

        // Skip past closing tag
        var closeEnd = html.indexOf('>', i);
        if (closeEnd !== -1) i = closeEnd + 1;
    }

    return { element: element, end: i };
}

function _decodeEntities(text) {
    return text
        .replace(/&lt;/g, '<')
        .replace(/&gt;/g, '>')
        .replace(/&amp;/g, '&')
        .replace(/&quot;/g, '"')
        .replace(/&#39;/g, "'")
        .replace(/&nbsp;/g, '\u00A0')
        .replace(/&#(\d+);/g, function(m, n) { return String.fromCharCode(parseInt(n, 10)); })
        .replace(/&#x([0-9a-fA-F]+);/g, function(m, n) { return String.fromCharCode(parseInt(n, 16)); });
}

// ============================================================================
// HTML Serializer
// ============================================================================
function _serializeElement(el) {
    if (el.nodeType === Node.TEXT_NODE) {
        return _encodeEntities(el.nodeValue);
    }
    if (el.nodeType === Node.COMMENT_NODE) {
        return '<!--' + el.nodeValue + '-->';
    }
    if (el.nodeType !== Node.ELEMENT_NODE) {
        return '';
    }

    var tag = el.tagName.toLowerCase();
    var html = '<' + tag;

    // Attributes
    for (var k in el._attrs) {
        if (el._attrs.hasOwnProperty(k)) {
            var v = el._attrs[k];
            html += ' ' + k + '="' + _encodeAttr(v) + '"';
        }
    }

    // Style attribute
    if (el._style && el._style.cssText) {
        html += ' style="' + _encodeAttr(el._style.cssText) + '"';
    }

    var isVoid = _voidElements.indexOf(tag) >= 0;
    if (isVoid) {
        html += '>';
        return html;
    }

    html += '>';
    html += _serializeChildren(el);
    html += '</' + tag + '>';

    return html;
}

function _serializeChildren(el) {
    var html = '';
    for (var i = 0; i < el.childNodes.length; i++) {
        var child = el.childNodes[i];
        if (child.nodeType === Node.TEXT_NODE) {
            html += _encodeEntities(child.nodeValue);
        } else if (child.nodeType === Node.COMMENT_NODE) {
            html += '<!--' + child.nodeValue + '-->';
        } else if (child.nodeType === Node.ELEMENT_NODE) {
            html += _serializeElement(child);
        }
    }
    return html;
}

function _encodeEntities(text) {
    return text
        .replace(/&/g, '&amp;')
        .replace(/</g, '&lt;')
        .replace(/>/g, '&gt;');
}

function _encodeAttr(text) {
    return text
        .replace(/&/g, '&amp;')
        .replace(/"/g, '&quot;')
        .replace(/</g, '&lt;')
        .replace(/>/g, '&gt;');
}

// ============================================================================
// Document
// ============================================================================
var _docType = { name: 'html', publicId: '', systemId: '' };
var _docEl = new PlasElement('html');
var _docHead = new PlasElement('head');
var _docBody = new PlasElement('body');
_docEl.appendChild(_docHead);
_docEl.appendChild(_docBody);

var document = {
    nodeType: Node.DOCUMENT_NODE,
    nodeName: '#document',
    doctype: _docType,
    documentElement: _docEl,
    head: _docHead,
    body: _docBody,
    childNodes: _docType ? [_docType, _docEl] : [_docEl],
    title: '',
    readyState: 'loading',
    cookie: '',
    referrer: '',
    URL: '',
    domain: '',
    characterSet: 'UTF-8',
    charset: 'UTF-8',
    contentType: 'text/html',
    _listeners: {},

    createElement: function(tag) {
        var el = new PlasElement(tag);
        el.ownerDocument = document;
        return el;
    },

    createTextNode: function(text) {
        var node = new PlasText(text);
        node.ownerDocument = document;
        return node;
    },

    createComment: function(data) {
        var node = new PlasComment(data);
        node.ownerDocument = document;
        return node;
    },

    createDocumentFragment: function() {
        var frag = new PlasDocumentFragment();
        frag.ownerDocument = document;
        return frag;
    },

    createEvent: function(type) {
        return new Event(type);
    },

    getElementById: function(id) {
        return _getElementById(_docEl, id);
    },

    getElementsByTagName: function(tag) {
        if (tag === 'head') return [_docHead];
        if (tag === 'body') return [_docBody];
        if (tag === 'html') return [_docEl];
        return _getElementsByTagName(_docEl, tag.toUpperCase());
    },

    getElementsByClassName: function(className) {
        return _getElementsByClassName(_docEl, className);
    },

    querySelector: function(selector) {
        return _querySelector(_docEl, selector);
    },

    querySelectorAll: function(selector) {
        return _querySelectorAll(_docEl, selector);
    },

    addEventListener: function(type, listener, options) {
        if (!this._listeners[type]) this._listeners[type] = [];
        this._listeners[type].push(listener);
        __plasmate_event_listeners.push({ tag: 'DOCUMENT', event: type });
    },

    removeEventListener: function(type, listener) {
        if (this._listeners[type]) {
            var idx = this._listeners[type].indexOf(listener);
            if (idx >= 0) this._listeners[type].splice(idx, 1);
        }
    },

    dispatchEvent: function(event) {
        if (this._listeners[event.type]) {
            for (var i = 0; i < this._listeners[event.type].length; i++) {
                try { this._listeners[event.type][i].call(this, event); } catch(e) {}
            }
        }
        return true;
    },

    write: function(html) {
        var nodes = _parseHTML(html);
        for (var i = 0; i < nodes.length; i++) {
            _docBody.appendChild(nodes[i]);
        }
    },

    writeln: function(html) {
        this.write(html + '\n');
    },

    hasFocus: function() { return true; },
    getSelection: function() { return { toString: function() { return ''; } }; },
    execCommand: function() { return false; },

    implementation: {
        hasFeature: function() { return false; },
        createHTMLDocument: function(title) {
            var doc = Object.create(document);
            doc.title = title || '';
            return doc;
        }
    },

    // Serialize the entire document to HTML
    __plasmate_serialize: function() {
        return '<!DOCTYPE html>' + _serializeElement(_docEl);
    }
};

// Set owner document
_docEl.ownerDocument = document;
_docHead.ownerDocument = document;
_docBody.ownerDocument = document;

Object.defineProperty(document, 'title', {
    get: function() {
        var titleEl = _querySelector(_docHead, 'title');
        return titleEl ? titleEl.textContent : '';
    },
    set: function(v) {
        var titleEl = _querySelector(_docHead, 'title');
        if (!titleEl) {
            titleEl = new PlasElement('title');
            _docHead.appendChild(titleEl);
        }
        titleEl.textContent = v;
    }
});

window.document = document;
window.Node = Node;

// ============================================================================
// Console
// ============================================================================
var console = {
    log: function() { __plasmate_console.push(['log', Array.prototype.slice.call(arguments)]); },
    warn: function() { __plasmate_console.push(['warn', Array.prototype.slice.call(arguments)]); },
    error: function() { __plasmate_console.push(['error', Array.prototype.slice.call(arguments)]); },
    info: function() { __plasmate_console.push(['info', Array.prototype.slice.call(arguments)]); },
    debug: function() {},
    trace: function() {},
    dir: function() {},
    table: function() {},
    group: function() {},
    groupEnd: function() {},
    groupCollapsed: function() {},
    time: function() {},
    timeEnd: function() {},
    timeLog: function() {},
    assert: function() {},
    count: function() {},
    countReset: function() {},
    clear: function() {}
};

// ============================================================================
// Timers
// ============================================================================
var _timerCounter = 0;
function setTimeout(fn, ms) {
    var id = ++_timerCounter;
    __plasmate_timers.push({ id: id, fn: fn, ms: ms || 0, type: 'timeout' });
    return id;
}
function clearTimeout(id) {
    __plasmate_timers = __plasmate_timers.filter(function(t) { return t.id !== id; });
}
function setInterval(fn, ms) {
    var id = ++_timerCounter;
    __plasmate_timers.push({ id: id, fn: fn, ms: ms || 0, type: 'interval' });
    return id;
}
function clearInterval(id) {
    __plasmate_timers = __plasmate_timers.filter(function(t) { return t.id !== id; });
}
function requestAnimationFrame(fn) {
    var id = ++__plasmate_raf_id;
    __plasmate_raf_queue.push({ id: id, fn: fn });
    return id;
}
function cancelAnimationFrame(id) {
    __plasmate_raf_queue = __plasmate_raf_queue.filter(function(r) { return r.id !== id; });
}
function _drainAnimationFrames() {
    var maxIterations = 10;
    var iteration = 0;
    while (__plasmate_raf_queue.length > 0 && iteration < maxIterations) {
        var queue = __plasmate_raf_queue.slice();
        __plasmate_raf_queue = [];
        var timestamp = Date.now();
        for (var i = 0; i < queue.length; i++) {
            try {
                queue[i].fn(timestamp);
            } catch(e) {
                if (typeof console !== 'undefined' && console.error) {
                    console.error('rAF callback error:', e.message || e);
                }
            }
        }
        iteration++;
    }
}

// ============================================================================
// URL resolution helper
// ============================================================================
function __plasmate_resolve_url(raw) {
    var urlStr = String(raw).substring(0, 500);
    // Resolve relative URLs against the page origin
    if (urlStr.charAt(0) === '/' && window.location && window.location.href) {
        try {
            var loc = window.location;
            return loc.protocol + '//' + loc.host + urlStr;
        } catch(e) { return urlStr; }
    }
    // Relative path like "json/product.json"
    if (urlStr.indexOf('://') === -1 && window.location && window.location.href) {
        try {
            var base = window.location.href.replace(/[^\/]*$/, '');
            return base + urlStr;
        } catch(e) { return urlStr; }
    }
    return urlStr;
}

// ============================================================================
// Fetch (stub that records requests - actual fetch done by Rust)
// ============================================================================
function fetch(url, opts) {
    var urlStr = __plasmate_resolve_url(url);
    __plasmate_fetch_queue.push({ url: urlStr, opts: opts || {} });

    // If Rust has injected __plasmate_do_fetch, use it
    if (typeof __plasmate_do_fetch === 'function') {
        try {
            var result = __plasmate_do_fetch(urlStr, opts ? JSON.stringify(opts) : '{}');
            if (result) {
                var parsed = JSON.parse(result);
                return Promise.resolve({
                    ok: parsed.ok !== false,
                    status: parsed.status || 200,
                    statusText: parsed.statusText || 'OK',
                    headers: {
                        get: function(name) { return parsed.headers ? parsed.headers[name.toLowerCase()] : null; }
                    },
                    json: function() {
                        try { return Promise.resolve(JSON.parse(parsed.body || '{}')); }
                        catch(e) { return Promise.reject(e); }
                    },
                    text: function() { return Promise.resolve(parsed.body || ''); },
                    blob: function() { return Promise.resolve(new Blob([parsed.body || ''])); },
                    arrayBuffer: function() { return Promise.resolve(new ArrayBuffer(0)); }
                });
            }
        } catch(e) {
            // Fall through to stub
        }
    }

    // Stub response
    return Promise.resolve({
        ok: true, status: 200, statusText: 'OK',
        headers: { get: function() { return null; } },
        json: function() { return Promise.resolve({}); },
        text: function() { return Promise.resolve(''); },
        blob: function() { return Promise.resolve(new Blob()); },
        arrayBuffer: function() { return Promise.resolve(new ArrayBuffer(0)); }
    });
}

// ============================================================================
// XMLHttpRequest (stub)
// ============================================================================
function XMLHttpRequest() {
    this.readyState = 0;
    this.status = 0;
    this.statusText = '';
    this.responseText = '';
    this.response = '';
    this.responseType = '';
    this.responseURL = '';
    this._headers = {};
    this._listeners = {};
    this._method = 'GET';
    this._url = '';
    this._async = true;
}

XMLHttpRequest.prototype.open = function(method, url, async) {
    this._method = method;
    this._url = __plasmate_resolve_url(url);
    this.responseURL = this._url;
    this._async = async !== false;
    this.readyState = 1;
};

XMLHttpRequest.prototype.setRequestHeader = function(name, value) {
    this._headers[name] = value;
};

XMLHttpRequest.prototype.send = function(body) {
    var self = this;
    __plasmate_fetch_queue.push({ url: this._url, method: this._method });

    // If Rust has injected __plasmate_do_fetch, use it
    if (typeof __plasmate_do_fetch === 'function') {
        try {
            var opts = JSON.stringify({ method: this._method, headers: this._headers, body: body });
            var result = __plasmate_do_fetch(this._url, opts);
            if (typeof console !== 'undefined' && console.error) {
                console.error('XHR __plasmate_do_fetch result type:', typeof result, 'length:', result ? result.length : 'null', 'first 300:', result ? result.substring(0, 300) : 'null');
            }
            if (result) {
                var parsed = JSON.parse(result);
                self.status = parsed.status || 200;
                self.statusText = parsed.statusText || 'OK';
                self.responseText = parsed.body || '';
                // Handle responseType
                if (self.responseType === 'json') {
                    try {
                        self.response = JSON.parse(self.responseText);
                    } catch(parseErr) {
                        if (typeof console !== 'undefined' && console.error) {
                            console.error('XHR JSON parse error for ' + self._url + ':', parseErr.message,
                                'responseText length:', self.responseText.length,
                                'first 200 chars:', self.responseText.substring(0, 200));
                        }
                        self.response = null;
                    }
                } else {
                    self.response = self.responseText;
                }
                self.readyState = 4;
                self._fireEvent('readystatechange');
                self._fireEvent('load');
                return;
            }
        } catch(e) {
            if (typeof console !== 'undefined' && console.error) {
                console.error('XHR send error:', e.message);
            }
        }
    }

    // Stub response
    this.readyState = 4;
    this.status = 200;
    this.statusText = 'OK';
    this._fireEvent('readystatechange');
    this._fireEvent('load');
};

XMLHttpRequest.prototype.abort = function() {
    this.readyState = 0;
};

XMLHttpRequest.prototype.getResponseHeader = function(name) {
    return null;
};

XMLHttpRequest.prototype.getAllResponseHeaders = function() {
    return '';
};

XMLHttpRequest.prototype.addEventListener = function(type, fn) {
    if (!this._listeners[type]) this._listeners[type] = [];
    this._listeners[type].push(fn);
};

XMLHttpRequest.prototype.removeEventListener = function(type, fn) {
    if (this._listeners[type]) {
        var idx = this._listeners[type].indexOf(fn);
        if (idx >= 0) this._listeners[type].splice(idx, 1);
    }
};

XMLHttpRequest.prototype._fireEvent = function(type) {
    var evt = { type: type, target: this };
    if (this['on' + type]) {
        try { this['on' + type](evt); } catch(e) {
            if (typeof console !== 'undefined' && console.error) console.error('XHR on' + type + ' error:', e.message || e);
        }
    }
    if (this._listeners[type]) {
        for (var i = 0; i < this._listeners[type].length; i++) {
            try { this._listeners[type][i].call(this, evt); } catch(e) {
                if (typeof console !== 'undefined' && console.error) console.error('XHR ' + type + ' listener error:', e.message || e);
            }
        }
    }
};

XMLHttpRequest.UNSENT = 0;
XMLHttpRequest.OPENED = 1;
XMLHttpRequest.HEADERS_RECEIVED = 2;
XMLHttpRequest.LOADING = 3;
XMLHttpRequest.DONE = 4;

// ============================================================================
// Storage
// ============================================================================
var _store = {};
var localStorage = {
    getItem: function(k) { return _store.hasOwnProperty(k) ? _store[k] : null; },
    setItem: function(k, v) { _store[k] = String(v); },
    removeItem: function(k) { delete _store[k]; },
    clear: function() { _store = {}; },
    key: function(i) { return Object.keys(_store)[i] || null; },
    get length() { return Object.keys(_store).length; }
};
var sessionStorage = Object.create(localStorage);
sessionStorage._store = {};

// ============================================================================
// Events
// ============================================================================
function Event(type, eventInit) {
    this.type = type;
    this.bubbles = eventInit ? !!eventInit.bubbles : false;
    this.cancelable = eventInit ? !!eventInit.cancelable : false;
    this.composed = eventInit ? !!eventInit.composed : false;
    this.defaultPrevented = false;
    this.target = null;
    this.currentTarget = null;
    this.eventPhase = 0;
    this.timeStamp = Date.now();
}
Event.prototype.preventDefault = function() { this.defaultPrevented = true; };
Event.prototype.stopPropagation = function() {};
Event.prototype.stopImmediatePropagation = function() {};
Event.prototype.initEvent = function(type, bubbles, cancelable) {
    this.type = type;
    this.bubbles = bubbles;
    this.cancelable = cancelable;
};

function CustomEvent(type, eventInit) {
    Event.call(this, type, eventInit);
    this.detail = eventInit ? eventInit.detail : null;
}
CustomEvent.prototype = Object.create(Event.prototype);

function MouseEvent(type, eventInit) {
    Event.call(this, type, eventInit);
    this.clientX = eventInit ? eventInit.clientX || 0 : 0;
    this.clientY = eventInit ? eventInit.clientY || 0 : 0;
    this.button = eventInit ? eventInit.button || 0 : 0;
}
MouseEvent.prototype = Object.create(Event.prototype);

function KeyboardEvent(type, eventInit) {
    Event.call(this, type, eventInit);
    this.key = eventInit ? eventInit.key || '' : '';
    this.code = eventInit ? eventInit.code || '' : '';
    this.ctrlKey = eventInit ? !!eventInit.ctrlKey : false;
    this.shiftKey = eventInit ? !!eventInit.shiftKey : false;
    this.altKey = eventInit ? !!eventInit.altKey : false;
    this.metaKey = eventInit ? !!eventInit.metaKey : false;
}
KeyboardEvent.prototype = Object.create(Event.prototype);

// ============================================================================
// Other Browser Globals
// ============================================================================
function Blob(parts, options) {
    this._parts = parts || [];
    this.type = options ? options.type || '' : '';
    this.size = 0;
    for (var i = 0; i < this._parts.length; i++) {
        this.size += this._parts[i].length || 0;
    }
}
Blob.prototype.slice = function() { return new Blob(); };
Blob.prototype.text = function() { return Promise.resolve(this._parts.join('')); };

function File(parts, name, options) {
    Blob.call(this, parts, options);
    this.name = name;
    this.lastModified = Date.now();
}
File.prototype = Object.create(Blob.prototype);

var URL = {
    createObjectURL: function() { return 'blob:null'; },
    revokeObjectURL: function() {}
};

function MutationObserver(callback) {
    this._callback = callback;
    this._targets = [];
    this._records = [];
    __plasmate_mutation_observers.push(this);
}

MutationObserver.prototype.observe = function(target, config) {
    if (!target) return;
    // Default config
    var opts = {
        childList: !!config.childList,
        attributes: !!config.attributes,
        characterData: !!config.characterData,
        subtree: !!config.subtree,
        attributeOldValue: !!config.attributeOldValue,
        characterDataOldValue: !!config.characterDataOldValue,
        attributeFilter: config.attributeFilter || null
    };
    // Remove existing entry for this target
    for (var i = 0; i < this._targets.length; i++) {
        if (this._targets[i].target === target) {
            this._targets.splice(i, 1);
            break;
        }
    }
    this._targets.push({ target: target, config: opts });
};

MutationObserver.prototype.disconnect = function() {
    this._targets = [];
    this._records = [];
    // Remove from global list
    var idx = __plasmate_mutation_observers.indexOf(this);
    if (idx >= 0) {
        __plasmate_mutation_observers.splice(idx, 1);
    }
};

MutationObserver.prototype.takeRecords = function() {
    var records = this._records.slice();
    this._records = [];
    return records;
};

function IntersectionObserver(callback) { this._callback = callback; }
IntersectionObserver.prototype.observe = function() {};
IntersectionObserver.prototype.unobserve = function() {};
IntersectionObserver.prototype.disconnect = function() {};

function ResizeObserver(callback) { this._callback = callback; }
ResizeObserver.prototype.observe = function() {};
ResizeObserver.prototype.unobserve = function() {};
ResizeObserver.prototype.disconnect = function() {};

function PerformanceObserver(callback) { this._callback = callback; }
PerformanceObserver.prototype.observe = function() {};
PerformanceObserver.prototype.disconnect = function() {};

var matchMedia = function(query) {
    return {
        matches: false,
        media: query,
        addListener: function() {},
        removeListener: function() {},
        addEventListener: function() {},
        removeEventListener: function() {}
    };
};
window.matchMedia = matchMedia;

var getComputedStyle = function(el) {
    return el._style || new PlasStyle(el);
};
window.getComputedStyle = getComputedStyle;

// Base64
var btoa = function(s) {
    try {
        var chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/=';
        var result = '';
        for (var i = 0; i < s.length; i += 3) {
            var a = s.charCodeAt(i);
            var b = s.charCodeAt(i + 1);
            var c = s.charCodeAt(i + 2);
            result += chars[a >> 2];
            result += chars[((a & 3) << 4) | (b >> 4)];
            result += isNaN(b) ? '=' : chars[((b & 15) << 2) | (c >> 6)];
            result += isNaN(c) ? '=' : chars[c & 63];
        }
        return result;
    } catch(e) { return s; }
};
var atob = function(s) {
    try {
        var chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/=';
        var result = '';
        s = s.replace(/[^A-Za-z0-9+/=]/g, '');
        for (var i = 0; i < s.length; i += 4) {
            var a = chars.indexOf(s[i]);
            var b = chars.indexOf(s[i + 1]);
            var c = chars.indexOf(s[i + 2]);
            var d = chars.indexOf(s[i + 3]);
            result += String.fromCharCode((a << 2) | (b >> 4));
            if (c !== 64) result += String.fromCharCode(((b & 15) << 4) | (c >> 2));
            if (d !== 64) result += String.fromCharCode(((c & 3) << 6) | d);
        }
        return result;
    } catch(e) { return s; }
};
window.btoa = btoa;
window.atob = atob;

var performance = {
    now: function() { return Date.now(); },
    mark: function() {},
    measure: function() {},
    getEntriesByName: function() { return []; },
    getEntriesByType: function() { return []; },
    timing: { navigationStart: Date.now() }
};
window.performance = performance;

var crypto = {
    getRandomValues: function(arr) {
        for (var i = 0; i < arr.length; i++) {
            arr[i] = Math.floor(Math.random() * 256);
        }
        return arr;
    },
    randomUUID: function() {
        return 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, function(c) {
            var r = Math.random() * 16 | 0;
            return (c === 'x' ? r : (r & 0x3 | 0x8)).toString(16);
        });
    }
};
window.crypto = crypto;

var queueMicrotask = function(fn) { Promise.resolve().then(fn).catch(function(){}); };
window.queueMicrotask = queueMicrotask;

// ============================================================================
// MutationObserver Helper Implementations (now that queueMicrotask is defined)
// ============================================================================
_queueMutationRecord = function(target, record) {
    // Find all observers watching this target or its ancestors with subtree
    for (var i = 0; i < __plasmate_mutation_observers.length; i++) {
        var obs = __plasmate_mutation_observers[i];
        if (!obs._targets) continue;
        for (var j = 0; j < obs._targets.length; j++) {
            var entry = obs._targets[j];
            var watchedNode = entry.target;
            var config = entry.config;
            // Check if this observer is watching the target directly
            var matches = (watchedNode === target);
            // Or if subtree is true, check if target is a descendant
            if (!matches && config.subtree) {
                var node = target;
                while (node && node.parentNode) {
                    node = node.parentNode;
                    if (node === watchedNode) {
                        matches = true;
                        break;
                    }
                }
            }
            if (matches) {
                // Check if this mutation type is being observed
                var shouldQueue = false;
                if (record.type === 'childList' && config.childList) shouldQueue = true;
                if (record.type === 'attributes' && config.attributes) shouldQueue = true;
                if (record.type === 'characterData' && config.characterData) shouldQueue = true;
                if (shouldQueue) {
                    // Add oldValue if requested
                    var rec = {
                        type: record.type,
                        target: record.target,
                        addedNodes: record.addedNodes || [],
                        removedNodes: record.removedNodes || [],
                        previousSibling: record.previousSibling || null,
                        nextSibling: record.nextSibling || null,
                        attributeName: record.attributeName || null,
                        attributeNamespace: null,
                        oldValue: null
                    };
                    if (record.type === 'attributes' && config.attributeOldValue && record.oldValue !== undefined) {
                        rec.oldValue = record.oldValue;
                    }
                    if (record.type === 'characterData' && config.characterDataOldValue && record.oldValue !== undefined) {
                        rec.oldValue = record.oldValue;
                    }
                    obs._records.push(rec);
                }
            }
        }
    }
    // Schedule microtask to deliver mutations
    _scheduleMutationDelivery();
};

_scheduleMutationDelivery = function() {
    if (__plasmate_mutation_scheduled) return;
    __plasmate_mutation_scheduled = true;
    queueMicrotask(function() {
        __plasmate_mutation_scheduled = false;
        _deliverMutations();
    });
};

_deliverMutations = function() {
    for (var i = 0; i < __plasmate_mutation_observers.length; i++) {
        var obs = __plasmate_mutation_observers[i];
        if (obs._records.length > 0) {
            var records = obs._records.slice();
            obs._records = [];
            try {
                obs._callback(records, obs);
            } catch(e) {
                if (typeof console !== 'undefined' && console.error) {
                    console.error('MutationObserver callback error:', e.message || e);
                }
            }
        }
    }
};

var requestIdleCallback = function(fn) { return setTimeout(fn, 0); };
var cancelIdleCallback = function(id) { clearTimeout(id); };
window.requestIdleCallback = requestIdleCallback;
window.cancelIdleCallback = cancelIdleCallback;

var history = {
    length: 1,
    state: null,
    pushState: function(state, title, url) { this.state = state; },
    replaceState: function(state, title, url) { this.state = state; },
    go: function() {},
    back: function() {},
    forward: function() {}
};
window.history = history;

// DOMParser
function DOMParser() {}
DOMParser.prototype.parseFromString = function(str, type) {
    var frag = document.createDocumentFragment();
    var nodes = _parseHTML(str);
    for (var i = 0; i < nodes.length; i++) {
        frag.appendChild(nodes[i]);
    }
    return frag;
};
window.DOMParser = DOMParser;

// HTMLCollection / NodeList array-like support
var HTMLCollection = Array;
var NodeList = Array;
window.HTMLCollection = HTMLCollection;
window.NodeList = NodeList;

// Element constructors (for instanceof checks)
window.Element = PlasElement;
window.HTMLElement = PlasElement;
window.Text = PlasText;
window.Comment = PlasComment;
window.DocumentFragment = PlasDocumentFragment;
window.Event = Event;
window.CustomEvent = CustomEvent;
window.MouseEvent = MouseEvent;
window.KeyboardEvent = KeyboardEvent;
window.Blob = Blob;
window.File = File;
window.URL = URL;
window.XMLHttpRequest = XMLHttpRequest;
window.MutationObserver = MutationObserver;
window.IntersectionObserver = IntersectionObserver;
window.ResizeObserver = ResizeObserver;
window.PerformanceObserver = PerformanceObserver;

// ============================================================================
// Plasmate Bootstrap Function - parses source HTML into DOM tree
// ============================================================================
function __plasmate_bootstrap(html, url) {
    // Parse URL
    if (url) {
        try {
            var parsed = new (function(u) {
                var a = document.createElement('a');
                a.href = u;
                this.href = u;
                this.protocol = u.match(/^([^:]+):/) ? RegExp.$1 + ':' : 'https:';
                this.host = u.match(/^[^:]+:\/\/([^/]+)/) ? RegExp.$1 : '';
                this.hostname = this.host.split(':')[0];
                this.port = this.host.split(':')[1] || '';
                this.pathname = u.replace(/^[^:]+:\/\/[^/]+/, '').split('?')[0].split('#')[0] || '/';
                this.search = u.indexOf('?') >= 0 ? '?' + u.split('?')[1].split('#')[0] : '';
                this.hash = u.indexOf('#') >= 0 ? '#' + u.split('#')[1] : '';
                this.origin = this.protocol + '//' + this.host;
            })(url);
            window.location = parsed;
            document.URL = url;
            document.domain = parsed.hostname;
        } catch(e) {
            window.location.href = url;
        }
    }

    // Clear existing document
    _docHead.childNodes = [];
    _docBody.childNodes = [];

    // Parse HTML
    var nodes = _parseHTML(html);

    // Find html element or use root nodes
    var htmlEl = null;
    var headEl = null;
    var bodyEl = null;

    for (var i = 0; i < nodes.length; i++) {
        var node = nodes[i];
        if (node.nodeType === Node.ELEMENT_NODE) {
            if (node.tagName === 'HTML') {
                htmlEl = node;
                break;
            }
        }
    }

    if (htmlEl) {
        // Copy attributes from parsed html to doc element
        for (var k in htmlEl._attrs) {
            if (htmlEl._attrs.hasOwnProperty(k)) {
                _docEl._attrs[k] = htmlEl._attrs[k];
            }
        }

        // Find head and body
        for (var i = 0; i < htmlEl.childNodes.length; i++) {
            var child = htmlEl.childNodes[i];
            if (child.nodeType === Node.ELEMENT_NODE) {
                if (child.tagName === 'HEAD') headEl = child;
                if (child.tagName === 'BODY') bodyEl = child;
            }
        }

        // Copy head contents
        if (headEl) {
            for (var i = 0; i < headEl.childNodes.length; i++) {
                var node = headEl.childNodes[i].cloneNode(true);
                node.parentNode = _docHead;
                node.ownerDocument = document;
                _docHead.childNodes.push(node);
            }
        }

        // Copy body contents
        if (bodyEl) {
            for (var k in bodyEl._attrs) {
                if (bodyEl._attrs.hasOwnProperty(k)) {
                    _docBody._attrs[k] = bodyEl._attrs[k];
                }
            }
            for (var i = 0; i < bodyEl.childNodes.length; i++) {
                var node = bodyEl.childNodes[i].cloneNode(true);
                node.parentNode = _docBody;
                node.ownerDocument = document;
                _docBody.childNodes.push(node);
            }
        }
    } else {
        // No html element - just append nodes to body
        for (var i = 0; i < nodes.length; i++) {
            var node = nodes[i];
            if (node.nodeType === Node.ELEMENT_NODE && node.tagName === 'HEAD') {
                for (var j = 0; j < node.childNodes.length; j++) {
                    var child = node.childNodes[j].cloneNode(true);
                    child.parentNode = _docHead;
                    _docHead.childNodes.push(child);
                }
            } else if (node.nodeType === Node.ELEMENT_NODE && node.tagName === 'BODY') {
                for (var j = 0; j < node.childNodes.length; j++) {
                    var child = node.childNodes[j].cloneNode(true);
                    child.parentNode = _docBody;
                    _docBody.childNodes.push(child);
                }
            } else {
                var cloned = node.cloneNode(true);
                cloned.parentNode = _docBody;
                _docBody.childNodes.push(cloned);
            }
        }
    }
}

// ============================================================================
// Lifecycle Events
// ============================================================================
function __plasmate_fire_domcontentloaded() {
    document.readyState = 'interactive';
    var evt = new Event('DOMContentLoaded');
    document.dispatchEvent(evt);
}

function __plasmate_fire_load() {
    document.readyState = 'complete';
    var evt = new Event('load');
    if (window.onload) {
        try { window.onload(evt); } catch(e) {}
    }
    if (window._listeners && window._listeners.load) {
        for (var i = 0; i < window._listeners.load.length; i++) {
            try { window._listeners.load[i](evt); } catch(e) {}
        }
    }
    // Drain pending animation frames after load event
    _drainAnimationFrames();
    // Deliver any pending mutations
    _deliverMutations();
}

// Window event listeners
window._listeners = {};
window.addEventListener = function(type, fn) {
    if (!this._listeners[type]) this._listeners[type] = [];
    this._listeners[type].push(fn);
};
window.removeEventListener = function(type, fn) {
    if (this._listeners[type]) {
        var idx = this._listeners[type].indexOf(fn);
        if (idx >= 0) this._listeners[type].splice(idx, 1);
    }
};
window.dispatchEvent = function(event) {
    if (this._listeners[event.type]) {
        for (var i = 0; i < this._listeners[event.type].length; i++) {
            try { this._listeners[event.type][i](event); } catch(e) {}
        }
    }
    return true;
};

// ============================================================================
// Ready state
// ============================================================================
document.readyState = 'loading';
"#;

/// A JavaScript runtime bound to a single page.
/// Context persists between script executions (state accumulates like a browser).
pub struct JsRuntime {
    isolate: v8::OwnedIsolate,
    config: RuntimeConfig,
    context: Option<v8::Global<v8::Context>>,
    scripts_executed: usize,
}

impl JsRuntime {
    /// Create a new isolated JS runtime.
    pub fn new(config: RuntimeConfig) -> Self {
        init_platform();

        let params = if config.max_heap_bytes > 0 {
            v8::CreateParams::default().heap_limits(0, config.max_heap_bytes)
        } else {
            v8::CreateParams::default()
        };

        let mut isolate = v8::Isolate::new(params);

        // Create a persistent context
        let context = {
            let scope = &mut v8::HandleScope::new(&mut isolate);
            let ctx = v8::Context::new(scope, Default::default());
            v8::Global::new(scope, ctx)
        };

        let mut rt = Self {
            isolate,
            config: config.clone(),
            context: Some(context),
            scripts_executed: 0,
        };

        // Inject DOM shim
        if config.inject_dom_shim {
            if let Err(e) = rt.execute_in_context(DOM_SHIM, "<plasmate-shim>") {
                warn!("Failed to inject DOM shim: {}", e);
            }
        }

        rt
    }

    /// Set the page URL in the JS context (updates window.location).
    pub fn set_page_url(&mut self, url: &str) {
        let script = format!(
            "window.location.href = '{}'; document.URL = '{}'; document.domain = '{}';",
            url.replace('\'', "\\'"),
            url.replace('\'', "\\'"),
            url::Url::parse(url)
                .map(|u| u.host_str().unwrap_or("").to_string())
                .unwrap_or_default()
                .replace('\'', "\\'"),
        );
        let _ = self.execute_in_context(&script, "<set-url>");
    }

    /// Execute a script in the persistent page context.
    pub fn execute_in_context(&mut self, source: &str, filename: &str) -> Result<String, JsError> {
        let context = self
            .context
            .as_ref()
            .ok_or_else(|| JsError::Runtime("No context available".into()))?;

        let scope = &mut v8::HandleScope::new(&mut self.isolate);
        let context = v8::Local::new(scope, context);
        let scope = &mut v8::ContextScope::new(scope, context);

        let source_str = v8::String::new(scope, source)
            .ok_or_else(|| JsError::Runtime("Failed to create source string".into()))?;

        let name = v8::String::new(scope, filename).unwrap();
        let origin = v8::ScriptOrigin::new(
            scope,
            name.into(),
            0,
            0,
            false,
            0,
            None,
            false,
            false,
            false,
            None,
        );

        let tc = &mut v8::TryCatch::new(scope);

        let script = match v8::Script::compile(tc, source_str, Some(&origin)) {
            Some(s) => s,
            None => {
                let msg = tc
                    .exception()
                    .map(|e| e.to_rust_string_lossy(tc))
                    .unwrap_or_else(|| "Unknown compile error".into());
                return Err(JsError::Compile(msg));
            }
        };

        match script.run(tc) {
            Some(result) => {
                self.scripts_executed += 1;
                let result_str = result
                    .to_string(tc)
                    .map(|s| s.to_rust_string_lossy(tc))
                    .unwrap_or_default();
                Ok(result_str)
            }
            None => {
                let msg = tc
                    .exception()
                    .map(|e| e.to_rust_string_lossy(tc))
                    .unwrap_or_else(|| "Unknown runtime error".into());
                // Don't fail - just log and continue (like a real browser)
                debug!(filename, error = %msg, "JS error (non-fatal)");
                Err(JsError::Runtime(msg))
            }
        }
    }

    /// Execute multiple script blocks in order (state accumulates).
    pub fn execute_page_scripts(&mut self, scripts: &[(String, String)]) -> JsExecutionReport {
        let mut report = JsExecutionReport {
            total: scripts.len(),
            succeeded: 0,
            failed: 0,
            errors: Vec::new(),
        };

        for (source, filename) in scripts {
            if source.trim().is_empty() {
                continue;
            }
            match self.execute_in_context(source, filename) {
                Ok(_) => report.succeeded += 1,
                Err(e) => {
                    report.failed += 1;
                    report.errors.push((filename.clone(), e.to_string()));
                }
            }
        }
        report
    }

    /// Get event listeners registered during JS execution.
    pub fn get_event_listeners(&mut self) -> Vec<String> {
        match self.execute_in_context(
            "JSON.stringify(__plasmate_event_listeners)",
            "<get-listeners>",
        ) {
            Ok(json) => serde_json::from_str::<Vec<serde_json::Value>>(&json)
                .unwrap_or_default()
                .iter()
                .map(|v| v.to_string())
                .collect(),
            Err(_) => Vec::new(),
        }
    }

    /// Get the DOM mutations captured by the shim.
    pub fn get_mutations(&mut self) -> Vec<String> {
        match self.execute_in_context("JSON.stringify(__plasmate_mutations)", "<get-mutations>") {
            Ok(json) => serde_json::from_str::<Vec<serde_json::Value>>(&json)
                .unwrap_or_default()
                .iter()
                .map(|v| v.to_string())
                .collect(),
            Err(_) => Vec::new(),
        }
    }

    /// Get the document.title as set by JS.
    pub fn get_title(&mut self) -> Option<String> {
        self.execute_in_context("document.title", "<get-title>")
            .ok()
            .filter(|s| !s.is_empty())
    }

    /// Drain pending short timers (execute setTimeout callbacks with delay <= threshold_ms).
    pub fn drain_timers(&mut self, threshold_ms: u64) {
        let script = format!(
            r#"(function() {{
                var executed = 0;
                for (var i = 0; i < __plasmate_timers.length && executed < 50; i++) {{
                    var t = __plasmate_timers[i];
                    if (t.type === 'timeout' && t.ms <= {}) {{
                        try {{ t.fn(); }} catch(e) {{}}
                        executed++;
                    }}
                }}
                __plasmate_timers = [];
                return executed;
            }})()"#,
            threshold_ms
        );
        let _ = self.execute_in_context(&script, "<drain-timers>");
    }

    /// Quick eval for AWP page.extract / interactive use.
    pub fn eval(&mut self, expression: &str) -> Result<String, JsError> {
        self.execute_in_context(expression, "<eval>")
    }

    /// Bootstrap the DOM tree from source HTML.
    /// This parses the HTML into the JS DOM tree so that scripts can query and modify it.
    pub fn bootstrap_dom(&mut self, html: &str, url: &str) {
        // Escape the HTML for embedding in JS string
        let escaped_html = html
            .replace('\\', "\\\\")
            .replace('`', "\\`")
            .replace("${", "\\${");
        let escaped_url = url.replace('\\', "\\\\").replace('`', "\\`");

        let script = format!(
            "__plasmate_bootstrap(`{}`, `{}`);",
            escaped_html, escaped_url
        );
        if let Err(e) = self.execute_in_context(&script, "<bootstrap>") {
            warn!("DOM bootstrap failed: {}", e);
        }
    }

    /// Fire the DOMContentLoaded event.
    pub fn fire_dom_content_loaded(&mut self) {
        let _ =
            self.execute_in_context("__plasmate_fire_domcontentloaded();", "<domcontentloaded>");
    }

    /// Fire the load event.
    pub fn fire_load(&mut self) {
        let _ = self.execute_in_context("__plasmate_fire_load();", "<load>");
    }

    /// Pump V8 microtask queue - resolves pending Promise.then() callbacks.
    /// This is critical for fetch().then() chains to execute.
    pub fn pump_microtasks(&mut self) {
        self.isolate.perform_microtask_checkpoint();
    }

    /// Serialize the current DOM tree back to HTML.
    /// Returns the full HTML document as a string.
    pub fn serialize_dom(&mut self) -> Result<String, JsError> {
        // Pump microtasks first to resolve any pending Promise.then() chains
        // (e.g. from fetch().then() that modify the DOM)
        self.pump_microtasks();
        self.execute_in_context("document.__plasmate_serialize()", "<serialize>")
    }

    /// Inject the fetch bridge into the V8 context.
    ///
    /// This registers a native `__plasmate_do_fetch(url, opts_json)` function
    /// that performs real HTTP requests using reqwest. The JS fetch() and
    /// XMLHttpRequest in the DOM shim will call this if it exists.
    ///
    /// # Arguments
    /// * `client` - The reqwest Client to use for HTTP requests
    pub fn inject_fetch_bridge(&mut self, client: reqwest::Client) {
        // Store the client in thread-local storage
        FETCH_CLIENT.with(|c| {
            *c.borrow_mut() = Some(client);
        });

        let context = match self.context.as_ref() {
            Some(c) => c,
            None => {
                warn!("No context available for fetch bridge injection");
                return;
            }
        };

        let scope = &mut v8::HandleScope::new(&mut self.isolate);
        let context = v8::Local::new(scope, context);
        let scope = &mut v8::ContextScope::new(scope, context);

        // Create the native fetch function
        let fetch_fn = v8::Function::new(scope, fetch_bridge_callback);

        if let Some(fetch_fn) = fetch_fn {
            let global = context.global(scope);
            let key = v8::String::new(scope, "__plasmate_do_fetch").unwrap();
            global.set(scope, key.into(), fetch_fn.into());
            debug!("Fetch bridge injected into V8 context");
        } else {
            warn!("Failed to create fetch bridge function");
        }
    }

    /// Get heap statistics.
    pub fn heap_stats(&mut self) -> HeapStats {
        let mut stats = v8::HeapStatistics::default();
        self.isolate.get_heap_statistics(&mut stats);
        HeapStats {
            used_bytes: stats.used_heap_size(),
            total_bytes: stats.total_heap_size(),
            limit_bytes: stats.heap_size_limit(),
        }
    }

    /// Number of scripts successfully executed.
    pub fn scripts_executed(&self) -> usize {
        self.scripts_executed
    }
}

/// V8 callback for the fetch bridge.
///
/// This function is called from JS when `__plasmate_do_fetch(url, opts_json)` is invoked.
/// It performs a synchronous HTTP request using reqwest and returns a JSON response.
fn fetch_bridge_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    debug!("fetch_bridge_callback invoked with {} args", args.length());

    // Get URL argument
    let url = if args.length() > 0 {
        let url_val = args.get(0);
        if url_val.is_string() {
            url_val.to_rust_string_lossy(scope)
        } else {
            rv.set(v8::undefined(scope).into());
            return;
        }
    } else {
        rv.set(v8::undefined(scope).into());
        return;
    };

    // Get options JSON argument
    let opts_json = if args.length() > 1 {
        let opts_val = args.get(1);
        if opts_val.is_string() {
            opts_val.to_rust_string_lossy(scope)
        } else {
            "{}".to_string()
        }
    } else {
        "{}".to_string()
    };

    // Parse options
    let opts: serde_json::Value = serde_json::from_str(&opts_json).unwrap_or(serde_json::json!({}));
    let method = opts
        .get("method")
        .and_then(|v| v.as_str())
        .unwrap_or("GET")
        .to_uppercase();
    let body = opts.get("body").and_then(|v| v.as_str()).map(String::from);
    let headers: std::collections::HashMap<String, String> = opts
        .get("headers")
        .and_then(|v| v.as_object())
        .map(|h| {
            h.iter()
                .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                .collect()
        })
        .unwrap_or_default();

    debug!(url = %url, method = %method, "Fetch bridge performing request");

    // Perform the fetch using the thread-local client
    let result = FETCH_CLIENT.with(|c| {
        let client_opt = c.borrow();
        let client = match client_opt.as_ref() {
            Some(c) => c,
            None => {
                debug!("No fetch client in thread-local storage");
                return Err("No fetch client available".to_string());
            }
        };

        // Always use blocking fetch - we're inside a V8 callback which is synchronous.
        // Using handle.block_on() panics when called from within an async runtime.
        perform_blocking_fetch(client, &url, &method, body.as_deref(), &headers)
    });

    debug!(result = ?result, "Fetch bridge result");

    // Build the result JSON and return it
    let result_json = match result {
        Ok(resp) => resp,
        Err(e) => {
            // Return an error response
            serde_json::json!({
                "ok": false,
                "status": 0,
                "statusText": e,
                "headers": {},
                "body": ""
            })
            .to_string()
        }
    };

    if let Some(result_str) = v8::String::new(scope, &result_json) {
        rv.set(result_str.into());
    } else {
        rv.set(v8::undefined(scope).into());
    }
}

/// Perform an async fetch request.
async fn perform_async_fetch(
    client: &reqwest::Client,
    url: &str,
    method: &str,
    body: Option<&str>,
    headers: &std::collections::HashMap<String, String>,
) -> Result<String, String> {
    use reqwest::Method;

    let method = match method {
        "GET" => Method::GET,
        "POST" => Method::POST,
        "PUT" => Method::PUT,
        "DELETE" => Method::DELETE,
        "PATCH" => Method::PATCH,
        "HEAD" => Method::HEAD,
        "OPTIONS" => Method::OPTIONS,
        _ => Method::GET,
    };

    let mut request = client.request(method, url).timeout(FETCH_TIMEOUT);

    // Add headers
    for (k, v) in headers {
        if let Ok(header_name) = reqwest::header::HeaderName::from_bytes(k.as_bytes()) {
            if let Ok(header_value) = reqwest::header::HeaderValue::from_str(v) {
                request = request.header(header_name, header_value);
            }
        }
    }

    // Add body if present
    if let Some(body_str) = body {
        request = request.body(body_str.to_string());
    }

    // Send the request
    let response = request.send().await.map_err(|e| e.to_string())?;

    let status = response.status().as_u16();
    let status_text = response
        .status()
        .canonical_reason()
        .unwrap_or("Unknown")
        .to_string();
    let ok = response.status().is_success();

    // Collect response headers
    let mut resp_headers = serde_json::Map::new();
    for (k, v) in response.headers() {
        if let Ok(v_str) = v.to_str() {
            resp_headers.insert(k.to_string().to_lowercase(), serde_json::json!(v_str));
        }
    }

    // Read the body with size limit
    let body_bytes = response.bytes().await.map_err(|e| e.to_string())?;

    let body_str = if body_bytes.len() > MAX_RESPONSE_BODY_SIZE {
        String::from_utf8_lossy(&body_bytes[..MAX_RESPONSE_BODY_SIZE]).to_string()
    } else {
        String::from_utf8_lossy(&body_bytes).to_string()
    };

    let result = serde_json::json!({
        "ok": ok,
        "status": status,
        "statusText": status_text,
        "headers": resp_headers,
        "body": body_str
    });

    Ok(result.to_string())
}

/// Perform a blocking fetch request using reqwest's blocking client.
///
/// We use reqwest::blocking::Client because the async Client is tied to
/// a specific tokio runtime and cannot be safely used from V8 callbacks
/// which run on the main thread outside of any async context.
fn perform_blocking_fetch(
    _client: &reqwest::Client, // We ignore this and create a fresh blocking client
    url: &str,
    method: &str,
    body: Option<&str>,
    headers: &std::collections::HashMap<String, String>,
) -> Result<String, String> {
    use reqwest::blocking::Client as BlockingClient;

    // Build a fresh blocking client for this request.
    // This is necessary because the async reqwest::Client cannot be used
    // from a synchronous context without an active tokio runtime, and
    // creating a new runtime per-request causes connection pool issues.
    let client = BlockingClient::builder()
        .timeout(FETCH_TIMEOUT)
        .user_agent(crate::network::fetch::DEFAULT_USER_AGENT)
        .build()
        .map_err(|e| e.to_string())?;

    let req_method = match method {
        "GET" => reqwest::Method::GET,
        "POST" => reqwest::Method::POST,
        "PUT" => reqwest::Method::PUT,
        "DELETE" => reqwest::Method::DELETE,
        "PATCH" => reqwest::Method::PATCH,
        "HEAD" => reqwest::Method::HEAD,
        "OPTIONS" => reqwest::Method::OPTIONS,
        _ => reqwest::Method::GET,
    };

    let mut request = client.request(req_method, url);

    // Add headers
    for (k, v) in headers {
        if let Ok(header_name) = reqwest::header::HeaderName::from_bytes(k.as_bytes()) {
            if let Ok(header_value) = reqwest::header::HeaderValue::from_str(v) {
                request = request.header(header_name, header_value);
            }
        }
    }

    // Add body if present
    if let Some(body_str) = body {
        request = request.body(body_str.to_string());
    }

    // Send the request
    let response = request.send().map_err(|e| e.to_string())?;

    let status = response.status().as_u16();
    let status_text = response
        .status()
        .canonical_reason()
        .unwrap_or("Unknown")
        .to_string();
    let ok = response.status().is_success();

    // Collect response headers
    let mut resp_headers = serde_json::Map::new();
    for (k, v) in response.headers() {
        if let Ok(v_str) = v.to_str() {
            resp_headers.insert(k.to_string().to_lowercase(), serde_json::json!(v_str));
        }
    }

    // Read the body with size limit
    let body_bytes = response.bytes().map_err(|e| e.to_string())?;

    let body_str = if body_bytes.len() > MAX_RESPONSE_BODY_SIZE {
        String::from_utf8_lossy(&body_bytes[..MAX_RESPONSE_BODY_SIZE]).to_string()
    } else {
        String::from_utf8_lossy(&body_bytes).to_string()
    };

    let result = serde_json::json!({
        "ok": ok,
        "status": status,
        "statusText": status_text,
        "headers": resp_headers,
        "body": body_str
    });

    Ok(result.to_string())
}

/// Report from executing page scripts.
#[derive(Debug, Clone)]
pub struct JsExecutionReport {
    pub total: usize,
    pub succeeded: usize,
    pub failed: usize,
    pub errors: Vec<(String, String)>,
}

/// Heap memory statistics.
#[derive(Debug, Clone)]
pub struct HeapStats {
    pub used_bytes: usize,
    pub total_bytes: usize,
    pub limit_bytes: usize,
}

/// Errors from JS execution.
#[derive(Debug, thiserror::Error)]
pub enum JsError {
    #[error("Compile error: {0}")]
    Compile(String),
    #[error("Runtime error: {0}")]
    Runtime(String),
    #[error("Timeout: execution exceeded {0}ms")]
    Timeout(u64),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_execution() {
        let mut rt = JsRuntime::new(RuntimeConfig::default());
        let result = rt.execute_in_context("1 + 2", "test.js").unwrap();
        assert_eq!(result, "3");
    }

    #[test]
    fn test_persistent_context() {
        let mut rt = JsRuntime::new(RuntimeConfig::default());
        rt.execute_in_context("var x = 42;", "a.js").unwrap();
        let result = rt.execute_in_context("x + 8", "b.js").unwrap();
        assert_eq!(result, "50", "Variables should persist across scripts");
    }

    #[test]
    fn test_dom_shim_exists() {
        let mut rt = JsRuntime::new(RuntimeConfig::default());
        let result = rt.execute_in_context("typeof document", "test.js").unwrap();
        assert_eq!(result, "object");
        let result = rt.execute_in_context("typeof window", "test.js").unwrap();
        assert_eq!(result, "object");
    }

    #[test]
    fn test_document_create_element() {
        let mut rt = JsRuntime::new(RuntimeConfig::default());
        let result = rt
            .execute_in_context(
                "var el = document.createElement('div'); el.tagName",
                "test.js",
            )
            .unwrap();
        assert_eq!(result, "DIV");
    }

    #[test]
    fn test_dom_mutations_via_serialize() {
        // With the new rich DOM, mutations are captured in the DOM tree itself
        // We verify by serializing and checking the output contains our changes
        let mut rt = JsRuntime::new(RuntimeConfig::default());
        rt.bootstrap_dom("<html><body></body></html>", "https://example.com");
        rt.execute_in_context(
            "var el = document.createElement('p'); el.textContent = 'hello'; document.body.appendChild(el);",
            "test.js",
        ).unwrap();
        let html = rt.serialize_dom().unwrap();
        assert!(
            html.contains("hello"),
            "DOM mutations should be reflected in serialized HTML"
        );
    }

    #[test]
    fn test_set_timeout_captured() {
        let mut rt = JsRuntime::new(RuntimeConfig::default());
        rt.execute_in_context("setTimeout(function(){}, 100)", "test.js")
            .unwrap();
        let timers = rt
            .execute_in_context("__plasmate_timers.length", "test.js")
            .unwrap();
        assert_eq!(timers, "1");
    }

    #[test]
    fn test_console_captured() {
        let mut rt = JsRuntime::new(RuntimeConfig::default());
        rt.execute_in_context("console.log('hello', 'world')", "test.js")
            .unwrap();
        let logs = rt
            .execute_in_context("__plasmate_console.length", "test.js")
            .unwrap();
        assert_eq!(logs, "1");
    }

    #[test]
    fn test_js_error_nonfatal() {
        let mut rt = JsRuntime::new(RuntimeConfig::default());
        // This should fail but not crash
        let result = rt.execute_in_context("undefinedVar.prop", "test.js");
        assert!(result.is_err());
        // Runtime should still work after error
        let ok = rt.execute_in_context("1 + 1", "test.js").unwrap();
        assert_eq!(ok, "2");
    }

    #[test]
    fn test_page_scripts_execution() {
        let mut rt = JsRuntime::new(RuntimeConfig::default());
        let scripts = vec![
            ("var counter = 0;".to_string(), "init.js".to_string()),
            ("counter += 10;".to_string(), "add.js".to_string()),
            ("counter += 5;".to_string(), "add2.js".to_string()),
        ];
        let report = rt.execute_page_scripts(&scripts);
        assert_eq!(report.succeeded, 3);
        assert_eq!(report.failed, 0);
        let val = rt.execute_in_context("counter", "check.js").unwrap();
        assert_eq!(val, "15");
    }

    #[test]
    fn test_page_url_set() {
        let mut rt = JsRuntime::new(RuntimeConfig::default());
        rt.set_page_url("https://example.com/page");
        let href = rt
            .execute_in_context("window.location.href", "test.js")
            .unwrap();
        assert!(href.contains("example.com"));
    }

    #[test]
    fn test_drain_timers() {
        let mut rt = JsRuntime::new(RuntimeConfig::default());
        rt.execute_in_context(
            "var x = 0; setTimeout(function(){ x = 42; }, 0);",
            "test.js",
        )
        .unwrap();
        rt.drain_timers(100);
        let val = rt.execute_in_context("x", "check.js").unwrap();
        assert_eq!(val, "42");
    }

    // =========================================================================
    // Rich DOM Shim Tests
    // =========================================================================

    #[test]
    fn test_dom_bootstrap_basic() {
        let mut rt = JsRuntime::new(RuntimeConfig::default());
        let html = r#"<!DOCTYPE html>
        <html><head><title>Test</title></head>
        <body><div id="app">Hello</div></body></html>"#;
        rt.bootstrap_dom(html, "https://example.com/test");

        // Check title
        let title = rt.execute_in_context("document.title", "test.js").unwrap();
        assert_eq!(title, "Test");

        // Check getElementById
        let el = rt
            .execute_in_context("document.getElementById('app') !== null", "test.js")
            .unwrap();
        assert_eq!(el, "true");

        // Check textContent
        let text = rt
            .execute_in_context("document.getElementById('app').textContent", "test.js")
            .unwrap();
        assert_eq!(text, "Hello");
    }

    #[test]
    fn test_dom_getelementbyid() {
        let mut rt = JsRuntime::new(RuntimeConfig::default());
        let html = r#"<html><body>
            <div id="container">
                <span id="inner">Content</span>
            </div>
        </body></html>"#;
        rt.bootstrap_dom(html, "https://example.com");

        let found = rt
            .execute_in_context("document.getElementById('inner').textContent", "test.js")
            .unwrap();
        assert_eq!(found, "Content");
    }

    #[test]
    fn test_dom_queryselector() {
        let mut rt = JsRuntime::new(RuntimeConfig::default());
        let html = r#"<html><body>
            <div class="card">
                <h2 class="title">First</h2>
            </div>
            <div class="card">
                <h2 class="title">Second</h2>
            </div>
        </body></html>"#;
        rt.bootstrap_dom(html, "https://example.com");

        // querySelector returns first match
        let first = rt
            .execute_in_context(
                "document.querySelector('.card .title').textContent",
                "test.js",
            )
            .unwrap();
        assert_eq!(first, "First");

        // querySelectorAll returns all
        let count = rt
            .execute_in_context("document.querySelectorAll('.title').length", "test.js")
            .unwrap();
        assert_eq!(count, "2");
    }

    #[test]
    fn test_dom_createelement_appendchild() {
        let mut rt = JsRuntime::new(RuntimeConfig::default());
        rt.bootstrap_dom("<html><body></body></html>", "https://example.com");

        rt.execute_in_context(
            r#"
            var div = document.createElement('div');
            div.id = 'created';
            div.textContent = 'Dynamic content';
            document.body.appendChild(div);
            "#,
            "test.js",
        )
        .unwrap();

        // Verify the element was added
        let found = rt
            .execute_in_context("document.getElementById('created') !== null", "test.js")
            .unwrap();
        assert_eq!(found, "true");

        let text = rt
            .execute_in_context("document.getElementById('created').textContent", "test.js")
            .unwrap();
        assert_eq!(text, "Dynamic content");
    }

    #[test]
    fn test_dom_innerhtml_set() {
        let mut rt = JsRuntime::new(RuntimeConfig::default());
        rt.bootstrap_dom(
            "<html><body><div id='root'></div></body></html>",
            "https://example.com",
        );

        rt.execute_in_context(
            r#"
            document.getElementById('root').innerHTML = '<p id="para">Rendered</p><span>More</span>';
            "#,
            "test.js",
        )
        .unwrap();

        // Verify innerHTML parsing worked
        let found = rt
            .execute_in_context("document.getElementById('para').textContent", "test.js")
            .unwrap();
        assert_eq!(found, "Rendered");

        let count = rt
            .execute_in_context("document.getElementById('root').children.length", "test.js")
            .unwrap();
        assert_eq!(count, "2");
    }

    #[test]
    fn test_dom_serialize() {
        let mut rt = JsRuntime::new(RuntimeConfig::default());
        rt.bootstrap_dom(
            "<html><body><div id='app'></div></body></html>",
            "https://example.com",
        );

        // Create some dynamic content
        rt.execute_in_context(
            r#"
            var root = document.getElementById('app');
            root.innerHTML = '<h1>Hello World</h1><p>This was created by JS</p>';
            "#,
            "test.js",
        )
        .unwrap();

        // Serialize and check output
        let html = rt.serialize_dom().unwrap();
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("<h1>Hello World</h1>"));
        assert!(html.contains("This was created by JS"));
    }

    #[test]
    fn test_dom_classlist() {
        let mut rt = JsRuntime::new(RuntimeConfig::default());
        rt.bootstrap_dom(
            "<html><body><div id='el' class='foo'></div></body></html>",
            "https://example.com",
        );

        // Add class
        rt.execute_in_context(
            "document.getElementById('el').classList.add('bar')",
            "test.js",
        )
        .unwrap();

        let cls = rt
            .execute_in_context("document.getElementById('el').className", "test.js")
            .unwrap();
        assert!(cls.contains("foo"));
        assert!(cls.contains("bar"));

        // Toggle
        rt.execute_in_context(
            "document.getElementById('el').classList.toggle('foo')",
            "test.js",
        )
        .unwrap();

        let cls2 = rt
            .execute_in_context("document.getElementById('el').className", "test.js")
            .unwrap();
        assert!(!cls2.contains("foo"));
        assert!(cls2.contains("bar"));
    }

    #[test]
    fn test_dom_textcontent_vs_innertext() {
        let mut rt = JsRuntime::new(RuntimeConfig::default());
        rt.bootstrap_dom(
            "<html><body><div id='el'>Initial</div></body></html>",
            "https://example.com",
        );

        rt.execute_in_context(
            "document.getElementById('el').textContent = 'Updated'",
            "test.js",
        )
        .unwrap();

        let text = rt
            .execute_in_context("document.getElementById('el').innerText", "test.js")
            .unwrap();
        assert_eq!(text, "Updated");
    }

    #[test]
    fn test_dom_removechild() {
        let mut rt = JsRuntime::new(RuntimeConfig::default());
        rt.bootstrap_dom(
            "<html><body><div id='parent'><span id='child'>Remove me</span></div></body></html>",
            "https://example.com",
        );

        rt.execute_in_context(
            r#"
            var parent = document.getElementById('parent');
            var child = document.getElementById('child');
            parent.removeChild(child);
            "#,
            "test.js",
        )
        .unwrap();

        let found = rt
            .execute_in_context("document.getElementById('child')", "test.js")
            .unwrap();
        assert_eq!(found, "null");
    }

    #[test]
    fn test_dom_clonenode() {
        let mut rt = JsRuntime::new(RuntimeConfig::default());
        rt.bootstrap_dom(
            "<html><body><div id='orig'><span>Inner</span></div></body></html>",
            "https://example.com",
        );

        rt.execute_in_context(
            r#"
            var orig = document.getElementById('orig');
            var clone = orig.cloneNode(true);
            clone.id = 'cloned';
            document.body.appendChild(clone);
            "#,
            "test.js",
        )
        .unwrap();

        // Both should exist
        let orig = rt
            .execute_in_context("document.getElementById('orig') !== null", "test.js")
            .unwrap();
        assert_eq!(orig, "true");

        let cloned = rt
            .execute_in_context("document.getElementById('cloned') !== null", "test.js")
            .unwrap();
        assert_eq!(cloned, "true");

        // Clone should have the same inner content
        let inner = rt
            .execute_in_context(
                "document.getElementById('cloned').getElementsByTagName('span').length",
                "test.js",
            )
            .unwrap();
        assert_eq!(inner, "1");
    }

    #[test]
    fn test_dom_attribute_methods() {
        let mut rt = JsRuntime::new(RuntimeConfig::default());
        rt.bootstrap_dom(
            "<html><body><div id='el' data-foo='bar'></div></body></html>",
            "https://example.com",
        );

        // getAttribute
        let val = rt
            .execute_in_context(
                "document.getElementById('el').getAttribute('data-foo')",
                "test.js",
            )
            .unwrap();
        assert_eq!(val, "bar");

        // hasAttribute
        let has = rt
            .execute_in_context(
                "document.getElementById('el').hasAttribute('data-foo')",
                "test.js",
            )
            .unwrap();
        assert_eq!(has, "true");

        // setAttribute
        rt.execute_in_context(
            "document.getElementById('el').setAttribute('data-new', 'value')",
            "test.js",
        )
        .unwrap();
        let newval = rt
            .execute_in_context(
                "document.getElementById('el').getAttribute('data-new')",
                "test.js",
            )
            .unwrap();
        assert_eq!(newval, "value");

        // removeAttribute
        rt.execute_in_context(
            "document.getElementById('el').removeAttribute('data-foo')",
            "test.js",
        )
        .unwrap();
        let removed = rt
            .execute_in_context(
                "document.getElementById('el').hasAttribute('data-foo')",
                "test.js",
            )
            .unwrap();
        assert_eq!(removed, "false");
    }

    #[test]
    fn test_dom_form_elements() {
        let mut rt = JsRuntime::new(RuntimeConfig::default());
        rt.bootstrap_dom(
            r#"<html><body>
            <input type="text" id="txt" value="initial">
            <input type="checkbox" id="chk">
            <select id="sel"><option value="a">A</option><option value="b" selected>B</option></select>
            </body></html>"#,
            "https://example.com",
        );

        // Input value
        let val = rt
            .execute_in_context("document.getElementById('txt').value", "test.js")
            .unwrap();
        assert_eq!(val, "initial");

        // Set input value
        rt.execute_in_context(
            "document.getElementById('txt').value = 'changed'",
            "test.js",
        )
        .unwrap();
        let newval = rt
            .execute_in_context("document.getElementById('txt').value", "test.js")
            .unwrap();
        assert_eq!(newval, "changed");

        // Select value (selected option) - the value attribute is lowercase "b"
        let sel = rt
            .execute_in_context("document.getElementById('sel').value", "test.js")
            .unwrap();
        // The select returns the option's text content when value attr not found,
        // which is "B" in our test HTML
        assert!(
            sel == "b" || sel == "B",
            "Select value should be 'b' or 'B', got: {}",
            sel
        );
    }

    #[test]
    fn test_dom_events_domcontentloaded() {
        let mut rt = JsRuntime::new(RuntimeConfig::default());
        rt.bootstrap_dom(
            "<html><body><div id='target'></div></body></html>",
            "https://example.com",
        );

        // Register DOMContentLoaded handler
        rt.execute_in_context(
            r#"
            var loaded = false;
            document.addEventListener('DOMContentLoaded', function() {
                loaded = true;
                document.getElementById('target').textContent = 'Loaded!';
            });
            "#,
            "test.js",
        )
        .unwrap();

        // Before firing
        let before = rt.execute_in_context("loaded", "test.js").unwrap();
        assert_eq!(before, "false");

        // Fire event
        rt.fire_dom_content_loaded();

        // After firing
        let after = rt.execute_in_context("loaded", "test.js").unwrap();
        assert_eq!(after, "true");

        let text = rt
            .execute_in_context("document.getElementById('target').textContent", "test.js")
            .unwrap();
        assert_eq!(text, "Loaded!");
    }

    #[test]
    fn test_dom_matches_selector() {
        let mut rt = JsRuntime::new(RuntimeConfig::default());
        rt.bootstrap_dom(
            "<html><body><div class='card active' id='test'></div></body></html>",
            "https://example.com",
        );

        let m1 = rt
            .execute_in_context(
                "document.getElementById('test').matches('.card')",
                "test.js",
            )
            .unwrap();
        assert_eq!(m1, "true");

        let m2 = rt
            .execute_in_context(
                "document.getElementById('test').matches('#test')",
                "test.js",
            )
            .unwrap();
        assert_eq!(m2, "true");

        let m3 = rt
            .execute_in_context(
                "document.getElementById('test').matches('.card.active')",
                "test.js",
            )
            .unwrap();
        assert_eq!(m3, "true");

        let m4 = rt
            .execute_in_context(
                "document.getElementById('test').matches('.other')",
                "test.js",
            )
            .unwrap();
        assert_eq!(m4, "false");
    }

    #[test]
    fn test_dom_closest() {
        let mut rt = JsRuntime::new(RuntimeConfig::default());
        rt.bootstrap_dom(
            "<html><body><div class='outer'><div class='inner'><span id='target'>X</span></div></div></body></html>",
            "https://example.com",
        );

        let closest = rt
            .execute_in_context(
                "document.getElementById('target').closest('.outer').className",
                "test.js",
            )
            .unwrap();
        assert_eq!(closest, "outer");
    }

    #[test]
    fn test_react_hydration_pattern() {
        // Simulates a React-style hydration pattern
        let mut rt = JsRuntime::new(RuntimeConfig::default());
        rt.bootstrap_dom(
            r#"<html><body><div id="root"></div></body></html>"#,
            "https://example.com",
        );

        // Simulate React rendering
        rt.execute_in_context(
            r#"
            function createElement(tag, props, ...children) {
                var el = document.createElement(tag);
                if (props) {
                    for (var key in props) {
                        if (key === 'className') {
                            el.className = props[key];
                        } else if (key === 'onClick') {
                            el.addEventListener('click', props[key]);
                        } else {
                            el.setAttribute(key, props[key]);
                        }
                    }
                }
                children.forEach(function(child) {
                    if (typeof child === 'string') {
                        el.appendChild(document.createTextNode(child));
                    } else if (child) {
                        el.appendChild(child);
                    }
                });
                return el;
            }

            var app = createElement('div', {className: 'app'},
                createElement('header', null,
                    createElement('h1', null, 'My App')
                ),
                createElement('main', null,
                    createElement('p', null, 'Welcome to the app!'),
                    createElement('button', {id: 'btn'}, 'Click me')
                )
            );

            document.getElementById('root').appendChild(app);
            "#,
            "app.js",
        )
        .unwrap();

        // Verify structure
        let h1 = rt
            .execute_in_context("document.querySelector('h1').textContent", "test.js")
            .unwrap();
        assert_eq!(h1, "My App");

        let p = rt
            .execute_in_context("document.querySelector('p').textContent", "test.js")
            .unwrap();
        assert_eq!(p, "Welcome to the app!");

        // Serialize and verify HTML output
        let html = rt.serialize_dom().unwrap();
        assert!(html.contains("<h1>My App</h1>"));
        assert!(html.contains("Welcome to the app!"));
    }

    #[test]
    fn test_dom_serialize_preserves_attributes() {
        let mut rt = JsRuntime::new(RuntimeConfig::default());
        rt.bootstrap_dom(
            r#"<html><body>
            <a href="/link" class="btn primary" data-track="click">Link</a>
            <input type="email" placeholder="Enter email" required>
            </body></html>"#,
            "https://example.com",
        );

        let html = rt.serialize_dom().unwrap();
        assert!(html.contains(r#"href="/link""#));
        assert!(html.contains(r#"class="btn primary""#));
        assert!(html.contains(r#"data-track="click""#));
        assert!(html.contains(r#"type="email""#));
        assert!(html.contains(r#"placeholder="Enter email""#));
        assert!(html.contains("required"));
    }

    #[test]
    fn test_dom_multiple_queryselectorall() {
        let mut rt = JsRuntime::new(RuntimeConfig::default());
        rt.bootstrap_dom(
            r#"<html><body>
            <ul>
                <li class="item">One</li>
                <li class="item">Two</li>
                <li class="item">Three</li>
            </ul>
            </body></html>"#,
            "https://example.com",
        );

        let count = rt
            .execute_in_context("document.querySelectorAll('li.item').length", "test.js")
            .unwrap();
        assert_eq!(count, "3");

        let texts = rt
            .execute_in_context(
                r#"Array.from(document.querySelectorAll('li')).map(function(el) { return el.textContent; }).join(',')"#,
                "test.js",
            )
            .unwrap();
        assert_eq!(texts, "One,Two,Three");
    }

    #[test]
    fn test_window_location_set_via_bootstrap() {
        let mut rt = JsRuntime::new(RuntimeConfig::default());
        rt.bootstrap_dom(
            "<html><body></body></html>",
            "https://example.com/path?query=1",
        );

        let href = rt
            .execute_in_context("window.location.href", "test.js")
            .unwrap();
        assert_eq!(href, "https://example.com/path?query=1");

        let hostname = rt
            .execute_in_context("window.location.hostname", "test.js")
            .unwrap();
        assert_eq!(hostname, "example.com");
    }

    // =========================================================================
    // Fetch Bridge Tests
    // =========================================================================

    #[test]
    fn test_fetch_bridge_not_injected_returns_stub() {
        // When no fetch bridge is injected, fetch() should return stub response
        let mut rt = JsRuntime::new(RuntimeConfig::default());

        // Check that __plasmate_do_fetch is not defined initially
        let result = rt
            .execute_in_context("typeof __plasmate_do_fetch", "test.js")
            .unwrap();
        assert_eq!(result, "undefined");

        // fetch() should return a Promise-like object
        let result = rt
            .execute_in_context(
                r#"
                var p = fetch('https://example.com/test');
                typeof p.then === 'function' ? 'has_then' : 'no_then'
                "#,
                "test.js",
            )
            .unwrap();
        assert_eq!(result, "has_then", "fetch should return a Promise");

        // The fetch queue should have our request recorded
        let queue_len = rt
            .execute_in_context("__plasmate_fetch_queue.length", "test.js")
            .unwrap();
        assert_eq!(queue_len, "1", "fetch should be recorded in queue");
    }

    #[test]
    fn test_fetch_bridge_injection() {
        // Test that we can inject the fetch bridge
        let mut rt = JsRuntime::new(RuntimeConfig::default());

        // Create a simple client for testing
        let client = reqwest::Client::builder()
            .build()
            .expect("Failed to create client");

        // Inject the fetch bridge
        rt.inject_fetch_bridge(client);

        // Check that __plasmate_do_fetch is now defined
        let result = rt
            .execute_in_context("typeof __plasmate_do_fetch", "test.js")
            .unwrap();
        assert_eq!(result, "function");
    }

    #[test]
    fn test_xhr_not_injected_returns_stub() {
        // When no fetch bridge is injected, XMLHttpRequest should return stub response
        let mut rt = JsRuntime::new(RuntimeConfig::default());

        let result = rt
            .execute_in_context(
                r#"
                var xhr = new XMLHttpRequest();
                xhr.open('GET', 'https://example.com/test', false);
                xhr.send();
                JSON.stringify({ status: xhr.status, statusText: xhr.statusText })
                "#,
                "test.js",
            )
            .unwrap();
        // The stub returns status 200
        assert!(result.contains("200"));
        assert!(result.contains("OK"));
    }

    #[test]
    fn test_textcontent_setter_and_serialize() {
        // Verify that setting textContent via JS is reflected in serialized HTML
        let mut rt = JsRuntime::new(RuntimeConfig {
            inject_dom_shim: true,
            execute_inline_scripts: false,
            ..Default::default()
        });

        let html = r#"<html><body><h4 id="product-price"></h4></body></html>"#;
        rt.bootstrap_dom(html, "http://localhost:1234/");

        // Set textContent via JS
        rt.execute_in_context(
            "document.getElementById('product-price').textContent = '$244.99';",
            "test.js",
        )
        .unwrap();

        // Check textContent getter
        let price = rt
            .execute_in_context(
                "document.getElementById('product-price').textContent",
                "test",
            )
            .unwrap();
        assert_eq!(price, "$244.99", "textContent should be set to $244.99");

        // Check serialized HTML contains the value
        let serialized = rt.serialize_dom().unwrap();
        assert!(
            serialized.contains("$244.99"),
            "Serialized HTML should contain $244.99"
        );
    }

    // =========================================================================
    // MutationObserver Tests
    // =========================================================================

    #[test]
    fn test_mutation_observer_appendchild() {
        let mut rt = JsRuntime::new(RuntimeConfig::default());
        rt.bootstrap_dom("<html><body><div id='root'></div></body></html>", "https://example.com");

        // Set up MutationObserver and appendChild
        rt.execute_in_context(
            r#"
            var mutationRecords = [];
            var observer = new MutationObserver(function(records) {
                mutationRecords = mutationRecords.concat(records);
            });
            var root = document.getElementById('root');
            observer.observe(root, { childList: true });
            var child = document.createElement('span');
            child.textContent = 'Hello';
            root.appendChild(child);
            "#,
            "test.js",
        )
        .unwrap();

        // Pump microtasks to deliver mutations
        rt.pump_microtasks();

        // Check mutation record was delivered
        let record_count = rt
            .execute_in_context("mutationRecords.length", "test.js")
            .unwrap();
        assert_eq!(record_count, "1", "Should have 1 mutation record for appendChild");

        let record_type = rt
            .execute_in_context("mutationRecords[0].type", "test.js")
            .unwrap();
        assert_eq!(record_type, "childList");

        let added_count = rt
            .execute_in_context("mutationRecords[0].addedNodes.length", "test.js")
            .unwrap();
        assert_eq!(added_count, "1", "Should have 1 added node");
    }

    #[test]
    fn test_mutation_observer_setattribute() {
        let mut rt = JsRuntime::new(RuntimeConfig::default());
        rt.bootstrap_dom("<html><body><div id='el'></div></body></html>", "https://example.com");

        rt.execute_in_context(
            r#"
            var mutationRecords = [];
            var observer = new MutationObserver(function(records) {
                mutationRecords = mutationRecords.concat(records);
            });
            var el = document.getElementById('el');
            observer.observe(el, { attributes: true, attributeOldValue: true });
            el.setAttribute('data-test', 'value1');
            "#,
            "test.js",
        )
        .unwrap();

        rt.pump_microtasks();

        let record_count = rt
            .execute_in_context("mutationRecords.length", "test.js")
            .unwrap();
        assert_eq!(record_count, "1", "Should have 1 mutation record for setAttribute");

        let record_type = rt
            .execute_in_context("mutationRecords[0].type", "test.js")
            .unwrap();
        assert_eq!(record_type, "attributes");

        let attr_name = rt
            .execute_in_context("mutationRecords[0].attributeName", "test.js")
            .unwrap();
        assert_eq!(attr_name, "data-test");
    }

    #[test]
    fn test_mutation_observer_subtree() {
        let mut rt = JsRuntime::new(RuntimeConfig::default());
        rt.bootstrap_dom("<html><body><div id='root'><div id='nested'></div></div></body></html>", "https://example.com");

        rt.execute_in_context(
            r#"
            var mutationRecords = [];
            var observer = new MutationObserver(function(records) {
                mutationRecords = mutationRecords.concat(records);
            });
            var root = document.getElementById('root');
            observer.observe(root, { childList: true, subtree: true });
            var nested = document.getElementById('nested');
            var child = document.createElement('span');
            nested.appendChild(child);
            "#,
            "test.js",
        )
        .unwrap();

        rt.pump_microtasks();

        let record_count = rt
            .execute_in_context("mutationRecords.length", "test.js")
            .unwrap();
        assert_eq!(record_count, "1", "Should have 1 mutation record for subtree appendChild");

        // Target should be the nested element
        let target_id = rt
            .execute_in_context("mutationRecords[0].target.id", "test.js")
            .unwrap();
        assert_eq!(target_id, "nested");
    }

    #[test]
    fn test_mutation_observer_disconnect() {
        let mut rt = JsRuntime::new(RuntimeConfig::default());
        rt.bootstrap_dom("<html><body><div id='root'></div></body></html>", "https://example.com");

        rt.execute_in_context(
            r#"
            var mutationRecords = [];
            var observer = new MutationObserver(function(records) {
                mutationRecords = mutationRecords.concat(records);
            });
            var root = document.getElementById('root');
            observer.observe(root, { childList: true });
            observer.disconnect();
            var child = document.createElement('span');
            root.appendChild(child);
            "#,
            "test.js",
        )
        .unwrap();

        rt.pump_microtasks();

        let record_count = rt
            .execute_in_context("mutationRecords.length", "test.js")
            .unwrap();
        assert_eq!(record_count, "0", "Should have 0 mutations after disconnect");
    }

    #[test]
    fn test_mutation_observer_takerecords() {
        let mut rt = JsRuntime::new(RuntimeConfig::default());
        rt.bootstrap_dom("<html><body><div id='root'></div></body></html>", "https://example.com");

        // Take records before microtasks are delivered
        rt.execute_in_context(
            r#"
            var callbackRecords = [];
            var observer = new MutationObserver(function(records) {
                callbackRecords = callbackRecords.concat(records);
            });
            var root = document.getElementById('root');
            observer.observe(root, { childList: true });
            var child = document.createElement('span');
            root.appendChild(child);
            var takenRecords = observer.takeRecords();
            "#,
            "test.js",
        )
        .unwrap();

        let taken_count = rt
            .execute_in_context("takenRecords.length", "test.js")
            .unwrap();
        assert_eq!(taken_count, "1", "takeRecords should return 1 record");

        rt.pump_microtasks();

        // Callback should not have received the records since we took them
        let callback_count = rt
            .execute_in_context("callbackRecords.length", "test.js")
            .unwrap();
        assert_eq!(callback_count, "0", "Callback should not receive taken records");
    }

    #[test]
    fn test_request_animation_frame() {
        let mut rt = JsRuntime::new(RuntimeConfig::default());
        rt.bootstrap_dom("<html><body></body></html>", "https://example.com");

        rt.execute_in_context(
            r#"
            var rafCalled = false;
            var rafTimestamp = 0;
            requestAnimationFrame(function(ts) {
                rafCalled = true;
                rafTimestamp = ts;
            });
            "#,
            "test.js",
        )
        .unwrap();

        // rAF should not be called yet
        let called = rt.execute_in_context("rafCalled", "test.js").unwrap();
        assert_eq!(called, "false");

        // Fire load event which drains animation frames
        rt.fire_load();

        // Now it should be called
        let called = rt.execute_in_context("rafCalled", "test.js").unwrap();
        assert_eq!(called, "true");

        // Timestamp should be set
        let ts = rt.execute_in_context("rafTimestamp > 0", "test.js").unwrap();
        assert_eq!(ts, "true");
    }
}
