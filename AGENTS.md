# AGENTS.md — 项目约定

## CSS 规范

- **全局禁止使用 `position: fixed`**(包括探针、吸底条、遮罩等一切场景)。
  全屏遮罩与吸顶/吸底一律用 `position: absolute`(配合高层滚动容器)或
  `position: sticky` 实现;需要"覆盖整个视口"的元素挂在 `document.body`
  下的第一层,用 `absolute; inset: 0`。
- **全局禁止 `height: 100%`(以及任何百分比高度链)**,严重性与
  `position: fixed` 同级。百分比高度的解析基准在 iOS PWA、安全区、
  flex 嵌套等场景下不可靠(曾导致底部空白带、底栏被挤出视口、
  内容沉底等多起问题)。**占满视口一律用 `position: absolute;
  inset: 0`**(钉在初始包含块的几何边界上,不依赖任何高度单位;
  任何视口高度单位都不可靠:`100%` 链欠填、`100dvh` 在 WebKit
  standalone 不含底部安全区——实测 1194pt 视口只给 1160、
  `-webkit-fill-available` 无效)。本项目 html/body/#app/.app 四层
  即此写法。
  占满父容器用 flex(`flex: 1`,必要时配 `min-height: 0`);
  进度条填充等场景直接写字面高度,不引用父元素。
- 布局尺寸(内外边距、宽高、间距、字号、行高、圆角)一律用 `rem`;
  `px` 仅允许用于:**≤4px 的光学微调量**(发丝边框、1–4px 的
  margin/padding/圆角/间距等,这些不随字号缩放)、阴影、以及必须对齐
  屏幕物理区域的避让量(如 macOS 红绿灯避让)。
- 响应式断点用**容器查询**(`@container`),不用媒体查询:容器查询的
  `rem` 跟随应用字号设置,媒体查询不跟随。宽、高断点统一查询 `html`
  尺寸容器(`html { height: 100%; container-type: size; }`,见
  style.css)。例外:`html.pwa-pad` 的规则(选择器落在 html 自身,容器
  查询选不到容器自己,只能保留媒体查询)。
- **禁止对样式做任何形式的批量(脚本/正则)替换**,包括但不限于 px→rem、
  px→pt、颜色值替换、选择器重命名。历史教训:一次 px→rem 批量转换制造了
  属性后缀丢失(`border-bottom-width`→`border-bottom`、`margin-left`→
  `margin`、`border-top-*-radius`→`border-top`、`padding-bottom`→
  `padding`)、类名截断(`.bottombar`→`.bottom`)等十余处静默破坏,部分
  数天后才暴露。样式修改必须**逐处手工编辑**,改完全量目检受影响界面。
- **lucide 图标尺寸不能用 `:size` 数字**(按 px 渲染不缩放),也不能用
  `width`/`height` 属性透传(组件模板对这两个属性有显式绑定,外部
  attrs 覆盖不了)。正确做法:内联样式且单位用 **rem**(任意档位,
  如 `style="width: 0.875rem; height: 0.875rem"`),或组件内
  `:style="{ width: size, height: size }"`(size 必须是 rem 值)。
- 颜色/阴影/圆角/层级/等宽字体优先用 style.css `:root` 里的设计
  token(`--brand-weak`、`--shadow-modal`、`--radius-pill`、`--z-modal`
  等);新弹窗遮罩/外壳复用全局 `.modal-mask` / `.dialog` 类,不要
  手抄配方。
  
## 测试规范

- 所有验证不得污染真实数据:写操作只允许针对临时创建、验证完即删的
  题本;优先使用只读接口(如 PDF preview)。
- 前端逻辑改动补充 `vitest` 单测(`npm test`);后端保持 `cargo test`。

