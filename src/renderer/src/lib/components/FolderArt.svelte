<script lang="ts">
  import Icon from './Icon.svelte'
  import { library } from '../stores/library.svelte'
  import { getArt } from '../stores/artCache.svelte'

  interface Props {
    folderId: string
    size?: number
  }

  const { folderId, size = 28 }: Props = $props()

  const artIds = $derived.by(() => {
    const folder = library.folders.find(f => f.id === folderId)
    if (!folder) return []
    const result: string[] = []
    for (const song of library.songs) {
      if (!song.path.startsWith(folder.path)) continue
      result.push(song.id)
      if (result.length === 4) break
    }
    return result
  })
</script>

<div class="folder-art" style="width:{size}px;height:{size}px">
  {#if artIds.length === 0 || !getArt(artIds[0])}
    <div class="fallback">
      <Icon name="folder" size={size * 0.55} />
    </div>
  {:else if artIds.length === 1}
    <img src={getArt(artIds[0])} alt="" class="single" />
  {:else}
    <div class="grid">
      {#each Array(4) as _, i}
        <div class="cell">
          {#if getArt(artIds[i])}
            <img src={getArt(artIds[i])} alt="" />
          {:else if getArt(artIds[artIds.length - 1])}
            <img src={getArt(artIds[artIds.length - 1])} alt="" />
          {/if}
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .folder-art {
    border-radius: var(--radius);
    overflow: hidden;
    flex-shrink: 0;
  }

  .fallback {
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--on-surface-variant);
  }

  .single {
    width: 100%;
    height: 100%;
    object-fit: cover;
    display: block;
  }

  .grid {
    width: 100%;
    height: 100%;
    display: grid;
    grid-template-columns: 1fr 1fr;
    grid-template-rows: 1fr 1fr;
    gap: 1px;
    background: var(--outline-variant);
  }

  .cell {
    overflow: hidden;
    background: var(--surface-container-high);
  }

  .cell img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    display: block;
  }
</style>
