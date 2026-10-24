<script setup lang="ts">
import { ref, computed } from 'vue';
import type { Question, Option } from '@/types/quiz';
import Dialog from 'primevue/dialog';
import Button from 'primevue/button';
import InputText from 'primevue/inputtext';
import Message from 'primevue/message';

const props = withDefaults(
  defineProps<{
    visible: boolean;
    mode?: 'create' | 'append';
  }>(),
  {
    mode: 'create',
  }
);

const emit = defineEmits<{
  (e: 'update:visible', value: boolean): void;
  (e: 'import', data: { title: string; description: string; questions: Question[] }): void;
}>();

const activeTab = ref<'file' | 'paste'>('file');
const rawText = ref('');
const importedTitle = ref('');
const importedDescription = ref('');
const parsedQuestions = ref<Question[]>([]);
const parseError = ref<string | null>(null);
const parseWarning = ref<string | null>(null);
const fileName = ref<string | null>(null);
const isDragging = ref(false);

const optionIcons = ['▲', '◆', '●', '■'];
const optionColors = [
  'text-rose-400 bg-rose-500/10 border-rose-500/20',
  'text-blue-400 bg-blue-500/10 border-blue-500/20',
  'text-amber-400 bg-amber-500/10 border-amber-500/20',
  'text-emerald-400 bg-emerald-500/10 border-emerald-500/20',
];

function close() {
  emit('update:visible', false);
}

// Reset state when opening
function resetState() {
  rawText.value = '';
  importedTitle.value = '';
  importedDescription.value = '';
  parsedQuestions.value = [];
  parseError.value = null;
  parseWarning.value = null;
  fileName.value = null;
}

// -------------------------------------------------------------
// Parsers
// -------------------------------------------------------------

function parseCSVLine(line: string): string[] {
  const result: string[] = [];
  let current = '';
  let inQuotes = false;

  for (let i = 0; i < line.length; i++) {
    const char = line[i];
    if (char === '"') {
      if (inQuotes && line[i + 1] === '"') {
        current += '"';
        i++;
      } else {
        inQuotes = !inQuotes;
      }
    } else if (char === ',' && !inQuotes) {
      result.push(current.trim());
      current = '';
    } else {
      current += char;
    }
  }
  result.push(current.trim());
  return result;
}

function parseCSV(content: string): Question[] {
  const lines = content
    .split(/\r?\n/)
    .map((l) => l.trim())
    .filter((l) => l.length > 0);

  if (lines.length === 0) return [];

  // Check if first line is a header
  const firstRow = parseCSVLine(lines[0]);
  const isHeader =
    firstRow.some((col) => /question|prompt/i.test(col)) ||
    firstRow.some((col) => /option|answer|choice/i.test(col));

  const dataLines = isHeader ? lines.slice(1) : lines;
  const questions: Question[] = [];

  for (let idx = 0; idx < dataLines.length; idx++) {
    const cols = parseCSVLine(dataLines[idx]);
    if (cols.length < 3) continue;

    const questionText = cols[0];
    if (!questionText) continue;

    // Expected format: Question, Opt1, Opt2, Opt3 (opt), Opt4 (opt), Correct (1-4 or A-D or text), TimeSec (opt)
    // Find options
    const optionTexts: string[] = [];
    let correctIndicator = '';
    let timeLimitSecs = 20;

    // Last column could be time if numeric <= 300
    let lastColIndex = cols.length - 1;
    const maybeTime = parseInt(cols[lastColIndex], 10);
    if (!isNaN(maybeTime) && maybeTime > 0 && maybeTime <= 300 && cols.length >= 4) {
      timeLimitSecs = maybeTime;
      lastColIndex--;
    }

    // Now look for correct answer indicator (either 1-4, A-D, or matching option text)
    if (lastColIndex >= 2) {
      correctIndicator = cols[lastColIndex];
      // Remaining columns in between are options
      for (let c = 1; c < lastColIndex; c++) {
        if (cols[c].length > 0) {
          optionTexts.push(cols[c]);
        }
      }
    }

    if (optionTexts.length < 2) continue;

    // Determine which option is correct
    let correctIdx = 0;
    const trimmedIndicator = correctIndicator.trim().toLowerCase();

    if (/^[1-4]$/.test(trimmedIndicator)) {
      correctIdx = parseInt(trimmedIndicator, 10) - 1;
    } else if (trimmedIndicator === 'a') {
      correctIdx = 0;
    } else if (trimmedIndicator === 'b') {
      correctIdx = 1;
    } else if (trimmedIndicator === 'c') {
      correctIdx = 2;
    } else if (trimmedIndicator === 'd') {
      correctIdx = 3;
    } else {
      // Try matching by exact option text
      const foundIdx = optionTexts.findIndex(
        (o) => o.toLowerCase() === trimmedIndicator || trimmedIndicator.includes(o.toLowerCase())
      );
      if (foundIdx !== -1) {
        correctIdx = foundIdx;
      }
    }

    const options: Option[] = optionTexts.map((optText, oIdx) => ({
      id: `opt_${Date.now()}_${idx}_${oIdx}`,
      text: optText,
      is_correct: oIdx === correctIdx,
    }));

    questions.push({
      id: `q_${Date.now()}_${idx}`,
      question_text: questionText,
      time_limit_secs: timeLimitSecs,
      points: 1000,
      order_num: idx + 1,
      options,
    });
  }

  return questions;
}

