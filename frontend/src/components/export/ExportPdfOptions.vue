<script setup lang="ts">
import type { LineSpec } from '../../book'
import { LINE_STYLES } from '../../book'
import { FONT_STEPS, useExportPrefs } from '../../composables/useExportPrefs'

const { opts } = useExportPrefs()

type SepKey = 'sepQ' | 'sepHead' | 'sepFoot'

function pickLine(key: SepKey, ev: Event) {
  opts[key].style = (ev.target as HTMLSelectElement).value as LineSpec['style']
}

function pickFontScale(ev: Event) {
  const label = (ev.target as HTMLSelectElement).value
  opts.fontScale = FONT_STEPS.find((s) => s.label === label)?.fs ?? 1
}

const SLOT_PLACEHOLDERS = {
  headerBinding: `(c) => c.date + ' · 共 ' + c.totalQuestions + ' 题' + (c.seed ? ' 🎲 ' + c.seed : '')`,
  headerCenter: `(c) => c.subject`,
  headerFlip: `(c) => c.title + (c.answers ? ' - 答案' : '')`,
  footerBinding: `''`,
  footerCenter: `(c) => c.title`,
  footerFlip: `(c) => c.page`,
}

const META_PLACEHOLDERS = {
  workbook: `(q) => q.chapter + (q.locate ? ' @ ' + q.locate : '')`,
  answers: `(q) => q.chapter + (q.locate ? ' @ ' + q.locate : '')`,
}
</script>

