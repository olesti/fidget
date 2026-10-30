/** "system" | "light" | "dark" -> data-theme on <html>. Shared by the settings window and the panel. */
export function applyTheme(theme: string) {
  const dark = theme === "dark" || (theme === "system" && matchMedia("(prefers-color-scheme: dark)").matches);
  document.documentElement.dataset.theme = dark ? "dark" : "light";
}

export const savedTheme = () => localStorage.getItem("theme") ?? "system";
