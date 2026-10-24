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
      --bg: #14090C;
      --card: #221015;
      --card-highlight: #2E161D;
      --border: #4A2430;
      --primary: #A3263F;
      --primary-hover: #BF304E;
      --accent: #F2A7B8;
      --accent-soft: #F8D7DE;
      --text: #F6E9EC;
      --text-muted: #B9959D;
      --font-serif: "Songti SC", "Source Han Serif SC", "Noto Serif CJK SC", "STSong", "SimSun", Georgia, serif;
      --font-sans: system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", "PingFang SC", "Microsoft YaHei", sans-serif;
    }

    @media (prefers-color-scheme: light) {
      :root {
        --bg: #FBEFF1;
        --card: #FFFFFF;
        --card-highlight: #FFF5F7;
        --border: #E8C4CC;
        --primary: #A3263F;
        --primary-hover: #821D31;
        --accent: #C44D66;
        --accent-soft: #7A2538;
        --text: #4A0E1C;
        --text-muted: #8C4F5E;
      }
    }

    * {
      box-sizing: border-box;
      margin: 0;
      padding: 0;
    }

    html, body {
      min-height: 100dvh;
      background-color: var(--bg);
      color: var(--text);
      font-family: var(--font-sans);
      overflow-x: hidden;
      background-image: radial-gradient(circle at 50% 0%, rgba(163, 38, 63, 0.15), transparent 75%);
    }

    @media (prefers-reduced-motion: reduce) {
      ::view-transition-group(*),
      ::view-transition-old(*),
      ::view-transition-new(*) {
        animation: none !important;
      }
      * {
        animation-duration: 0.01ms !important;
        animation-iteration-count: 1 !important;
        transition-duration: 0.01ms !important;
      }
    }
  </style>
</svelte:head>

<main class="app-root">
  {@render children()}
</main>

<style>
  .app-root {
    min-height: 100dvh;
    display: flex;
    flex-direction: column;
  }
</style>
