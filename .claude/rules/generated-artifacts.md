# Generated Artifacts

- Never edit `src/bindings.ts` manually. Change its Rust/Specta source definitions instead.
- Never run Sage automatically to regenerate `src/bindings.ts`; Sage is a GUI application.
- Regenerate bindings with `pnpm generate:bindings` when needed; this command must not launch Sage.
- Release builds must never generate or modify `src/bindings.ts`.
- Translation catalogs (`src/locales/**/*.po`) follow the rules in `translations.md`.
- If a task requires generated artifacts to be refreshed, report that follow-up rather than hand-editing them.
