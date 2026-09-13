import { describe, expect, it } from 'vitest'
import {
  applyImageMapping,
  autoImageNames,
  buildDoc,
  buildHeaderLine,
  clampHeadings,
  defaultFileName,
  docDefects,
  extractImageRefs,
  nextNumberedBase,
  nextOptionLetter,
  parseHeader,
  sanitizeFileName,
  splitBlocks,
  splitDoc,
} from './mdoc'

const CHOICE = `> [数学] 示例题库 · 演示 | 第3章 一元函数微分学的概念 @ P29-10 > 选择题

设 $f(x)$，则（　）。

(A) 不连续

(B) 连续，但不可导

## 答案

**(D)**。

## 解析

连续性：$\\lim x=0$。

## 备注

易错。`

describe('splitBlocks（与后端 paragraphs 一致）', () => {
  it('按空行分段并 trim', () => {
    expect(splitBlocks('a\nb\n\n\nc')).toEqual(['a\nb', 'c'])
    expect(splitBlocks('  a  \n  b\n\n c ')).toEqual(['a\nb', 'c'])
    expect(splitBlocks('\n\n\n')).toEqual([])
  })

  it('代码围栏内的空行不分段', () => {
    expect(splitBlocks('前文\n\n```rust\nlet a = 1;\n\nlet b = 2;\n```\n\n后文')).toEqual([
      '前文',
      '```rust\nlet a = 1;\n\nlet b = 2;\n```',
      '后文',
    ])
  })
})

describe('parseHeader（与后端 parse_header 一致）', () => {
  it('完整五字段', () => {
    const m = parseHeader('[数学] 示例题库 · 演示 | 第1章 极限 @ P15-11 > 填空题')
    expect(m).toEqual({
      subject: '数学',
      origin: '示例题库 · 演示',
      chapter: '第1章 极限',
      locate: 'P15-11',
      qtype: '填空题',
    })
  })

  it('部分字段', () => {
    const m = parseHeader('2026 上海高考 @ P25-12 > 选择题')
    expect(m.subject).toBe('')
    expect(m.origin).toBe('2026 上海高考')
    expect(m.chapter).toBe('')
    expect(m.locate).toBe('P25-12')
    expect(m.qtype).toBe('选择题')

    const n = parseHeader('[数学] 某书')
    expect(n.origin).toBe('某书')
    expect(n.locate).toBe('')

    const o = parseHeader('| 第6章 @ P50-6')
    expect(o.origin).toBe('')
    expect(o.chapter).toBe('第6章')
  })

  it('后置分隔符出现后跳过更早的分隔符', () => {
    const m = parseHeader('来源 @ P1-1 | 杂项')
    expect(m.origin).toBe('来源')
    expect(m.locate).toBe('P1-1 | 杂项')
  })
})

