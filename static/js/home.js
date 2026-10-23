// Home Page Logic
document.addEventListener("DOMContentLoaded", () => {
  const quizListEl = document.getElementById("quiz-list");
  const joinForm = document.getElementById("join-form");
  const pinInput = document.getElementById("join-pin");
  const nicknameInput = document.getElementById("join-nickname");
  const soundToggleBtn = document.getElementById("btn-sound-toggle");
  const soundIcon = document.getElementById("sound-icon");

  // Format PIN input: numbers only
  pinInput.addEventListener("input", (e) => {
    e.target.value = e.target.value.replace(/\D/g, "");
  });

  // Sound toggle
  function updateSoundIcon() {
    soundIcon.textContent = window.soundFX.isMuted() ? "🔇" : "🔊";
  }
  updateSoundIcon();

  soundToggleBtn.addEventListener("click", () => {
    const isMuted = window.soundFX.toggleMute();
    updateSoundIcon();
    showToast(isMuted ? "Sound muted" : "Sound enabled");
  });

  // Load available quizzes
  async function loadQuizzes() {
    try {
      const quizzes = await window.api.get("/api/quizzes");
      if (!quizzes || quizzes.length === 0) {
        quizListEl.innerHTML = `
          <div class="text-center py-6 text-white/60 text-sm">
            No quizzes found. <a href="/edit" class="text-amber-400 underline font-bold">Create one now</a>!
          </div>
        `;
        return;
      }

      quizListEl.innerHTML = quizzes
        .map(
          (q) => `
          <div class="p-3.5 rounded-xl bg-white/10 hover:bg-white/15 border border-white/10 transition flex items-center justify-between gap-3">
            <div class="overflow-hidden">
              <h3 class="font-bold text-white text-base truncate">${escapeHtml(q.title)}</h3>
              <p class="text-xs text-white/60 truncate">${escapeHtml(q.description || "No description")} &bull; ${q.question_count} questions</p>
            </div>
            <button 
              data-quiz-id="${q.id}" 
              class="btn-host-quiz px-3.5 py-1.5 rounded-lg bg-amber-500 hover:bg-amber-400 text-black font-black text-xs uppercase tracking-wider transition shrink-0 shadow"
            >
              Host
            </button>
          </div>
        `
        )
        .join("");

      // Bind host buttons
      document.querySelectorAll(".btn-host-quiz").forEach((btn) => {
        btn.addEventListener("click", async (e) => {
          const quizId = e.target.getAttribute("data-quiz-id");
          e.target.disabled = true;
          e.target.textContent = "Creating...";

          try {
            window.soundFX.click();
            const res = await window.api.post("/api/rooms", { quiz_id: quizId });
            sessionStorage.setItem("ezsn_quiz_host_token", res.host_token);
            sessionStorage.setItem("ezsn_quiz_host_pin", res.pin);
            sessionStorage.setItem("ezsn_quiz_game_id", res.game_id);
            window.location.href = `/host?pin=${res.pin}`;
          } catch (err) {
            e.target.disabled = false;
            e.target.textContent = "Host";
            showToast(err.message || "Failed to create room", true);
          }
        });
      });
    } catch (err) {
      quizListEl.innerHTML = `
        <div class="text-center py-6 text-red-400 text-sm">
          Failed to load quizzes. Please try refreshing.
        </div>
      `;
    }
  }

  // Join form submission
  joinForm.addEventListener("submit", async (e) => {
    e.preventDefault();
    const pin = pinInput.value.trim();
    const nickname = nicknameInput.value.trim();

    if (pin.length !== 6) {
      showToast("Please enter a valid 6-digit PIN", true);
      return;
    }
    if (!nickname) {
      showToast("Please enter a nickname", true);
      return;
    }

    const joinBtn = document.getElementById("btn-join");
    joinBtn.disabled = true;
    joinBtn.innerHTML = "<span>Checking Room...</span>";

    try {
      window.soundFX.click();
      const status = await window.api.get(`/api/rooms/${pin}`);
      if (status.status !== "lobby") {
        throw new Error("This game has already started. You cannot join right now.");
      }

      sessionStorage.setItem("ezsn_quiz_player_pin", pin);
      sessionStorage.setItem("ezsn_quiz_player_nickname", nickname);
      window.location.href = `/play?pin=${pin}`;
    } catch (err) {
      joinBtn.disabled = false;
      joinBtn.innerHTML = "<span>Enter Game</span> <span>🚀</span>";
      showToast(err.message || "Room not found or expired", true);
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

  loadQuizzes();
});
