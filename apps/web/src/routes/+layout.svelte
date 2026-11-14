<script lang="ts">
  import { resolve } from "$app/paths";
  import { page } from "$app/state";
  import { connectionStatus } from "$lib/request.svelte";

  let { children } = $props();
</script>

<meta name="viewport" content="width=device-width, initial-scale=1.0" />

<title>Watchtower</title>

{#snippet Tab(
  path: string,
  label: string,
  level: 0 | 1 | 2 | 3 | 4 = 0,
  trimVerticalLine: boolean = false
)}
  {@const selected = decodeURIComponent(page.url.pathname) === path}
  <!-- eslint-disable svelte/no-navigation-without-resolve -->
  <div
    class="tab-container"
    style="grid-template-columns: repeat({level}, 1fr) {5 - level}fr;"
  >
    <!-- eslint-disable-next-line svelte/require-each-key -->
    {#each { length: level }, i}
      <div class="spacer">
        <div
          class="vertical-line"
          class:trim={trimVerticalLine && i === level - 1}
        ></div>
        {#if i === level - 1}
          <div class="horizontal-line"></div>
        {/if}
      </div>
    {/each}

    <a href={path} title={label} class:selected>{label}</a>
  </div>
  <!-- eslint-enable svelte/no-navigation-without-resolve -->
{/snippet}

<nav>
  {@render Tab(resolve("/"), "Overview")}
  {@render Tab(resolve("/machines"), "Machines")}
  {#if Array.isArray(connectionStatus.connectedMachines)}
    {#each connectionStatus.connectedMachines as machine, i (machine)}
      {@render Tab(
        resolve(`/machines/${machine}`),
        machine,
        1,
        i === connectionStatus.connectedMachines.length - 1
      )}
    {/each}
  {/if}
</nav>

<main>
  {@render children()}
</main>

<style lang="scss">
  @use "sass:color";

  :global {
    body {
      background-color: #181818;
    }

    * {
      color: #ddd;

      $scrollbar-track-colour: #0000;
      $scrollbar-thumb-colour: #382e48;
      $scrollbar-thumb-hover-colour: #3c344a;

      scrollbar-width: thin;
      scrollbar-color: $scrollbar-thumb-colour $scrollbar-track-colour;

      ::-moz-scrollbar-button,
      ::-webkit-scrollbar-button {
        display: none;
      }

      ::-webkit-scrollbar-thumb:hover {
        scrollbar-color: $scrollbar-thumb-colour $scrollbar-thumb-hover-colour;
      }
    }
  }

  $navbar-width: 250px;

  main {
    $spacing: 3ch;

    position: absolute;
    top: 0;
    left: calc($navbar-width + $spacing);

    // add spacing to the right
    width: calc(100% - $navbar-width - 2 * $spacing);
    height: 100%;
  }

  $tab-margin: 0.25em;
  nav {
    position: fixed;
    z-index: 999;
    top: 0;
    left: 0;

    height: 100vh;
    width: $navbar-width;
    overflow-y: scroll;

    display: flex;
    flex-direction: column;
    align-items: center;

    background-color: #333;

    a {
      $base-colour: #444;
      $selected-colour: #818;

      display: block;
      padding: 0.3em 0;
      margin: $tab-margin 0;
      border-radius: 5px;
      width: 100%;
      overflow: hidden;

      text-align: center;
      text-decoration: none;
      text-overflow: ellipsis;

      background-color: $base-colour;
      transition: background-color 0.1s;

      justify-self: end;

      &.selected {
        background-color: $selected-colour;
      }

      &:hover {
        background-color: color.mix($base-colour, #fff, 90%);
      }

      &.selected:hover {
        background-color: color.mix($selected-colour, #fff, 90%);
      }
    }
  }

  .tab-container {
    width: 80%;
    display: grid;

    &:first-child {
      margin-top: calc(2 * $tab-margin);
    }

    &:last-child {
      margin-bottom: calc(2 * $tab-margin);
    }
  }

  .spacer {
    width: 100%;
    display: flex;
  }

  $line-thickness: 2px;
  .vertical-line {
    width: 50%;
    height: 100%;
    border-right: $line-thickness solid #8888;
    box-sizing: border-box;

    &.trim {
      height: calc(50% + $line-thickness);
    }
  }

  .horizontal-line {
    width: 50%;
    height: 50%;
    border-bottom: $line-thickness solid #8888;
  }
</style>
