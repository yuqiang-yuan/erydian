# Erydian 开发笔记（GPUI / gpui-kit）

本项目基于 gpui-kit（GPUI + gpui_kit::component / base / assets）开发。以下是开发过程中积累的关键概念和模式，供后续参考。

> 遇到不确定的 API，可去 `/home/yuanyq/work/opensource/gpui-component` 下查找 gpui-kit 的示例代码。

---

## 1. 文件对话框

| 需求 | API | 返回类型 |
|---|---|---|
| **保存文件**（输入新路径） | `cx.prompt_for_new_path(dir, suggested_name)` | `oneshot::Receiver<Result<Option<PathBuf>>>` |
| 打开/选择已有文件 | `cx.prompt_for_paths(PathPromptOptions)` | `oneshot::Receiver<Result<Option<Vec<PathBuf>>>>` |

两者都是**异步**的，需配合 `cx.spawn_in(window, async move |this, cx| {...}).detach()`。

`PathPromptOptions` 字段：`files` / `directories` / `multiple` / `prompt`。**没有文件类型过滤器**（要类型过滤需引入 `rfd` crate）。

异步等待后要更新 view，必须用 `this.update_in(window, |this, window, cx| {...})` 拿回 `&mut self`：

```rust
cx.spawn_in(window, async move |this, cx| {
    let path = path.await.ok().flatten().flatten().and_then(|mut v| v.pop());
    let Some(path) = path else { return; };
    this.update_in(window, |this, window, cx| {
        this.save_file(path, window, cx);
    }).ok();
}).detach();
```

### `prompt_for_paths` 返回值层层剥开

```
.await      -> Result<Result<Option<Vec<PathBuf>>>, RecvError>
.ok()       -> Option<Result<Option<Vec<PathBuf>>>>
.flatten()  -> Option<Option<Vec<PathBuf>>>
.flatten()  -> Option<Vec<PathBuf>>        // None = 用户取消
```

---

## 2. 菜单 Action 派发机制（基于焦点）

`AppMenuBar` / `PopupMenu` 的 action 派发是**焦点上下文派发**：

1. 打开菜单瞬间：`action_context = window.focused(cx)`（记录当前焦点）
2. 确认菜单项时：先把焦点还原给 `action_context`，再 `window.dispatch_action(action, cx)`

`dispatch_action` 是**向当前焦点元素派发，沿 DOM 树冒泡**。命中 `.on_action(...)` 注册的祖先即处理。

**两个条件同时满足，action 才能派发到：**
- 焦点元素仍然有效（还在 DOM 树里，没被卸载）
- 从焦点元素冒泡能到达注册了 `.on_action` 的祖先

### 焦点悬空（dangling）问题

被切走的场景里如果持有焦点（Input/Select/Checkbox/Button 等任何 `track_focus` 的组件），场景卸载后焦点悬空，菜单 action 派发链断裂 → 菜单不工作。

**判断标准：被卸载的场景有没有正持有焦点**。有则切场景后必须重新 focus。

### 首帧焦点兜底

```rust
let focus_handle_clone = focus_handle.clone();
window.defer(cx, move |window, cx| {
    if window.focused(cx).is_none() {
        focus_handle_clone.focus(window, cx);
    }
});
```

`window.defer` 在**当前帧渲染提交之后**执行，此时 `.track_focus` 已生效。`if window.focused(cx).is_none()` 兜底：没有焦点时才抢。

### 场景切换后必须重新 focus

```rust
fn on_schema_created_action(&mut self, ..., cx: &mut Context<Self>) {
    self.scene = Scene::Editor;
    cx.notify();
    self.focus_handle.focus(window, cx);  // 复用已有的 handle，不要新建
}
```

**关键**：用 `new()` 时创建并经 render 里 `.track_focus` 绑定的那个 handle（永在 DOM 的容器），**不要用 `cx.focus_handle()` 新建**。

---

## 3. FocusHandle

`FocusHandle` 是个轻量级"焦点标识"（id + 引用计数），不是元素本身：

```rust
pub struct FocusHandle {
    id: FocusId,
    tab_index: isize,
    tab_stop: bool,
    handles: Arc<FocusMap>,
}
```

### `FocusHandle::focus` 的本质

```rust
pub fn focus(&self, window: &mut Window, cx: &mut App) {
    window.focus(self, cx);
}
```

