# Erydian 多方言 ER 设计器 —— 架构方案

> 整理自 2026-09 的讨论。作为后续实现的参照蓝图，实现顺序和细节可随进度调整。

## 0. 核心原则（一段话）

**中间模型（IR）与数据库方言彻底分离**：IR 只存"设计语义"（概念级的类型、结构、关系），
方言差异全部收拢到一个 `Dialect` trait 后面。IR 与 UI 也分离：视图不感知方言，
方言只声明"我有什么"，视图统一决定"怎么画"。新增一种 RDB = 新增一个方言模块，不碰其他代码。

## 1. 总体分层

```
┌─────────────────────────────────────────────┐
│  view/          GPUI 视图：通用渲染器 + 面板  │  ← 只认识 AttrSpec，不认识具体 RDB
├─────────────────────────────────────────────┤
│  dialect/       行为层：trait Dialect        │  ← 类型映射 / DDL 生成 / 引用 / 能力
│    ├ postgres.rs │ mysql.rs │ sqlite.rs ...  │     + 各方言的 typed Props（数据）
├─────────────────────────────────────────────┤
│  model/         数据层：IR                    │  ← 纯数据，不出现 SQL 字符串
│    Table / Column / Relation / LogicalType   │
├─────────────────────────────────────────────┤
│  document/      文件格式：Document            │  ← model + layout + 元数据 + version
└─────────────────────────────────────────────┘
        ↕ sqlparser：SQL 文本 ↔ AST（外包），自己只做 AST ↔ IR
```

依赖方向：view → dialect → model → document（或 document 与 model 同层，model 是其中一部分）。
dialect 内部的 Props struct 与 model 的 extras 容器是唯一交叉点，形状松耦合。

## 2. IR：中间模型（model/）

### 2.1 逻辑类型 LogicalType

类型映射永远有损（PG `text` vs MySQL `longtext`），所以语义层存**概念**，方言层决定写法：

```rust
pub enum LogicalType {
    Integer { bits: u8, unsigned: bool },
    Decimal { precision: u8, scale: u8 },
    Varchar { len: Option<u32> },
    Text,
    Boolean,
    Date,
    Timestamp { tz: bool },
    Json,
    Uuid,
    Binary,
    Auto { inner: Box<LogicalType> },   // 自增语义
    Custom(String),                     // 映射不上的原样透传，往返不丢
}
```

`Custom(String)` 与 dialect extras 里的 `unknown` 容器是同一个思想：**解析时遇到不认识的东西
原样保留，导出时带回去**——ER 工具处理真实库导出的尊严底线。

### 2.2 Schema / Database 属性

概念错位提醒：**MySQL 的 "database/schema" 是库级对象（对标 PG 的 Database）**；
PG 的 Schema 只是库内命名空间。属性面板按这个对齐建。

```rust
pub struct SchemaProperties {
    pub name: String,
    pub owner: Option<String>,        // PG 有，MySQL database 无 → Option
    pub charset: Option<String>,      // MySQL 有，PG 无
    pub collation: Option<String>,    // MySQL 有；PG 该语义归 encoding/locale
    pub comment: Option<String>,      // 两边都有，写法不同（方言负责）
    // PG 特有（encoding、tablespace 等）→ 方言扩展属性容器（见 2.3 的 extras 模式）
}
```

### 2.3 Table：共有语义 + 方言扩展容器

**数据差异用 struct 字段，不用 trait**（见 §4 判据）。方言特有的表属性放
`TableExtras` 里每方言一个 typed struct：

```rust
pub struct Table {
    pub id: EntityId,
    pub name: String,
    pub schema_id: Option<EntityId>,
    pub comment: Option<String>,          // 行内 vs COMMENT ON 是方言的事
    pub columns: Vec<Column>,             // Vec 顺序 = DDL 列顺序（物理有序，别排序）
    pub indexes: Vec<Index>,
    pub partitioning: Option<Partitioning>,  // MySQL/PG 都支持分区 → 语义进 IR
    pub extras: TableExtras,
}

#[derive(Default, Serialize, Deserialize)]
pub struct TableExtras {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mysql: Option<mysql::TableProps>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub postgres: Option<postgres::TableProps>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unknown: Option<BTreeMap<String, PropValue>>,  // 逃生口：往返不丢
}

// dialect/mysql.rs —— 类型化，非法值解析期就报错
pub struct TableProps {
    pub engine: Option<String>,             // InnoDB / MyISAM
    pub charset: Option<String>,
    pub collation: Option<String>,
    pub row_format: Option<RowFormat>,      // 枚举而非 String
    pub auto_increment_start: Option<u64>,  // 运行时状态：UI 藏、导出默认不带
}

// dialect/postgres.rs
pub struct TableProps {
    pub owner: Option<String>,
    pub unlogged: Option<bool>,
    pub tablespace: Option<String>,
    pub fillfactor: Option<u8>,
    pub rls_enabled: Option<bool>,
}
```