describe('buildHeaderLine / buildDoc round-trip', () => {
  it('所有字段组合都能解析回自身', () => {
    const metas = [
      { subject: '数学', origin: '张宇1000题', chapter: '第5章 微分', locate: 'P42-12', qtype: '选择题' },
      { subject: '', origin: '2026 上海高考', chapter: '', locate: 'P25-12', qtype: '' },
      { subject: '数学', origin: '', chapter: '', locate: '', qtype: '' },
      { subject: '', origin: '', chapter: '第6章', locate: '', qtype: '证明题' },
      { subject: '', origin: '', chapter: '', locate: '', qtype: '' },
    ]
    for (const m of metas) {
      expect(parseHeader(buildHeaderLine(m))).toEqual(m)
    }
  })

  it('拼装文档 → 拆分 → 字段一致', () => {
    const meta = { subject: '数学', origin: '某卷', chapter: '第2章', locate: 'P10-1', qtype: '多选题' }
    const sections = {
      stem: '下列正确的是（　）。\n\n(A) 甲\n\n(B) 乙',
      answer: '**(A)(C)**。',
      solution: '甲丙正确。',
      notes: '本题易错。',
    }
    const md = buildDoc(meta, sections)
    expect(md.startsWith('> [数学] 某卷 | 第2章 @ P10-1 > 多选题\n')).toBe(true)
    expect(md).toContain('\n## 答案\n\n**(A)(C)**。\n')
    const back = splitDoc(md)
    expect(back.meta).toEqual(meta)
    expect(back.sections).toEqual(sections)
  })

  it('空段落省略，空元信息省略首行', () => {
    const md = buildDoc(
      { subject: '', origin: '', chapter: '', locate: '', qtype: '' },
      { stem: '题干______。', answer: '', solution: '步骤。', notes: '' },
    )
    expect(md).toBe('题干______。\n\n## 解析\n\n步骤。')
    const back = splitDoc(md)
    expect(back.sections.answer).toBe('')
    expect(back.sections.solution).toBe('步骤。')
  })

  it('拆分 CHOICE 样例（含选项段落留在题干内）', () => {
    const { meta, sections } = splitDoc(CHOICE)
    expect(meta.locate).toBe('P29-10')
    expect(sections.stem).toBe('设 $f(x)$，则（　）。\n\n(A) 不连续\n\n(B) 连续，但不可导')
    expect(sections.answer).toBe('**(D)**。')
    expect(sections.solution).toBe('连续性：$\\lim x=0$。')
    expect(sections.notes).toBe('易错。')
  })

  it('CRLF 与无头部分隔', () => {
    const md = '> @ P1-1\r\n\r\n题干（　）。\r\n\r\n## 答案\r\n\r\n**(A)**。'
    const { sections } = splitDoc(md)
    expect(sections.stem).toBe('题干（　）。')
    expect(sections.answer).toBe('**(A)**。')
    const bare = splitDoc('题干______。\n\n## 答案\n\n42。')
    expect(bare.meta.locate).toBe('')
    expect(bare.sections.stem).toBe('题干______。')
  })
})

describe('clampHeadings（最高三级标题）', () => {
  it('一二级标题钳制为三级，更深级别不动', () => {
    expect(clampHeadings('# 大标题', 6)).toEqual({ text: '### 大标题', caret: 8 })
    expect(clampHeadings('## 二级\n正文\n#### 四级', 99)).toEqual({
      text: '### 二级\n正文\n#### 四级',
      caret: 100,
    })
  })

  it('无空格的 # 不受影响（避免打断正输入）', () => {
    const r = clampHeadings('#变量 x 的值', 1)
    expect(r.text).toBe('#变量 x 的值')
  })

  it('只移动光标之后的钳制', () => {

    expect(clampHeadings('# abc', 0).caret).toBe(0)

    expect(clampHeadings('# abc\nnext', 6).caret).toBe(8)
  })

  it('空行与普通行不变', () => {
    expect(clampHeadings('', 0)).toEqual({ text: '', caret: 0 })
    expect(clampHeadings('普通文本\n> 引用', 8)).toEqual({ text: '普通文本\n> 引用', caret: 8 })
  })

  it('围栏内的 # 行是代码，不被钳制', () => {
    const text = '```python\n# 注释\nn = 1\n```\n\n# 真标题'
    expect(clampHeadings(text, 99).text).toBe('```python\n# 注释\nn = 1\n```\n\n### 真标题')
  })
})

