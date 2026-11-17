<script lang="ts">
  import { notifications } from "$lib/notifications.svelte";
  import Notification from "./Notification.svelte";

  // TODO: fix everything
  // this system is actually atrocious
  notifications.onNewNotification = (id) => {
    setTimeout(() => {
      notifications.notifications.splice(
        notifications.notifications.findIndex(
          (notification) => notification.id === id
        ),
        1
      );
    }, 5000);
  };
</script>

<div class="container">
  {#each notifications.notifications as notification (notification.id)}
    <Notification>{notification.message}</Notification>
  {/each}
</div>

<style lang="scss">
  .container {
    position: fixed;
    bottom: 10px;
    left: 50%;
    translate: -50%;
    width: 250px;
    z-index: 999;

    display: flex;
    flex-direction: column;
  }
</style>
