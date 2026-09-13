import { describe, expect, it } from 'vitest'
import { displayLength, paraHtml, richText, richTextSingle } from './math'

describe('richText', () => {
  it('escapes html but keeps bold markers', () => {
    const html = richText('a<b> & **粗体** c')
    expect(html).toContain('&lt;b&gt;')
    expect(html).toContain('&amp;')
    expect(html).toContain('<strong>粗体</strong>')
  })

  it('renders inline and display math', () => {
    const html = richText('inline $x^2$ 和块级 $$y_1$$')
    expect(html).toContain('katex')
    expect(html).toContain('q-math')
  })

  it('converts newlines to <br/> in multi-line mode', () => {
    expect(richText('a\nb')).toContain('<br/>')
  })

  it('does not break newlines inside rendered math', () => {
    const html = richText('$$\\begin{cases}x\\\\ y\\end{cases}$$')
    expect(html).not.toContain('katex<br')
  })

  it('相邻 display 公式合并进一个容器（不留空行）', () => {
    const html = richText('$$a=1$$\n$$b=2$$')
    expect((html.match(/class="q-math"/g) ?? []).length).toBe(1)
    expect(html).toContain('q-math-sep')

    expect(html).not.toContain('sep/><br')
  })

  it('公式与文字相邻时不合并', () => {
    const html = richText('$$a=1$$\n普通文字\n$$b=2$$')
    expect((html.match(/class="q-math"/g) ?? []).length).toBe(2)
    expect(html).not.toContain('q-math-sep')
  })

  it('空行分隔的两个块公式保持独立（不合并）', () => {
    const html = richText('$$a=1$$\n\n$$b=2$$')
    expect((html.match(/class="q-math"/g) ?? []).length).toBe(2)
    expect(html).not.toContain('q-math-sep')

    expect(html).toContain('<br/><br/>')
  })
})

describe('richTextSingle', () => {
  it('collapses newlines to spaces', () => {
    expect(richTextSingle('a\nb')).toBe('a b')
  })

  it('renders display math inline (no q-math wrapper)', () => {
    const html = richTextSingle('$$x$$')
    expect(html).toContain('katex')
    expect(html).not.toContain('q-math')
  })

  it('bold can span a math placeholder', () => {
    const html = richTextSingle('**求 $M$**：')
    const strong = html.match(/<strong>(.*?)<\/strong>/)![1]
    expect(strong).toContain('katex')
  })
})

describe('paraHtml', () => {
  it('renders svg references as images', () => {
    const html = paraHtml('![[图 1.svg]]')
    expect(html).toContain('q-svg-wrap')
    expect(html).toContain('src="/api/assets/')
  })

  it('renders standalone $$ paragraphs as display math', () => {
    const html = paraHtml('$$x+1$$')
    expect(html).toContain('q-math')
    expect(html).toContain('katex-display')
  })

  it('renders quotes with the marker stripped', () => {
    const html = paraHtml('> 提示\n> 第二行')
    expect(html).toContain('q-quote')
    expect(html).not.toContain('&gt; ')
  })

  it('renders plain paragraphs in <p>', () => {
    expect(paraHtml('普通段落')).toMatch(/^<p>.*<\/p>$/)
  })

  it('invalid tex degrades to a code block instead of throwing', () => {
    const html = richText('$\\thisisnotacommand{$')
    expect(typeof html).toBe('string')
  })
})

describe('paraHtml 块级元素', () => {
  it('无序/有序列表', () => {
    const ul = paraHtml('- 甲\n- 乙')
    expect(ul).toContain('<ul class="md-list">')
    expect(ul).toContain('<li>甲</li>')
    const ol = paraHtml('1. 第一步\n2. 第二步')
    expect(ol).toContain('<ol class="md-list">')
    expect(ol).toContain('<li>第一步</li>')
  })

  it('表格', () => {
    const t = paraHtml('| a | b |\n|---|---|\n| 1 | 2 |')
    expect(t).toContain('<table class="md-table">')
    expect(t).toContain('<th>a</th>')
    expect(t).toContain('<td>2</td>')

    expect(paraHtml('| a | b |\n普通段落')).not.toContain('md-table')
  })

  it('代码块与行内代码', () => {
    const pre = paraHtml('```html\n<div>x</div>\n```')
    expect(pre).toContain('<pre class="md-code">')
    expect(pre).toContain('&lt;div&gt;')
    const inline = paraHtml('使用 `count()` 函数')
    expect(inline).toContain('<code class="md-icode">count()</code>')
  })

  it('斜体与链接', () => {
    const html = paraHtml('*重点* 和 [官网](https://example.com/a?b=1)')
    expect(html).toContain('<em>重点</em>')
    expect(html).toContain('<a href="https://example.com/a?b=1"')
  })

  it('公式与行内代码不受行内规则破坏', () => {
    const html = paraHtml('$a*b*c$ 与 `<b> **字** </b>`')
    expect(html).toContain('katex')

    expect(html).toContain('<code class="md-icode">&lt;b&gt; **字** &lt;/b&gt;</code>')
    expect(html).not.toContain('<strong>字</strong>')
  })
})

describe('displayLength（答案显示长度）', () => {
  it('纯文本按字符计', () => {
    expect(displayLength('42。')).toBe(3)
    expect(displayLength('一二三四五六七八九十一二三四五六七八九十一')).toBe(21)
  })

  it('LaTeX 命令与结构符不计，只计可见内容', () => {
    expect(displayLength('$\\dfrac{1}{2}$')).toBeLessThanOrEqual(3)
    expect(displayLength('$x^2+y^2$')).toBeLessThanOrEqual(6)

    expect(displayLength('$\\operatorname{sgn}(x)$')).toBeLessThanOrEqual(7)
  })

  it('加粗/行内代码标记不计', () => {
    expect(displayLength('**(A) 甲**')).toBe(5)
    expect(displayLength('`code`')).toBe(4)
  })

  it('图片嵌入不计', () => {
    expect(displayLength('![[图.svg]]')).toBe(0)
  })

  it('混合内容', () => {
    expect(displayLength('$k<0$，则 $f(x)$ 取得极大值')).toBeLessThanOrEqual(16)
  })
})
