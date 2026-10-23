// Player Screen Controller
document.addEventListener("DOMContentLoaded", () => {
  const urlParams = new URLSearchParams(window.location.search);
  const pin = urlParams.get("pin") || sessionStorage.getItem("ezsn_quiz_player_pin");
  const nickname = sessionStorage.getItem("ezsn_quiz_player_nickname");
  const storedPlayerId = sessionStorage.getItem("ezsn_quiz_player_id");

  if (!pin || !nickname) {
    alert("Missing PIN or nickname. Redirecting to home.");
    window.location.href = "/";
    return;
  }

  // Views
  const viewLobby = document.getElementById("player-view-lobby");
  const viewIntro = document.getElementById("player-view-intro");
  const viewQuestion = document.getElementById("player-view-question");
  const viewResult = document.getElementById("player-view-result");
  const viewLeaderboard = document.getElementById("player-view-leaderboard");
  const viewPodium = document.getElementById("player-view-podium");

  const answersGrid = document.getElementById("player-answers-grid");
  const submittedOverlay = document.getElementById("player-submitted-overlay");

  // Header Elements
  document.getElementById("player-pin-label").textContent = pin;
  document.getElementById("player-nickname-label").textContent = nickname;
  const scoreBadge = document.getElementById("player-score-badge");

  function switchView(activeView) {
    [viewLobby, viewIntro, viewQuestion, viewResult, viewLeaderboard, viewPodium].forEach((v) => {
      v.classList.add("hidden");
    });
    activeView.classList.remove("hidden");
  }

  let currentQuestionOptions = [];
  let currentQuestionId = null;
  let hasAnsweredCurrentQuestion = false;

  const ws = new QuizWebSocket();
  ws.connect(() => {
    // Connect as player (supports reconnection with storedPlayerId)
    ws.send("player_connect", {
      pin,
      nickname,
      player_id: storedPlayerId || null,
    });
  });

  ws.on("connected", (data) => {
    if (data.player_id) {
      sessionStorage.setItem("ezsn_quiz_player_id", data.player_id);
    }
    console.log("Player joined successfully:", data);
  });

  ws.on("error", (data) => {
    showToast(data.message, true);
    setTimeout(() => {
      window.location.href = "/";
    }, 2500);
  });

  ws.on("player_kicked", (data) => {
    alert(data.reason || "You were removed by the host.");
    sessionStorage.clear();
    window.location.href = "/";
  });

  ws.on("lobby_state", () => {
    switchView(viewLobby);
  });

  ws.on("game_starting", () => {
    window.soundFX.click();
    showToast("Game starting!");
  });

  ws.on("question_intro", (data) => {
    hasAnsweredCurrentQuestion = false;
    switchView(viewIntro);
    document.getElementById("player-intro-q-num").textContent = `Question ${data.question_index}`;

    let count = data.countdown_secs;
    const timerEl = document.getElementById("player-intro-timer");
    timerEl.textContent = count;

    const interval = setInterval(() => {
      count--;
      if (count >= 1) {
        timerEl.textContent = count;
        window.soundFX.tick();
      } else {
        clearInterval(interval);
      }
    }, 1000);
  });

  ws.on("question_active", (data) => {
    switchView(viewQuestion);
    currentQuestionOptions = data.options;
    currentQuestionId = data.options && data.options[0] ? data.options[0].id : null; // or pass question_id
    hasAnsweredCurrentQuestion = data.already_answered;

    if (hasAnsweredCurrentQuestion) {
      answersGrid.classList.add("hidden");
      submittedOverlay.classList.remove("hidden");
    } else {
      submittedOverlay.classList.add("hidden");
      answersGrid.classList.remove("hidden");
    }

    // Render / map options to buttons
    const buttons = answersGrid.querySelectorAll("button");
    buttons.forEach((btn, idx) => {
      btn.disabled = hasAnsweredCurrentQuestion;
      if (data.options && data.options[idx]) {
        btn.classList.remove("hidden");
        btn.setAttribute("data-option-id", data.options[idx].id);
        const textSpan = btn.querySelector(".answer-btn-text");
        if (textSpan) textSpan.textContent = data.options[idx].text;
      } else {
        btn.classList.add("hidden");
      }
    });
  });

  // Handle answer button taps
  answersGrid.querySelectorAll("button").forEach((btn) => {
    btn.addEventListener("click", () => {
      if (hasAnsweredCurrentQuestion) return;
      hasAnsweredCurrentQuestion = true;

      const optionId = btn.getAttribute("data-option-id");
      window.soundFX.click();
      if (navigator.vibrate) navigator.vibrate([60]);

      // Show immediate waiting screen
      answersGrid.classList.add("hidden");
      submittedOverlay.classList.remove("hidden");

      // Submit via WebSocket
      // For question_id, backend validates against the active question
      ws.send("player_submit_answer", {
        question_id: "", // Server matches active question
        option_id: optionId,
      });
    });
  });

  ws.on("answer_accepted", () => {
    showToast("Answer received!");
  });

  ws.on("answer_rejected", (data) => {
    showToast(data.reason || "Answer rejected", true);
  });

  ws.on("question_result", (data) => {
    switchView(viewResult);

    const isCorrect = data.player_correct === true;
    const points = data.points_awarded || 0;
    const streak = data.streak || 0;

    if (data.total_score !== undefined && data.total_score !== null) {
      scoreBadge.textContent = `${data.total_score.toLocaleString()} pts`;
    }

    const card = document.getElementById("result-card-bg");
    const icon = document.getElementById("result-icon");
    const title = document.getElementById("result-title");
    const pointsBox = document.getElementById("result-points-box");
    const streakBox = document.getElementById("result-streak-box");

    if (isCorrect) {
      window.soundFX.correct();
      if (navigator.vibrate) navigator.vibrate([100, 50, 100]);
      card.className = "glass-panel p-8 w-full border-2 border-emerald-400 bg-emerald-900/30";
      icon.textContent = "🎉";
      title.textContent = "Correct!";
      title.className = "text-3xl font-black text-emerald-300 mb-2";
      pointsBox.textContent = `+${points.toLocaleString()} pts`;
      pointsBox.classList.remove("hidden");
      streakBox.textContent = streak > 1 ? `🔥 Answer Streak: ${streak}` : "Streak started!";
      streakBox.classList.remove("hidden");
    } else {
      window.soundFX.wrong();
      if (navigator.vibrate) navigator.vibrate([200]);
      card.className = "glass-panel p-8 w-full border-2 border-red-500 bg-red-900/30";
      icon.textContent = "❌";
      title.textContent = "Incorrect";
      title.className = "text-3xl font-black text-red-300 mb-2";
      pointsBox.textContent = "+0 pts";
      streakBox.textContent = "Streak lost";
    }
  });

  ws.on("leaderboard_update", (data) => {
    switchView(viewLeaderboard);
    if (data.my_score !== undefined && data.my_score !== null) {
      scoreBadge.textContent = `${data.my_score.toLocaleString()} pts`;
      document.getElementById("player-score-display").textContent = `Score: ${data.my_score.toLocaleString()} pts`;
    }

    const rankDisplay = document.getElementById("player-rank-display");
    if (data.my_rank) {
      let suffix = "th";
      if (data.my_rank === 1) suffix = "st 🥇";
      else if (data.my_rank === 2) suffix = "nd 🥈";
      else if (data.my_rank === 3) suffix = "rd 🥉";
      rankDisplay.textContent = `#${data.my_rank} ${suffix}`;
    } else {
      rankDisplay.textContent = "--";
    }
  });

  ws.on("podium", (data) => {
    switchView(viewPodium);

    const rankEl = document.getElementById("player-final-rank");
    const iconEl = document.getElementById("player-podium-icon");
    const scoreEl = document.getElementById("player-final-score");

    const myEntry = data.full_results && data.full_results.find((r) => r.nickname === nickname);
    if (myEntry) {
      scoreEl.textContent = `Final Score: ${myEntry.score.toLocaleString()} pts`;
    }

    if (data.my_rank === 1) {
      window.soundFX.fanfare();
      window.confetti.burst(150);
      iconEl.textContent = "🥇";
      rankEl.textContent = "1st Place Champion!";
      rankEl.className = "text-4xl font-black text-amber-300 my-3";
    } else if (data.my_rank === 2) {
      window.soundFX.fanfare();
      window.confetti.burst(80);
      iconEl.textContent = "🥈";
      rankEl.textContent = "2nd Place!";
      rankEl.className = "text-4xl font-black text-slate-200 my-3";
    } else if (data.my_rank === 3) {
      window.soundFX.fanfare();
      window.confetti.burst(60);
      iconEl.textContent = "🥉";
      rankEl.textContent = "3rd Place!";
      rankEl.className = "text-4xl font-black text-amber-600 my-3";
    } else if (data.my_rank) {
      iconEl.textContent = "👏";
      rankEl.textContent = `${data.my_rank}th Place`;
      rankEl.className = "text-4xl font-black text-white/90 my-3";
    } else {
      rankEl.textContent = "Game Complete";
    }
  });
});
