// Game History Controller
document.addEventListener("DOMContentLoaded", () => {
  const historyContainer = document.getElementById("history-container");
  const resultsModal = document.getElementById("results-modal");
  const modalQuizTitle = document.getElementById("modal-quiz-title");
  const modalGameMeta = document.getElementById("modal-game-meta");
  const modalResultsTbody = document.getElementById("modal-results-tbody");
  const btnCloseModal = document.getElementById("btn-close-modal");
  const btnModalDismiss = document.getElementById("btn-modal-dismiss");

  async function loadHistory() {
    try {
      const history = await window.api.get("/api/history");
      if (!history || history.length === 0) {
        historyContainer.innerHTML = `
          <div class="glass-panel p-12 text-center text-white/60">
            <span class="text-4xl block mb-2">📜</span>
            <p class="font-bold text-lg text-white mb-1">No completed games yet</p>
            <p class="text-sm">Host a quiz and complete all questions to see full rankings and winner records here!</p>
          </div>
        `;
        return;
      }

      historyContainer.innerHTML = history
        .map((g) => {
          const dateStr = new Date(g.created_at).toLocaleString();
          const winnerText = g.winner_nickname
            ? `👑 Winner: <span class="font-bold text-amber-300">${escapeHtml(g.winner_nickname)}</span> (${(g.winner_score || 0).toLocaleString()} pts)`
            : "No winner recorded";

          return `
          <div class="glass-panel p-5 flex items-center justify-between gap-4 flex-wrap">
            <div>
              <div class="flex items-center gap-3 mb-1">
                <h3 class="font-extrabold text-white text-lg">${escapeHtml(g.quiz_title)}</h3>
                <span class="px-2 py-0.5 rounded bg-white/10 text-xs font-mono text-amber-400 font-bold">PIN: ${g.room_pin}</span>
              </div>
              <div class="flex items-center gap-3 text-xs text-white/60">
                <span>📅 ${dateStr}</span>
                <span>&bull;</span>
                <span>👥 ${g.total_players} players</span>
              </div>
              <div class="text-xs text-white/80 mt-2">
                ${winnerText}
              </div>
            </div>

            <div>
              <button 
                data-game-id="${g.id}" 
                data-title="${escapeHtml(g.quiz_title)}"
                data-pin="${g.room_pin}"
                data-date="${dateStr}"
                class="btn-view-results btn-secondary text-xs py-2 px-4 font-bold"
              >
                View Standings &rarr;
              </button>
            </div>
          </div>
        `;
        })
        .join("");

      bindViewResults();
    } catch (err) {
      historyContainer.innerHTML = `
        <div class="text-center py-12 text-red-400">
          Failed to load history: ${err.message}
        </div>
      `;
    }
  }

  function bindViewResults() {
    document.querySelectorAll(".btn-view-results").forEach((btn) => {
      btn.addEventListener("click", async () => {
        const gameId = btn.getAttribute("data-game-id");
        const title = btn.getAttribute("data-title");
        const pin = btn.getAttribute("data-pin");
        const date = btn.getAttribute("data-date");

        modalQuizTitle.textContent = title;
        modalGameMeta.textContent = `PIN: ${pin} • Played on ${date}`;
        modalResultsTbody.innerHTML = `<tr><td colspan="4" class="text-center py-6 text-white/50">Loading results...</td></tr>`;
        resultsModal.classList.remove("hidden");

        try {
          const results = await window.api.get(`/api/history/${gameId}`);
          if (!results || results.length === 0) {
            modalResultsTbody.innerHTML = `<tr><td colspan="4" class="text-center py-6 text-white/50">No player results found for this game.</td></tr>`;
            return;
          }

          modalResultsTbody.innerHTML = results
            .map((r) => {
              let badge = `#${r.final_rank}`;
              if (r.final_rank === 1) badge = "🥇 1st";
              else if (r.final_rank === 2) badge = "🥈 2nd";
              else if (r.final_rank === 3) badge = "🥉 3rd";

              return `
              <tr class="hover:bg-white/5 transition">
                <td class="py-3 px-3 font-black text-amber-400">${badge}</td>
                <td class="py-3 px-3 font-bold text-white">${escapeHtml(r.nickname)}</td>
                <td class="py-3 px-3 font-black text-white">${r.total_score.toLocaleString()} pts</td>
                <td class="py-3 px-3 text-white/80">${r.correct_count} / ${r.total_answered}</td>
              </tr>
            `;
            })
            .join("");
        } catch (err) {
          modalResultsTbody.innerHTML = `<tr><td colspan="4" class="text-center py-6 text-red-400">${err.message}</td></tr>`;
        }
      });
    });
  }

  function closeModal() {
    resultsModal.classList.add("hidden");
  }

  btnCloseModal.addEventListener("click", closeModal);
  btnModalDismiss.addEventListener("click", closeModal);

  function escapeHtml(str) {
    if (!str) return "";
    return str
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;")
      .replace(/'/g, "&#039;");
  }

  loadHistory();
});