function parseJSON(content: string): { title?: string; description?: string; questions: Question[] } {
  const data = JSON.parse(content);
  let title = '';
  let description = '';
  let rawList: any[] = [];

  if (Array.isArray(data)) {
    rawList = data;
  } else if (typeof data === 'object' && data !== null) {
    title = data.title || '';
    description = data.description || '';
    if (Array.isArray(data.questions)) {
      rawList = data.questions;
    }
  }

  const questions: Question[] = rawList.map((item: any, qIdx: number) => {
    const qText = item.question_text || item.text || item.question || item.prompt || `Question ${qIdx + 1}`;
    const timeLimit = item.time_limit_sec || item.time_limit_secs || item.time || 20;
    const points = item.points || 1000;

    let options: Option[] = [];
    if (Array.isArray(item.options)) {
      options = item.options.map((opt: any, oIdx: number) => {
        if (typeof opt === 'string') {
          return {
            id: `opt_${Date.now()}_${qIdx}_${oIdx}`,
            text: opt,
            is_correct: oIdx === (item.correct_index ?? item.answer_index ?? 0),
          };
        }
        return {
          id: opt.id || `opt_${Date.now()}_${qIdx}_${oIdx}`,
          text: opt.option_text || opt.text || opt.title || `Option ${oIdx + 1}`,
          is_correct: !!opt.is_correct || !!opt.correct,
        };
      });
    }

    // Ensure at least one correct option
    if (options.length > 0 && !options.some((o) => o.is_correct)) {
      options[0].is_correct = true;
    }

    return {
      id: item.id || `q_${Date.now()}_${qIdx}`,
      question_text: qText,
      time_limit_secs: timeLimit,
      points,
      order_num: item.order_num || item.order_index || qIdx + 1,
      options,
    };
  });

  return { title, description, questions };
}

