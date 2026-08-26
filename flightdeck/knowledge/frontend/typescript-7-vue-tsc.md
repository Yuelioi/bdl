# ⚠ TypeScript 7 breaks vue-tsc 3.3.7

## Symptom

`vue-tsc --noEmit` exits before checking the project:

```text
ERR_PACKAGE_PATH_NOT_EXPORTED: Package subpath './lib/tsc' is not defined by "exports" in typescript/package.json
```

## Root cause

The dependency upgrade installed TypeScript 7.0.2 alongside vue-tsc 3.3.7. That vue-tsc release resolves `typescript/lib/tsc`, while TypeScript 7 no longer exports that private subpath.

## Current fix

Pin `apps/desktop` to `typescript: 5.9.3`, regenerate the pnpm lockfile, and run the build again. Do not suppress the error or skip `vue-tsc`; upgrade both packages together once compatibility is documented.
