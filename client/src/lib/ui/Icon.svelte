<script lang="ts">
  // Ikony jsou zabalené v bundlu (§13.5), žádná síť. Čárové, 1.75 px, jako v hlavním designu.
  type Name = "back" | "plus" | "send" | "settings" | "users" | "lock" | "check" | "check2" | "clock" | "shield" | "trash" | "copy" | "chat" | "alert" | "close" | "search" | "qr" | "user" | "info" | "share" | "chevron" | "shieldcheck" | "paste" | "userplus" | "bubble" | "edit" | "key";
  let { name, size = 22 }: { name: Name; size?: number } = $props();

  const paths: Record<Name, string> = {
    back: "M19 12H5M11 6l-6 6 6 6",
    plus: "M12 5v14M5 12h14",
    send: "M12 19V5M5.5 11.5 12 5l6.5 6.5",
    settings: "M12 15.5a3.5 3.5 0 1 0 0-7 3.5 3.5 0 0 0 0 7ZM19.4 13.5l1.5 1.2-1.5 2.6-1.8-.6a7 7 0 0 1-1.6.9L15.7 19h-3l-.3-1.4a7 7 0 0 1-1.6-.9l-1.8.6-1.5-2.6 1.4-1.2a7 7 0 0 1 0-1.9L6.5 10.4 8 7.8l1.8.6a7 7 0 0 1 1.6-.9L11.7 5h3l.3 1.5a7 7 0 0 1 1.6.9l1.8-.6 1.5 2.6-1.4 1.2c.1.6.1 1.3 0 1.9Z",
    users: "M16 19v-1.5a3.5 3.5 0 0 0-3.5-3.5h-4A3.5 3.5 0 0 0 5 17.500V19M10.500 11a3 3 0 1 0 0-6 3 3 0 0 0 0 6ZM19 19v-1.500a3.500 3.500 0 0 0-2.500-3.300M15.500 5.200a3 3 0 0 1 0 5.600",
    lock: "M7 11V8a5 5 0 0 1 10 0v3M6 11h12v9H6z",
    check: "M5 12.500 10 17.500 19 7",
    check2: "M2.500 12.500 7.500 17.500 16.500 7M11 16.500l1 1L21.500 7",
    clock: "M12 21a9 9 0 1 0 0-18 9 9 0 0 0 0 18ZM12 7v5l3 2",
    shield: "M12 3 5 6v5.500c0 4.200 2.800 7.500 7 9 4.200-1.500 7-4.800 7-9V6z",
    trash: "M4 7h16M9 7V4h6v3M6.500 7l.8 13h9.400l.8-13M10 11v5M14 11v5",
    copy: "M9 9h10v11H9zM5 15V4h10",
    chat: "M5 5h14v10H10l-5 4z",
    alert: "M12 8v5M12 17v.01M12 3 2.500 20h19z",
    close: "M6 6l12 12M18 6 6 18",
    search: "M11 18a7 7 0 1 0 0-14 7 7 0 0 0 0 14ZM20 20l-4-4",
    qr: "M4 4h6v6H4zM14 4h6v6h-6zM4 14h6v6H4zM14 14h2v2h-2zM18 18h2v2h-2zM14 18v2M20 14v2",
    user: "M12 12a4 4 0 1 0 0-8 4 4 0 0 0 0 8ZM4.500 20a7.500 7.500 0 0 1 15 0",
    info: "M12 21a9 9 0 1 0 0-18 9 9 0 0 0 0 18ZM12 11v5M12 8v.01",
    share: "M12 4v11M7.500 8.500 12 4l4.500 4.500M5 13v6h14v-6",
    chevron: "M9 6l6 6-6 6",
    shieldcheck: "M12 3 5 6v5.500c0 4.200 2.800 7.500 7 9 4.200-1.500 7-4.800 7-9V6zM9 12l2.200 2.200L15.500 10",
    paste: "M9 4h6v3H9zM7 5.500H5v15h14v-15h-2",
    userplus: "M10 12a4 4 0 1 0 0-8 4 4 0 0 0 0 8ZM3 20a7 7 0 0 1 13-3.500M19 8v6M16 11h6",
    edit: "M4 20h4L19 9l-4-4L4 16zM13.500 6.500l4 4",
    key: "M14.500 14a4.500 4.500 0 1 0-4-2.500L4 18v2h3v-2h2v-2h2l1.200-1.200c.7.200 1.500.2 2.300.2ZM16.500 8.500v.01",
    bubble: "M4 6a2 2 0 0 1 2-2h12a2 2 0 0 1 2 2v9a2 2 0 0 1-2 2h-6l-4.500 3.500V17H6a2 2 0 0 1-2-2z",
  };
</script>

<svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
  <path d={paths[name]} />
</svg>