`Window::focus` 做的事：
1. 守卫：焦点被禁用或已是这个 handle → 直接返回
2. `self.focus = Some(handle.id)`（设焦点 id）
3. `focus_generation += 1`（版本号，判断焦点是否变化）
4. 清待按键 + `refresh()`（重绘 focus ring）

**`focus()` 只是写一个焦点 id，真正的元素关联靠 `track_focus` 在渲染期建立。** focus 要生效，handle 必须被 track 到一个还在 DOM 里的元素上。

### 三种情况对比

| 情况 | handle 有效? | 元素在 DOM? | focus 结果 |
|---|---|---|---|
| 正常可聚焦元素 | ✅ | ✅ | 有效 ✅ |
| `cx.focus_handle()` 新建但没 track | ✅ | ❌ | 冒泡链空 ❌ |
| 被卸载的元素 | ✅ | ❌ | 悬空 ❌ |
| 永在 DOM 的容器 | ✅ | ✅ | 永远有效 ✅ |

### 相关方法

| 方法 | 作用 |
|---|---|
| `handle.focus(window, cx)` | 设焦点到这个 handle |
| `handle.is_focused(window)` | 是不是当前焦点 |
| `handle.contains_focused(window, cx)` | 子树是否包含当前焦点 |
| `window.focused(cx)` | 拿到当前焦点 handle（可能 None） |
| `window.blur(cx)` | 取消所有焦点 |

---

## 4. 'static 闭包与 move + clone 模式

gpui 里所有"以后才执行"的东西——`cx.listener`、`on_click`、`cx.spawn`/`cx.spawn_in` 的 async block、`defer` 回调——**闭包都必须 `'static`**。

### `'static` 的真正含义

闭包不借用任何"活不过它自己"的东西。满足途径：
- **own（move）**：搬进来，没有引用了，自然满足
- **借 `'static` 的数据**：借用目标本身 `'static`（如静态变量、字面量），借用也 `'static`
- **借短命数据**：❌ 不满足

### move + clone 的分工

| | 职责 |
|---|---|
| 外层 `move \|...\|` | 把变量按值搬进 `'static` 闭包，解决生命周期（让闭包 `'static`） |
| 闭包内 `.clone()` | 产出多个副本供多次使用，解决"一个值要用 N 次"（不消耗原值） |

**两者职责不同，缺一不可。** 只写 `.clone()` 不写 `move`，闭包仍不是 `'static`。

```rust
// ✅ 正确模式
.on_click(cx.listener(move |this, _, window, cx| {
    window.dispatch_action(Box::new(FileOpenedAction {
        path: path_clone.clone(),  // clone 解决多次触发
    }), cx);
}))
```

### `Clone` vs `Copy`

- `Clone::clone(&self)` 只借用不消耗，可反复调用
- `Copy` 是隐式复制，赋值/传参时自动 copy

gpui 的 `Font` 是 `Clone` 不是 `Copy`（`#[derive(Clone, Debug, Eq, PartialEq, Hash)]`）。所以循环里要用 `font.clone()`：

```rust
move |bounds, window, cx| {
    for table in ... {
        let runs = vec![TextRun {
            font: font.clone(),  // clone 只借用，font 还在，下轮还能用
            ...
        }];
    }
}
```

### 判断类型是否 Copy

- 试探法：加 `#[derive(Copy, Clone)]`，编译器会指出哪个字段阻止 Copy
- 规律：整数/浮点/bool/char/`&T`/元组（若元素都 Copy）是 Copy；`String`/`Vec`/`Box`/`PathBuf`/`&mut T`/struct（未 derive）都不是 Copy

---

## 5. Entity<T> vs Option<Entity<T>> vs Entity<Option<T>>

### 核心判断标准

**这个值需不需要被"在它不存在时"还持有并观察？**

| | `Option<Entity<T>>` | `Entity<Option<T>>` |
|---|---|---|
| Entity 可能根本不存在？ | ✅ | ❌ Entity 一直存在 |
| 存在之后基本不变？ | ✅ | 不必要 |
| 内部值会频繁变化（None↔Some）？ | ❌ | ✅ |
| 需要在"不存在"状态下仍被 observe？ | ❌ | ✅ |
| 创建需要专门的 `Context<T>`？ | ✅ `cx.new` 直接给 | ❌ update 闭包给的是 `Context<Option<T>>` |

