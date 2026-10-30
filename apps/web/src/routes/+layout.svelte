<script lang="ts">
  let { children } = $props();
</script>

{@render children()}

<meta name="viewport" content="width=device-width, initial-scale=1.0" />

<style lang="scss">
  :global {
    body {
      background-color: #181818;
    }

    * {
      color: #ddd;
    }
  }
</style>