### 2.4 Column

```rust
pub struct Column {
    pub id: EntityId,
    pub name: String,
    pub logical_type: LogicalType,
    pub nullable: bool,
    pub default: Option<ColumnDefault>,   // 结构化，不是字符串
    pub comment: Option<String>,
    pub collate: Option<String>,          // PG 列级 COLLATE 才是 collation 灵活处
}
pub enum ColumnDefault {                 // 语义化，方言翻译成 CURRENT_TIMESTAMP 等
    Null, Now, Sequence(String), Literal(Value), Custom(String),
}
```

unsigned / auto_increment 等列级差异走 `LogicalType`（`Integer { unsigned }` / `Auto`），不进 Table。

### 2.5 Relation（关系）

**FK 不要存进 Table**——ER 语义里外键就是画布上的线（relations），
DDL 生成时由方言物化成约束。relations 与 Table.foreign_keys 各存一份迟早打架。

```rust
pub struct Relation {
    pub id: EntityId,
    pub kind: RelationKind,              // OneToOne / OneToMany / ManyToMany
    pub source: RelationEnd,
    pub target: RelationEnd,
    pub on_delete: Option<ForeignKeyAction>,   // RESTRICT / CASCADE / SET NULL
    pub on_update: Option<ForeignKeyAction>,
    pub name: Option<String>,
}
pub struct RelationEnd {
    pub table: EntityId,
    pub columns: Vec<String>,            // 复合外键支持；**将来改为列 id 引用**（见 §6.2）
}
```

ManyToMany 存成"两条关系 + 提示"，由方言决定生成 join 表 DDL 还是只画线，
IR 不凭空造不存在的表。

## 3. 方言层（dialect/）

### 3.1 trait Dialect —— 只承载**行为**差异

同一份 IR，不同数据库产出不同的字符串/决策：

```rust
pub trait Dialect: Send + Sync {
    fn name(&self) -> &'static str;

    // 类型映射
    fn to_sql_type(&self, t: &LogicalType) -> String;
    fn from_sql_type(&self, raw: &str) -> LogicalType;

    // DDL 生成（模板方法：默认骨架，方言 override 差异点）
    fn generate_table_ddl(&self, table: &Table, model: &Model) -> String;
    fn column_definition(&self, col: &Column) -> String { /* 默认实现 */ }

    // 标识符引用：`name` vs "name" vs [name]
    fn quote_ident(&self, name: &str) -> String;

    // 能力声明 —— UI 也读它
    fn capabilities(&self) -> &Capabilities;

    // 属性面板声明（见 §5）
    fn database_attributes(&self) -> Vec<AttrSpec>;
    fn table_attributes(&self) -> Vec<TableAttrSpec>;
}

pub struct Capabilities {
    pub auto_increment: AutoIncStyle,        // Serial / Identity / AutoIncrement / None
    pub inline_comment: bool,                // MySQL 行内 COMMENT vs COMMENT ON
    pub supports_enum: bool,
    pub supports_index_type: bool,           // USING btree/hash
    pub supports_on_update_timestamp: bool,
}
```

示例（模板方法）：

```rust
// 默认骨架（trait 内）
fn column_definition(&self, col: &Column) -> String {
    format!("{} {}{}{}", self.quote_ident(&col.name),
        self.to_sql_type(&col.logical_type), /* nullable, default, autoinc */)
}
// MySQL override：追加 COMMENT '...'；PG override：自增用 GENERATED AS IDENTITY
```

### 3.2 SQL 解析：外包 sqlparser

`sqlparser 0.54` 自带各家方言解析器。分工：**它负责 SQL 文本 → AST（含各家语法差异），
我们只做 AST → IR 的语义映射（含各家类型/特性差异）**。

