export function bytes(n: number): string {
  if (!Number.isFinite(n) || n <= 0) return "0 B";
  const units = ["B", "KiB", "MiB", "GiB", "TiB"];
  let i = 0;
  let v = n;
  while (v >= 1024 && i < units.length - 1) {
    v /= 1024;
    i++;
  }
  return `${v.toFixed(v < 10 && i > 0 ? 2 : v < 100 ? 1 : 0)} ${units[i]}`;
}

export function seconds(n: number | null | undefined): string {
  if (n == null || !Number.isFinite(n)) return "—";
  const h = Math.floor(n / 3600);
  const m = Math.floor((n % 3600) / 60);
  const s = n - h * 3600 - m * 60;
  if (h > 0) return `${h}:${pad(m)}:${pad(Math.floor(s))}`;
  return `${pad(m)}:${pad(Math.floor(s))}.${String(Math.floor((s % 1) * 1000)).padStart(3, "0").slice(0, 2)}`;
}

function pad(n: number): string {
  return String(n).padStart(2, "0");
}

export function kbps(n: number | null | undefined): string {
  if (n == null || !Number.isFinite(n)) return "—";
  if (n >= 1000) return `${(n / 1000).toFixed(2)} Mbps`;
  return `${n.toFixed(0)} kbps`;
}

export function lufs(n: number | null | undefined): string {
  if (n == null || !Number.isFinite(n)) return "—";
  return `${n.toFixed(2)} LUFS`;
}

export function dbtp(n: number | null | undefined): string {
  if (n == null || !Number.isFinite(n)) return "—";
  return `${n.toFixed(2)} dBTP`;
}

export function hexOffset(n: number): string {
  return `0x${n.toString(16).padStart(8, "0")}`;
}