<template>
  <div class="m-body">
    <div class="field">
      <span class="lbl">纸张</span>
      <div class="seg fill">
        <button :class="{ on: opts.paper === 'A4' }" @click="opts.paper = 'A4'">A4</button>
        <button :class="{ on: opts.paper === 'B5' }" @click="opts.paper = 'B5'">B5</button>
      </div>
    </div>
    <div class="field">
      <span class="lbl">每页题数</span>
      <select
        class="fill"
        :value="opts.perPage"
        @change="opts.perPage = Number(($event.target as HTMLSelectElement).value)"
      >
        <option v-for="n in 4" :key="n" :value="n">{{ n }} 题</option>
        <option :value="0">尽可能多</option>
      </select>
    </div>
    <div class="field">
      <span class="lbl">字号</span>
      <select
        class="fill"
        :value="FONT_STEPS.find((s) => s.fs === opts.fontScale)?.label ?? '五号'"
        @change="pickFontScale"
      >
        <option v-for="s in FONT_STEPS" :key="s.label" :value="s.label">{{ s.label }}</option>
      </select>
    </div>
    <div class="field">
      <span class="lbl">选项排布</span>
      <select
        class="fill"
        :value="opts.optsPerRow"
        @change="opts.optsPerRow = Number(($event.target as HTMLSelectElement).value) as 1 | 2 | 4"
      >
        <option :value="4">每行 4 个</option>
        <option :value="2">每行 2 个</option>
        <option :value="1">每行 1 个</option>
      </select>
    </div>
    <div class="field">
      <span class="lbl">题间分隔线</span>
      <select class="fill" :value="opts.sepQ.style" @change="pickLine('sepQ', $event)">
        <option v-for="st in LINE_STYLES" :key="st.value" :value="st.value">{{ st.label }}</option>
      </select>
    </div>

    <button class="toggle-row" type="button" @click="opts.showMeta = !opts.showMeta">
      <input type="checkbox" :checked="opts.showMeta" tabindex="-1" />
      <span>包含题目元信息</span>
    </button>

    <button class="toggle-row" type="button" @click="opts.bindingLong = !opts.bindingLong">
      <input type="checkbox" :checked="opts.bindingLong" tabindex="-1" />
      <span>长边装订</span>
    </button>
    <button v-if="opts.bindingLong" class="toggle-row" type="button" @click="opts.firstPageLeft = !opts.firstPageLeft">
      <input type="checkbox" :checked="opts.firstPageLeft" tabindex="-1" />
      <span>首页在左</span>
    </button>
    <div v-if="opts.bindingLong" class="field">
      <span class="lbl">长边装订避让</span>
      <input
        type="number"
        class="fill"
        min="0"
        max="30"
        step="1"
        :value="opts.bindingExtra"
        @change="opts.bindingExtra = Math.max(0, Math.min(30, Number(($event.target as HTMLInputElement).value) || 0))"
      />
      <span class="unit">mm</span>
    </div>

    <details class="adv">
      <summary>高级</summary>
      <div class="adv-sec">
        <div class="adv-title">页边距</div>
        <div class="margin-grid adv-grid">
            <label class="hf-row"><span class="hf-lab">上</span>
              <input v-model.number="opts.marginT" class="fn-input" type="number" min="5" max="40" /><span class="unit">mm</span></label>
            <label class="hf-row"><span class="hf-lab">下</span>
              <input v-model.number="opts.marginB" class="fn-input" type="number" min="5" max="40" /><span class="unit">mm</span></label>
            <label class="hf-row"><span class="hf-lab">左</span>
              <input v-model.number="opts.marginL" class="fn-input" type="number" min="5" max="40" /><span class="unit">mm</span></label>
            <label class="hf-row"><span class="hf-lab">右</span>
              <input v-model.number="opts.marginR" class="fn-input" type="number" min="5" max="40" /><span class="unit">mm</span></label>
        </div>
      </div>

      <div class="adv-sec">
        <div class="adv-title">题目分隔线</div>
        <div class="adv-grid">
          <label class="hf-row"><span class="hf-lab">线宽</span>
            <input v-model.number="opts.sepQ.width" class="fn-input" type="number" min="0.2" max="4" step="0.1" /><span class="unit">pt</span></label>
          <label class="hf-row"><span class="hf-lab">间隔</span>
            <input v-model.number="opts.sepQ.gap" class="fn-input" type="number" min="0.5" max="12" step="0.5" /><span class="unit">pt</span></label>
        </div>
      </div>

      <div class="adv-sec">
        <div class="adv-title">页眉线</div>
        <div class="adv-grid">
          <label class="hf-row"><span class="hf-lab">样式</span>
            <select class="fn-input" :value="opts.sepHead.style" @change="pickLine('sepHead', $event)">
              <option v-for="st in LINE_STYLES" :key="st.value" :value="st.value">{{ st.label }}</option>
            </select></label>
          <label class="hf-row"><span class="hf-lab">线宽</span>
            <input v-model.number="opts.sepHead.width" class="fn-input" type="number" min="0.2" max="4" step="0.1" /><span class="unit">pt</span></label>
          <label class="hf-row"><span class="hf-lab">间隔</span>
            <input v-model.number="opts.sepHead.gap" class="fn-input" type="number" min="0.5" max="12" step="0.5" /><span class="unit">pt</span></label>
        </div>
      </div>

      <div class="adv-sec">
        <div class="adv-title">页脚线</div>
        <div class="adv-grid">
          <label class="hf-row"><span class="hf-lab">样式</span>
            <select class="fn-input" :value="opts.sepFoot.style" @change="pickLine('sepFoot', $event)">
              <option v-for="st in LINE_STYLES" :key="st.value" :value="st.value">{{ st.label }}</option>
            </select></label>
          <label class="hf-row"><span class="hf-lab">线宽</span>
            <input v-model.number="opts.sepFoot.width" class="fn-input" type="number" min="0.2" max="4" step="0.1" /><span class="unit">pt</span></label>
          <label class="hf-row"><span class="hf-lab">间隔</span>
            <input v-model.number="opts.sepFoot.gap" class="fn-input" type="number" min="0.5" max="12" step="0.5" /><span class="unit">pt</span></label>
        </div>
      </div>

      <div class="adv-sec">
        <div class="adv-title">题目元信息</div>
        <details class="doc-fold"><summary>说明与参数</summary>
          <p class="adv-doc">每道题右下角的灰色小字，题本与答案本各一个函数。留空显示默认的「章节 @ 定位」。</p>
        <pre class="adv-code">(q: {
  index: number     // 题号，从 1 开始
  total: number     // 题目总数
  chapter: string   // 章节
  locate: string    // 定位，如 P29-10
  subject: string   // 学科
  answers: boolean  // 是否答案本
}) => string</pre>
        </details>
        <div class="adv-grid">
          <label class="hf-row"><span class="hf-lab">题本</span>
            <textarea v-model="opts.workbookMeta" class="fn-input fn-area" :placeholder="META_PLACEHOLDERS.workbook" spellcheck="false" rows="1"></textarea></label>
          <label class="hf-row"><span class="hf-lab">答案本</span>
            <textarea v-model="opts.answersMeta" class="fn-input fn-area" :placeholder="META_PLACEHOLDERS.answers" spellcheck="false" rows="1"></textarea></label>
        </div>
      </div>

      <div class="adv-sec">
        <div class="adv-title">页眉 / 页脚</div>
        <details class="doc-fold"><summary>说明与参数</summary>
          <p class="adv-doc">页眉页脚各分左中右三槽，每槽一个函数，返回字符串按纯文本排版，支持 emoji；留空使用默认。</p>
        <pre class="adv-code">(c: {
  page: number           // 当前页码，从 1 开始
  totalPages: number     // 总页数
  totalQuestions: number // 题目总数
  title: string          // 题本名
  date: Date             // 导出日期，当天零点
  dateStrs: {            // 日期各部件的现成字符串
    year: string         // '2026'
    month: string        // '09'
    day: string          // '04'
    ymd: string          // '2026-09-04'
    zh: string           // '2026年9月4日'
    compact: string      // '20260904'
  }
  seed: string           // 种子，快速刷题为空串
  subject: string        // 学科，取第一题
  answers: boolean       // 是否答案本
  paper: 'A4' | 'B5'     // 纸型
  binding: boolean       // 是否长边装订
  firstPageLeft: boolean // 长边装订时首页在左
}) => string</pre>
        </details>
        <div class="rule-group">
          <div class="adv-title sub">页眉</div>
          <label class="hf-row"><span class="hf-lab">{{ opts.bindingLong ? '装订侧' : '左' }}</span>
            <textarea v-model="opts.headerBinding" class="fn-input fn-area" :placeholder="SLOT_PLACEHOLDERS.headerBinding" spellcheck="false" rows="1"></textarea></label>
          <label class="hf-row"><span class="hf-lab">中央</span>
            <textarea v-model="opts.headerCenter" class="fn-input fn-area" :placeholder="SLOT_PLACEHOLDERS.headerCenter" spellcheck="false" rows="1"></textarea></label>
          <label class="hf-row"><span class="hf-lab">{{ opts.bindingLong ? '翻页侧' : '右' }}</span>
            <textarea v-model="opts.headerFlip" class="fn-input fn-area" :placeholder="SLOT_PLACEHOLDERS.headerFlip" spellcheck="false" rows="1"></textarea></label>
        </div>
        <div class="rule-group">
          <div class="adv-title sub">页脚</div>
          <label class="hf-row"><span class="hf-lab">{{ opts.bindingLong ? '装订侧' : '左' }}</span>
            <textarea v-model="opts.footerBinding" class="fn-input fn-area" :placeholder="SLOT_PLACEHOLDERS.footerBinding" spellcheck="false" rows="1"></textarea></label>
          <label class="hf-row"><span class="hf-lab">中央</span>
            <textarea v-model="opts.footerCenter" class="fn-input fn-area" :placeholder="SLOT_PLACEHOLDERS.footerCenter" spellcheck="false" rows="1"></textarea></label>
          <label class="hf-row"><span class="hf-lab">{{ opts.bindingLong ? '翻页侧' : '右' }}</span>
            <textarea v-model="opts.footerFlip" class="fn-input fn-area" :placeholder="SLOT_PLACEHOLDERS.footerFlip" spellcheck="false" rows="1"></textarea></label>
        </div>
      </div>

      <div class="adv-sec">
        <div class="adv-title">自定义样式</div>
        <details class="doc-fold"><summary>说明与示例</summary>
          <p class="adv-doc">
          追加在默认 Typst 设置之后，可覆盖默认值。常用写法：<br />
          <code>#set text(size: 11pt, weight: "regular")</code> 调全局字体、字号、字重、颜色；<br />
          <code>#set par(leading: 1.3em, spacing: 1.5em, justify: true)</code> 调行距、段距、两端对齐；<br />
          <code>#let ansbox(body) = block(stroke: 1pt + gray, inset: 3mm, align(center, body))</code> 调答案框；<br />
          <code>#let qblock(body) = block(.., stroke: (bottom: (paint: gray, dash: "solid")), body)</code> 调题目分隔线；<br />
          <code>#let fig(key) = image(.., width: 60mm)</code> 调题图宽度。<br />
          纸张与页边距由上方「纸张 / 长边装订」控制，这里 <code>#set page</code> 的修改会被逐页定义覆盖。
        </p>
        </details>
        <textarea
          v-model="opts.customStyle"
          class="style-input"
          rows="6"
          spellcheck="false"
          placeholder="#set par(leading: 1.3em)&#10;#set text(size: 11pt)"
        ></textarea>
      </div>

    </details>
  </div>
