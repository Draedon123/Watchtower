<script lang="ts">
  let { children } = $props();
</script>

{@render children()}

<style lang="scss">
  :global {
    body {
      background-color: black;
    }

    * {
      color: white;
    }
  }
</style>
