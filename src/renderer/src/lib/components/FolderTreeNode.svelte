<script lang="ts">
  import Icon from './Icon.svelte'
  import FolderTreeNode from './FolderTreeNode.svelte'
  import { ui } from '../stores/ui.svelte'

  export interface FolderNode {
    path: string
    name: string
    songCount: number
    children: FolderNode[]
  }

  let { node, depth = 0 }: { node: FolderNode; depth?: number } = $props()

  let expanded = $state(false)
  let hovered = $state(false)

  const isActive = $derived(ui.selectedSubFolderPath === node.path)
  const isPinned = $derived(ui.pinnedFolders.some(f => f.path === node.path))
  const hasChildren = $derived(node.children.length > 0)
</script>

<div class="tree-node">
  <div
    class="node-row"
    class:active={isActive}
    style="padding-left: {8 + depth * 12}px"
    role="button"
    tabindex="0"
    onmouseenter={() => hovered = true}
    onmouseleave={() => hovered = false}
    onclick={() => ui.navigateToSubFolder(node.path)}
    onkeydown={(e) => e.key === 'Enter' && ui.navigateToSubFolder(node.path)}
  >
    <button
      class="chevron-btn"
      style="opacity: {hasChildren ? 1 : 0}; pointer-events: {hasChildren ? 'auto' : 'none'}"
      onclick={(e) => { e.stopPropagation(); expanded = !expanded }}
      tabindex="-1"
    >
      <Icon name={expanded ? 'chevron-down' : 'chevron-right'} size={10} />
    </button>
    <Icon name="folder" size={13} />
    <span class="node-name">{node.name}</span>
    {#if hovered}
      <button
        class="pin-btn"
        class:pinned={isPinned}
        onclick={(e) => { e.stopPropagation(); isPinned ? ui.unpinFolder(node.path) : ui.pinFolder(node.path, node.name) }}
        title={isPinned ? 'Unpin' : 'Pin'}
        tabindex="-1"
      >
        <Icon name="pin" size={11} />
      </button>
    {:else if isPinned}
      <span class="pinned-dot"></span>
    {/if}
    <span class="node-count">{node.songCount}</span>
  </div>

  {#if expanded && hasChildren}
    {#each node.children as child}
      <FolderTreeNode node={child} depth={depth + 1} />
    {/each}
  {/if}
</div>

<style>
  .tree-node {
    display: flex;
    flex-direction: column;
  }

  .node-row {
    display: flex;
    align-items: center;
    gap: 5px;
    padding-right: 6px;
    height: 28px;
    border-radius: var(--radius);
    cursor: pointer;
    color: var(--on-surface-variant);
    font-size: 12px;
    transition: background 0.1s, color 0.1s;
    user-select: none;
    outline: none;
  }

  .node-row:hover { background: var(--surface-container); color: var(--on-surface); }
  .node-row.active { background: var(--primary-container); color: var(--on-primary-container); }

  .chevron-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 14px;
    flex-shrink: 0;
    color: inherit;
    padding: 0;
  }

  .node-name {
    flex: 1;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    min-width: 0;
  }

  .node-count {
    font-size: 10px;
    color: var(--outline);
    flex-shrink: 0;
  }

  .pin-btn {
    display: flex;
    align-items: center;
    color: var(--outline);
    flex-shrink: 0;
    border-radius: 3px;
    padding: 1px;
    line-height: 1;
  }

  .pin-btn:hover, .pin-btn.pinned { color: var(--secondary); }

  .pinned-dot {
    width: 4px;
    height: 4px;
    border-radius: 50%;
    background: var(--secondary);
    flex-shrink: 0;
  }
</style>
