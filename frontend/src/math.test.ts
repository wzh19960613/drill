import { describe, expect, it } from 'vitest'
import { paraHtml, richText, richTextSingle } from './math'

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