describe('图片引用', () => {
  it('提取按序去重', () => {
    expect(extractImageRefs('![[a.svg]] 和 ![[b.png]] 还有 ![[a.svg]]')).toEqual(['a.svg', 'b.png'])
  })

  it('替换临时引用与改名', () => {
    const md = '题 ![[u1-0.png]]\n\n![[旧图.svg]]'
    expect(applyImageMapping(md, [['u1-0.png', 'P1-01.png'], ['旧图.svg', '新图.svg']])).toBe(
      '题 ![[P1-01.png]]\n\n![[新图.svg]]',
    )
    expect(applyImageMapping(md, [['x.svg', 'x.svg']])).toBe(md)
  })

  it('替换是同时的：a→b 与 b→c 不级联', () => {

    expect(applyImageMapping('![[a.svg]] ![[b.svg]]', [['a.svg', 'b.svg'], ['b.svg', 'c.svg']])).toBe(
      '![[b.svg]] ![[c.svg]]',
    )

    expect(
      applyImageMapping('![[u1.svg]] 和 ![[G-01.svg]]', [
        ['u1.svg', 'G-01.svg'],
        ['G-01.svg', 'G-02.svg'],
      ]),
    ).toBe('![[G-01.svg]] 和 ![[G-02.svg]]')
  })
})

describe('命名逻辑', () => {
  it('题目 NN 取第一个未占用编号', () => {
    expect(nextNumberedBase('题目', [])).toBe('题目 01')
    expect(nextNumberedBase('题目', ['题目 01.md', '题目 02.md'])).toBe('题目 03')
    expect(nextNumberedBase('题目', ['题目 1.md'])).toBe('题目 01')
  })

  it('默认文件名：定位 > 来源 > 题目 NN', () => {
    expect(defaultFileName({ subject: '', origin: '某卷', chapter: '', locate: 'P1-1', qtype: '' }, [])).toBe('P1-1')
    expect(defaultFileName({ subject: '', origin: '某卷', chapter: '', locate: '', qtype: '' }, [])).toBe('某卷')
    expect(defaultFileName({ subject: '', origin: '', chapter: '', locate: '', qtype: '' }, ['题目 01.md'])).toBe('题目 02')
  })

  it('图片默认名 题目文件名-NN.ext 且跳过占用', () => {
    const temps = [{ id: 'u1.png' }, { id: 'u2.jpg' }, { id: 'u3.svg' }]
    expect(autoImageNames('P42-12', temps, [])).toEqual([
      'P42-12-01.png',
      'P42-12-01.jpg',
      'P42-12-01.svg',
    ])
    expect(autoImageNames('P42-12', temps, ['P42-12-01.png'])).toEqual([
      'P42-12-02.png',
      'P42-12-01.jpg',
      'P42-12-01.svg',
    ])
    expect(autoImageNames('X', [{ id: 'u1.png' }, { id: 'u2.png' }], [])).toEqual(['X-01.png', 'X-02.png'])
  })

  it('文件名校验（与后端一致）', () => {
    expect(sanitizeFileName('P42-12')).toBe('P42-12')
    expect(sanitizeFileName('  P42-12.md ')).toBe('P42-12')

    expect(sanitizeFileName('题目.md.md')).toBe('题目')
    expect(sanitizeFileName('题目 01')).toBe('题目 01')
    expect(sanitizeFileName('')).toBeNull()
    expect(sanitizeFileName('a/b')).toBeNull()
    expect(sanitizeFileName('..')).toBeNull()
    expect(sanitizeFileName('.hidden')).toBeNull()
  })
})

describe('编辑器校验与选项', () => {
  it('题目为空 / 答案解析皆无', () => {
    expect(docDefects({ stem: '', answer: '42。', solution: '', notes: '' })).toEqual({ stemBlank: true, noAnswer: false })
    expect(docDefects({ stem: '题。', answer: '', solution: '', notes: '备注不算' })).toEqual({ stemBlank: false, noAnswer: true })
    expect(docDefects({ stem: '题。', answer: '', solution: '步骤。', notes: '' })).toEqual({
      stemBlank: false,
      noAnswer: false,
    })
  })

  it('下一个选项字母', () => {
    expect(nextOptionLetter('题干')).toBe('A')
    expect(nextOptionLetter('题干\n\n(A) 甲\n\n(B) 乙')).toBe('C')
    expect(nextOptionLetter('（a) 小写不算')).toBe('A')
  })
})
