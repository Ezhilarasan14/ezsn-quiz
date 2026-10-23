// Quiz Creator and Editor Controller
document.addEventListener("DOMContentLoaded", () => {
  const viewLibrary = document.getElementById("view-library");
  const viewEditor = document.getElementById("view-editor");
  const libraryContainer = document.getElementById("library-container");

  const btnNewQuiz = document.getElementById("btn-new-quiz");
  const btnBackToLibrary = document.getElementById("btn-back-to-library");
  const btnSaveQuiz = document.getElementById("btn-save-quiz");
  const btnSaveQuizBottom = document.getElementById("btn-save-quiz-bottom");
  const btnAddQuestion = document.getElementById("btn-add-question");
  const btnPreviewQuiz = document.getElementById("btn-preview-quiz");
  const btnClosePreview = document.getElementById("btn-close-preview");
  const btnModalClose = document.getElementById("btn-modal-close");
  const previewModal = document.getElementById("preview-modal");
  const previewBody = document.getElementById("preview-body");

  const quizTitleInput = document.getElementById("quiz-title-input");
  const quizDescInput = document.getElementById("quiz-desc-input");
  const questionsContainer = document.getElementById("questions-container");
  const questionCountBadge = document.getElementById("question-count-badge");
  const editorPageTitle = document.getElementById("editor-page-title");

  let currentEditingQuizId = null;
  let questions = [];

  // Load quizzes library
  async function loadLibrary() {
    try {
      const list = await window.api.get("/api/quizzes");
      if (!list || list.length === 0) {
        libraryContainer.innerHTML = `
          <div class="col-span-full text-center py-12 glass-panel p-8">
            <p class="text-white/60 mb-4">No quizzes available yet.</p>
            <button id="btn-create-first" class="btn-primary py-2 px-6 font-bold text-sm">Create Your First Quiz</button>
          </div>
        `;
        document.getElementById("btn-create-first")?.addEventListener("click", () => openEditor());
        return;
      }

      libraryContainer.innerHTML = list
        .map(
          (q) => `
        <div class="glass-panel p-5 flex flex-col justify-between gap-4">
          <div>
            <div class="flex items-start justify-between gap-2 mb-2">
              <h3 class="font-extrabold text-white text-lg leading-snug">${escapeHtml(q.title)}</h3>
              <span class="px-2.5 py-1 rounded-full bg-amber-500/20 text-amber-300 font-black text-xs shrink-0">
                ${q.question_count} Qs
              </span>
            </div>
            <p class="text-white/60 text-xs line-clamp-2">${escapeHtml(q.description || "No description")}</p>
          </div>

          <div class="pt-3 border-t border-white/10 flex items-center justify-between gap-2 flex-wrap">
            <div class="flex items-center gap-2">
              <button data-action="edit" data-id="${q.id}" class="text-xs font-bold text-white/80 hover:text-white px-2.5 py-1 rounded bg-white/10 hover:bg-white/20 transition">
                ✏️ Edit
              </button>
              <button data-action="duplicate" data-id="${q.id}" class="text-xs font-bold text-white/80 hover:text-white px-2.5 py-1 rounded bg-white/10 hover:bg-white/20 transition">
                📋 Copy
              </button>
              <button data-action="delete" data-id="${q.id}" class="text-xs font-bold text-red-400 hover:text-red-300 px-2.5 py-1 rounded bg-red-500/10 hover:bg-red-500/20 transition">
                🗑️ Delete
              </button>
            </div>
            <button data-action="host" data-id="${q.id}" class="btn-primary text-xs py-1.5 px-3.5 font-black">
              Host &rarr;
            </button>
          </div>
        </div>
      `
        )
        .join("");

      bindLibraryActions();
    } catch (err) {
      libraryContainer.innerHTML = `
        <div class="col-span-full text-center py-12 text-red-400">
          Failed to load quizzes: ${err.message}
        </div>
      `;
    }
  }

  function bindLibraryActions() {
    libraryContainer.querySelectorAll("[data-action]").forEach((btn) => {
      btn.addEventListener("click", async () => {
        const action = btn.getAttribute("data-action");
        const id = btn.getAttribute("data-id");

        if (action === "edit") {
          loadQuizForEditing(id);
        } else if (action === "duplicate") {
          btn.disabled = true;
          try {
            await window.api.post(`/api/quizzes/${id}/duplicate`, {});
            showToast("Quiz duplicated!");
            loadLibrary();
          } catch (err) {
            showToast(err.message, true);
            btn.disabled = false;
          }
        } else if (action === "delete") {
          if (confirm("Are you sure you want to delete this quiz? This action cannot be undone.")) {
            try {
              await window.api.delete(`/api/quizzes/${id}`);
              showToast("Quiz deleted");
              loadLibrary();
            } catch (err) {
              showToast(err.message, true);
            }
          }
        } else if (action === "host") {
          try {
            const res = await window.api.post("/api/rooms", { quiz_id: id });
            sessionStorage.setItem("ezsn_quiz_host_token", res.host_token);
            sessionStorage.setItem("ezsn_quiz_host_pin", res.pin);
            sessionStorage.setItem("ezsn_quiz_game_id", res.game_id);
            window.location.href = `/host?pin=${res.pin}`;
          } catch (err) {
            showToast(err.message, true);
          }
        }
      });
    });
  }

  async function loadQuizForEditing(id) {
    try {
      const quiz = await window.api.get(`/api/quizzes/${id}`);
      currentEditingQuizId = quiz.id;
      editorPageTitle.textContent = "Edit Quiz";
      quizTitleInput.value = quiz.title;
      quizDescInput.value = quiz.description || "";

      questions = quiz.questions.map((q) => ({
        question_text: q.question_text,
        time_limit_sec: q.time_limit_sec,
        points: q.points,
        options: q.options.map((o) => ({
          option_text: o.option_text,
          is_correct: o.is_correct,
        })),
      }));

      renderQuestionsList();
      viewLibrary.classList.add("hidden");
      viewEditor.classList.remove("hidden");
    } catch (err) {
      showToast("Failed to load quiz: " + err.message, true);
    }
  }

  function openEditor() {
    currentEditingQuizId = null;
    editorPageTitle.textContent = "Create New Quiz";
    quizTitleInput.value = "";
    quizDescInput.value = "";

    // Default with 1 starter question
    questions = [
      {
        question_text: "",
        time_limit_sec: 20,
        points: 1000,
        options: [
          { option_text: "", is_correct: true },
          { option_text: "", is_correct: false },
          { option_text: "", is_correct: false },
          { option_text: "", is_correct: false },
        ],
      },
    ];

    renderQuestionsList();
    viewLibrary.classList.add("hidden");
    viewEditor.classList.remove("hidden");
  }

  function renderQuestionsList() {
    questionCountBadge.textContent = questions.length;

    const shapes = ["▲ Triangle", "◆ Diamond", "● Circle", "■ Square"];
    const colorBorders = ["border-red-500", "border-blue-500", "border-amber-500", "border-emerald-500"];

    questionsContainer.innerHTML = questions
      .map(
        (q, qIdx) => `
      <div class="glass-panel p-6 space-y-4 relative border-l-4 border-amber-400" data-q-idx="${qIdx}">
        <div class="flex items-center justify-between gap-4">
          <span class="font-extrabold text-amber-400 text-sm">Question ${qIdx + 1}</span>
          <div class="flex items-center gap-3">
            <div class="flex items-center gap-2">
              <label class="text-xs text-white/60 font-bold">Timer:</label>
              <select class="q-timer glass-input text-xs py-1 px-2.5 w-auto" data-q-idx="${qIdx}">
                <option value="5" ${q.time_limit_sec === 5 ? "selected" : ""}>5 sec</option>
                <option value="10" ${q.time_limit_sec === 10 ? "selected" : ""}>10 sec</option>
                <option value="15" ${q.time_limit_sec === 15 ? "selected" : ""}>15 sec</option>
                <option value="20" ${q.time_limit_sec === 20 ? "selected" : ""}>20 sec</option>
                <option value="30" ${q.time_limit_sec === 30 ? "selected" : ""}>30 sec</option>
                <option value="60" ${q.time_limit_sec === 60 ? "selected" : ""}>60 sec</option>
              </select>
            </div>

            <div class="flex items-center gap-2">
              <label class="text-xs text-white/60 font-bold">Points:</label>
              <select class="q-points glass-input text-xs py-1 px-2.5 w-auto" data-q-idx="${qIdx}">
                <option value="500" ${q.points === 500 ? "selected" : ""}>500 pts</option>
                <option value="1000" ${q.points === 1000 ? "selected" : ""}>1000 pts</option>
                <option value="2000" ${q.points === 2000 ? "selected" : ""}>2000 pts</option>
              </select>
            </div>

            ${
              questions.length > 1
                ? `
              <button class="btn-remove-q text-red-400 hover:text-red-300 font-bold text-xs p-1" data-q-idx="${qIdx}" title="Delete Question">
                ✕ Delete
              </button>
            `
                : ""
            }
          </div>
        </div>

        <div>
          <input 
            type="text" 
            class="q-text-input glass-input text-base font-bold w-full" 
            data-q-idx="${qIdx}" 
            placeholder="Type your question here..." 
            value="${escapeHtml(q.question_text)}"
          >
        </div>

        <!-- 4 Options Grid -->
        <div class="grid grid-cols-1 md:grid-cols-2 gap-3 pt-2">
          ${q.options
            .map(
              (opt, optIdx) => `
            <div class="p-3 rounded-xl bg-white/5 border ${colorBorders[optIdx % 4]} flex items-center gap-3">
              <input 
                type="radio" 
                name="correct-q-${qIdx}" 
                class="opt-correct-radio w-5 h-5 accent-emerald-500 cursor-pointer" 
                data-q-idx="${qIdx}" 
                data-opt-idx="${optIdx}" 
                ${opt.is_correct ? "checked" : ""}
                title="Mark as correct answer"
              >
              <div class="flex-1">
                <span class="text-[10px] font-bold uppercase tracking-wider text-white/50 block">${shapes[optIdx % 4]}</span>
                <input 
                  type="text" 
                  class="opt-text-input w-full bg-transparent text-white font-bold outline-none text-sm placeholder-white/40" 
                  data-q-idx="${qIdx}" 
                  data-opt-idx="${optIdx}" 
                  placeholder="Answer option ${optIdx + 1}" 
                  value="${escapeHtml(opt.option_text)}"
                >
              </div>
            </div>
          `
            )
            .join("")}
        </div>
      </div>
    `
      )
      .join("");

    bindEditorInputs();
  }

  function bindEditorInputs() {
    // Question text changes
    document.querySelectorAll(".q-text-input").forEach((inp) => {
      inp.addEventListener("input", (e) => {
        const qIdx = parseInt(e.target.getAttribute("data-q-idx"));
        questions[qIdx].question_text = e.target.value;
      });
    });

    // Option text changes
    document.querySelectorAll(".opt-text-input").forEach((inp) => {
      inp.addEventListener("input", (e) => {
        const qIdx = parseInt(e.target.getAttribute("data-q-idx"));
        const optIdx = parseInt(e.target.getAttribute("data-opt-idx"));
        questions[qIdx].options[optIdx].option_text = e.target.value;
      });
    });

    // Correct radio toggle
    document.querySelectorAll(".opt-correct-radio").forEach((radio) => {
      radio.addEventListener("change", (e) => {
        const qIdx = parseInt(e.target.getAttribute("data-q-idx"));
        const optIdx = parseInt(e.target.getAttribute("data-opt-idx"));
        questions[qIdx].options.forEach((o, i) => {
          o.is_correct = i === optIdx;
        });
      });
    });

    // Timer change
    document.querySelectorAll(".q-timer").forEach((sel) => {
      sel.addEventListener("change", (e) => {
        const qIdx = parseInt(e.target.getAttribute("data-q-idx"));
        questions[qIdx].time_limit_sec = parseInt(e.target.value);
      });
    });

    // Points change
    document.querySelectorAll(".q-points").forEach((sel) => {
      sel.addEventListener("change", (e) => {
        const qIdx = parseInt(e.target.getAttribute("data-q-idx"));
        questions[qIdx].points = parseInt(e.target.value);
      });
    });

    // Remove question
    document.querySelectorAll(".btn-remove-q").forEach((btn) => {
      btn.addEventListener("click", (e) => {
        const qIdx = parseInt(e.target.getAttribute("data-q-idx"));
        questions.splice(qIdx, 1);
        renderQuestionsList();
      });
    });
  }

  // Add new question button
  btnAddQuestion.addEventListener("click", () => {
    questions.push({
      question_text: "",
      time_limit_sec: 20,
      points: 1000,
      options: [
        { option_text: "", is_correct: true },
        { option_text: "", is_correct: false },
        { option_text: "", is_correct: false },
        { option_text: "", is_correct: false },
      ],
    });
    renderQuestionsList();
    // Scroll to new question
    setTimeout(() => {
      window.scrollTo({ top: document.body.scrollHeight, behavior: "smooth" });
    }, 50);
  });

  // Save Quiz
  async function saveQuiz() {
    const title = quizTitleInput.value.trim();
    const description = quizDescInput.value.trim();

    if (!title) {
      showToast("Please enter a quiz title", true);
      quizTitleInput.focus();
      return;
    }

    if (questions.length === 0) {
      showToast("Quiz must have at least one question", true);
      return;
    }

    for (let i = 0; i < questions.length; i++) {
      const q = questions[i];
      if (!q.question_text.trim()) {
        showToast(`Question ${i + 1} text cannot be empty`, true);
        return;
      }

      const validOptions = q.options.filter((o) => o.option_text.trim() !== "");
      if (validOptions.length < 2) {
        showToast(`Question ${i + 1} must have at least 2 filled options`, true);
        return;
      }

      const hasCorrect = q.options.some((o) => o.is_correct && o.option_text.trim() !== "");
      if (!hasCorrect) {
        showToast(`Question ${i + 1} must have a correct option selected with text`, true);
        return;
      }
    }

    const payload = {
      title,
      description: description || null,
      questions: questions.map((q) => ({
        question_text: q.question_text.trim(),
        time_limit_sec: q.time_limit_sec,
        points: q.points,
        options: q.options
          .filter((o) => o.option_text.trim() !== "")
          .map((o) => ({
            option_text: o.option_text.trim(),
            is_correct: o.is_correct,
          })),
      })),
    };

    try {
      if (currentEditingQuizId) {
        await window.api.put(`/api/quizzes/${currentEditingQuizId}`, payload);
        showToast("Quiz updated successfully!");
      } else {
        await window.api.post("/api/quizzes", payload);
        showToast("Quiz created successfully!");
      }

      viewEditor.classList.add("hidden");
      viewLibrary.classList.remove("hidden");
      loadLibrary();
    } catch (err) {
      showToast(err.message, true);
    }
  }

  btnSaveQuiz.addEventListener("click", saveQuiz);
  btnSaveQuizBottom.addEventListener("click", saveQuiz);

  // Preview Modal
  btnPreviewQuiz.addEventListener("click", () => {
    previewBody.innerHTML = `
      <div class="mb-4">
        <h4 class="text-2xl font-black text-white">${escapeHtml(quizTitleInput.value || "Untitled Quiz")}</h4>
        <p class="text-white/60 text-sm">${escapeHtml(quizDescInput.value || "No description")}</p>
      </div>
      <div class="space-y-4">
        ${questions
          .map(
            (q, idx) => `
          <div class="p-4 rounded-xl bg-white/10 border border-white/10">
            <div class="flex items-center justify-between mb-2">
              <span class="font-bold text-amber-400 text-sm">Q${idx + 1}: ${escapeHtml(q.question_text || "Untitled Question")}</span>
              <span class="text-xs text-white/50">${q.time_limit_sec}s &bull; ${q.points}pts</span>
            </div>
            <div class="grid grid-cols-2 gap-2 text-xs">
              ${q.options
                .map(
                  (o) => `
                <div class="p-2 rounded bg-white/5 border ${o.is_correct ? "border-emerald-400 font-bold text-emerald-300" : "border-white/10 text-white/80"}">
                  ${o.is_correct ? "✓ " : ""}${escapeHtml(o.option_text || "(empty)")}
                </div>
              `
                )
                .join("")}
            </div>
          </div>
        `
          )
          .join("")}
      </div>
    `;
    previewModal.classList.remove("hidden");
  });

  function closePreview() {
    previewModal.classList.add("hidden");
  }
  btnClosePreview.addEventListener("click", closePreview);
  btnModalClose.addEventListener("click", closePreview);

  btnNewQuiz.addEventListener("click", openEditor);
  btnBackToLibrary.addEventListener("click", () => {
    viewEditor.classList.add("hidden");
    viewLibrary.classList.remove("hidden");
    loadLibrary();
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

  loadLibrary();
});
