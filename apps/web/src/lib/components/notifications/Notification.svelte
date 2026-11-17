<script lang="ts">
  import type { Snippet } from "svelte";

  type Props = {
    children: Snippet<[]>;
  };

  let { children }: Props = $props();
</script>

<div class="container">
  <p class="message">{@render children()}</p>
</div>

<style lang="scss">
  .container {
    text-align: center;

    background-color: #000;
    border: 2px solid #707;
    border-radius: 10px;

    margin: 5px 0;

    box-shadow: 0 0 5px 3px #000a;
  }
</style>
