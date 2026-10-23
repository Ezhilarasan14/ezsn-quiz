import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { wsClient } from '@/services/websocket';
import { soundService } from '@/services/audio';
import { useBrandingStore } from './branding';
import type { GameState, Player, QuestionStats, PodiumEntry } from '@/types/game';
import type { OptionView, LeaderboardEntry, ServerMessage } from '@/types/ws';

export const useGameStore = defineStore('game', () => {
  const brandingStore = useBrandingStore();

  // Connection & Room state
  const isConnected = ref(false);
  const role = ref<'host' | 'player' | null>(null);
  const roomPin = ref<string>('');
  const quizTitle = ref<string>('');
  const hostToken = ref<string>('');

  // Player state
  const playerId = ref<string>('');
  const nickname = ref<string>('');
  const playerEmail = ref<string>(localStorage.getItem('ezsn_player_email') || '');
  const score = ref<number>(0);
  const streak = ref<number>(0);
  const rank = ref<number>(0);

  // Active Game State
  const gameState = ref<GameState>('LOBBY');
  const players = ref<Player[]>([]);
  const playerCount = computed(() => players.value.length);

  // Question State
  const questionIndex = ref<number>(0);
  const totalQuestions = ref<number>(0);
  const questionText = ref<string>('');
  const timeLimitSecs = ref<number>(20);
  const timeLeftSecs = ref<number>(20);
  const options = ref<OptionView[]>([]);
  const selectedOptionId = ref<string | null>(null);
  const answerSubmitted = ref<boolean>(false);
  const answerAccepted = ref<boolean>(false);

  // Host Realtime Metrics
  const answeredCount = ref<number>(0);

  // Reveal / Result state
  const correctOptionIds = ref<string[]>([]);
  const optionCounts = ref<Record<string, number>>({});
  const lastPointsAwarded = ref<number>(0);
  const lastAnswerCorrect = ref<boolean | null>(null);

  // Final Results (Scoreboard shown only at end)
  const podium = ref<PodiumEntry[]>([]);
  const fullResults = ref<LeaderboardEntry[]>([]);

  // Timer reference
  let timerInterval: number | null = null;

  // Initialize WebSocket handlers
  let wasConnected = false;
  function initWebSocket() {
    wsClient.onStatus((connected) => {
      isConnected.value = connected;
      if (connected && wasConnected) {
        // Automatic re-handshake on reconnection
        if (role.value === 'player' && roomPin.value && nickname.value) {
          wsClient.send({
            type: 'player_connect',
            payload: {
              pin: roomPin.value,
              nickname: nickname.value,
              player_id: playerId.value || undefined,
              email: playerEmail.value || undefined,
            },
          });
        } else if (role.value === 'host' && roomPin.value && hostToken.value) {
          wsClient.send({
            type: 'host_connect',
            payload: { pin: roomPin.value, token: hostToken.value },
          });
        }
      }
      if (connected) {
        wasConnected = true;
      }
    });

    wsClient.onMessage((msg: ServerMessage) => {
      handleServerMessage(msg);
    });

    wsClient.connect();
  }

  function handleServerMessage(msg: ServerMessage) {
    switch (msg.type) {
      case 'connected':
        role.value = msg.payload.role;
        roomPin.value = msg.payload.pin;
        if (msg.payload.player_id) playerId.value = msg.payload.player_id;
        if (msg.payload.nickname) nickname.value = msg.payload.nickname;
        break;

      case 'theme_config':
        brandingStore.applyTheme(msg.payload.theme);
        break;

      case 'lobby_state':
        gameState.value = 'LOBBY';
        roomPin.value = msg.payload.pin;
        quizTitle.value = msg.payload.quiz_title;
        players.value = msg.payload.players.map((p) => ({
          id: p.player_id,
          nickname: p.nickname,
          score: 0,
          streak: 0,
          connected: p.is_connected,
        }));
        break;

      case 'player_joined':
        if (!players.value.some((p) => p.id === msg.payload.player_id)) {
          players.value.push({
            id: msg.payload.player_id,
            nickname: msg.payload.nickname,
            score: 0,
            streak: 0,
            connected: true,
          });
        }
        break;

      case 'player_left':
        players.value = players.value.filter((p) => p.id !== msg.payload.player_id);
        break;

      case 'game_starting':
        gameState.value = 'LOBBY';
        break;

      case 'question_intro':
        gameState.value = 'QUESTION';
        questionIndex.value = msg.payload.question_index;
        totalQuestions.value = msg.payload.total_questions;
        questionText.value = msg.payload.question_text;
        selectedOptionId.value = null;
        answerSubmitted.value = false;
        answerAccepted.value = false;
        correctOptionIds.value = [];
        answeredCount.value = 0;
        break;

      case 'question_active':
        gameState.value = 'QUESTION';
        questionIndex.value = msg.payload.question_index;
        totalQuestions.value = msg.payload.total_questions;
        questionText.value = msg.payload.question_text;
        timeLimitSecs.value = msg.payload.time_limit_sec;
        options.value = msg.payload.options;
        startCountdown(msg.payload.time_limit_sec);
        break;

      case 'answer_accepted':
        answerAccepted.value = true;
        break;

      case 'host_answer_count_update':
        answeredCount.value = msg.payload.answered_count;
        break;

      case 'question_result':
        stopCountdown();
        gameState.value = 'REVEAL';
        correctOptionIds.value = msg.payload.correct_option_ids;
        optionCounts.value = msg.payload.option_counts;

        if (msg.payload.player_correct !== undefined) {
          lastAnswerCorrect.value = msg.payload.player_correct;
          lastPointsAwarded.value = msg.payload.points_awarded || 0;
          if (msg.payload.total_score !== undefined) score.value = msg.payload.total_score;
          if (msg.payload.streak !== undefined) streak.value = msg.payload.streak;

          if (msg.payload.player_correct) {
            soundService.playCorrect();
          } else {
            soundService.playWrong();
          }
        }
        break;

      case 'podium':
        stopCountdown();
        gameState.value = 'PODIUM';
        podium.value = msg.payload.podium;
        fullResults.value = msg.payload.full_results;
        if (msg.payload.my_rank) rank.value = msg.payload.my_rank;
        soundService.playFanfare();
        break;

      case 'player_kicked':
        disconnect();
        alert('You have been removed from the room.');
        break;
    }
  }

  function startCountdown(seconds: number) {
    stopCountdown();
    timeLeftSecs.value = seconds;
    timerInterval = window.setInterval(() => {
      if (timeLeftSecs.value > 0) {
        timeLeftSecs.value--;
        if (timeLeftSecs.value <= 5 && timeLeftSecs.value > 0) {
          soundService.playTick();
        }
      } else {
        stopCountdown();
      }
    }, 1000);
  }

  function stopCountdown() {
    if (timerInterval !== null) {
      clearInterval(timerInterval);
      timerInterval = null;
    }
  }

  // Client Actions
  function joinAsPlayer(pin: string, name: string, email?: string) {
    roomPin.value = pin;
    nickname.value = name;
    role.value = 'player';
    if (email) {
      playerEmail.value = email.trim();
      localStorage.setItem('ezsn_player_email', email.trim());
    }
    initWebSocket();
    wsClient.send({
      type: 'player_connect',
      payload: {
        pin,
        nickname: name,
        player_id: playerId.value || undefined,
        email: playerEmail.value || undefined,
      },
    });
  }

  function connectAsHost(pin: string, token: string) {
    roomPin.value = pin;
    hostToken.value = token;
    role.value = 'host';
    initWebSocket();
    wsClient.send({
      type: 'host_connect',
      payload: { pin, token },
    });
  }

  function submitAnswer(optionId: string) {
    if (answerSubmitted.value || gameState.value !== 'QUESTION') return;
    selectedOptionId.value = optionId;
    answerSubmitted.value = true;
    wsClient.send({
      type: 'player_submit_answer',
      payload: { question_id: `q_${questionIndex.value}`, option_id: optionId },
    });
  }

  function startGame() {
    wsClient.send({ type: 'host_start_game' });
  }

  function nextQuestion() {
    wsClient.send({ type: 'host_next_question' });
  }

  function kickPlayer(pId: string) {
    wsClient.send({
      type: 'host_kick_player',
      payload: { player_id: pId },
    });
  }

  function disconnect() {
    stopCountdown();
    wsClient.disconnect();
    gameState.value = 'LOBBY';
    selectedOptionId.value = null;
    answerSubmitted.value = false;
  }

  return {
    isConnected,
    role,
    roomPin,
    quizTitle,
    playerId,
    nickname,
    score,
    streak,
    rank,
    gameState,
    players,
    playerCount,
    questionIndex,
    totalQuestions,
    questionText,
    timeLimitSecs,
    timeLeftSecs,
    options,
    selectedOptionId,
    answerSubmitted,
    answerAccepted,
    answeredCount,
    correctOptionIds,
    optionCounts,
    lastPointsAwarded,
    lastAnswerCorrect,
    podium,
    fullResults,
    joinAsPlayer,
    connectAsHost,
    submitAnswer,
    startGame,
    nextQuestion,
    kickPlayer,
    disconnect,
  };
});
