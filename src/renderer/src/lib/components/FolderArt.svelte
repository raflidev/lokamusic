<script lang="ts">
  import Icon from './Icon.svelte'
  import { library } from '../stores/library.svelte'

  interface Props {
    folderId: string
    size?: number
  }

  const { folderId, size = 28 }: Props = $props()

  const arts = $derived.by(() => {
    const folder = library.folders.find(f => f.id === folderId)
    if (!folder) return []
    const seen = new Set<string>()
    const result: string[] = []
    for (const song of library.songs) {
      if (!song.albumArt || !song.path.startsWith(folder.path)) continue
      if (!seen.has(song.albumArt)) {
        seen.add(song.albumArt)
        result.push(song.albumArt)
        if (result.length === 4) break
      }
    }
    return result
  })
</script>

<div class="folder-art" style="width:{size}px;height:{size}px">
  {#if arts.length === 0}
    <div class="fallback">
      <Icon name="folder" size={size * 0.55} />
    </div>
  {:else if arts.length === 1}
    <img src={arts[0]} alt="" class="single" />
  {:else}
    <div class="grid">
      {#each Array(4) as _, i}
        <div class="cell">
          {#if arts[i]}
            <img src={arts[i]} alt="" />
          {:else if arts[arts.length - 1]}
            <img src={arts[arts.length - 1]} alt="" />
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
