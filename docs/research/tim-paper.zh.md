# Trade Intent Models：跨市场交易与 Agent 策略执行的语义接口

Bill Sun and Alpha.Dev team (https://alpha.dev/)

## 摘要

交易系统至今仍然被资产类别、交易场所、执行方式与技术栈所割裂。一个单一的经济想法，往往需要分别映射到券商 API、中心化交易所、去中心化协议、期权交易场所、预测市场以及链上交易流程之中，而这些系统各自拥有不同的语法与操作假设。本文主张，应当把 trade intent 视为交易系统中的第一类语义层。所谓 trade intent model，本质上是一种领域专用语言，它把含糊的人类请求编译成精确、可机读验证、且与具体执行后端解耦的经济动作表示。它在系统中的角色，类似于搜索里的 semantic parsing，或优化中的 CVXPY 这类高层建模语言：用户只描述要达成什么，系统将其编译成规范化表示，再由下游模块把这一表示翻译成可执行指令。本文的核心论点是，一个设计良好的 trade intent model 会在四个维度创造价值：语义精度、跨 venue 互操作性、策略可组合性，以及 agent 化自动执行。更重要的是，它是更大一套 action-oriented workflow 的表示层：用户只给出一个想法，系统即可生成候选策略，并在验证后完成一键部署。

## 1. 引言

问题的核心非常简单：人的交易意图是高层的，而执行基础设施却是碎片化、低层化的。用户通常以结果、暴露、对冲与约束来思考问题；但交易 venue 暴露给外部的往往是订单类型、产品标识符、链上交易格式、路由规则和互不兼容的 API。把前者直接映射到后者，天然会导致脆弱的系统。

而且，这个问题正在因交易入口的变化而迅速加剧。对于人类用户而言，越来越多的输入并不是标准化订单，而是 chat、voice，甚至可以称为 vibe trading 的模糊指令，例如“帮我在收盘前拿一点上行暴露”“如果市场反转就帮我对冲”“在多个 venue 中买到这个 thesis 的最佳表达”。这些表达对人类来说自然，但并不是安全的执行格式。它们富含意图，却严重缺乏规格。

对于机器端而言，越来越多的用户正在把通用型或非专业 agent 直接连接到钱包、broker API 与 DeFi protocol 上。这些 agent 可以理解自然语言、调用工具、执行多步金融动作，但它们通常缺少一个稳定的语义层，用来隔离“用户真正想表达的意思”和“不可逆的高权限金融执行动作”。在这种情况下，非结构化 prompt 实际上就变成了直接的 action surface。

近期事件说明了这一点。2026 年 2 月，基于 OpenClaw 的 autonomous crypto agent Lobstar Wilde 因误解了一次小额转账请求，向陌生地址误转了约 5240 万枚 LOBSTAR。2026 年 3 月，中国互联网金融协会则警示称，将 OpenClaw 一类系统用于金融场景，可能暴露敏感凭证，并触发错误转账或非预期投资购买。

这些案例共同指向一个系统问题：真正缺失的并不总是预测能力，也未必是智能合约本身，而是位于含糊语言与高权限动作之间的严格中间层。

这种错位具体表现为三类持续性失败。第一，自然语言表达能力强，但带有歧义。第二，执行接口精确，却彼此割裂。第三，如果没有稳定的中间表示，自动化系统就难以审计、难以调试；一旦出错，人们往往无法分辨究竟是理解错了用户目标、构造错了策略，还是把策略错误地翻译成了 venue 指令。

本文提出的解决方案，是在用户表达与执行之间引入一个规范化的语义层。系统不再把用户请求直接映射成 venue order，而是先把它编译成经济意义上的规范化表示，再由下游模块把这一表示转译成执行计划与具体动作。

对应的编译流程可以写成：

**用户想法或自然语言指令 -> 高层 Trade Intent -> 执行任务与执行策略 -> Venue-Specific Orders and Transactions**

这个框架会彻底改变人们理解交易软件的方式。目标不再只是“更好地下单”，而是构建一个面向 action 的语义接口，使系统能够围绕同一个表示层完成 parsing、validation、optimization、routing、simulation、auditing，以及最终的 agent-driven execution。

一个典型的自然语言请求如下：

`在 Solana 上买 10 美元的 NVDAx；在以太坊二层 Base 与 Arbitrum 中选择流动性更好的 venue 买 5 美元的 PAX Gold；买一张关于 “2026 年 2 月 NVDA 能否突破 200 美元” 的 Yes 预测市场合约，再买一张关于 “今年美联储会降息 2 次” 的合约；同时通过 Alpaca via MCP 建立一个 6 月 6 日到期、190/200 行权价的 AAPL bull call spread。`

这个请求在经济意义上完全连贯，但它直接落地时需要面对高度异构的基础设施。Trade intent model 的作用，就是把它视为一个单一的结构化 trade vector，而不是一包彼此无关的 API 调用。

## 2. 什么是 Trade Intent Model？

Trade intent model 是对期望经济动作的形式化中间表示。最恰当的理解方式，是把它视为一种用于交易语义的领域专用语言。

它的作用并不只是记录订单，而是以一种精确、规范、与执行后端解耦且可供机器操作的方式，表达用户真正想要的东西。

一个好的 intent schema 应当表达经济意义，而不是 venue 的表层语法。它至少需要容纳以下要素：经济目标、工具与动作类型、组合结构、约束条件、执行偏好，以及从 parsing 到 validation 再到 lowering 的 provenance。也正是因为有了这一层，不同表述的相似请求才能映射到相近的结构，下游模块也才能围绕同一个对象进行算法化推理，而不是一条条处理孤立指令。

## 3. 为什么 Trade Intent Model 重要？

Trade intent model 至少在四个维度产生价值。

1. 语义精度与可调试性。系统可以在执行前发现信息缺失、约束冲突或不可行请求。
2. 跨 venue 互操作性。现货、期权、perp、预测市场、broker 流程与链上动作都可以通过同一语义接口被表示。
3. 策略可组合性。multi-leg package、路由替代方案、成本与风险权衡、以及 portfolio-level constraints 都可以被联合评估。
4. Agent 化自动执行。parser、validator、simulator、execution planner、monitor 与 agent 可以围绕同一个共享对象协同工作，而不必依赖脆弱的 prompt glue 或 venue-specific adapter。

一旦缺少这一中间表示，这些优势就会退化成黑箱自动化。

## 4. 分层执行（Hierarchical Execution）

执行本身也必须分层。最好的类比来自机器人系统：high-level planning 与 low-level motor execution 是两层不同的问题。交易系统同样应当把 high-level trade intent 与 low-level execution tasks / policies 区分开来。但在本文语境下，这一分层执行栈应被理解为对当前仓库工作的扩展，而不是对现有代码已经完整实现这些层的陈述。

高层 trade intent 是规划层。它以不保留经济性歧义的方式承载用户目标，同时又提供了进行 cross-venue reasoning、liquidity aggregation 与 policy-aware routing 的正确界面。例如，`在收盘前买入 10 万股 AAPL`，或者 `本周买入 100 万美元名义的 NVDA call option`，都不应被直接当作单一订单处理，而应先被表示成包含名义规模、时间范围、impact tolerance、紧迫度与 venue constraints 的 portfolio-level object，然后再往下分解成低层 execution task。

这些低层 task 属于执行层。一个高层 portfolio trade 可能展开成许多 child order，而每个 child order 又可能由不同的 execution policy 驱动。视 market structure 而定，这些 task 可以通过 TWAP、VWAP、broker-native smart routing、链上的 naive slicing、AMM / aggregator execution，或者包含 low-latency market-structure / macro-sensitive signal 的更复杂执行策略来完成。它们不是不同的用户意图，而是同一个 intent 在不同市场微观结构中的不同编译结果。

这种分层还为 microstructure-level improvement 提供了明确位置。同一个 intent，可以通过 low-latency tactic、order-book-aware placement、queue-sensitive routing、microstructure alpha，或者更优的链上 route selection 与 timing 来持续改进。语义层负责 economic correctness，执行层负责 realized execution quality。用户在自然语言层只指定一次 portfolio objective，而系统可以在不改变原始 intent 的前提下，不断搜索更优的低层实现。需要明确的是，当前 GitHub 仓库并未实现这一完整的 planning-and-execution hierarchy；它是在本文中被提出和延展的下一层架构。

## 5. 系统视角：从 Idea 到 Agentic Execution

这一模块的最终目标，不应被理解为一个“更方便的券商界面”，而更像是一层 action-description layer 与 agent workflow substrate：用户只提供一个 idea，系统便能合成候选策略、完成验证，并支持一键部署到 agent platform。

对应的概念性 pipeline 如下：

1. Idea ingestion。用户提供 thesis、持仓目标、对冲目标，或多步动作请求。
2. Semantic parsing into high-level trade intent。系统把输入规范化为 canonical、machine-verifiable 的高层对象，以支持 cross-venue reasoning 与 liquidity aggregation。
3. Strategy synthesis and task decomposition。系统生成候选实现，并把大额或带时间约束的 intent 分解成 executable task 与 child order。
4. Validation and policy checks。系统检查 feasibility、liquidity、risk、permission 与 portfolio consistency。
5. Low-level execution policy selection。系统选择具体执行方式，例如 TWAP、VWAP、smart routing、AMM-aware splitting 或 low-latency microstructure-informed enhancement。
6. Execution compilation and deployment。系统把选中的策略 lowering 成 venue-specific order、transaction 或 MCP workflow，并部署到 agent platform。

Trade intent schema 正是贯穿这些阶段的稳定接口。配套的 GitHub 仓库 `galpha-ai/intent-kit`（`https://github.com/galpha-ai/intent-kit`）应被理解为这一架构中 contract layer 的当前参考实现：schema definition、template generation、parsing、validation 与 dispatch。也正因为如此，它天然适合增量式的开放开发：parser、schema、validator、simulator、adapter 与 agent 可以独立演化，但仍通过共享表示保持兼容。

## 6. 一个实用的 Trade Intent DSL 应具备什么原则？

一个可落地的 trade intent language 至少应满足五个原则。

1. 语义完整性。它必须表达 portfolio、conditional logic、dependency 与非平凡 execution preference，而不是只覆盖单 venue order。
2. 规范化结构。表面措辞不同但经济含义相近的请求，应映射到接近的结构形式。
3. 关注点分离。intent layer 负责表达用户在经济上的目标；后续层负责决定如何实现。
4. 可验证性。每一轮翻译都应可审计，并保留从 request 到 parsed intent，再到 selected strategy 与 final execution artifact 的 provenance。
5. 可扩展性。新的 instrument、venue、chain 与 action type 应能在不重构整个表示层的前提下被纳入。

一个 wrapper 只是翻译语法；真正的 trade intent model 组织的是语义。

为避免过度表述，需要明确区分“当前仓库已实现的部分”和“本文提出的完整系统”。当前仓库实现的是 semantic gateway layer：intent schema、agent-facing template、parsing、validation 与 dispatch；而 high-level portfolio planning、strategy synthesis、execution-policy search 与 low-level algorithmic execution 则属于建立在这一接口之上的未来层。

## 7. 开放问题

仍然存在若干重要研究问题：schema 的抽象层级如何选择；lowering 过程中如何保证 semantic preservation；在用户请求不完整时哪些字段可以安全推断；partial fill 与 cross-venue failure recovery 如何处理；以及在更 agentic 的系统中，risk、policy 与 audit layer 应如何约束 agent autonomy。这些都不是边缘问题，而是 deployment-grade trade intent system 的核心设计约束。

## 8. 结论

Trade intent model 在人类交易想法与异构执行基础设施之间提供了一个语义接口。它把含糊输入转化为精确、可机读验证、且与具体执行后端解耦的经济动作表示。它的价值远不止更整洁的 order syntax：一个设计良好的 trade intent layer 能实现 cross-venue interoperability、multi-leg strategy composability、可调试性提升，以及 high-level economic intent 与 low-level execution policy 的原则性分离。从这个意义上说，trade intent model 不应被视为某个产品 feature，而应被视为一种基础设施：它是让碎片化市场、异构 API 与 agent platform 能通过同一种 economic-intent language 协同工作的共享语义底座。
