const notifications: {
  nextId: number;
  notifications: { id: number; message: string }[];
  readonly addNotification: (message: string) => number;
  onNewNotification?: (id: number) => void;
} = $state({
  nextId: 0,
  notifications: [],
  addNotification(message) {
    const id = notifications.nextId++;

    notifications.notifications.push({
      id,
      message,
    });

    notifications.onNewNotification?.(id);

    return id;
  },
});

export { notifications };
