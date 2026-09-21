// Fill missing JSON object fields recursively. Arrays are user choices, not
// positional defaults; false, zero, null and an empty array remain explicit values.
export const fillMissingDefaults = <T>(defaults: T, saved: unknown): T => {
  if (saved === undefined) return JSON.parse(JSON.stringify(defaults)) as T
  if (isObject(defaults) && isObject(saved)) {
    return Object.fromEntries(Object.entries(defaults).map(([key, value]) => [
      key, fillMissingDefaults(value, Object.hasOwn(saved, key) ? saved[key] : undefined),
    ])) as T
  }
  return JSON.parse(JSON.stringify(saved)) as T
}

const isObject = (value: unknown): value is Record<string, unknown> =>
  value !== null && typeof value === 'object' && !Array.isArray(value)
