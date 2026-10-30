<script lang="ts">
  // Ikony jsou zabalené v bundlu (§13.5), žádná síť. Čárové, 1.75 px, jako v hlavním designu.
  type Name = "back" | "plus" | "send" | "settings" | "users" | "lock" | "check" | "check2" | "clock" | "shield" | "trash" | "copy" | "chat" | "alert" | "close";
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
  };
</script>

<svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
  <path d={paths[name]} />
</svg>