### 实例

- **`EditorView`**（可能不存在、一旦存在就稳定、创建需 `Context<EditorView>`）→ `Option<Entity<EditorView>>`
- **选中 id**（会变 None↔Some(a)↔Some(b)、要被多个 view observe）→ `Entity<Option<String>>`

### 创建需要 `Context<T>` 的 view

`EditorView::new` 需要 `&mut Context<EditorView>`，**唯一能拿到它的地方是 `cx.new(|cx| ...)` 的闭包**：

```rust
// ✅ 正确：cx.new 的闭包参数是 &mut Context<EditorView>
let editor = cx.new(|cx| EditorView::new(schema, window, cx));
self.editor_view = Some(editor);

// ❌ 错误：Entity<Option<EditorView>> 的 update 闭包给的是 Context<Option<EditorView>>
self.editor_view.update(cx, |view, cx| *view = Some(EditorView::new(..., &mut *cx)));
```

---

## 6. update entity 后必须 notify

GPUI 是响应式：**改值不通知 = 没人知道**。

```rust
selected_id.update(cx, |id, cx| {       // 闭包第二个参数是 entity 自己的 Context
    *id = Some(table_id);
    cx.notify();                         // ← 必须用这个 cx notify，才会通知 selected_id 的 observer
});
```

### notify 的核心规则

**`cx.notify()` 只通知"这个 cx 所属的那个 entity 的 observer"。**

| 想通知谁 | 用哪个 cx |
|---|---|
| 观察 `selected_id` 的 view | `selected_id.update(cx, \|_, cx\| cx.notify())` 里的 cx |
| 观察 `EditorView` 的 view | `EditorView` 方法里的 `cx.notify()` |
| 观察 `schema` 的 view | `schema.update(cx, \|_, cx\| cx.notify())` 里的 cx |

**`update` 闭包的第二个参数就是被 update entity 自己的 Context，不要用 `_` 忽略它。**

### 什么时候不需要 notify

| 场景 | 要 notify? |
|---|---|
| 事件回调（click/drag/action）里改 entity | ✅ 要 |
| 渲染期（canvas prepaint/paint 回调）改 entity | ❌ 不要（会重渲染循环） |

---

## 7. observe vs observe_in

### `cx.observe`（没有 window）

```rust
pub fn observe<W>(
    &mut self,
    entity: &Entity<W>,
    on_notify: impl FnMut(&mut T, Entity<W>, &mut Context<T>) + 'static,
) -> Subscription
```

闭包**没有 window 参数**。适合只读状态、更新自身字段。

### `cx.observe_in`（有 window）

```rust
pub fn observe_in<V2>(
    &mut self,
    observed: &Entity<V2>,
    window: &mut Window,
    on_notify: impl FnMut(&mut T, Entity<V2>, &mut Window, &mut Context<T>) + 'static,
) -> Subscription
```

闭包**有 window 参数**。适合需要在回调里用 window（如 `cx.new` 创建需要 Window 的 view）。

### observe 触发时序

**observe 回调一定在 render 之前执行**（同一轮事件循环里，notify 先触发 observer、后触发重绘）。所以 observe 回调里创建的 view，在同轮 render 里一定能读到。

### Subscription 必须持有

`observe`/`observe_in` 返回 `Subscription`，一旦 drop 观察就失效。必须存到结构体字段：

```rust
_subscriptions: Vec<Subscription>,
```

### 在 observe 回调里创建 view

```rust
let sid_sub = cx.observe_in(&sid, window, |this, ent, window, cx| match ent.read(cx) {
    Some(SelectedItem::Table(_)) if this.table_detail_view.is_none() => {
        this.table_detail_view = Some(cx.new(|cx| {
            TableDetailView::new(this.schema.clone(), this.selected_item.clone(), window, cx)
        }));
    }
    _ => {}
});
```

---

## 8. 构造 view 时是否需要 window

**取决于构造函数里干不干"碰窗口"的事：**

| 构造时要做的事 | 需要 window? |
|---|---|
| 测量字体/文本（`window.text_system()`） | ✅ |
| 设置光标、焦点 | ✅ |
| 弹出浮层（popover/date picker） | ✅ |
| 注册窗口级监听（`observe_in`、`observe_window_bounds`） | ✅ |
| 纯数据初始化 | ❌ |
| 只是 `cx.new(...)` 创建子 entity | ❌ |
| 只是 `cx.observe(...)`（不带 window） | ❌ |

