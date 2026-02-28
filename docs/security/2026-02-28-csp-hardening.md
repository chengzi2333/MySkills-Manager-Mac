# CSP Hardening Note (2026-02-28)

## Applied

- `script-src` removed `'unsafe-inline'`.
- Current policy keeps: `script-src 'self' 'wasm-unsafe-eval'`.
- `style-src` removed `'unsafe-inline'`.
- Current policy keeps: `style-src 'self'`.

## Validation Checklist

1. `npm run build` succeeds.
2. `npx tsx --test test/cspPolicy.test.ts` passes.
3. Smoke-check desktop pages after launch:
   - Dashboard chart cards render.
   - Logs table and pagination render.
   - Tools page badges and controls render.
