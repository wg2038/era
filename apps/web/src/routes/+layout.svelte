<script lang="ts">
  import type { Snippet } from 'svelte';
  import { onNavigate } from '$app/navigation';

  interface Props {
    children: Snippet;
  }

  const { children }: Props = $props();

  onNavigate((navigation) => {
    if (!document.startViewTransition) return;
    return new Promise((resolve) => {
      document.startViewTransition(async () => {
        resolve();
        await navigation.complete;
      });
    });
  });
</script>

<svelte:head>
  <style>
    :root {
      color-scheme: dark;
      --font-body: system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", "PingFang SC", "Hiragino Sans GB", "Microsoft YaHei", sans-serif;
      --font-serif: "Songti SC", "SimSun", "Noto Serif CJK SC", serif;
    }
    body {
      margin: 0;
      padding: 0;
      font-family: var(--font-body);
      background-color: #121110;
      color: #e5e0d8;
    }
    @media (prefers-reduced-motion: reduce) {
      ::view-transition-group(*),
      ::view-transition-old(*),
      ::view-transition-new(*) {
        animation: none !important;
      }
    }
  </style>
</svelte:head>

<main class="min-h-screen">
  {@render children()}
</main>
