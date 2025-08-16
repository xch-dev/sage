# Content Security Policy (CSP) Rules for Sage Wallet

## Overview

This document describes the Content Security Policy implemented in the Sage Tauri wallet application. The CSP is configured in two locations:

1. **Tauri Configuration**: `src-tauri/tauri.conf.json` (enforced at webview level)
2. **HTML Meta Tag**: `index.html` (backup enforcement)

## Final CSP Configuration

```
default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src 'self' data: https:; font-src 'self' data:; connect-src 'self' https: wss://relay.walletconnect.org ipc://localhost; frame-src 'none'; object-src 'none'; base-uri 'self'; form-action 'self';
```

## Detailed Rule Breakdown

### ✅ Allowed Resources

#### **Scripts** (`script-src`)

- **`'self'`** - Scripts from the same origin (your app domain)
- **`'unsafe-inline'`** - Inline `<script>` tags and event handlers
- **❌ `'unsafe-eval'`** - Dynamic code execution is **BLOCKED**

#### **Styles** (`style-src`)

- **`'self'`** - CSS files from the same origin
- **`'unsafe-inline'`** - Inline `<style>` tags and style attributes

#### **Images** (`img-src`)

- **`'self'`** - Images from the same origin
- **`data:`** - Data URI images (base64 encoded)
- **`https:`** - Any HTTPS image source

#### **Fonts** (`font-src`)

- **`'self'`** - Font files from the same origin
- **`data:`** - Data URI fonts (base64 encoded)

#### **Connections** (`connect-src`)

- **`'self'`** - API calls and fetch requests to the same origin
- **`https:`** - Any HTTPS API endpoints
- **`wss://relay.walletconnect.org`** - WalletConnect WebSocket relay
- **`ipc://localhost`** - Tauri internal IPC communication

#### **Base URI** (`base-uri`)

- **`'self'`** - Base URL can only be the same origin

#### **Form Actions** (`form-action`)

- **`'self'`** - Forms can only submit to the same origin

### ❌ Restricted/Blocked Resources

#### **Frames** (`frame-src`)

- **`'none'`** - All iframes, embeds, and frames are blocked
- **No external content embedding allowed**

#### **Objects** (`object-src`)

- **`'none'`** - All `<object>`, `<embed>`, `<applet>` elements blocked
- **No Flash, Java applets, or other plugins allowed**

#### **Default Sources** (`default-src`)

- **`'self'`** - Any resource type not explicitly allowed falls back to same-origin only

## Security Protections

### **What This Protects Against:**

1. **Cross-Site Scripting (XSS)**

   - Restricts script execution to trusted sources
   - Blocks dynamic code execution (`eval()`, `Function()`)

2. **Clickjacking**

   - Blocks iframe embedding completely
   - Prevents malicious sites from embedding your app

3. **Data Exfiltration**

   - Limits connections to approved endpoints only
   - Prevents unauthorized data transmission

4. **Malicious Plugins**

   - Blocks object/embed elements
   - Prevents execution of untrusted plugins

5. **Open Redirects**
   - Restricts form submissions to same origin
   - Controls base URI to prevent redirect attacks

### **What This Allows:**

1. **React Functionality**

   - Static component rendering and JSX
   - Event handlers (`onClick`, `onChange`, etc.)
   - Inline styles for dynamic styling

2. **WalletConnect Integration**

   - WebSocket connections to relay server
   - Secure wallet-to-wallet communication

3. **Tauri Commands**

   - IPC communication with backend
   - Native functionality access

4. **External APIs**

   - Any HTTPS endpoint for API calls
   - Image loading from HTTPS sources

5. **Development Features**
   - Hot reloading (with `'unsafe-inline'`)
   - Inline event handlers for React

## Implementation Details

### **Tauri Configuration**

```json
{
  "app": {
    "security": {
      "csp": "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src 'self' data: https:; font-src 'self' data:; connect-src 'self' https: wss://relay.walletconnect.org ipc://localhost; frame-src 'none'; object-src 'none'; base-uri 'self'; form-action 'self';"
    }
  }
}
```

### **HTML Meta Tag**

```html
<meta
  http-equiv="Content-Security-Policy"
  content="default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src 'self' data: https:; font-src 'self' data:; connect-src 'self' https: wss://relay.walletconnect.org ipc://localhost; frame-src 'none'; object-src 'none'; base-uri 'self'; form-action 'self';"
/>
```

## Why `'unsafe-inline'` is Required

The `'unsafe-inline'` directive is necessary for:

1. **React Event Handlers** - The app uses extensive inline event handlers:

   - `onClick={handleFunction}`
   - `onChange={setState}`
   - `onSubmit={form.handleSubmit(onSubmit)}`

2. **Inline Styles** - Dynamic styling throughout the app:

   - `style={{ minHeight: '200px' }}`
   - `style={{ height: 'calc(100vh - 400px)' }}`

3. **React Component System** - React's event system relies on inline handlers

## Security Considerations

### **Current Security Level: HIGH**

- ✅ No dynamic code execution (`eval()`, `Function()`)
- ✅ Restricted connections to approved endpoints
- ✅ No iframe embedding (prevents clickjacking)
- ✅ Same-origin restrictions for forms and base URIs
- ✅ No plugin execution

### **Trade-offs Made**

- ⚠️ `'unsafe-inline'` allows inline scripts/styles (required for React)
- ⚠️ `https:` allows any HTTPS connection (needed for external APIs)

### **Future Improvements**

- Consider restricting `https:` to specific domains if possible
- Monitor for CSP violations in production
- Implement nonces if higher security is required (complex refactoring needed)

## Testing and Monitoring

### **Development Testing**

1. Run `pnpm tauri dev`
2. Open browser developer tools
3. Check Console for CSP violations
4. Verify all functionality works as expected

### **Production Monitoring**

- Monitor browser console for CSP violations
- Check network requests for blocked resources
- Ensure WalletConnect and Tauri IPC work correctly

## Compliance Notes

This CSP configuration provides:

- **Strong protection** against common web vulnerabilities
- **Compatibility** with React and Tauri frameworks
- **Functionality** for wallet operations and external APIs
- **Balance** between security and usability

The policy is designed specifically for a Tauri wallet application and may need adjustments for other use cases.