function parsePlainText(content: string): Question[] {
  // Common format from ChatGPT / Quiz generators:
  // 1. Question text?
  // A) Option 1
  // B) Option 2*
  // C) Option 3
  // D) Option 4
  // or Answer: B

  const blocks = content.split(/\n\s*\n/);
  const questions: Question[] = [];

  for (let qIdx = 0; qIdx < blocks.length; qIdx++) {
    const lines = blocks[qIdx]
      .split(/\r?\n/)
      .map((l) => l.trim())
      .filter((l) => l.length > 0);

    if (lines.length < 3) continue;

    // First line is usually question prompt
    let qPrompt = lines[0].replace(/^(?:Q\d*[:.]?|\d+[\.\)]|\*\*Question\s*\d*:\*\*)\s*/i, '').trim();

    const optionLines: { text: string; isCorrect: boolean }[] = [];
    let answerIndicator: string | null = null;

    for (let i = 1; i < lines.length; i++) {
      const line = lines[i];

      // Check if it's an answer key line: "Answer: A" or "Correct: 2"
      const ansMatch = line.match(/^(?:Answer|Correct|Key)[:\s]+([A-Da-d1-4])/i);
      if (ansMatch) {
        answerIndicator = ansMatch[1].toUpperCase();
        continue;
      }

      // Check option pattern: "A) Text" or "A. Text" or "1. Text" or "[x] Text"
      const optMatch = line.match(/^(?:[A-Da-d1-4][\.\)]|\([A-Da-d1-4]\)|[-*•])\s*(.*)$/);
      if (optMatch) {
        let optText = optMatch[1].trim();
        let isCorrect = false;

        // Check for trailing asterisk or (correct)
        if (optText.endsWith('*')) {
          optText = optText.slice(0, -1).trim();
          isCorrect = true;
        } else if (/\((?:correct|answer)\)$/i.test(optText)) {
          optText = optText.replace(/\((?:correct|answer)\)$/i, '').trim();
          isCorrect = true;
        }

        optionLines.push({ text: optText, isCorrect });
      }
    }

    if (optionLines.length < 2) continue;

    // If answer indicator was provided separately, apply it
    if (answerIndicator) {
      let correctIdx = -1;
      if (['A', '1'].includes(answerIndicator)) correctIdx = 0;
      else if (['B', '2'].includes(answerIndicator)) correctIdx = 1;
      else if (['C', '3'].includes(answerIndicator)) correctIdx = 2;
      else if (['D', '4'].includes(answerIndicator)) correctIdx = 3;

      if (correctIdx >= 0 && correctIdx < optionLines.length) {
        optionLines.forEach((o, idx) => (o.isCorrect = idx === correctIdx));
      }
    }

    // Default to first option if none marked
    if (!optionLines.some((o) => o.isCorrect)) {
      optionLines[0].isCorrect = true;
    }

    questions.push({
      id: `q_${Date.now()}_${qIdx}`,
      question_text: qPrompt,
      time_limit_secs: 20,
      points: 1000,
      order_num: qIdx + 1,
      options: optionLines.map((o, oIdx) => ({
        id: `opt_${Date.now()}_${qIdx}_${oIdx}`,
        text: o.text,
        is_correct: o.isCorrect,
      })),
    });
  }

  return questions;
}

function processContent(text: string, sourceName?: string) {
  parseError.value = null;
  parseWarning.value = null;

  const trimmed = text.trim();
  if (!trimmed) {
    parseError.value = 'File or text content is empty.';
    parsedQuestions.value = [];
    return;
  }

  // 1. Try parsing as JSON
  if (trimmed.startsWith('{') || trimmed.startsWith('[')) {
    try {
      const res = parseJSON(trimmed);
      if (res.title && !importedTitle.value) {
        importedTitle.value = res.title;
      }
      if (res.description && !importedDescription.value) {
        importedDescription.value = res.description;
      }
      if (res.questions.length > 0) {
        parsedQuestions.value = res.questions;
        if (!importedTitle.value && sourceName) {
          importedTitle.value = sourceName.replace(/\.[^/.]+$/, '').replace(/[-_]/g, ' ');
        }
        validateQuestions();
        return;
      }
    } catch (e: any) {
      // If it looks like JSON but failed, record error
      if (trimmed.startsWith('{')) {
        parseError.value = `JSON Parse Error: ${e.message}`;
        return;
      }
    }
  }

  // 2. Try parsing as CSV
  if (trimmed.includes(',') && trimmed.includes('\n')) {
    const csvQuestions = parseCSV(trimmed);
    if (csvQuestions.length > 0) {
      parsedQuestions.value = csvQuestions;
      if (!importedTitle.value && sourceName) {
        importedTitle.value = sourceName.replace(/\.[^/.]+$/, '').replace(/[-_]/g, ' ');
      }
      validateQuestions();
      return;
    }
  }

  // 3. Fallback: Parse plain text / AI question format
  const textQuestions = parsePlainText(trimmed);
  if (textQuestions.length > 0) {
    parsedQuestions.value = textQuestions;
    if (!importedTitle.value && sourceName) {
      importedTitle.value = sourceName.replace(/\.[^/.]+$/, '').replace(/[-_]/g, ' ');
    }
    validateQuestions();
    return;
  }

  parseError.value =
    'Unable to recognize question format. Please check the sample format below (JSON, CSV, or formatted text).';
  parsedQuestions.value = [];
}