</template>

<style scoped>
.m-body {
  margin-top: 0.875rem;
  display: flex;
  flex-direction: column;
  gap: 0.875rem;
}

.unit {
  flex: none;
  font-size: 0.7812rem;
  color: var(--muted);
}

.adv-sec .unit {
  width: 1.5rem;
}

.margin-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 0.375rem 1rem;
}

.adv-title.sub {
  font-size: 0.75rem;
  color: var(--muted);
}

.doc-fold {
  margin: 0 0 0.5rem;
}

.doc-fold summary {
  cursor: pointer;
  user-select: none;
  font-size: 0.7188rem;
  color: var(--brand);
}

.doc-fold .adv-doc {
  margin-top: 0.375rem;
}

.adv-grid,
.rule-group {
  display: flex;
  flex-direction: column;
  gap: 0.375rem;
}

.rule-group {
  padding:4px;
}

.seg {
  margin-top: 0.875rem;
}

.seg button {
  padding: 0.8125rem 0;
  min-height: 3rem;
  font-size: 0.9375rem;
}

.adv {
  border: 1px solid var(--line);
  border-radius:0.625rem;
  padding: 0.625rem 0.75rem;
}

.adv summary {
  cursor: pointer;
  user-select: none;
  font-size: 0.8438rem;
  color: var(--muted);
  font-weight: 600;
}

