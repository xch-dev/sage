if ('__TAURI__' in window) {
  var __TAURI_PLUGIN_BARCODE_SCANNER__ = (function (n) {
    'use strict';
    function e(n, e, t, s) {
      if ('a' === t && !s)
        throw new TypeError('Private accessor was defined without a getter');
      if ('function' == typeof e ? n !== e || !s : !e.has(n))
        throw new TypeError(
          'Cannot read private member from an object whose class did not declare it',
        );
      return 'm' === t ? s : 'a' === t ? s.call(n) : s ? s.value : e.get(n);
    }
    function t(n, e, t, s, r) {
      if ('function' == typeof e || !e.has(n))
        throw new TypeError(
          'Cannot write private member to an object whose class did not declare it',
        );
      return e.set(n, t), t;
    }
    var s, r, a;
    'function' == typeof SuppressedError && SuppressedError;
    const i = '__TAURI_TO_IPC_KEY__';
    class c {
      constructor() {
        (this.__TAURI_CHANNEL_MARKER__ = !0),
          s.set(this, () => {}),
          r.set(this, 0),
          a.set(this, []),
          (this.id = (function (n, e = !1) {
            return window.__TAURI_INTERNALS__.transformCallback(n, e);
          })(({ message: n, id: i }) => {
            if (i == e(this, r, 'f'))
              for (
                e(this, s, 'f').call(this, n), t(this, r, e(this, r, 'f') + 1);
                e(this, r, 'f') in e(this, a, 'f');

              ) {
                const n = e(this, a, 'f')[e(this, r, 'f')];
                e(this, s, 'f').call(this, n),
                  delete e(this, a, 'f')[e(this, r, 'f')],
                  t(this, r, e(this, r, 'f') + 1);
              }
            else e(this, a, 'f')[i] = n;
          }));
      }
      set onmessage(n) {
        t(this, s, n);
      }
      get onmessage() {
        return e(this, s, 'f');
      }
      [((s = new WeakMap()), (r = new WeakMap()), (a = new WeakMap()), i)]() {
        return `__CHANNEL__:${this.id}`;
      }
      toJSON() {
        return this[i]();
      }
    }
    class o {
      constructor(n, e, t) {
        (this.plugin = n), (this.event = e), (this.channelId = t);
      }
      async unregister() {
        return _(`plugin:${this.plugin}|remove_listener`, {
          event: this.event,
          channelId: this.channelId,
        });
      }
    }
    async function _(n, e = {}, t) {
      return window.__TAURI_INTERNALS__.invoke(n, e, t);
    }
    var u;
    return (
      (n.Format = void 0),
      ((u = n.Format || (n.Format = {})).QRCode = 'QR_CODE'),
      (u.UPC_A = 'UPC_A'),
      (u.UPC_E = 'UPC_E'),
      (u.EAN8 = 'EAN_8'),
      (u.EAN13 = 'EAN_13'),
      (u.Code39 = 'CODE_39'),
      (u.Code93 = 'CODE_93'),
      (u.Code128 = 'CODE_128'),
      (u.Codabar = 'CODABAR'),
      (u.ITF = 'ITF'),
      (u.Aztec = 'AZTEC'),
      (u.DataMatrix = 'DATA_MATRIX'),
      (u.PDF417 = 'PDF_417'),
      (n.cancel = async function () {
        await _('plugin:barcode-scanner|cancel');
      }),
      (n.checkPermissions = async function () {
        return await (async function (n) {
          return _(`plugin:${n}|check_permissions`);
        })('barcode-scanner').then((n) => n.camera);
      }),
      (n.openAppSettings = async function () {
        await _('plugin:barcode-scanner|open_app_settings');
      }),
      (n.requestPermissions = async function () {
        return await (async function (n) {
          return _(`plugin:${n}|request_permissions`);
        })('barcode-scanner').then((n) => n.camera);
      }),
      (n.scan = async function (n) {
        return await _('plugin:barcode-scanner|scan', { ...n });
      }),
      (n.startScan = async function (n, e) {
        return (
          await _('plugin:barcode-scanner|start_scan', { ...n }),
          console.log('Start scanning'),
          await (async function (n, e, t) {
            const s = new c();
            return (
              (s.onmessage = t),
              _(`plugin:${n}|register_listener`, { event: e, handler: s }).then(
                () => new o(n, e, s.id),
              )
            );
          })('barcode-scanner', 'barcode-detected', e)
        );
      }),
      (n.stopScan = async function () {
        await _('plugin:barcode-scanner|stop_scan');
      }),
      n
    );
  })({});
  Object.defineProperty(window.__TAURI__, 'barcodeScanner', {
    value: __TAURI_PLUGIN_BARCODE_SCANNER__,
  });
}
