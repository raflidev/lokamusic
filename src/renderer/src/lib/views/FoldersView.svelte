<script lang="ts">
  import Icon from '../components/Icon.svelte'
  import FolderArt from '../components/FolderArt.svelte'
  import { library } from '../stores/library.svelte'
  import { ui } from '../stores/ui.svelte'
  import type { WatchedFolder } from '../../../../../preload/index.d'

  let folderToRemove = $state<WatchedFolder | null>(null)

  function formatBytes(b: number): string {
    if (!b) return '0 B'
    if (b < 1024) return `${b} B`
    if (b < 1024 * 1024) return `${(b / 1024).toFixed(0)} KB`
    if (b < 1024 * 1024 * 1024) return `${(b / (1024 * 1024)).toFixed(0)} MB`
    return `${(b / (1024 * 1024 * 1024)).toFixed(1)} GB`
  }

  function formatDate(ts?: number): string {
    if (!ts) return 'Never'
    const d = new Date(ts)
    return d.toLocaleDateString('en-US', { month: 'short', day: 'numeric', year: 'numeric' })
  }

  async function scanNewFolder() {
    const path = await window.electronAPI.invoke('dialog:select-folder') as string | null
    if (!path) return
    const folder = await window.electronAPI.invoke('library:add-folder', path) as WatchedFolder | null
    if (folder) library.addFolder(folder)
  }

  async function rescanFolder(id: string) {
    await window.electronAPI.invoke('library:scan-folder', id)
  }

  async function confirmRemove() {
    if (!folderToRemove) return
    const id = folderToRemove.id
    folderToRemove = null
    library.removeFolder(id)
    await window.electronAPI.invoke('library:remove-folder', id)
  }
</script>

