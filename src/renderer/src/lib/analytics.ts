import posthog from 'posthog-js'

export function initAnalytics() {
  posthog.init('phc_pjFKEMqbHZR2eqMqJUCTRhyoGZApkD3wtuBzuizshqYj', {
    api_host: 'https://us.i.posthog.com',
    person_profiles: 'never',
    autocapture: false,
    capture_pageview: false,
    persistence: 'localStorage',
  })
}

export function trackEvent(event: string, properties?: Record<string, unknown>) {
  posthog.capture(event, properties)
}
