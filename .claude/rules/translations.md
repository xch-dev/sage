# Translations

Translation catalogs (`src/locales/**/*.po`) are only regenerated when a release is being cut.

- Do not run `pnpm extract` or modify `src/locales/**/*.po` in feature or fix PRs. Every extraction rewrites line references and message ordering across all locales, so PRs that include it constantly conflict with each other.
- Still wrap new user-facing strings with Lingui (`t`, `<Trans>`, etc.) as usual; they will be picked up by the next extraction.
- Only regenerate or edit `.po` catalogs when explicitly asked, typically as part of release preparation.
- If a change leaves catalogs out of date, mention that the next release will need `pnpm extract` rather than committing it.
