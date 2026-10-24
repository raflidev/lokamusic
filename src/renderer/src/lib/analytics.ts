type PostHogClient = typeof import('posthog-js')['default']

const POSTHOG_KEY = 'phc_pjFKEMqbHZR2eqMqJUCTRhyoGZApkD3wtuBzuizshqYj'

let client: PostHogClient | null = null
let initializing: Promise<void> | null = null
let enabled = false

/**
 * Loads PostHog lazily (only when analytics is enabled) and applies the user's
 * opt-in/opt-out setting. Safe to call multiple times.
 */
export async function initAnalytics(on: boolean) {
  enabled = on

  if (!on) {
    client?.opt_out_capturing()
    return
  }

  if (!client) {
    initializing ??= import('posthog-js')
      .then(({ default: posthog }) => {
        posthog.init(POSTHOG_KEY, {
          api_host: 'https://us.i.posthog.com',
          person_profiles: 'never',
          autocapture: false,
          capture_pageview: false,
          persistence: 'localStorage',
        })
        client = posthog
      })
      .catch(() => {
        // Analytics must never break the app (offline, CSP, blocked chunk).
      })
      .finally(() => {
        initializing = null
      })
    await initializing
  }

  // The user may have disabled analytics while the chunk was loading.
  if (enabled) {
    client?.opt_in_capturing()
  } else {
    client?.opt_out_capturing()
  }
}

export function trackEvent(event: string, properties?: Record<string, unknown>) {
  if (!enabled || !client) return
  client.capture(event, properties)
}
