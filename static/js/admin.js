/**
 * EZSN Quiz - Admin Dashboard & Organization Branding System
 */
document.addEventListener("DOMContentLoaded", () => {
  let adminToken = localStorage.getItem("admin_token");
  let currentAdmin = null;
  let currentOrg = null;
  let allQuizzes = [];
  let editingQuizForTheme = null;

  // DOM Elements
  const viewLogin = document.getElementById("view-login");
  const viewDashboard = document.getElementById("view-dashboard");
  const formLogin = document.getElementById("form-login");
  const loginError = document.getElementById("login-error");
  const btnLogout = document.getElementById("btn-logout");
  const adminBadge = document.getElementById("admin-user-badge");
  const loggedAdminName = document.getElementById("logged-admin-name");

  // Tab Navigation
  const tabButtons = document.querySelectorAll(".nav-tab");
  const tabPanes = document.querySelectorAll(".tab-pane");

  tabButtons.forEach((btn) => {
    btn.addEventListener("click", () => {
      const targetId = btn.dataset.tab;
      tabButtons.forEach((b) => {
        b.classList.remove("active", "text-white");
        b.classList.add("text-white/70");
      });
      btn.classList.add("active", "text-white");
      btn.classList.remove("text-white/70");

      tabPanes.forEach((p) => p.classList.add("hidden"));
      const targetPane = document.getElementById(targetId);
      if (targetPane) targetPane.classList.remove("hidden");

      if (targetId === "tab-quizzes") fetchQuizzes();
      if (targetId === "tab-branding") fetchOrganization();
      if (targetId === "tab-history") fetchHistory();
    });
  });

  // Check Existing Session
  checkAuth();

  async function checkAuth() {
    if (!adminToken) {
      showLogin();
      return;
    }

    try {
      const res = await fetch("/api/auth/me", {
        headers: { Authorization: `Bearer ${adminToken}` },
      });

      if (res.ok) {
        const data = await res.json();
        currentAdmin = data.admin;
        currentOrg = data.organization;
        showDashboard();
      } else {
        localStorage.removeItem("admin_token");
        adminToken = null;
        showLogin();
      }
    } catch (e) {
      console.warn("Auth check failed:", e);
      showLogin();
    }
  }

  function showLogin() {
    viewLogin.classList.remove("hidden");
    viewDashboard.classList.add("hidden");
    adminBadge.classList.add("hidden");
    adminBadge.classList.remove("flex");
  }

  function showDashboard() {
    viewLogin.classList.add("hidden");
    viewDashboard.classList.remove("hidden");
    adminBadge.classList.remove("hidden");
    adminBadge.classList.add("flex");
    if (currentAdmin) loggedAdminName.textContent = currentAdmin.username;

    fetchQuizzes();
    fetchOrganization();
  }

  // Login Form Submission
  formLogin.addEventListener("submit", async (e) => {
    e.preventDefault();
    loginError.classList.add("hidden");

    const username = document.getElementById("login-username").value.trim();
    const password = document.getElementById("login-password").value;

    try {
      const res = await fetch("/api/auth/login", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ username, password }),
      });

      const data = await res.json();
      if (res.ok) {
        adminToken = data.token;
        localStorage.setItem("admin_token", adminToken);
        currentAdmin = data.admin;
        currentOrg = data.organization;
        showDashboard();
        showToast("Signed in successfully!");
      } else {
        loginError.textContent = data.error || "Login failed. Check your credentials.";
        loginError.classList.remove("hidden");
      }
    } catch (err) {
      loginError.textContent = "Network error. Please try again.";
      loginError.classList.remove("hidden");
    }
  });

  // Logout
  btnLogout.addEventListener("click", async () => {
    if (adminToken) {
      try {
        await fetch("/api/auth/logout", {
          method: "POST",
          headers: { Authorization: `Bearer ${adminToken}` },
        });
      } catch (e) {}
    }
    localStorage.removeItem("admin_token");
    adminToken = null;
    currentAdmin = null;
    showLogin();
    showToast("Logged out");
  });

  // ==========================================
  // TAB 1: QUIZZES MANAGEMENT
  // ==========================================
  async function fetchQuizzes() {
    try {
      const res = await fetch("/api/quizzes");
      if (!res.ok) throw new Error("Failed to load quizzes");
      allQuizzes = await res.json();

      document.getElementById("metric-quiz-count").textContent = allQuizzes.length;
      const totalQ = allQuizzes.reduce((acc, q) => acc + (q.question_count || 0), 0);
      document.getElementById("metric-question-count").textContent = totalQ;

      renderQuizList(allQuizzes);
    } catch (e) {
      document.getElementById("admin-quiz-list").innerHTML =
        `<div class="text-center py-12 text-red-400 col-span-2">Error loading quizzes.</div>`;
    }
  }

  function renderQuizList(quizzes) {
    const listEl = document.getElementById("admin-quiz-list");
    if (!quizzes || quizzes.length === 0) {
      listEl.innerHTML = `
        <div class="text-center py-12 text-white/50 col-span-2 space-y-3">
          <p>No quizzes found.</p>
          <a href="/edit" class="btn-primary py-2 px-6 text-sm inline-block">+ Create First Quiz</a>
        </div>
      `;
      return;
    }

    listEl.innerHTML = quizzes
      .map((q) => {
        const hasCustomTheme = !!(q.theme_primary_color || q.theme_logo_url);
        const themeBadge = hasCustomTheme
          ? `<span class="px-2.5 py-1 rounded-full text-xs font-black bg-pink-500/20 text-pink-300 border border-pink-500/30">Custom Theme</span>`
          : `<span class="px-2.5 py-1 rounded-full text-xs font-black bg-white/10 text-white/60">Org Theme</span>`;

        return `
        <div class="glass-panel p-5 rounded-2xl border border-white/10 hover:border-white/20 transition-all flex flex-col justify-between gap-4">
          <div>
            <div class="flex items-start justify-between gap-2 mb-2">
              <h4 class="text-lg font-black text-white leading-tight">${escapeHtml(q.title)}</h4>
              ${themeBadge}
            </div>
            <p class="text-xs text-white/60 line-clamp-2 mb-3">${escapeHtml(q.description || "No description provided.")}</p>
            <div class="flex items-center gap-3 text-xs text-white/50">
              <span class="font-bold text-indigo-400">📝 ${q.question_count} Questions</span>
              <span>•</span>
              <span>Created ${new Date(q.created_at).toLocaleDateString()}</span>
            </div>
          </div>

          <div class="pt-3 border-t border-white/10 flex flex-wrap items-center justify-between gap-2">
            <button class="btn-primary py-1.5 px-4 text-xs font-black flex items-center gap-1.5" onclick="launchQuiz('${q.id}')">
              <span>🚀</span> <span>Launch</span>
            </button>
            <div class="flex items-center gap-1.5">
              <button class="btn-secondary text-xs py-1.5 px-3" onclick="openThemeModal('${q.id}')" title="Customize Branding">
                🎨 Theme
              </button>
              <a href="/edit?id=${q.id}" class="btn-secondary text-xs py-1.5 px-3" title="Edit Questions">
                ✏️ Edit
              </a>
              <button class="btn-secondary text-xs py-1.5 px-2.5" onclick="duplicateQuiz('${q.id}')" title="Duplicate">
                📋
              </button>
              <button class="btn-secondary text-xs py-1.5 px-2.5 text-red-300 hover:text-red-100" onclick="deleteQuiz('${q.id}', '${escapeHtml(q.title)}')" title="Delete">
                🗑️
              </button>
            </div>
          </div>
        </div>
      `;
      })
      .join("");
  }

  // Search filter
  document.getElementById("input-search-quizzes").addEventListener("input", (e) => {
    const query = e.target.value.toLowerCase();
    const filtered = allQuizzes.filter(
      (q) => q.title.toLowerCase().includes(query) || (q.description && q.description.toLowerCase().includes(query))
    );
    renderQuizList(filtered);
  });

  // Launch live room
  window.launchQuiz = async (quizId) => {
    try {
      const res = await fetch("/api/rooms", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ quiz_id: quizId }),
      });
      if (!res.ok) throw new Error("Could not create game room");
      const data = await res.json();
      window.location.href = `/host?pin=${data.pin}&token=${data.host_token}`;
    } catch (e) {
      showToast(e.message || "Failed to launch game", true);
    }
  };

  // Duplicate Quiz
  window.duplicateQuiz = async (quizId) => {
    try {
      const res = await fetch(`/api/quizzes/${quizId}/duplicate`, { method: "POST" });
      if (!res.ok) throw new Error("Could not duplicate quiz");
      showToast("Quiz duplicated!");
      fetchQuizzes();
    } catch (e) {
      showToast(e.message || "Duplicate failed", true);
    }
  };

  // Delete Quiz
  window.deleteQuiz = async (quizId, title) => {
    if (!confirm(`Are you sure you want to delete "${title}"? This cannot be undone.`)) return;
    try {
      const res = await fetch(`/api/quizzes/${quizId}`, { method: "DELETE" });
      if (!res.ok) throw new Error("Could not delete quiz");
      showToast("Quiz deleted");
      fetchQuizzes();
    } catch (e) {
      showToast(e.message || "Delete failed", true);
    }
  };

  // ==========================================
  // TAB 2: ORGANIZATION BRANDING
  // ==========================================
  const orgNameInput = document.getElementById("org-name");
  const orgLogoInput = document.getElementById("org-logo-input");
  const orgLogoPreview = document.getElementById("org-logo-preview");
  const orgLogoPlaceholder = document.getElementById("org-logo-placeholder");
  let currentLogoUrl = "";

  const colorPrimary = document.getElementById("org-primary-color");
  const hexPrimary = document.getElementById("org-primary-hex");
  const colorSecondary = document.getElementById("org-secondary-color");
  const hexSecondary = document.getElementById("org-secondary-hex");
  const colorBg = document.getElementById("org-bg-color");
  const hexBg = document.getElementById("org-bg-hex");
  const colorBtn = document.getElementById("org-btn-color");
  const hexBtn = document.getElementById("org-btn-hex");

  // Live preview elements
  const previewBox = document.getElementById("theme-preview-box");
  const previewLogo = document.getElementById("preview-logo");
  const previewLogoIcon = document.getElementById("preview-logo-icon");
  const previewOrgName = document.getElementById("preview-org-name");
  const previewBtn = document.getElementById("preview-btn");

  async function fetchOrganization() {
    try {
      const res = await fetch("/api/organization");
      if (!res.ok) return;
      currentOrg = await res.json();

      orgNameInput.value = currentOrg.name || "EZSN Quiz Platform";
      currentLogoUrl = currentOrg.logo_url || "";
      updateLogoPreview(currentLogoUrl);

      setThemeColors(
        currentOrg.primary_color || "#6366f1",
        currentOrg.secondary_color || "#ec4899",
        currentOrg.background_color || "#0f172a",
        currentOrg.button_color || "#3b82f6"
      );

      document.getElementById("metric-branding-status").textContent = currentOrg.name || "Configured";
    } catch (e) {
      console.warn("Could not fetch organization:", e);
    }
  }

  function updateLogoPreview(url) {
    if (url && url.trim() !== "") {
      orgLogoPreview.src = url;
      orgLogoPreview.classList.remove("hidden");
      orgLogoPlaceholder.classList.add("hidden");

      previewLogo.src = url;
      previewLogo.classList.remove("hidden");
      previewLogoIcon.classList.add("hidden");
    } else {
      orgLogoPreview.classList.add("hidden");
      orgLogoPlaceholder.classList.remove("hidden");

      previewLogo.classList.add("hidden");
      previewLogoIcon.classList.remove("hidden");
    }
  }

  function setThemeColors(pri, sec, bg, btn) {
    colorPrimary.value = pri;
    hexPrimary.value = pri;
    colorSecondary.value = sec;
    hexSecondary.value = sec;
    colorBg.value = bg;
    hexBg.value = bg;
    colorBtn.value = btn;
    hexBtn.value = btn;

    updateLivePreview();
  }

  function updateLivePreview() {
    const pri = hexPrimary.value;
    const sec = hexSecondary.value;
    const bg = hexBg.value;
    const btn = hexBtn.value;
    const name = orgNameInput.value || "EZSN Quiz";

    previewBox.style.backgroundColor = bg;
    previewBox.style.backgroundImage = `radial-gradient(circle at top right, ${pri}33, transparent 55%), radial-gradient(circle at bottom left, ${sec}33, transparent 55%)`;
    previewOrgName.textContent = name;
    previewBtn.style.backgroundColor = btn;
    previewBtn.style.borderColor = pri;
  }

  // Two-way bindings for colors
  [
    [colorPrimary, hexPrimary],
    [colorSecondary, hexSecondary],
    [colorBg, hexBg],
    [colorBtn, hexBtn],
  ].forEach(([colorInput, hexInput]) => {
    colorInput.addEventListener("input", () => {
      hexInput.value = colorInput.value.toUpperCase();
      updateLivePreview();
    });
    hexInput.addEventListener("input", () => {
      if (/^#[0-9A-Fa-f]{6}$/.test(hexInput.value)) {
        colorInput.value = hexInput.value;
        updateLivePreview();
      }
    });
  });

  orgNameInput.addEventListener("input", updateLivePreview);

  // Preset Palette click
  document.querySelectorAll(".preset-btn").forEach((btn) => {
    btn.addEventListener("click", () => {
      setThemeColors(
        btn.dataset.primary,
        btn.dataset.secondary,
        btn.dataset.bg,
        btn.dataset.btn
      );
    });
  });

  // Handle Logo Upload via Base64 to /api/upload
  orgLogoInput.addEventListener("change", async (e) => {
    const file = e.target.files[0];
    if (!file) return;

    const reader = new FileReader();
    reader.onload = async (evt) => {
      const base64Data = evt.target.result;
      updateLogoPreview(base64Data);

      try {
        const uploadRes = await fetch("/api/upload", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ filename: file.name, data: base64Data }),
        });

        if (uploadRes.ok) {
          const resData = await uploadRes.json();
          currentLogoUrl = resData.url;
          updateLogoPreview(currentLogoUrl);
          showToast("Logo uploaded successfully!");
        } else {
          showToast("Failed to upload image", true);
        }
      } catch (err) {
        showToast("Error uploading file", true);
      }
    };
    reader.readAsDataURL(file);
  });

  // Save Organization Branding
  document.getElementById("btn-save-branding").addEventListener("click", async () => {
    if (!adminToken) return showLogin();

    const payload = {
      name: orgNameInput.value.trim() || "EZSN Quiz Platform",
      logo_url: currentLogoUrl || null,
      primary_color: hexPrimary.value,
      secondary_color: hexSecondary.value,
      background_color: hexBg.value,
      button_color: hexBtn.value,
    };

    try {
      const res = await fetch("/api/organization", {
        method: "PUT",
        headers: {
          "Content-Type": "application/json",
          Authorization: `Bearer ${adminToken}`,
        },
        body: JSON.stringify(payload),
      });

      if (res.ok) {
        currentOrg = await res.json();
        showToast("Organization branding saved!");
        if (window.applyBrandTheme) window.applyBrandTheme(currentOrg);
      } else {
        showToast("Failed to save branding", true);
      }
    } catch (e) {
      showToast("Network error saving branding", true);
    }
  });

  // ==========================================
  // TAB 3: LEADERBOARDS & HISTORY
  // ==========================================
  async function fetchHistory() {
    try {
      const res = await fetch("/api/history");
      if (!res.ok) throw new Error("Failed to load history");
      const historyItems = await res.json();

      const container = document.getElementById("admin-history-list");
      if (!historyItems || historyItems.length === 0) {
        container.innerHTML = `<div class="text-center py-12 text-white/50">No completed games found yet.</div>`;
        return;
      }

      container.innerHTML = historyItems
        .map((item) => {
          return `
          <div class="glass-panel p-5 rounded-2xl border border-white/10 flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4">
            <div>
              <div class="flex items-center gap-2 mb-1">
                <span class="text-xs font-black px-2 py-0.5 rounded bg-indigo-500/20 text-indigo-300">PIN: ${item.room_pin}</span>
                <h4 class="text-lg font-black text-white">${escapeHtml(item.quiz_title)}</h4>
              </div>
              <div class="text-xs text-white/60 flex items-center gap-3">
                <span>👥 ${item.total_players} players</span>
                <span>•</span>
                <span>${new Date(item.created_at).toLocaleString()}</span>
              </div>
            </div>

            <div class="flex items-center gap-4 w-full sm:w-auto justify-between sm:justify-end">
              ${
                item.winner_nickname
                  ? `<div class="text-right">
                       <span class="text-xs text-amber-300 font-extrabold block">👑 ${escapeHtml(item.winner_nickname)}</span>
                       <span class="text-xs text-white/50 font-bold">${item.winner_score?.toLocaleString() || 0} pts</span>
                     </div>`
                  : `<span class="text-xs text-white/40">Incomplete</span>`
              }
              <a href="/history" class="btn-secondary text-xs py-2 px-4">View Standings &rarr;</a>
            </div>
          </div>
        `;
        })
        .join("");
    } catch (e) {
      document.getElementById("admin-history-list").innerHTML =
        `<div class="text-center py-12 text-red-400">Failed to load history.</div>`;
    }
  }

  document.getElementById("btn-refresh-history").addEventListener("click", fetchHistory);

  // ==========================================
  // QUIZ-SPECIFIC THEME MODAL
  // ==========================================
  const modalTheme = document.getElementById("modal-quiz-theme");
  const modalTitle = document.getElementById("modal-quiz-title");
  const modalOverride = document.getElementById("modal-override-theme");
  const modalFields = document.getElementById("modal-custom-theme-fields");
  const modalLogoFile = document.getElementById("modal-logo-file");
  const modalLogoUrl = document.getElementById("modal-logo-url");
  const modalPri = document.getElementById("modal-primary-color");
  const modalSec = document.getElementById("modal-secondary-color");
  const modalBg = document.getElementById("modal-bg-color");
  const modalBtn = document.getElementById("modal-btn-color");

  window.openThemeModal = async (quizId) => {
    try {
      const res = await fetch(`/api/quizzes/${quizId}`);
      if (!res.ok) throw new Error("Could not load quiz details");
      editingQuizForTheme = await res.json();

      modalTitle.textContent = `Theme: ${editingQuizForTheme.title}`;

      const hasCustom = !!(editingQuizForTheme.theme_primary_color || editingQuizForTheme.theme_logo_url);
      modalOverride.checked = hasCustom;
      if (hasCustom) {
        modalFields.classList.remove("hidden");
        modalLogoUrl.value = editingQuizForTheme.theme_logo_url || "";
        modalPri.value = editingQuizForTheme.theme_primary_color || "#6366f1";
        modalSec.value = editingQuizForTheme.theme_secondary_color || "#ec4899";
        modalBg.value = editingQuizForTheme.theme_bg_color || "#0f172a";
        modalBtn.value = editingQuizForTheme.theme_btn_color || "#3b82f6";
      } else {
        modalFields.classList.add("hidden");
      }

      modalTheme.classList.remove("hidden");
    } catch (e) {
      showToast(e.message || "Failed to load quiz", true);
    }
  };

  modalOverride.addEventListener("change", () => {
    if (modalOverride.checked) {
      modalFields.classList.remove("hidden");
    } else {
      modalFields.classList.add("hidden");
    }
  });

  modalLogoFile.addEventListener("change", (e) => {
    const file = e.target.files[0];
    if (!file) return;
    const reader = new FileReader();
    reader.onload = async (evt) => {
      try {
        const uploadRes = await fetch("/api/upload", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ filename: file.name, data: evt.target.result }),
        });
        if (uploadRes.ok) {
          const data = await uploadRes.json();
          modalLogoUrl.value = data.url;
          showToast("Quiz logo uploaded!");
        }
      } catch (err) {
        showToast("Failed to upload quiz logo", true);
      }
    };
    reader.readAsDataURL(file);
  });

  function closeThemeModal() {
    modalTheme.classList.add("hidden");
    editingQuizForTheme = null;
  }

  document.getElementById("btn-close-theme-modal").addEventListener("click", closeThemeModal);
  document.getElementById("btn-cancel-quiz-theme").addEventListener("click", closeThemeModal);

  document.getElementById("btn-save-quiz-theme").addEventListener("click", async () => {
    if (!editingQuizForTheme) return;

    const isOverride = modalOverride.checked;
    const updatePayload = {
      title: editingQuizForTheme.title,
      description: editingQuizForTheme.description,
      organization_id: editingQuizForTheme.organization_id || "org_default",
      theme_logo_url: isOverride ? modalLogoUrl.value || null : null,
      theme_primary_color: isOverride ? modalPri.value : null,
      theme_secondary_color: isOverride ? modalSec.value : null,
      theme_bg_color: isOverride ? modalBg.value : null,
      theme_btn_color: isOverride ? modalBtn.value : null,
      questions: editingQuizForTheme.questions.map((q) => ({
        question_text: q.question_text,
        time_limit_sec: q.time_limit_sec,
        points: q.points,
        options: q.options.map((o) => ({
          option_text: o.option_text,
          is_correct: o.is_correct,
        })),
      })),
    };

    try {
      const res = await fetch(`/api/quizzes/${editingQuizForTheme.id}`, {
        method: "PUT",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(updatePayload),
      });

      if (res.ok) {
        showToast("Quiz theme updated!");
        closeThemeModal();
        fetchQuizzes();
      } else {
        showToast("Failed to update quiz theme", true);
      }
    } catch (e) {
      showToast("Error updating theme", true);
    }
  });

  function escapeHtml(str) {
    if (!str) return "";
    return str
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;")
      .replace(/'/g, "&#039;");
  }
});