.adv-sec {
  margin-top: 0.75rem;
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.adv-sec + .adv-sec {
  margin-top: 0.875rem;
}

.adv-title {
  font-size: 0.8125rem;
  font-weight: 600;
  color: var(--ink);
}

.adv-doc {
  margin: 0 0 0.5rem;
  font-size: 0.7188rem;
  line-height: 1.8;
  color: var(--muted);
}

.adv-doc code {
  font-family: var(--font-mono);
  font-size: 0.6875rem;
  background: var(--panel);
  border-radius:4px;
  padding:0 4px;
}

.adv-code {
  margin: 0 0 0.5rem;
  padding: 0.5rem 0.625rem;
  background: var(--panel);
  border-radius:0.5rem;
  font-family: var(--font-mono);
  font-size: 0.6875rem;
  line-height: 1.7;
  color: var(--ink);
  overflow-x: auto;
  white-space: pre;
}

.hf-row {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.hf-lab {
  flex: none;
  width: 3.25rem;
  font-size: 0.75rem;
  color: var(--muted);
}

.fn-area {
  resize: vertical;
  min-height: 2.25rem;
  height: 2.25rem;
  line-height: 1.5;
  overflow-y: auto;
  white-space: pre;
  font-family: var(--font-mono);
}

.fn-input {
  flex: 1;
  min-width: 0;
  font-family: var(--font-mono);
  font-size: 0.7188rem;
  padding: 0.375rem 0.5rem;
}

.style-input {
  width: 100%;
  font-family: var(--font-mono);
  font-size: 0.7188rem;
  line-height: 1.6;
  padding: 0.5rem;
}
</style>
