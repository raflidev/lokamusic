// Path helpers shared by the renderer. Paths are stored with forward slashes,
// but Windows is case-insensitive, so comparisons normalize accordingly.
const caseInsensitive = typeof navigator !== 'undefined' && /windows/i.test(navigator.userAgent)

function normalize(path: string): string {
  const normalized = path.replace(/\\/g, '/').replace(/\/+$/, '')
  return caseInsensitive ? normalized.toLowerCase() : normalized
}

/** Returns true when `child` is `parent` itself or lives inside it. */
export function isPathInside(child: string, parent: string): boolean {
  const normalizedChild = normalize(child)
  const normalizedParent = normalize(parent)
  if (!normalizedParent) return true
  return normalizedChild === normalizedParent || normalizedChild.startsWith(normalizedParent + '/')
}

/**
 * Returns the path inside `parent` (e.g. "Rock/album"), or null when `child`
 * is not inside `parent`. Keeps the original casing of `child`.
 */
export function relativePath(child: string, parent: string): string | null {
  if (!isPathInside(child, parent)) return null
  const trimmedParent = parent.replace(/\\/g, '/').replace(/\/+$/, '')
  if (!trimmedParent) return child
  if (child.length <= trimmedParent.length) return ''
  return child.slice(trimmedParent.length + 1)
}