function validateQuestions() {
  const count = parsedQuestions.value.length;
  if (count === 0) {
    parseError.value = 'No valid questions could be extracted from the content.';
    return;
  }

  const missingOptions = parsedQuestions.value.filter((q) => q.options.length < 2);
  if (missingOptions.length > 0) {
    parseWarning.value = `${missingOptions.length} question(s) had fewer than 2 options.`;
  }
}

// -------------------------------------------------------------
// File Handlers
// -------------------------------------------------------------

function handleFileUpload(event: Event) {
  const target = event.target as HTMLInputElement;
  const file = target.files?.[0];
  if (!file) return;
  readFile(file);
}

function handleDrop(event: DragEvent) {
  isDragging.value = false;
  const file = event.dataTransfer?.files?.[0];
  if (!file) return;
  readFile(file);
}

function readFile(file: File) {
  fileName.value = file.name;
  const reader = new FileReader();
  reader.onload = (e) => {
    const content = e.target?.result as string;
    rawText.value = content;
    processContent(content, file.name);
  };
  reader.onerror = () => {
    parseError.value = 'Failed to read file.';
  };
  reader.readAsText(file);
}

function handlePasteChange() {
  processContent(rawText.value);
}

// -------------------------------------------------------------
// Sample Downloads
// -------------------------------------------------------------

