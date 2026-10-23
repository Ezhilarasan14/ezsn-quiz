/**
 * EZSN Quiz - Dynamic Organization & Quiz Branding Engine
 */
(function () {
  function hexToRgba(hex, alpha = 1) {
    if (!hex) return `rgba(99, 102, 241, ${alpha})`;
    let cleanHex = hex.replace("#", "");
    if (cleanHex.length === 3) {
      cleanHex = cleanHex.split("").map((c) => c + c).join("");
    }
    const num = parseInt(cleanHex, 16);
    if (isNaN(num)) return `rgba(99, 102, 241, ${alpha})`;
    const r = (num >> 16) & 255;
    const g = (num >> 8) & 255;
    const b = num & 255;
    return `rgba(${r}, ${g}, ${b}, ${alpha})`;
  }

  function applyBrandTheme(theme) {
    if (!theme) return;

    const primary = theme.primary_color || "#6366f1";
    const secondary = theme.secondary_color || "#ec4899";
    const bg = theme.background_color || "#0f172a";
    const btn = theme.button_color || "#3b82f6";
    const logoUrl = theme.logo_url;
    const orgName = theme.org_name || theme.name || "EZSN Quiz";

    const root = document.documentElement;
    root.style.setProperty("--brand-primary", primary);
    root.style.setProperty("--brand-secondary", secondary);
    root.style.setProperty("--brand-bg", bg);
    root.style.setProperty("--brand-btn", btn);

    // Dynamic background style
    document.body.style.backgroundColor = bg;
    document.body.style.backgroundImage = `
      radial-gradient(circle at 15% 20%, ${hexToRgba(primary, 0.22)} 0%, transparent 45%),
      radial-gradient(circle at 85% 80%, ${hexToRgba(secondary, 0.22)} 0%, transparent 45%),
      radial-gradient(circle at 50% 50%, ${hexToRgba(bg, 1)} 0%, ${bg} 100%)
    `;

    // Apply logo if element exists
    document.querySelectorAll(".brand-logo-img").forEach((img) => {
      if (logoUrl && logoUrl.trim() !== "") {
        img.src = logoUrl;
        img.classList.remove("hidden");
        const placeholder = img.parentElement ? img.parentElement.querySelector(".brand-logo-default") : null;
        if (placeholder) placeholder.classList.add("hidden");
      } else {
        img.classList.add("hidden");
        const placeholder = img.parentElement ? img.parentElement.querySelector(".brand-logo-default") : null;
        if (placeholder) placeholder.classList.remove("hidden");
      }
    });

    // Apply organization name
    document.querySelectorAll(".brand-org-name").forEach((el) => {
      el.textContent = orgName;
    });

    // Update dynamic buttons
    document.querySelectorAll(".btn-brand").forEach((button) => {
      button.style.backgroundColor = btn;
      button.style.borderColor = primary;
    });
  }

  async function fetchAndApplyOrgTheme() {
    try {
      const res = await fetch("/api/organization");
      if (res.ok) {
        const org = await res.json();
        applyBrandTheme(org);
        return org;
      }
    } catch (e) {
      console.warn("Could not load organization theme:", e);
    }
    return null;
  }

  window.applyBrandTheme = applyBrandTheme;
  window.fetchAndApplyOrgTheme = fetchAndApplyOrgTheme;

  // Auto-load organization branding on page load
  document.addEventListener("DOMContentLoaded", () => {
    fetchAndApplyOrgTheme();
  });
})();
