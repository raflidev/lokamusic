<script lang="ts">
  import Icon from '../components/Icon.svelte'

  const GITHUB_USER = 'raflidev'
  const REPO_URL = 'https://github.com/raflidev/lokamusic'

  function openLink(url: string) {
    window.open(url, '_blank')
  }
</script>

<div class="about-view">
  <div class="topbar">
    <h1 class="page-title">About</h1>
  </div>

  <div class="content">
    <div class="card profile-card">
      <div class="avatar">
        <Icon name="user" size={36} />
      </div>
      <div class="profile-info">
        <h2 class="username">@{GITHUB_USER}</h2>
        <p class="tagline">Developer of lokamusic</p>
      </div>
    </div>

    <div class="card links-card">
      <p class="label-sm">Links</p>
      <button class="link-row" onclick={() => openLink(`https://github.com/${GITHUB_USER}`)}>
        <span class="link-icon"><Icon name="github" size={16} /></span>
        <span class="link-text">github.com/{GITHUB_USER}</span>
        <span class="link-arrow"><Icon name="external-link" size={14} /></span>
      </button>
      <div class="divider"></div>
      <button class="link-row" onclick={() => openLink(REPO_URL)}>
        <span class="link-icon"><Icon name="folder" size={16} /></span>
        <span class="link-text">raflidev/lokamusic</span>
        <span class="link-badge">Repository</span>
        <span class="link-arrow"><Icon name="external-link" size={14} /></span>
      </button>
    </div>

    <div class="card app-card">
      <p class="label-sm">App</p>
      <div class="app-row">
        <span class="app-name">lokamusic</span>
        <span class="app-stack">Electron · Svelte 5 · TypeScript</span>
      </div>
    </div>
  </div>
</div>

<style>
  .about-view {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .topbar {
    padding: 16px 32px 8px;
    flex-shrink: 0;
  }

  .page-title {
    font-size: 22px;
    font-weight: 600;
    letter-spacing: -0.02em;
    color: var(--secondary);
  }

  .content {
    flex: 1;
    overflow-y: auto;
    padding: 16px 32px 32px;
    display: flex;
    flex-direction: column;
    gap: 12px;
    max-width: 520px;
  }

  .card {
    background: var(--surface-container-low);
    border: 1px solid var(--outline-variant);
    border-radius: var(--radius-lg);
    padding: 20px 24px;
  }

  /* Profile */
  .profile-card {
    display: flex;
    align-items: center;
    gap: 20px;
  }

  .avatar {
    width: 64px;
    height: 64px;
    border-radius: 50%;
    background: var(--primary-container);
    color: var(--on-primary-container);
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .username {
    font-size: 20px;
    font-weight: 600;
    letter-spacing: -0.02em;
    color: var(--on-surface);
    margin-bottom: 4px;
  }

  .tagline {
    font-size: 13px;
    color: var(--on-surface-variant);
  }

  /* Links */
  .links-card {
    display: flex;
    flex-direction: column;
    gap: 0;
    padding: 16px 0;
  }

  .links-card .label-sm {
    padding: 0 24px 12px;
  }

  .link-row {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 11px 24px;
    width: 100%;
    text-align: left;
    transition: background 0.12s;
    color: var(--on-surface);
  }

  .link-row:hover { background: var(--surface-container); }

  .link-icon { color: var(--on-surface-variant); display: flex; }
  .link-arrow { color: var(--outline); display: flex; margin-left: auto; }

  .link-text {
    font-size: 14px;
    color: var(--on-surface);
    flex: 1;
  }

  .link-badge {
    font-size: 10px;
    font-weight: 600;
    letter-spacing: 0.06em;
    background: var(--primary-container);
    color: var(--on-primary-container);
    padding: 2px 8px;
    border-radius: var(--radius-lg);
  }

  .divider {
    height: 1px;
    background: var(--outline-variant);
    margin: 0 24px;
  }

  /* App info */
  .app-card .label-sm { margin-bottom: 12px; }

  .app-row {
    display: flex;
    align-items: baseline;
    gap: 12px;
  }

  .app-name {
    font-size: 16px;
    font-weight: 600;
    color: var(--on-surface);
  }

  .app-stack {
    font-size: 12px;
    color: var(--outline);
  }
</style>