```rust
use sqlparser::parser::Parser;
// trait Dialect → sqlparser dialect 的映射函数

pub fn parse_ddl(sql: &str, dialect: &dyn Dialect) -> anyhow::Result<Model> {
    let stmts = Parser::parse_statements(sql, &*sql_dialect_of(dialect))?;
    // Statement::CreateTable / AlterTable / ... → IR
    // data_type 字符串经 dialect.from_sql_type() 转 LogicalType
    // 认不出的选项进 extras.unknown / LogicalType::Custom
}
```

### 3.3 SQL 生成：手写 emitter

不用 sqlparser 的 AST 逆序列化（注释、IF NOT EXISTS、排版控制力不够）。
模板方法 + 方言 hook（§3.1）。

### 3.4 落地节奏

先 **Postgres + MySQL** 两个（差异最有代表性：引号、注释、自增、类型名全覆盖），
`GenericDialect` 兜底。新增 RDB = 新文件 + 注册进静态表，不碰其他代码。

## 4. 数据差异 vs 行为差异 —— trait 出现的位置

> **同一种操作，不同数据库有不同做法 → trait 方法；
> 不同数据库拥有不同的字段 → struct 字段（分容器装）。**

| 对象 | 判定 | 原因 |
|---|---|---|
| `Table` + `TableExtras` | 数据 | 属性记录，MySQL/PG 字段集不同 |
| `Dialect` trait | 行为 | to_sql_type / quote_ident / DDL 生成：同一操作不同实现 |
| `LogicalType` enum + `Custom` | 数据 | 类型是值；`Custom("JSONB")` 兜底 |
| `AttrSpec` / `TableAttrSpec` | 数据的描述 | 方言返回"我有哪些属性"，UI 统一渲染 |
| 将来的校验规则（如 PG 列名 63 字符） | 行为 | `fn validate(...) -> Vec<Diagnostic>`，各库各实现 |

反模式备忘：给 `Table` 挂 `TableBehavior` trait、为方言造 `MySqlTable`/`PgTable` 类型
→ 类型爆炸，存储/serde/遍历全要 enum 包一层，只为省几个字段访问。IR 保持"哑数据"。

## 5. 属性面板：声明式，渲染方法不进 trait

**为什么渲染不进 Dialect trait**：
1. 依赖方向——方言层一旦 import gpui 组件，headless 导出 CLI、方言单测全被拖累；
2. 重复的正是最坏的那种重复——五份几乎一样的表单样板，UI 改布局动五个文件；
   方言间真正的差异（字段本身）反被淹没；
3. 渲染没法快照测试，DDL 生成可以。

拆法：**方言声明"有什么"（数据），视图统一画"怎么画"（写一次）**。

```rust
pub enum AttrKind {
    Text { default: Option<String> },
    Select { options: Vec<String> },          // charset / encoding 下拉
    Bool,
    Collation { charset_key: &'static str },  // 联动 charset 的特殊控件
}
pub struct AttrSpec {
    pub key: &'static str,        // "charset" / "encoding"
    pub label: &'static str,
    pub kind: AttrKind,
    pub required: bool,
}
```

各方言只剩差异本身（几行数据声明，不是样板重复）：

```rust
impl Dialect for MySqlDialect {
    fn database_attributes(&self) -> Vec<AttrSpec> {
        vec![name(), comment(),
             AttrSpec::select("charset", "Character Set", CHARSETS),
             AttrSpec::collation("collation", "Collation", "charset")]
    }
}
impl Dialect for PostgresDialect {
    fn database_attributes(&self) -> Vec<AttrSpec> {
        vec![name(), owner(), comment(),
             AttrSpec::select("encoding", "Encoding", PG_ENCODINGS),
             AttrSpec::text("lc_collate", "LC_COLLATE")]
    }
}
```

Table 级属性是**存储数据**（进 Document、参与 DDL），用带 get/set 闭包的
`TableAttrSpec`：闭包在方言模块里写（那里有 TableProps 完整类型），
UI 全程只见字符串，枚举解析和 None 处理由方言闭包承担。

```rust
fn table_attributes(&self) -> Vec<TableAttrSpec> {
    vec![
        spec_select("engine", "Engine", ENGINES,
            |t| t.extras.mysql.as_ref().and_then(|p| p.engine.clone()),
            |t, v| t.extras.mysql_mut().engine = v),
        ...
    ]
}
```

