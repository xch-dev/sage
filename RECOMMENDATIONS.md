# Sage Wallet - Recommendations Report

**Date:** February 10, 2026
**Version:** 0.12.8
**Focus:** UI maintainability, architecture improvements, developer experience, and project health

---

## Executive Summary

Sage is a well-architected wallet with strong Rust foundations and a modern React frontend. The primary areas for improvement center on: (1) frontend test coverage (currently zero), (2) large component decomposition, (3) the 9-deep context provider nesting in App.tsx, (4) consolidating the mixed state management approaches, and (5) securing the release and dependency pipeline. Below are detailed, prioritized recommendations organized by category.

---

## 1. Frontend Architecture & Maintainability

### 1.1 Break Down Oversized Components
**Priority:** HIGH | **Effort:** Medium

Several components exceed 300-400 lines and mix multiple concerns. These are the primary maintenance bottlenecks:

| Component | Lines | Issues |
|-----------|-------|--------|
| `ConfirmationDialog.tsx` | 400+ | Transaction display + signature management + advanced summary tabs + JSON export |
| `NftCard.tsx` | 400+ | Display + 4 dialog states + form handling + event listeners |
| `CoinList.tsx` | 300+ | Column definitions + sort/filter logic + rendering + header components |
| `MultiSelectActions.tsx` | 300+ | 3 action dialogs + NFT fetching + state sync |
| `WalletConnectContext.tsx` | 24.6KB | Session management + command handling + event processing |

**Recommended decomposition:**
- `ConfirmationDialog` → `TransactionSummary`, `SignaturePanel`, `AdvancedSummaryTabs`, `SpendBundleExport`
- `NftCard` → `NftCardDisplay`, `NftTransferDialog`, `NftBurnDialog`, `NftAssignDialog`
- `CoinList` → Extract column definitions to `coinColumns.ts`, sort/filter logic to `useCoinFilters.ts`
- `WalletConnectContext` → `WalletConnectSessionManager`, `WalletConnectCommandHandler`, `WalletConnectEventProcessor`

### 1.2 Flatten Context Provider Nesting
**Priority:** HIGH | **Effort:** Medium

`App.tsx` nests 9 context providers deep:
```
LanguageProvider → ThemeProvider → SafeAreaProvider → ErrorProvider →
BiometricProvider → WalletProvider → PeerProvider → WalletConnectProvider → PriceProvider
```

This creates:
- Deep re-render cascading when any provider value changes
- Difficulty reasoning about provider dependencies
- Testing complexity (must wrap with all providers)

**Recommendations:**
1. **Compose providers** with a `composeProviders()` utility to flatten the JSX nesting
2. **Move service-like contexts to Zustand:** PeerContext (auto-polls every 5s), PriceContext (price data), and SafeAreaContext are essentially global stores — they'd work better as Zustand slices
3. **Consider react-query/TanStack Query** for PeerContext and PriceContext polling patterns — this would give caching, stale-while-revalidate, and error/loading states for free

### 1.3 Consolidate State Management
**Priority:** MEDIUM | **Effort:** Medium

State is currently split across three mechanisms:
- **Zustand stores** (`state.ts`): wallet state, offer state, navigation
- **React Context** (8 contexts): wallet, errors, biometric, price, peer, language, safe-area, WalletConnect
- **React Router location state**: for passing data between pages (e.g., `splitNftOffers` in MakeOffer)

**Recommendation:** Adopt a clearer convention:
- **Zustand** for all app-wide state (migrate PeerContext, PriceContext, SafeAreaContext)
- **Context** only for dependency injection of services that need React tree awareness (Theme, Language, Error modals, WalletConnect)
- **URL params** instead of location state for shareable/bookmarkable state
- Document the convention in a `STATE_MANAGEMENT.md` or CLAUDE.md section

### 1.4 Add Error Boundaries
**Priority:** MEDIUM | **Effort:** Low

No React error boundaries exist. A crash in any component (e.g., malformed NFT data, unexpected API response) will crash the entire app.