gpui 按需传参：简单 view 签名干净，需要窗口的明确表态。代价是构造链上一旦有需要 window 的，上层都得传。

---

## 9. Canvas 绘制

### paint_quad vs ShapedLine::paint

| 方法 | 需要 `&mut App`? |
|---|---|
| `window.paint_quad(...)` | ❌ 只要 `&mut Window` |
| `window.paint_glyph(...)` | ❌ 只要 `&mut Window` |
| `ShapedLine::paint(...)` | ✅ 要 `&mut App` |

`ShapedLine` 的 `layout` 字段是 `pub(crate)`，外部拿不到字形布局，无法绕开 `ShapedLine::paint` 自己逐个 `paint_glyph`。

### 借用冲突：read schema + paint 需 &mut App

canvas paint 回调里读 `schema.read(cx)`（`&App`）和 `shaped.paint(..., &mut App)` 冲突。**解法：两阶段 + 轻量收集**——只提取 paint 需要的最小字段，read 借用结束后再 paint：

```rust
// 阶段一：read 借用内，只提取 paint 需要的轻量数据
struct PaintItem { top_left: Point<Pixels>, size: Size<Pixels>, name: String }
let items: Vec<PaintItem> = schema.read(cx).tables.iter()
    .filter_map(|t| {
        let g = t.graph.as_ref()?;
        Some(PaintItem { top_left: point(...), size: size(...), name: t.name.clone() })
    }).collect();
// read 借用结束

// 阶段二：paint，需要 &mut App，此时已无 read 借用
for item in &items {
    window.paint_quad(...);
    shaped.paint(..., &mut *cx);
}
```

只 clone 一个 `String`（name），不 clone 整个 `TableSpec`（含 `Vec<ColumnSpec>`、`BTreeMap`）。

---

## 10. children 接收 Option

`children` 签名（`element.rs:202`）：

```rust
fn children(self, children: impl IntoIterator<Item = impl IntoElement>) -> Self
```

收的是 `IntoIterator`，而 **Rust 标准库的 `Option<T>` 实现了 `IntoIterator`**（None→空迭代器、Some(x)→单元素迭代器）。所以：

```rust
.children(self.table_detail_view.clone())  // Option<Entity<TableDetailView>>
// None → 0 个子元素；Some(entity) → 1 个子元素
```

trait 链：`Option<Entity<T>>` → `IntoIterator`（标准库）→ `Item = Entity<T>` → `IntoElement`（`Entity<V: Render>`）。

这是表达"可选子视图"最简洁的方式，不用写 `if let Some`。对比 `child`（收单个 `IntoElement`，不能直接收 `Option`）。

---

## 11. FluentBuilder: when / when_some / when_else

定义在 `gpui-pre-0.3.2/src/util.rs`：

```rust
// 传 bool，闭包拿不到 Option 里的值
fn when(self, condition: bool, then: impl FnOnce(Self) -> Self) -> Self

// 传 Option<T>，闭包能拿到解包后的 T
fn when_some<T>(self, option: Option<T>, then: impl FnOnce(Self, T) -> Self) -> Self

// if/else
fn when_else(self, condition: bool, then, else_fn) -> Self
```

- **`when`** 收 `bool`，不能传 `Option`
- **`when_some`** 收 `Option<T>`，闭包里能用解包的 `T`
- 可选子元素用 `children(Option)` 比 `when_some` 更简洁；Some 时还要加包裹/样式则用 `when_some`

风格：`len() > 0` → `!is_empty()`（Clippy 也建议）。

---

## 12. Eq vs PartialEq

| | `PartialEq` | `Eq` |
|---|---|---|
| 有方法? | ✅ `eq`/`ne` | ❌ 空标记 |
| 要求自反性（`a == a`）? | ❌ | ✅ |
| `f32`/`f64` | ✅ 实现 | ❌ 不实现（NaN != NaN） |
| 可做 HashMap key? | ❌ | ✅（还需 Hash） |

`Eq: PartialEq`，实现 `Eq` 必须先实现 `PartialEq`。`#[derive(Eq)]` 只在所有字段都 `Eq` 时才能编译通过。

