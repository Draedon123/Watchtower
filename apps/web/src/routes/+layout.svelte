<script lang="ts">
  import { resolve } from "$app/paths";
  import { page } from "$app/state";

  let { children } = $props();
</script>

<meta name="viewport" content="width=device-width, initial-scale=1.0" />

<title>Watchtower</title>

{#snippet Tab(path: string, label: string)}
  {@const selected = page.url.pathname === path}
  <!-- eslint-disable-next-line svelte/no-navigation-without-resolve -->
  <a href={path} class:selected>{label}</a>
{/snippet}

<nav>
  {@render Tab(resolve("/"), "Overview")}
  {@render Tab(resolve("/machines"), "Machines")}
</nav>

<main>
  {@render children()}
</main>

<style lang="scss">
  :global {
    body {
      background-color: #181818;
    }

    * {
      color: #ddd;
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

  nav {
    position: fixed;
    z-index: 999;
    top: 0;
    left: 0;

    height: 100vh;
    width: $navbar-width;

    display: flex;
    flex-direction: column;
    align-items: center;

    background-color: #333;

    a {
      $base-colour: #444;
      $selected-colour: #818;
      $margin: 0.25em;

      display: block;
      width: 90%;
      padding: 0.3em 0;
      margin: $margin 0;
      border-radius: 5px;

      text-align: center;
      text-decoration: none;

      background-color: $base-colour;
      transition: background-color 0.1s;

      &:first-child {
        margin-top: calc(2 * $margin);
      }

      &:last-child {
        margin-bottom: calc(2 * $margin);
      }

      &.selected {
        background-color: $selected-colour;
      }

      &:hover {
        background-color: mix($base-colour, #fff, 90%);
      }

      &.selected:hover {
        background-color: mix($selected-colour, #fff, 90%);
      }
    }
  }
</style>