先例：gpui-component 的 settings 面板（SettingGroup / SettingField::switch(getter, setter)）
就是这个模式，结构感直接可借。

特殊控件逃生口（如 PG tablespace 需查系统目录才能下拉）：视图侧按 `dialect.name()`
注册自定义渲染函数的 map——隔离在视图层，不污染 trait。99% 走通用路径。

## 6. Document：文件格式（document/）

### 6.1 顶层结构：三部分平级

```rust
pub const DOCUMENT_FORMAT_VERSION: usize = 1;

#[derive(Serialize, Deserialize)]
pub struct Document {
    pub version: usize,                    // 格式版本，迁移的钥匙
    pub name: String,                      // "订单系统.er"
    pub origin_dialect: Option<String>,    // 建模时的方言提示，如 "postgres"
    pub created_at: i64,
    pub updated_at: i64,
    #[serde(default)]
    pub model: Model,                      // 设计语义（删不得）
    #[serde(default)]
    pub layout: Layout,                    // 画布呈现（删了也不丢设计）
}

pub struct Model {
    pub schemas: Vec<Schema>,              // 你正在做的 database 属性在这
    pub tables: Vec<Table>,
    pub relations: Vec<Relation>,
}

pub struct Layout {
    pub entities: HashMap<EntityId, EntityLayout>,  // model 有而 layout 无 → 默认布局
}
pub struct EntityLayout {
    pub x: f32, pub y: f32,
    pub z: Option<u32>,
    pub collapsed: Option<bool>,
    pub color: Option<String>,             // 用户手动分组色
}
```

model/layout 分离的理由：两者生命周期不同。从 SQL 导入 / 文档合并 / AI 生成实体
只动 model；layout 取不到就自动布局。将来"导出为无坐标的纯 schema"也免费。

### 6.2 稳定性要点

- **id**：ULID / uuid v7（时间有序、git diff 友好），别用自增数字（合并冲突）。
  **列也用 id 引用**（当前 RelationEnd.columns 存列名是过渡——重命名列是高频操作，
  存 id 天然稳定，名字只在表定义存一份；实现初期可先列名，重构要趁早）。
- **序列化顺序**：保存前 Vec 按 id 排序（列除外，保持物理顺序），
  同一设计两次保存产出逐字节相同的文件——对用 git 管理 .er 文件的用户是刚需。
- **serde 宽容**：所有新增字段 `#[serde(default)]`，可选项
  `skip_serializing_if = "Option::is_none"`（先例：gpui-component DockAreaState 的
  version + default + skip 模式）。
- **版本迁移**：读文件时 `version < DOCUMENT_FORMAT_VERSION` 就链式执行
  `fn migrate_v1_to_v2(...)`，每个版本一段。
- **原子保存**：写临时文件 + rename（AppSettings 那种直接 fs::write 对 Document
  不够——崩一半毁整份设计）。加 `.auto_save` + 最近 N 版历史目录，成本低救急价值高。
- 单文件 JSON（pretty），扩展名如 `.erydian`；将来换 toml/bincode 不动 struct。

### 6.3 全局状态接线（GPUI 侧）

当前方言放 GPUI Global（与 AppSettings 同级）；方言选择器、类型下拉、导出菜单都读它。
Document 实体本身建议 Entity<Document> + 写穿保存（参考 settings 的即时保存模式）。

## 7. 实现顺序建议

1. `model/`：EntityId、LogicalType、Schema 属性（当前任务）、Table/Column 最小版
2. `dialect/`：trait + MySQL + Postgres（Capabilities / 类型映射 / quote_ident）
3. DDL 生成 emitter（含快照测试：固定 IR → golden 文件）
4. 属性面板：AttrSpec + 通用渲染器（Schema 属性面板练兵）
5. Document 序列化 / 加载 / 版本迁移 + 原子保存
6. 解析：sqlparser 接入，AST → IR（含 extras.unknown / Custom 兜底）
7. Relation / 画布 / 反向解析往返测试（parse → generate 往返不丢）

测试策略：方言快照测试（IR → DDL golden 文件）+ 往返测试（SQL → IR → SQL 结构等价）。
这些测试是日后改 emitter 安全感的来源。