gpui 的 `Font` 派生了 `Eq + PartialEq + Hash`（`#[derive(Clone, Debug, Eq, PartialEq, Hash)]`），所以能做 HashMap key（字体缓存）。

---

## 13. 常用模式速查

### `'static` 闭包捕获外层变量

```rust
// 外层先 clone 得到 owned，内层 move 搬入，运行时再 clone 复用
let path_clone = f.path.clone();
.on_click(cx.listener(move |this, _, window, cx| {
    let cloned = path_clone.clone();
    window.dispatch_action(Box::new(SomeAction { path: cloned }), cx);
}))
```

### 选中状态联动

```rust
// A view：改选中 id
selection.update(cx, |id, cx| {
    *id = Some(table_id);
    cx.notify();   // 通知 observer
});

// B view：observe 选中 id
let _sub = cx.observe(&selected_id, |this, entity, cx| {
    this.highlighted = entity.read(cx).clone();
    cx.notify();
});
```

### 渲染期 update 不要 notify

canvas prepaint/paint 回调里改 entity 是渲染流程内部的即时计算，调 notify 会触发重渲染循环。

### `Entity` clone 廉价

`Entity<T>` clone 是引用计数 +1，非常廉价。解借用冲突的标准手法：**先 clone Entity，再 update**。

---

## 14. f32 注意事项

- 范围：`f32::MAX` ≈ 3.4e38，精度约 6-7 位有效数字
- `NaN != NaN`，不用 `==` 比较，用 `.is_nan()`
- 溢出变 `±INFINITY`，不 panic
- 金额/ID 等高精度场景用 `f64` 或整型，不用 `f32`
- gpui 的 `px` 是 `f32`，UI 尺寸用 `f32` 足够

---

## 15. font_size 的含义

`font_size` 是字体**高度维度**的量——对应 **1 em** 的尺寸，是字体的缩放基准。字符的实际宽度和高度各不相同，都不直接等于 `font_size`。

gpui 的字符宽度需单独查询：`em_width`（'m' 宽）、`ch_width`（'0' 宽）、`advance`（任意字符推进宽度）。

---

## 16. Window 是什么

`Window` 是 gpui 对"一个操作系统窗口"的完整抽象——它把"一块屏幕上的窗口"和"gpui 用来渲染它的所有状态"打包在一起。结构体定义在 `window.rs:1134`，内部字段可分几类：

### 平台窗口（OS 层）

```rust
platform_window: Box<dyn PlatformWindow>,   // 操作系统窗口句柄
display_id: Option<DisplayId>,
is_resizable: bool,
is_minimizable: bool,
viewport_size: Size<Pixels>,
bounds_observers: ...,
```

`platform_window` 是 trait 对象，背后是各平台具体实现（macOS 的 NSWindow、Linux 的 Wayland/X11 surface、Windows 的 HWND）。这是 Window 的"物理根基"——没有它就没有真正的窗口。

### 渲染状态（绘制层）

```rust
sprite_atlas: Arc<dyn PlatformAtlas>,       // 字形/图片图集
text_system: Arc<WindowTextSystem>,          // 文字系统
layout_engine: Option<TaffyLayoutEngine>,   // 布局引擎
rendered_frame: Frame,                       // 已渲染的帧
next_frame: Frame,                           // 下一帧
needs_present: Rc<Cell<bool>>,               // 是否需要呈现
scale_factor: f32,                           // 缩放因子（HiDPI）
```

这些是"怎么画"——图集缓存、布局计算、双帧缓冲。

### 元素树与渲染上下文（绘制时的栈）

```rust
root: Option<AnyView>,                       // 根视图
element_id_stack: SmallVec<[ElementId; 32]>, // 元素 id 栈（render 时构建）
text_style_stack: Vec<TextStyleRefinement>,  // 文本样式栈
content_mask_stack: Vec<ContentMask<Pixels>>, // 内容遮罩栈
element_offset_stack: Vec<Point<Pixels>>,     // 元素偏移栈
element_opacity: f32,                         // 当前元素透明度
```

这些是 render/paint 时的"渲染上下文"——canvas 回调、`paint_quad` 里拿到的 `&mut Window` 能画东西、算布局、压栈，就是因为 Window 自己持有这些栈。

### 焦点与输入