**Recommendation:**
- Add a top-level error boundary with a "Something went wrong" recovery UI
- Add per-route error boundaries so navigation remains functional when a page crashes
- React Router v6 supports `errorElement` on routes natively

### 1.5 Formalize the Component API Pattern
**Priority:** LOW | **Effort:** Low

The codebase uses inconsistent prop patterns:
- Some components use `PropsWithChildren<{...}>` (Header)
- Some define separate interface types (NftCard)
- Some use inline types

**Recommendation:** Standardize on exported `ComponentNameProps` interfaces for all non-trivial components, making them reusable and documentable.

---

## 2. Testing

### 2.1 Add Frontend Test Infrastructure
**Priority:** CRITICAL | **Effort:** Medium | **Status: IMPLEMENTED in [PR #739](https://github.com/xch-dev/sage/pull/739)**

**Current state:** ~~Zero frontend test files. No test framework configured.~~ PR #739 adds 170 frontend tests across 7 test files covering:

- **Test infrastructure:** Vitest + jsdom + Testing Library, with global mocks for all Tauri APIs and plugins
- **Pure function tests:** amount conversion (toMojos/fromMojos), address encoding/validation, hex utilities, URL validation, deepMerge
- **WalletConnect:** Zod schema validation for all 17 commands, handler tests for CHIP-0002, offers, and high-level commands
- **State:** Zustand store tests (wallet, offer, navigation) with mocked Tauri bindings
- **CI:** lint and frontend test steps added to the build workflow

### 2.2 Expand Rust Test Coverage
**Priority:** HIGH | **Effort:** Medium | **Status: IMPLEMENTED in [PR #739](https://github.com/xch-dev/sage/pull/739)**

~~Currently only `sage-wallet` has tests.~~ PR #739 adds 98 new Rust tests across 4 crates:
- **sage-keychain (16 tests):** encrypt/decrypt round-trips, wrong password, tampered data, keychain CRUD, serialization
- **sage-database (32 tests):** blocks, offers, collections, mempool items, type conversion utils (in-memory SQLite)
- **sage-config (23 tests):** config defaults/TOML round-trip, network inheritance, v1→v2 migration
- **sage parse (27 tests):** all parse_* functions (asset IDs, coins, hashes, signatures)

Still untested:
- `sage/endpoints` — request validation, error mapping, edge cases

### 2.3 Add E2E Testing
**Priority:** MEDIUM | **Effort:** High

For a financial application, end-to-end testing of critical flows is important:
- Wallet creation → seed backup → restore
- Send XCH → confirm → verify balance
- Offer create → share → accept
- WalletConnect pair → approve → sign

Tauri supports WebDriver testing via `tauri-driver`. Consider Playwright or Cypress with the Tauri test infrastructure.

---

## 3. Developer Experience

### 3.1 Type Safety Improvements
**Priority:** MEDIUM | **Effort:** Low

- **bindings.ts** (63KB) is auto-generated but imported throughout the codebase. Consider splitting it into domain-specific modules during generation (e.g., `bindings/wallet.ts`, `bindings/nft.ts`, `bindings/offers.ts`)
- Some `catch` blocks use implicit `any` types — add `unknown` typing and proper error narrowing
- The `Amount` type uses `string` for the amount field — consider a branded type or a wrapper class with arithmetic operations to prevent raw string manipulation errors

### 3.2 Reduce Bundle Size
**Priority:** LOW | **Effort:** Medium

The Vite config sets chunk warning at 2048KB, suggesting large bundles. Potential improvements:
- Lazy-load route components using `React.lazy()` + `Suspense` (currently all 31 pages are eagerly imported)
- Check if `framer-motion` and `@react-spring/web` are both needed (both provide animation capabilities)
- Audit `emoji-mart` import size — it's a heavy library used only for wallet emoji selection

### 3.3 Improve Development Documentation
**Priority:** MEDIUM | **Effort:** Low

The README covers basic setup but lacks:
- Architecture overview diagram
- State management conventions
- Adding new Tauri commands (Rust → Specta → bindings.ts → frontend)
- Adding new pages/routes
- Theme development guide
- Translation workflow details

---

## 4. UI/UX Improvements

### 4.1 Consistent Loading States
**Priority:** MEDIUM | **Effort:** Low

Loading states are handled inconsistently:
- Some pages use skeleton components
- Some use the `LoadingButton` from shadcn
- Some have no loading indicators at all

**Recommendation:** Create a `PageSkeleton` component and standardize loading patterns across all data-fetching pages.

### 4.2 Improve Mobile Responsiveness
**Priority:** MEDIUM | **Effort:** Medium

The app uses `md:` breakpoint for responsive layouts, but several issues suggest mobile needs more attention:
- Issue #704: Android keyboard closes during token search
- Layout component has separate `FullLayout` (desktop) and mobile sheet navigation
- Some components have hard-coded dimensions that don't adapt well

**Recommendation:** Audit all pages at mobile viewport sizes. Consider a `useIsMobile()` hook that returns true below the `md` breakpoint (currently using `platform()` which only detects OS, not viewport).

### 4.3 Improve Accessibility
**Priority:** MEDIUM | **Effort:** Medium

Good foundations exist (Radix UI with built-in a11y, ARIA labels on buttons), but gaps remain:
- No skip-to-content link
- Focus management after dialog close isn't always correct
- Color contrast may vary with custom themes (no contrast checking)
- Screen reader testing appears not to have been done
- Keyboard navigation for the NFT gallery grid

### 4.4 Theme System Robustness
**Priority:** LOW | **Effort:** Low

Issue #735 reports that app upgrades overwrite theme selection. The theme system could be more robust:
- Persist theme choice in a location that survives upgrades
- Validate theme CSS variables before applying (prevent broken themes from making app unusable)
- Add a "Reset theme" option
- Sanitize `backgroundImage` URLs in themes loaded from NFT metadata (security crossover — see Audit report)

---

## 5. Architecture & Infrastructure

### 5.1 Add Proper Logging Infrastructure
**Priority:** MEDIUM | **Effort:** Low

The Rust backend uses `tracing` with daily rotating log files, but:
- No structured logging format (just debug output)
- No log levels configurable per-module in production
- Frontend has no structured logging at all
- Debug logs can leak sensitive data (spend bundles, transaction details)

**Recommendation:** Configure `tracing-subscriber` with JSON output for production, add a frontend logging utility that pipes to Rust's tracing via Tauri events.

### 5.2 Dependency Management
**Priority:** MEDIUM | **Effort:** Low

- **Cargo.toml:** 60+ workspace dependencies are well-pinned. `cargo-machete` is in CI for unused dep detection. Good.
- **package.json:** 110+ npm packages. Consider:
  - Running `npm audit` / `pnpm audit` in CI
  - Adding Dependabot or Renovate for automated dependency updates
  - Reviewing if all 12 Radix UI packages are needed (some may be shadcn transitive deps)

### 5.3 CI/CD Pipeline Enhancements
**Priority:** LOW | **Effort:** Medium

Current CI runs: Prettier, Clippy, Cargo tests, cargo-machete, rustfmt, multi-platform builds. Missing:
- **Frontend linting** in CI (ESLint is configured but not in the CI pipeline)
- **License scanning** (important for an Apache-2.0 project with 170+ dependencies)
- **Binary size tracking** (Tauri apps can grow large)
- **SBOM generation** (Software Bill of Materials — increasingly required for security-sensitive apps)

### 5.4 Database Migration Safety
**Priority:** LOW | **Effort:** Low

5 migration files exist in `migrations/`. The migration system should:
- Have rollback migration files (currently forward-only)
- Be tested with `cargo test` (migration round-trip tests)
- Have schema snapshot tests to detect accidental changes

### 5.5 Consider Extracting a Shared Types Package
**Priority:** LOW | **Effort:** Medium

`sage-api` generates TypeScript types via Specta into `bindings.ts`. For the SDK use case (the `sage-client` crate and the planned webhooks in PR #694), consider:
- Publishing the API types as a standalone npm package
- This enables third-party tools to interact with Sage's RPC with type safety
- The OpenAPI spec generation (`cargo run --bin sage rpc generate_openapi`) is a good start — consider auto-publishing to a documentation site

---

## 6. Release Process

### 6.1 Automate the Release Checklist
**Priority:** MEDIUM | **Effort:** Medium

The README lists a 8-step manual release process:
1. Bump all crate versions
2. Bump iOS Info.plist versions
3. Run prettier
4. Extract translations
5. Create tagged release
6. Upload to TestFlight and Play Store
7. Generate OpenAPI spec
8. Upload to Docusaurus

**Recommendation:** Automate steps 1-4 and 7-8 in a `release.sh` script or CI workflow. Version bumps should be a single command (consider `cargo-release` or a custom script that bumps Cargo.toml, package.json, and Info.plist simultaneously).

### 6.2 Version Consistency
**Priority:** LOW | **Effort:** Low

Version 0.12.8 needs to be kept in sync across:
- `Cargo.toml` (workspace)
- `package.json`
- `src-tauri/tauri.conf.json`
- `src-tauri/gen/apple/sage-tauri_iOS/Info.plist`

A single `version.txt` or Cargo workspace version that other files derive from would prevent drift.

---

## 7. Strategic Recommendations

### 7.1 Address Defense-in-Depth Items Before Leaving Beta
The medium-priority security findings (see Audit report) — adding a CSP policy and sanitizing theme image URLs — should be addressed before any 1.0 release. These are low-effort hardening measures. The planned optional encryption password for the key file (infrastructure already exists in sage-keychain) would add a layer for security-conscious users.

### 7.2 Invest in Testing Before Major Refactors
Several of the recommendations above involve significant refactoring (context flattening, component decomposition, state consolidation). [PR #739](https://github.com/xch-dev/sage/pull/739) establishes the test infrastructure (170 frontend + 98 Rust tests) needed to safely pursue these refactors. E2E testing (section 2.3) remains the next testing investment.

### 7.3 Consider Feature Flags for Beta Features
Options contracts, WalletConnect, and the theme system are complex features with ongoing issues. Feature flags would allow shipping fixes to core wallet functionality without blocking on feature-specific bugs.

### 7.4 Community & Ecosystem
With 52 stars and 13 contributors, the project is growing. To support this:
- The open PRs (#720 secure-element, #728 scheme handler, #694 webhooks) have been open for months — consider a PR review cadence
- Issue #727 (deep links as WalletConnect alternative) and #729 (sync reflections) are architecture discussions that would benefit from RFCs or design docs
- The OpenAPI spec and SDK client position Sage well as a platform — lean into this with developer documentation

---

## Summary Matrix

| Category | Recommendation | Priority | Effort | Impact |
|----------|---------------|----------|--------|--------|
| Testing | ~~Add frontend test infra (Vitest)~~ | ~~CRITICAL~~ DONE | Medium | [PR #739](https://github.com/xch-dev/sage/pull/739) |
| Architecture | Break down oversized components | HIGH | Medium | Reduces maintenance burden |
| Architecture | Flatten context provider nesting | HIGH | Medium | Performance + testability |
| Testing | ~~Expand Rust test coverage~~ | ~~HIGH~~ DONE | Medium | [PR #739](https://github.com/xch-dev/sage/pull/739) |
| DX | Consolidate state management | MEDIUM | Medium | Reduces confusion |
| DX | Add error boundaries | MEDIUM | Low | Prevents full-app crashes |
| UI/UX | Consistent loading states | MEDIUM | Low | Polish |
| UI/UX | Mobile responsiveness audit | MEDIUM | Medium | Mobile user experience |
| Infra | Automate release process | MEDIUM | Medium | Reduces release risk |
| Infra | Add npm audit to CI | MEDIUM | Low | Dependency security |
| DX | Development documentation | MEDIUM | Low | Contributor onboarding |
| Architecture | Lazy-load routes | LOW | Medium | Performance |
| UI/UX | Accessibility audit | MEDIUM | Medium | Inclusivity |
| Infra | License scanning | LOW | Low | Compliance |

---

*This report is based on static analysis and code review. Recommendations should be validated against the team's priorities and roadmap.*