<div class="folders-view">
  <div class="topbar">
    <h1 class="page-title">File Manager</h1>
    <div class="topbar-right">
      <div class="search-wrap">
        <Icon name="search" size={14} />
        <input type="text" placeholder="Search folders…" />
      </div>
      <button class="icon-btn" title="Settings"><Icon name="settings" size={18} /></button>
      <button class="icon-btn" title="Profile"><Icon name="user" size={18} /></button>
    </div>
  </div>

  <div class="content">
    <!-- Hero -->
    <div class="hero">
      <div class="hero-text">
        <h2>Optimize Your Library</h2>
        <p>lokamusic scans your local directories to build a metadata-rich music library automatically.</p>
      </div>
      <div class="hero-actions">
        <button class="scan-btn" onclick={scanNewFolder}>
          Scan New Folder <Icon name="arrow-right" size={16} />
        </button>
        <div class="secondary-actions">
          <button class="secondary-btn disabled" disabled>
            <Icon name="cloud" size={20} />
            <span>Sync Cloud</span>
          </button>
        </div>
      </div>
    </div>

    <!-- Scan progress -->
    {#if ui.scanProgress}
      <div class="scan-card">
        <p class="label-sm">Current Operation</p>
        <div class="scan-row">
          <p class="scan-path">Scanning {library.folders.find(f => f.id === ui.scanProgress?.folderId)?.path ?? '…'}</p>
          <span class="scan-pct">{ui.scanProgress.percent}%</span>
        </div>
        <div class="progress-track">
          <div class="progress-fill" style="width: {ui.scanProgress.percent}%"></div>
        </div>
        <div class="scan-meta">
          <span>{ui.scanProgress.scanned} files indexed</span>
          <span class="dot">•</span>
          <span>Scanning…</span>
        </div>
      </div>
    {/if}

    <!-- Watched folders -->
    <div class="folders-section">
      <div class="section-header">
        <h3>Watched Folders</h3>
        <button class="link-btn">Manage All Permissions</button>
      </div>

      <div class="folders-grid">
        {#each library.folders as folder}
          <div class="folder-card" onclick={() => ui.navigateToFolder(folder.id)} role="button" tabindex="0" onkeydown={(e) => e.key === 'Enter' && ui.navigateToFolder(folder.id)}>
            <div class="folder-art-wrap">
              <FolderArt folderId={folder.id} size={130} />
              <span class="folder-badge">LOCAL DRIVE</span>
            </div>
            <div class="folder-info">
              <p class="folder-name">{folder.name}</p>
              <p class="folder-meta">{formatBytes(folder.sizeBytes)} · {folder.songCount} Tracks</p>
            </div>
            <div class="folder-actions">
              <button class="icon-action" onclick={(e) => { e.stopPropagation(); rescanFolder(folder.id) }} title="Rescan">
                <Icon name="refresh" size={14} />
              </button>
              <button class="icon-action danger" onclick={(e) => { e.stopPropagation(); folderToRemove = folder }} title="Remove">
                <Icon name="trash" size={14} />
              </button>
            </div>
          </div>
        {/each}

        <!-- Add new -->
        <button class="folder-card add-card" onclick={scanNewFolder}>
          <div class="add-icon"><Icon name="plus" size={28} /></div>
          <p class="folder-name">Connect New Folder</p>
          <p class="folder-meta">Scan a local directory</p>
        </button>
      </div>
    </div>

    <!-- Recent activity -->
    {#if library.scanHistory.length > 0}
      <div class="activity-section">
        <div class="section-header">
          <h3>Recent Import Activity</h3>
          <div class="activity-badges">
            <span class="activity-badge success">Success: {library.scanHistory.reduce((a, h) => a + h.success, 0)}</span>
            {#if library.scanHistory.some(h => h.errors > 0)}
              <span class="activity-badge error">Errors: {library.scanHistory.reduce((a, h) => a + h.errors, 0)}</span>
            {/if}
          </div>
        </div>

        <div class="activity-list">
          {#each library.scanHistory.slice(0, 10) as event}
            <div class="activity-row">
              <Icon name="check" size={14} class="check-icon" />
              <span class="activity-path">{event.folderPath}</span>
              <span class="activity-count">{event.success} tracks</span>
              <span class="activity-date">{formatDate(event.timestamp)}</span>
            </div>
          {/each}
        </div>
      </div>
    {/if}
  </div>
</div>

{#if folderToRemove}
  <div class="modal-backdrop" onclick={() => folderToRemove = null} role="presentation">
    <div class="modal" onclick={(e) => e.stopPropagation()} role="dialog" aria-modal="true">
      <div class="modal-icon"><Icon name="trash" size={22} /></div>
      <h3 class="modal-title">Remove Folder</h3>
      <p class="modal-body">
        <strong>{folderToRemove.name}</strong> will be removed from your library along with all its tracks.
        Files on disk will not be deleted.
      </p>
      <div class="modal-actions">
        <button class="modal-btn cancel" onclick={() => folderToRemove = null}>Cancel</button>
        <button class="modal-btn confirm" onclick={confirmRemove}>Remove</button>
      </div>
    </div>
  </div>
{/if}

<style>
  .folders-view {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .topbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px 32px 8px;
    flex-shrink: 0;
  }

  .page-title {
    font-size: 22px;
    font-weight: 600;
    letter-spacing: -0.02em;
    color: var(--secondary);
  }

  .topbar-right {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .search-wrap {
    display: flex;
    align-items: center;
    gap: 8px;
    background: var(--surface-container);
    border: 1px solid var(--outline-variant);
    border-radius: var(--radius-xl);
    padding: 7px 14px;
    color: var(--on-surface-variant);
  }

  .search-wrap input {
    background: none;
    border: none;
    outline: none;
    color: var(--on-surface);
    font-size: 13px;
    width: 180px;
  }

  .search-wrap input::placeholder { color: var(--outline); }

  .icon-btn {
    color: var(--on-surface-variant);
    padding: 8px;
    border-radius: var(--radius-lg);
    display: flex;
    transition: color 0.12s, background 0.12s;
  }

  .icon-btn:hover { color: var(--on-surface); background: var(--surface-container); }

  .content {
    flex: 1;
    overflow-y: auto;
    padding: 16px 32px 32px;
    display: flex;
    flex-direction: column;
    gap: 24px;
  }

  /* Hero */
  .hero {
    display: grid;
    grid-template-columns: 1fr auto;
    gap: 24px;
    align-items: start;
    padding: 32px;
    background: var(--surface-container-low);
    border: 1px solid var(--outline-variant);
    border-radius: var(--radius-lg);
  }

  .hero h2 {
    font-size: 30px;
    font-weight: 300;
    letter-spacing: -0.03em;
    color: var(--on-surface);
    margin-bottom: 8px;
  }

  .hero p {
    font-size: 14px;
    color: var(--on-surface-variant);
    max-width: 500px;
    line-height: 1.6;
  }

  .hero-actions {
    display: flex;
    flex-direction: column;
    gap: 8px;
    align-items: stretch;
  }

  .scan-btn {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    padding: 16px 24px;
    background: var(--secondary);
    color: var(--on-secondary);
    border-radius: var(--radius-lg);
    font-size: 15px;
    font-weight: 500;
    transition: background 0.12s;
    white-space: nowrap;
  }

  .scan-btn:hover { background: var(--tertiary); }

  .secondary-actions {
    display: flex;
    gap: 8px;
  }

  .secondary-btn {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 6px;
    padding: 14px 12px;
    background: var(--surface-container);
    border: 1px solid var(--outline-variant);
    border-radius: var(--radius-lg);
    font-size: 12px;
    color: var(--on-surface-variant);
    transition: border-color 0.12s, color 0.12s;
  }

  .secondary-btn:hover:not(.disabled) { border-color: var(--outline); color: var(--on-surface); }
  .secondary-btn.disabled { opacity: 0.4; cursor: not-allowed; }

  /* Scan card */
  .scan-card {
    padding: 24px;
    background: var(--surface-container-low);
    border: 1px solid var(--secondary-container);
    border-radius: var(--radius-lg);
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .scan-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .scan-path {
    font-size: 16px;
    font-weight: 500;
    color: var(--on-surface);
  }

  .scan-pct {
    font-size: 18px;
    font-weight: 600;
    color: var(--secondary);
  }

  .progress-track {
    height: 4px;
    background: var(--surface-container-high);
    border-radius: 2px;
    overflow: hidden;
  }

  .progress-fill {
    height: 100%;
    background: var(--secondary);
    border-radius: 2px;
    transition: width 0.3s;
  }

  .scan-meta {
    display: flex;
    gap: 8px;
    font-size: 12px;
    color: var(--on-surface-variant);
  }

  .dot { color: var(--outline); }

  /* Folders grid */
  .section-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 16px;
  }

  .section-header h3 {
    font-size: 16px;
    font-weight: 600;
    letter-spacing: -0.01em;
    color: var(--on-surface);
  }

  .link-btn {
    font-size: 13px;
    color: var(--secondary);
    transition: opacity 0.12s;
  }

  .link-btn:hover { opacity: 0.8; }

  .folders-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
    gap: 12px;
  }

  .folder-card {
    background: var(--surface-container-low);
    border: 1px solid var(--outline-variant);
    border-radius: var(--radius-lg);
    overflow: hidden;
    transition: border-color 0.12s;
    text-align: left;
    cursor: pointer;
  }

  .folder-card:hover { border-color: var(--outline); }

  .folder-art-wrap {
    height: 130px;
    background: var(--surface-container);
    position: relative;
    overflow: hidden;
  }

  .folder-art-wrap :global(.folder-art) {
    width: 100% !important;
    height: 100% !important;
    border-radius: 0;
  }

  .folder-badge {
    position: absolute;
    bottom: 8px;
    left: 10px;
    font-size: 9px;
    font-weight: 700;
    letter-spacing: 0.1em;
    background: rgba(0,0,0,0.6);
    padding: 3px 7px;
    border-radius: 2px;
    color: var(--on-surface-variant);
  }

  .folder-info {
    padding: 12px 14px 6px;
  }

  .folder-name {
    font-size: 14px;
    font-weight: 600;
    color: var(--on-surface);
    margin-bottom: 3px;
  }

  .folder-meta {
    font-size: 12px;
    color: var(--on-surface-variant);
  }

  .folder-actions {
    display: flex;
    gap: 4px;
    padding: 6px 10px 10px;
  }

  .icon-action {
    color: var(--on-surface-variant);
    padding: 5px;
    border-radius: var(--radius);
    display: flex;
    transition: color 0.12s;
  }

  .icon-action:hover { color: var(--on-surface); }
  .icon-action.danger:hover { color: var(--error); }

  .add-card {
    cursor: pointer;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 10px;
    min-height: 180px;
    background: var(--surface-container-lowest);
    border: 1px dashed var(--outline-variant);
  }

  .add-card:hover { border-color: var(--outline); }

  .add-icon {
    width: 52px;
    height: 52px;
    border-radius: var(--radius-lg);
    background: var(--surface-container);
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--on-surface-variant);
  }

  .add-card .folder-meta { text-align: center; font-size: 12px; color: var(--outline); }

  /* Activity */
  .activity-list {
    display: flex;
    flex-direction: column;
    gap: 0;
    background: var(--surface-container-low);
    border: 1px solid var(--outline-variant);
    border-radius: var(--radius-lg);
    overflow: hidden;
  }

  .activity-row {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 10px 16px;
    border-bottom: 1px solid var(--outline-variant);
    font-size: 13px;
  }

  .activity-row:last-child { border-bottom: none; }

  :global(.check-icon) { color: var(--primary); flex-shrink: 0; }

  .activity-path {
    flex: 1;
    color: var(--on-surface);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .activity-count { color: var(--primary); font-size: 12px; white-space: nowrap; }
  .activity-date { color: var(--outline); font-size: 12px; white-space: nowrap; }

  .activity-badges { display: flex; gap: 8px; }

  .activity-badge {
    font-size: 11px;
    font-weight: 600;
    padding: 3px 10px;
    border-radius: var(--radius-lg);
  }

  .activity-badge.success {
    background: var(--primary-container);
    color: var(--on-primary-container);
  }

  .activity-badge.error {
    background: var(--error-container);
    color: var(--error);
  }

  /* Modal */
  .modal-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.6);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
    backdrop-filter: blur(2px);
  }

  .modal {
    background: var(--surface-container-low);
    border: 1px solid var(--outline-variant);
    border-radius: var(--radius-lg);
    padding: 28px 32px;
    width: 360px;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
    text-align: center;
  }

  .modal-icon {
    width: 48px;
    height: 48px;
    border-radius: 50%;
    background: var(--error-container);
    color: var(--error);
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .modal-title {
    font-size: 17px;
    font-weight: 600;
    color: var(--on-surface);
    letter-spacing: -0.01em;
  }

  .modal-body {
    font-size: 13px;
    color: var(--on-surface-variant);
    line-height: 1.6;
  }

  .modal-body strong {
    color: var(--on-surface);
    font-weight: 500;
  }

  .modal-actions {
    display: flex;
    gap: 8px;
    margin-top: 4px;
    width: 100%;
  }

  .modal-btn {
    flex: 1;
    padding: 10px;
    border-radius: var(--radius);
    font-size: 14px;
    font-weight: 500;
    transition: background 0.12s, color 0.12s;
  }

  .modal-btn.cancel {
    background: var(--surface-container);
    color: var(--on-surface-variant);
    border: 1px solid var(--outline-variant);
  }

  .modal-btn.cancel:hover { background: var(--surface-container-high); color: var(--on-surface); }

  .modal-btn.confirm {
    background: var(--error-container);
    color: var(--error);
  }

  .modal-btn.confirm:hover { opacity: 0.88; }
</style>