function downloadSampleJSON() {
  const sample = {
    title: 'World Capitals Championship',
    description: 'Test your global geography skills with quick multi-choice questions.',
    questions: [
      {
        question_text: 'What is the capital city of France?',
        time_limit_sec: 20,
        points: 1000,
        options: [
          { option_text: 'Paris', is_correct: true },
          { option_text: 'Marseille', is_correct: false },
          { option_text: 'Lyon', is_correct: false },
          { option_text: 'Nice', is_correct: false },
        ],
      },
      {
        question_text: 'Which planet is known as the Red Planet?',
        time_limit_sec: 20,
        points: 1000,
        options: [
          { option_text: 'Venus', is_correct: false },
          { option_text: 'Mars', is_correct: true },
          { option_text: 'Jupiter', is_correct: false },
          { option_text: 'Saturn', is_correct: false },
        ],
      },
      {
        question_text: 'How many continents are there on Earth?',
        time_limit_sec: 15,
        points: 1000,
        options: [
          { option_text: '5', is_correct: false },
          { option_text: '6', is_correct: false },
          { option_text: '7', is_correct: true },
          { option_text: '8', is_correct: false },
        ],
      },
    ],
  };

  const blob = new Blob([JSON.stringify(sample, null, 2)], { type: 'application/json' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = 'sample_quiz_ezsn.json';
  a.click();
  URL.revokeObjectURL(url);
}

function downloadSampleCSV() {
  const csvContent = `Question,Option 1,Option 2,Option 3,Option 4,Correct Answer (1-4 or text),Time (seconds)
"What is the capital of France?","Paris","Marseille","Lyon","Nice",1,20
"Which planet is known as the Red Planet?","Venus","Mars","Jupiter","Saturn",2,20
"What is 12 x 12?","124","144","164","184",2,15
"How many colors are in a rainbow?","5","6","7","8",3,20`;

  const blob = new Blob([csvContent], { type: 'text/csv;charset=utf-8;' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = 'sample_quiz_ezsn.csv';
  a.click();
  URL.revokeObjectURL(url);
}

// -------------------------------------------------------------
// Submit
// -------------------------------------------------------------

function confirmImport() {
  if (parsedQuestions.value.length === 0) return;

  emit('import', {
    title: importedTitle.value.trim() || 'Imported Quiz Competition',
    description: importedDescription.value.trim(),
    questions: parsedQuestions.value,
  });

  close();
}
</script>

<template>
  <Dialog
    :visible="visible"
    @update:visible="emit('update:visible', $event)"
    modal
    :closable="true"
    :dismissableMask="true"
    class="max-w-3xl w-full mx-4 rounded-3xl overflow-hidden glass-panel border border-purple-400/30 p-0 shadow-2xl"
    :header="mode === 'append' ? 'Import Questions into Quiz' : 'Import Quiz Competition'"
  >
    <div class="p-6 space-y-6 max-h-[80vh] overflow-y-auto">
      <!-- Top Explanatory Banner & Sample Download Buttons -->
      <div class="glass-card rounded-2xl p-4 flex flex-col sm:flex-row items-start sm:items-center justify-between gap-3 border border-purple-400/20">
        <div class="space-y-0.5">
          <span class="text-xs font-bold text-white flex items-center space-x-1.5">
            <i class="pi pi-file-import text-purple-300" />
            <span>Supported Formats: JSON, CSV, or AI Plain Text</span>
          </span>
          <p class="text-[11px] text-purple-200/70">
            Upload a spreadsheet/JSON export or paste ChatGPT generated quiz prompts directly.
          </p>
        </div>

        <div class="flex items-center space-x-2 shrink-0">
          <Button
            label="Sample CSV"
            icon="pi pi-download"
            size="small"
            text
            class="glass-pill text-[11px] font-bold text-purple-200 hover:text-white py-1 px-2.5 rounded-xl border border-purple-400/30"
            @click="downloadSampleCSV"
          />
          <Button
            label="Sample JSON"
            icon="pi pi-download"
            size="small"
            text
            class="glass-pill text-[11px] font-bold text-purple-200 hover:text-white py-1 px-2.5 rounded-xl border border-purple-400/30"
            @click="downloadSampleJSON"
          />
        </div>
      </div>

      <!-- Mode Selector Tabs: File Upload vs Paste Text -->
      <div class="flex items-center space-x-2 border-b border-purple-400/20 pb-2">
        <button
          type="button"
          @click="activeTab = 'file'"
          :class="[
            'px-4 py-2 rounded-xl text-xs font-bold transition-all flex items-center space-x-2',
            activeTab === 'file'
              ? 'glass-panel text-white border border-purple-400/40 shadow-md shadow-purple-900/30'
              : 'text-purple-300/70 hover:text-white'
          ]"
        >
          <i class="pi pi-upload" />
          <span>Upload File (.json, .csv)</span>
        </button>

        <button
          type="button"
          @click="activeTab = 'paste'"
          :class="[
            'px-4 py-2 rounded-xl text-xs font-bold transition-all flex items-center space-x-2',
            activeTab === 'paste'
              ? 'glass-panel text-white border border-purple-400/40 shadow-md shadow-purple-900/30'
              : 'text-purple-300/70 hover:text-white'
          ]"
        >
          <i class="pi pi-file-edit" />
          <span>Paste Text / JSON / ChatGPT</span>
        </button>
      </div>

      <!-- TAB 1: FILE DROP ZONE -->
      <div v-if="activeTab === 'file'" class="space-y-4">
        <label
          @dragover.prevent="isDragging = true"
          @dragleave.prevent="isDragging = false"
          @drop.prevent="handleDrop"
          :class="[
            'border-2 border-dashed rounded-3xl p-8 flex flex-col items-center justify-center cursor-pointer transition-all text-center group',
            isDragging
              ? 'border-fuchsia-400 bg-fuchsia-500/10 scale-[1.01]'
              : 'border-purple-400/30 hover:border-purple-400/60 bg-purple-950/20 hover:bg-purple-950/40'
          ]"
        >
          <input
            type="file"
            accept=".json,.csv,.txt,.tsv"
            class="hidden"
            @change="handleFileUpload"
          />

          <div class="w-14 h-14 rounded-2xl bg-gradient-to-tr from-purple-600/30 via-fuchsia-600/20 to-pink-600/30 text-purple-200 flex items-center justify-center text-2xl mb-3 border border-purple-400/30 shadow-xl group-hover:scale-105 transition-transform">
            <i class="pi pi-cloud-upload text-xl" />
          </div>

          <h4 class="text-sm font-bold text-white mb-1">
            {{ fileName ? fileName : 'Click to browse or drop file here' }}
          </h4>
          <p class="text-xs text-purple-300/70">
            Accepts <span class="text-purple-200 font-mono font-bold">.json</span>, <span class="text-purple-200 font-mono font-bold">.csv</span>, or formatted text
          </p>
        </label>
      </div>

      <!-- TAB 2: PASTE RAW TEXT -->
      <div v-else class="space-y-2">
        <label class="text-xs font-bold uppercase tracking-wider text-purple-200">
          Paste Raw Quiz Content
        </label>
        <textarea
          v-model="rawText"
          @input="handlePasteChange"
          rows="7"
          placeholder="Paste JSON, CSV rows, or AI-generated quiz text:

1. What is the capital of Italy?
A) Milan
B) Rome*
C) Venice
D) Naples"
          class="glass-input w-full p-4 rounded-2xl text-xs font-mono text-white placeholder:text-purple-300/30 border border-purple-400/30 focus:border-purple-300 resize-y"
        />
      </div>

      <!-- ERROR / WARNING MESSAGES -->
      <Message v-if="parseError" severity="error" :closable="false" class="rounded-xl text-xs">
        {{ parseError }}
      </Message>
      <Message v-if="parseWarning" severity="warn" :closable="false" class="rounded-xl text-xs">
        {{ parseWarning }}
      </Message>

      <!-- PARSED PREVIEW SECTION -->
      <div v-if="parsedQuestions.length > 0" class="space-y-4 pt-2 border-t border-purple-400/20">
        <div class="flex items-center justify-between">
          <div class="flex items-center space-x-2">
            <span class="text-xs font-black uppercase tracking-wider text-emerald-400 flex items-center space-x-1">
              <i class="pi pi-check-circle" />
              <span>Parsed Successfully</span>
            </span>
            <span class="glass-pill px-2.5 py-0.5 rounded-full text-xs font-bold text-purple-200">
              {{ parsedQuestions.length }} Questions
            </span>
          </div>
        </div>

        <!-- Editable Title (for full quiz import) -->
        <div v-if="mode === 'create'" class="space-y-1.5">
          <label class="text-xs font-bold uppercase tracking-wider text-purple-200">Quiz Title</label>
          <InputText
            v-model="importedTitle"
            placeholder="e.g. World Capitals Championship"
            class="glass-input w-full font-bold text-sm py-2.5 rounded-xl text-white"
          />
        </div>

        <!-- Scrollable Questions Preview -->
        <div class="space-y-3 max-h-64 overflow-y-auto pr-1">
          <div
            v-for="(q, qIdx) in parsedQuestions"
            :key="q.id"
            class="glass-card rounded-2xl p-3.5 space-y-2 border border-purple-400/20"
          >
            <div class="flex items-center justify-between text-xs">
              <span class="font-extrabold text-white">
                {{ qIdx + 1 }}. {{ q.question_text }}
              </span>
              <span class="text-[11px] text-purple-300/80 font-mono">
                ⏱ {{ q.time_limit_secs }}s
              </span>
            </div>

            <!-- Options Chips -->
            <div class="grid grid-cols-1 sm:grid-cols-2 gap-1.5 pt-1">
              <div
                v-for="(opt, optIdx) in q.options"
                :key="opt.id"
                :class="[
                  'px-2.5 py-1.5 rounded-xl text-xs font-semibold flex items-center space-x-2 border',
                  opt.is_correct
                    ? 'border-emerald-500/50 bg-emerald-950/40 text-emerald-200 font-bold'
                    : 'border-purple-400/15 text-purple-200/80 bg-purple-950/20'
                ]"
              >
                <span class="w-4 h-4 rounded text-[10px] font-black flex items-center justify-center shrink-0" :class="optionColors[optIdx % 4]">
                  {{ optionIcons[optIdx % 4] }}
                </span>
                <span class="truncate">{{ opt.text }}</span>
                <span v-if="opt.is_correct" class="ml-auto text-emerald-400 text-[10px] font-black">✓ Correct</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Modal Footer Actions -->
    <template #footer>
      <div class="p-4 border-t border-purple-400/20 flex items-center justify-end space-x-3">
        <Button
          label="Cancel"
          text
          severity="secondary"
          class="glass-pill text-purple-200 hover:text-white px-4 py-2 text-xs"
          @click="close"
        />

        <Button
          :label="mode === 'append' ? `Import ${parsedQuestions.length} Questions` : 'Create & Save Quiz'"
          icon="pi pi-check"
          :disabled="parsedQuestions.length === 0"
          class="bg-gradient-to-r from-purple-600 via-fuchsia-600 to-pink-600 hover:from-purple-500 hover:to-pink-500 border border-purple-300/30 font-bold text-xs text-white px-6 py-2.5 rounded-xl shadow-lg shadow-purple-900/40 disabled:opacity-40 disabled:cursor-not-allowed"
          @click="confirmImport"
        />
      </div>
    </template>
  </Dialog>
</template>