```rust
focus: Option<FocusId>,                       // 当前焦点 id
focus_enabled: bool,
mouse_position: Point<Pixels>,
mouse_hit_test: HitTest,
modifiers: Modifiers,
focus_listeners: SubscriberSet<...>,
```

`window.focused(cx)`、`window.focus(handle, cx)` 能用，就是因为焦点状态在 Window 上。

### 观察者集合

```rust
bounds_observers, appearance_observers, activation_observers, focus_lost_listeners, ...
```

`cx.observe_window_bounds(window, ...)` 能工作，就是因为观察者注册在 Window 上。

### 一句话

**Window = OS 窗口句柄 + 渲染状态 + 渲染上下文栈 + 焦点/输入状态 + 观察者集合。** render/paint 回调里的 `&mut Window` 就是这些状态的"门面"。

---

## 17. Window 是如何创建的

入口 `cx.open_window`（`app.rs:1266`）：

```rust
pub fn open_window<V: 'static + Render>(
    &mut self,
    options: crate::WindowOptions,
    build_root_view: impl FnOnce(&mut Window, &mut App) -> Entity<V>,
) -> anyhow::Result<WindowHandle<V>>
```

### 五个关键步骤

**1. 分配 id + 创建物理窗口**

```rust
let id = cx.windows.insert(None);
Window::new(handle.into(), options, cx)
```

`Window::new` 通过平台层（`PlatformWindow`）真正创建 OS 窗口（NSWindow / Wayland/X11 surface / HWND）。`options`（`WindowOptions`）决定大小、装饰、标题栏等。

**2. 调你的闭包建根视图**

```rust
let root_view = build_root_view(&mut window, cx);
```

这就是 `main.rs` 里传的闭包：

```rust
cx.open_window(options, |window, cx| {
    let view = cx.new(|cx| MainView::new(window, cx));   // 建 MainView
    // ... defer focus 等
    cx.new(|cx| Root::new(view, window, cx))            // 包成 Root
})
```

此时 `window` 已创建好但还没渲染，你的闭包用它建根视图。**这也是为什么 `MainView::new` 能拿到 `window: &mut Window`——就是这里传进去的。**

**3. 根视图设进 Window**

```rust
window.root.replace(root_view.into());
```

Window 持有根视图的 `AnyView`。之后渲染时从 `window.root` 开始递归 render。

**4. 先画一帧**

```rust
let clear = window.draw(cx);
```

强制先画一帧——Windows 平台上常"抢不到第一帧"导致返回从没渲染过的窗口而 crash（`DispatchTree::root_node_id` 断言空节点）。

**5. 注册到 app，返回 handle**

```rust
cx.window_handles.insert(id, window.handle);
cx.windows.get_mut(id).unwrap().replace(Box::new(window));
Ok(handle)
```

Window 用 `Box` 装起来存进 app 的 `windows` 表。返回的 `WindowHandle<V>` 是轻量句柄，后面用它操作窗口。

### 层级关系

```
App
 ├─ windows: SlotMap<WindowId, Box<Window>>   ← 所有窗口的存储
 │    └─ Window
 │         ├─ platform_window: OS 窗口句柄（NSWindow / Wayland surface / HWND）
 │         ├─ root: AnyView（根视图，如 Root<MainView>）
 │         ├─ 渲染状态（Frame、布局引擎、图集）
 │         ├─ 焦点/输入状态
 │         └─ 观察者集合
 └─ window_handles: WindowId → AnyWindowHandle
```

- **App** 是全局，持有所有 Window
- **Window** 是单个窗口的完整状态（OS 句柄 + 渲染 + 焦点 + 输入）
- **WindowHandle<V>** 是 Window 的轻量引用，跨 view 传递用它

### 为什么 render 里能直接用 `&mut Window`

`MainView::render(&mut self, window: &mut Window, cx: &mut Context<Self>)` 拿到的 `window`，是 App 渲染该 view 所属窗口时借给你的当前窗口。渲染流程：

```
App 选中某个 Window 要重绘
  → window.draw(cx)
    → 从 window.root 开始递归 render
      → 每个元素的 render(&mut self, window: &mut Window, cx)
        → 用 window.paint_quad / window.text_system / cx.theme 等
```

`window` 不是你创建的，是 gpui 渲染管线调你的 render 时"借给你"的当前窗口。你在里面画的东西，最终通过 Window 的双帧缓冲呈现到 OS 窗口上。
