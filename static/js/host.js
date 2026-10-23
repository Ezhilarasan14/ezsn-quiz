// Host Screen Controller
document.addEventListener("DOMContentLoaded", () => {
  const urlParams = new URLSearchParams(window.location.search);
  const pin = urlParams.get("pin") || sessionStorage.getItem("ezsn_quiz_host_pin");
  const token = sessionStorage.getItem("ezsn_quiz_host_token");

  if (!pin || !token) {
    alert("Missing host credentials. Redirecting to home.");
    window.location.href = "/";
    return;
  }

  // Views
  const viewLobby = document.getElementById("view-lobby");
  const viewIntro = document.getElementById("view-intro");
  const viewQuestion = document.getElementById("view-question");
  const viewResult = document.getElementById("view-result");
  const viewLeaderboard = document.getElementById("view-leaderboard");
  const viewPodium = document.getElementById("view-podium");

  // Header Elements
  const hostPinDisplay = document.getElementById("host-pin-display");
  const hostQuizTitle = document.getElementById("host-quiz-title");
  const soundToggleBtn = document.getElementById("btn-sound-toggle");
  const soundIcon = document.getElementById("sound-icon");

  hostPinDisplay.textContent = pin;
  document.getElementById("lobby-pin").textContent = pin;
  document.getElementById("lobby-url").textContent = `${window.location.origin}/`;

  // Sound toggle
  function updateSoundIcon() {
    soundIcon.textContent = window.soundFX.isMuted() ? "🔇" : "🔊";
  }
  updateSoundIcon();
  soundToggleBtn.addEventListener("click", () => {
    window.soundFX.toggleMute();
    updateSoundIcon();
  });

  function switchView(activeView) {
    [viewLobby, viewIntro, viewQuestion, viewResult, viewLeaderboard, viewPodium].forEach((v) => {
      v.classList.add("hidden");
    });
    activeView.classList.remove("hidden");
  }

  // Timer reference
  let questionTimerInterval = null;
  let activeQuestionOptions = [];

  const ws = new QuizWebSocket();
  ws.connect(() => {
    // Authenticate as Host
    ws.send("host_connect", { pin, token });
  });

  // Server message handlers
  ws.on("connected", (data) => {
    console.log("Host connected successfully to room:", data.pin);
  });

  ws.on("error", (data) => {
    showToast(data.message, true);
  });

  ws.on("lobby_state", (data) => {
    switchView(viewLobby);
    hostQuizTitle.textContent = data.quiz_title;
    document.getElementById("player-count").textContent = data.player_count;

    const grid = document.getElementById("player-grid");
    if (!data.players || data.players.length === 0) {
      grid.innerHTML = `
        <div class="col-span-full text-center py-12 text-white/40 italic">
          Waiting for players to join with the PIN...
        </div>
      `;
      return;
    }

    grid.innerHTML = data.players
      .map(
        (p) => `
        <div 
          data-player-id="${p.player_id}"
          class="player-chip p-3 rounded-xl bg-white/10 hover:bg-red-500/30 border border-white/15 transition flex items-center justify-between cursor-pointer group"
          title="Click to remove player"
        >
          <div class="flex items-center gap-2 overflow-hidden">
            <span class="w-3 h-3 rounded-full ${p.is_connected ? "bg-emerald-400" : "bg-red-400"} shrink-0"></span>
            <span class="font-bold text-white text-sm truncate">${escapeHtml(p.nickname)}</span>
          </div>
          <span class="text-xs text-white/40 group-hover:text-red-300 font-bold ml-1">✕</span>
        </div>
      `
      )
      .join("");

    // Bind kick event
    grid.querySelectorAll(".player-chip").forEach((chip) => {
      chip.addEventListener("click", () => {
        const pid = chip.getAttribute("data-player-id");
        if (confirm("Remove this player from the game?")) {
          ws.send("host_kick_player", { player_id: pid });
        }
      });
    });
  });

  ws.on("player_joined", (data) => {
    window.soundFX.tick();
    showToast(`${data.nickname} joined!`);
  });

  ws.on("player_left", (data) => {
    showToast(`${data.nickname} disconnected`);
  });

  ws.on("game_starting", (data) => {
    window.soundFX.click();
    showToast("Game starting in 3 seconds!");
  });

  ws.on("question_intro", (data) => {
    switchView(viewIntro);
    document.getElementById("intro-q-counter").textContent = `Question ${data.question_index} of ${data.total_questions}`;
    document.getElementById("intro-q-text").textContent = data.question_text;

    let count = data.countdown_secs;
    const countEl = document.getElementById("intro-countdown");
    countEl.textContent = count;

    const timer = setInterval(() => {
      count--;
      if (count >= 1) {
        countEl.textContent = count;
        window.soundFX.tick();
      } else {
        clearInterval(timer);
      }
    }, 1000);
  });

  ws.on("question_active", (data) => {
    switchView(viewQuestion);
    activeQuestionOptions = data.options;

    document.getElementById("active-q-counter").textContent = `Question ${data.question_index}/${data.total_questions}`;
    document.getElementById("active-q-text").textContent = data.question_text;
    document.getElementById("active-answers-count").textContent = "0";

    const timerEl = document.getElementById("active-timer");
    let remaining = data.time_limit_sec;
    timerEl.textContent = remaining;
    timerEl.classList.remove("timer-warning");

    if (questionTimerInterval) clearInterval(questionTimerInterval);

    questionTimerInterval = setInterval(() => {
      remaining--;
      if (remaining <= 0) {
        timerEl.textContent = "0";
        clearInterval(questionTimerInterval);
      } else {
        timerEl.textContent = remaining;
        if (remaining <= 5) {
          timerEl.classList.add("timer-warning");
          window.soundFX.tick();
        }
      }
    }, 1000);

    // Render 4 answer cards
    const colorClasses = ["btn-red", "btn-blue", "btn-yellow", "btn-green"];
    const shapes = ["shape-triangle", "shape-diamond", "shape-circle", "shape-square"];

    const grid = document.getElementById("active-options-grid");
    grid.innerHTML = data.options
      .map(
        (opt, idx) => `
        <div class="btn-answer ${colorClasses[opt.color_idx || idx % 4]} flex items-center justify-start px-6">
          <div class="shape-icon"><div class="${shapes[opt.color_idx || idx % 4]}"></div></div>
          <span class="text-xl md:text-2xl font-bold ml-4 text-left">${escapeHtml(opt.text)}</span>
        </div>
      `
      )
      .join("");
  });

  ws.on("host_answer_count_update", (data) => {
    document.getElementById("active-answers-count").textContent = data.answered_count;
    document.getElementById("active-total-players").textContent = `/ ${data.total_players}`;
  });

  ws.on("question_result", (data) => {
    if (questionTimerInterval) clearInterval(questionTimerInterval);
    switchView(viewResult);

    const colorClasses = ["btn-red", "btn-blue", "btn-yellow", "btn-green"];
    const shapes = ["shape-triangle", "shape-diamond", "shape-circle", "shape-square"];

    const grid = document.getElementById("result-options-grid");
    grid.innerHTML = activeQuestionOptions
      .map((opt, idx) => {
        const count = (data.option_counts && data.option_counts[opt.id]) || 0;
        const isCorrect = data.correct_option_ids && data.correct_option_ids.includes(opt.id);

        return `
        <div class="btn-answer ${colorClasses[opt.color_idx || idx % 4]} flex flex-col items-start justify-between p-6 relative overflow-hidden ${
          isCorrect ? "ring-4 ring-white shadow-2xl" : "opacity-60"
        }">
          <div class="flex items-center justify-between w-full mb-3">
            <div class="flex items-center gap-3">
              <div class="shape-icon"><div class="${shapes[opt.color_idx || idx % 4]}"></div></div>
              <span class="text-xl font-bold">${escapeHtml(opt.text)}</span>
            </div>
            ${isCorrect ? `<span class="text-3xl">✅</span>` : ""}
          </div>
          <div class="w-full flex items-center justify-between pt-2 border-t border-white/20">
            <span class="text-xs uppercase font-extrabold tracking-wider text-white/80">Answers</span>
            <span class="text-2xl font-black">${count}</span>
          </div>
        </div>
      `;
      })
      .join("");
  });

  ws.on("leaderboard_update", (data) => {
    switchView(viewLeaderboard);
    const list = document.getElementById("leaderboard-list");

    if (!data.rankings || data.rankings.length === 0) {
      list.innerHTML = `<div class="text-center py-8 text-white/50">No scores yet</div>`;
      return;
    }

    list.innerHTML = data.rankings
      .slice(0, 5)
      .map((entry, idx) => {
        let badge = "";
        if (entry.rank === 1) badge = "🥇";
        else if (entry.rank === 2) badge = "🥈";
        else if (entry.rank === 3) badge = "🥉";

        let rankIndicator = "";
        if (entry.rank_change > 0) {
          rankIndicator = `<span class="text-xs font-bold text-emerald-400">▲ +${entry.rank_change}</span>`;
        } else if (entry.rank_change < 0) {
          rankIndicator = `<span class="text-xs font-bold text-red-400">▼ ${entry.rank_change}</span>`;
        }

        let streakBadge = "";
        if (entry.streak >= 2) {
          streakBadge = `<span class="px-2 py-0.5 rounded-full bg-amber-500/30 text-amber-300 text-xs font-black">🔥 ${entry.streak}</span>`;
        }

        return `
        <div class="glass-panel p-4 flex items-center justify-between gap-4 border-l-4 ${
          entry.rank === 1 ? "border-amber-400" : "border-white/20"
        }">
          <div class="flex items-center gap-4">
            <span class="text-2xl font-black w-8 text-center">${badge || `#${entry.rank}`}</span>
            <div>
              <div class="flex items-center gap-2">
                <span class="font-extrabold text-lg text-white">${escapeHtml(entry.nickname)}</span>
                ${streakBadge}
              </div>
              <div class="flex items-center gap-2">
                ${rankIndicator}
              </div>
            </div>
          </div>
          <div class="text-right">
            <span class="text-2xl font-black text-amber-300">${entry.score.toLocaleString()}</span>
            <span class="text-xs text-white/50 block">pts</span>
          </div>
        </div>
      `;
      })
      .join("");
  });

  ws.on("podium", (data) => {
    switchView(viewPodium);
    window.soundFX.fanfare();
    window.confetti.burst(150);
    setTimeout(() => window.confetti.burst(150), 800);

    const pillarsContainer = document.getElementById("podium-pillars");
    const top3 = data.podium || [];

    // Kahoot Podium layout: 2nd on left, 1st in center, 3rd on right
    const p1 = top3.find((p) => p.rank === 1);
    const p2 = top3.find((p) => p.rank === 2);
    const p3 = top3.find((p) => p.rank === 3);

    pillarsContainer.innerHTML = `
      <!-- 2nd Place -->
      <div class="podium-step">
        ${
          p2
            ? `
          <div class="font-black text-sm text-center mb-2 truncate max-w-[120px] text-slate-200">
            ${escapeHtml(p2.nickname)}
            <div class="text-xs text-white/70">${p2.score.toLocaleString()} pts</div>
          </div>
          <div class="podium-pillar pillar-2">🥈 2</div>
        `
            : `<div class="podium-pillar pillar-2 opacity-30">2</div>`
        }
      </div>

      <!-- 1st Place (Center / Highest) -->
      <div class="podium-step">
        ${
          p1
            ? `
          <div class="font-black text-base text-center mb-2 truncate max-w-[140px] text-amber-300">
            👑 ${escapeHtml(p1.nickname)}
            <div class="text-xs text-white/80">${p1.score.toLocaleString()} pts</div>
          </div>
          <div class="podium-pillar pillar-1">🥇 1</div>
        `
            : `<div class="podium-pillar pillar-1 opacity-30">1</div>`
        }
      </div>

      <!-- 3rd Place -->
      <div class="podium-step">
        ${
          p3
            ? `
          <div class="font-black text-sm text-center mb-2 truncate max-w-[120px] text-amber-600">
            ${escapeHtml(p3.nickname)}
            <div class="text-xs text-white/70">${p3.score.toLocaleString()} pts</div>
          </div>
          <div class="podium-pillar pillar-3">🥉 3</div>
        `
            : `<div class="podium-pillar pillar-3 opacity-30">3</div>`
        }
      </div>
    `;

    // Full results table
    const fullList = document.getElementById("podium-full-list");
    fullList.innerHTML = (data.full_results || [])
      .map(
        (r) => `
        <div class="p-2.5 rounded-lg bg-white/5 flex items-center justify-between text-sm">
          <div class="flex items-center gap-3">
            <span class="font-black w-6 text-white/60">#${r.rank}</span>
            <span class="font-bold text-white">${escapeHtml(r.nickname)}</span>
          </div>
          <span class="font-black text-amber-400">${r.score.toLocaleString()} pts</span>
        </div>
      `
      )
      .join("");
  });

  // Action buttons
  document.getElementById("btn-start-game").addEventListener("click", () => {
    window.soundFX.click();
    ws.send("host_start_game");
  });

  document.getElementById("btn-skip-question").addEventListener("click", () => {
    window.soundFX.click();
    ws.send("host_next_question");
  });

  document.getElementById("btn-next-to-leaderboard").addEventListener("click", () => {
    window.soundFX.click();
    ws.send("host_next_question");
  });

  document.getElementById("btn-leaderboard-next").addEventListener("click", () => {
    window.soundFX.click();
    ws.send("host_next_question");
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
