# The Applied Mathematician's Approach to Chip and Compute-System Design

**A Concrete Research Design in Two Nested Loops**

*Inner loop — construct and solve the right model of the engineered system. Outer loop — construct and act on a model of the evolving industrial world, coupled to the inner loop by scarcity prices.*

---

## Abstract

Existing AI-for-chip-design systems optimize inside a problem someone else formulated: the design variables, objectives, constraints, and simulators are given. This paper specifies, concretely, a system that *constructs* the formulation and then *searches formulation space* with an explicit, growing operator set. The inner loop defines the design problem as a typed intermediate representation $\mathcal{P} = (W, Z, H, X, J, C, U, B, S)$ — including abstraction boundaries $B$ and a semantic contract $S$ every reformulation must refine — and gives falsifiable mathematics for each component: compositional structure hypotheses drawn from a transferable structural prior and shipped as certificates with explicit validity domains; lower bounds with epistemic status that steer compute between improving the design and understanding the problem; a performance model grounded in resource-constrained scheduling; and a library of **reformulation operators** $R_1..R_{10}$, searched by lookahead tree search under value uncertainty and extended by operator induction from successful derivations. Bottlenecks are identified by **constraint value with uncertainty** — the marginal value of relaxation, of which the convex dual is one efficient estimator — and models are confronted with reality through three simulator roles (train, hidden-falsify, calibrate), an explicit model-discrepancy term separating inadequacy from noise, and information-maximizing **experiment selection**, not adversarial falsification alone. The outer loop lifts the formulation to belief-state control over a partially observed technology and supply world: readiness as a hidden state estimated from noisy signals, deployment as optimal stopping, supply, power, and facilities as capacity constraints whose prices coordinate the loops by column generation, and frontier expansion as *causal intervention* on the world model — with strategic counterparties flagged as the model's known frontier. The objective is discounted **useful delivered service**, with the valuation itself audited rather than assumed. Throughout, semantics remain stochastic while every experiment is deterministically replayable, so rewards are exactly recomputable. §22 pins each of eight research hypotheses to a controlled experiment.

## 0. Reading guide: the claims this program stands or falls on

The program is not a manifesto; it is eight falsifiable hypotheses (experiments in §22):

- **A.** Explicit mathematical models improve transfer across workload/technology shift.
- **B.** Simulator ensembles with hidden falsifiers beat Monte Carlo of one simulator.
- **C.** Constraint-value analysis discovers architecture bottlenecks that reward search misses.
- **D.** Training on reformulation operators produces more novel designs than parameter search.
- **E.** Modeling technology availability and supply changes designs materially.
- **F.** Optionality-preserving roadmaps beat committed roadmaps under realized uncertainty.
- **G.** Causal frontier-expansion actions improve long-horizon delivered-service value.
- **H.** *Formulation transfer*: trained on some design domains, the system constructs useful formulations in a domain it has never seen.

Compressed to its five scientific primitives, the program is: (1) **structure and bound certificates** — claims with validity domains, tests, and falsifiers, not beliefs; (2) **problem reformulation operators** — search over $(f, X, C, B)$, not only over $x$; (3) **constraint value → reformulation** — scarcity signals become scientific questions; (4) **model–experiment co-evolution** — train/falsify/calibrate simulator roles plus information-gain experiment selection; (5) **technical–industrial co-design** — prices, cuts, and counterexamples between the loops, with causal interventions on the future feasible set.

---

# Part I — The Inner Loop: Model the Engineered System

## 1. The design problem is the primary object

A conventional optimizer solves $\min_{x \in \mathcal{X}} J(x)$ s.t. $g_i(x) \le 0$. The architect's real output is the problem itself, which we make an explicit, mutable data structure — a **Problem IR**:

$$
\mathcal{P} = (W, Z, H, X, J, C, U, B, S)
$$

| Symbol | Meaning | Concrete instance (running example: LLM serving) |
|---|---|---|
| $W$ | workload distribution | mixture of prefill and decode; context $s \sim \text{LogNormal}$; batch process $B_t$ |
| $Z$ | representation | dataflow graph with per-edge byte counts, per-node FLOPs, reuse distances |
| $H$ | structural hypotheses | "decode is bandwidth-bound", "attention is low-rank-compressible" — each certified per §3 |
| $X$ | design variables | SRAM/HBM capacities, NoC topology, tile sizes, chiplet partition, precision |
| $J$ | objectives | tokens/s/W, tokens/s/\$, tail latency |
| $C$ | constraints | typed: logical, physical, resource, probabilistic, conventional (§6) |
| $U$ | uncertainty | workload drift, process variation, simulator model-form error |
| $B$ | abstraction boundaries | what is fixed parameter vs. design variable; HW/compiler/runtime responsibility split |
| $S$ | semantic contract | the task specification the design must satisfy: served model family, quality/accuracy floors, correctness obligations |

$B$ is what makes reformulation representable: moving an item across a boundary of $B$ is a formal edit to $\mathcal{P}$ (§7). $S$ is what keeps reformulation honest: it is the fixed point defining when two formulations still solve *the same problem* — every operator in §7 must refine it. The AI's competence is measured on $\mathcal{P}$-construction quality, not only on $x^*$ quality; §10 gives the metric.

## 2. Workload representation with numbers attached

Represent a workload phase as a computation on which two quantities are computed exactly: FLOPs $F$ and minimum data movement $Q$.

**Transformer decode, per token, per layer** (model dim $d$, batch $B$, context $s$, bytes/element $b$) [12]:

- GEMM (QKVO + MLP): $F \approx 24 B d^2$ FLOPs against $12 d^2 b$ weight bytes → arithmetic intensity $I = 2B/b$. At $B=1$, FP8: $I = 2$ FLOP/byte.
- Attention score/value: reads $2 s d b$ KV bytes for $\approx 4 B s d$ FLOPs → $I = 2B/b$ again.

A representative current accelerator has peak $\pi \approx 4.5\times10^{15}$ FLOP/s (dense FP8) and HBM bandwidth $\beta \approx 8\times10^{12}$ B/s [15], so the ridge point [3] is $\pi/\beta \approx 560$ FLOP/byte. Decode at small batch sits **two to three orders of magnitude** below the ridge: the machine is a memory system with an arithmetic unit attached. Prefill GEMMs ($I \sim 10^2$–$10^3$) sit above it. This single computation dictates the architecture conversation, and the AI must produce it unprompted from a workload trace.

**KV-cache capacity**, per token: $2 L d b$ bytes ($L$ layers). For $L=80$, $d=8192$, FP16: $2.6$ MB/token → an 8k-context sequence holds $21$ GB; a batch of 32 holds $687$ GB, far beyond one package's HBM [12]. Reformulations that dissolve this (each is an operator application from §7, changing the *formula*, not the operating point): grouped-query attention divides by group count $g$ [13]; quantization divides by $b_{16}/b_q$; paging converts a capacity constraint into a bandwidth+latency cost; recomputation trades $Q \to F$.

## 3. Structure discovery: each hypothesis is a test, not a sentence

Every structural hypothesis $H$ ships as a triple $(H, \text{statistic}, \text{refutation search})$. Hypotheses are not a flat menu but terms of a **compositional grammar**,

$$
H ::= \text{Sparse}(X) \mid \text{LowRank}(X, r) \mid \text{Separable}(f, \Pi) \mid \text{Monotone}(f, x_i) \mid \text{Invariant}(f, G) \mid H \land H \mid H \circ H,
$$

closed under conjunction and composition and stratified by scope (global → per-regime → local), so the system can express what real modeling actually finds — e.g. *sparse + block-low-rank + approximately separable, monotone in batch* — and structure discovery becomes symbolic model search rather than feature classification. The atomic tests:

| Structure | Formal claim | Test statistic | Refutation search |
|---|---|---|---|
| Low rank | $\|A - A_r\|_F / \|A\|_F \le \epsilon$ at rank $r$ | SVD tail energy on held-out workloads | maximize tail energy over workload perturbations |
| Sparsity | $\|x\|_0 \le k$, support $S$ stable | Jaccard$(S_{w}, S_{w'})$ across distribution shift | adversarial input search for support churn |
| Monotonicity | $\partial T / \partial B_{\text{mem}} \le 0$ | finite differences on a sampled grid | gradient ascent on the violation $\max(0, \Delta T)$ |
| Convexity | $f(\lambda x + (1{-}\lambda) y) \le \lambda f(x) + (1{-}\lambda) f(y)$ | random-chord violation rate | maximize the chord gap over $(x, y, \lambda)$ |
| Separability | $J(x) \approx \sum_i J_i(x_i)$ | $R^2$ of additive fit; interaction ANOVA | search for high-order interaction terms |
| Conservation | flow balance $A f = 0$ on the traffic graph | residual $\|Af\|$ on simulator traces | trace mining for buffering/drop events |
| Invariance | $f(gx) = f(x)$ for group $G$ | orbit variance | symmetry-breaking instance search |

**Where hypotheses come from: the structural prior.** $H$ is not proposed from nothing. The system carries an explicit prior $\pi_0(H \mid \phi(W, Z))$ over the hypothesis grammar above, conditioned on cheap workload features $\phi$ (graph statistics, reuse-distance profiles, operator mix) and seeded from domain knowledge that is itself structural: physics supplies conservation and locality (§6's "law" constraints induce flow-balance and distance-decay hypotheses); linear algebra supplies low-rank and separability candidates for tensor programs; regularity of the computation graph supplies invariance candidates. The prior is *transferable state, not folklore*: every certificate outcome (survived/revoked, on which workload family) is logged, and $\pi_0$ is re-fit across problems — hierarchical Bayes or simple frequency reweighting at first — so that hypothesis proposal itself improves with experience. This is the precise sense in which the system accumulates domain knowledge: as a calibrated prior over *which mathematical structures tend to survive falsification in which regimes*, rather than as text.

A hypothesis that survives its tests yields a **Structure Certificate**

$$
C_H = (H,\ \mathcal{D}_H,\ \epsilon_H,\ p_H,\ T_H,\ B_H)
$$

— the claim, its **validity domain**, tolerated approximation error, confidence, the tests run, and the counterexample budget spent. Certificates are cited by downstream models, and falsification acts on them with four graded outcomes, not two: a survived attack **strengthens** ($p_H \uparrow$); a marginal violation **weakens** ($\epsilon_H \uparrow$); a counterexample outside the core regime **localizes** ($\mathcal{D}_H \to \mathcal{D}_H'$, shrinking the domain — the usual scientific outcome: attention low-rankness may survive on long-context retrieval and die on code generation); only a violation inside the claimed domain at the claimed tolerance **revokes** ($p_H \to 0$), flagging every citing model for revision (§9). The training loop this induces — *conjecture → operational definition → test → refine domain or revoke* — is the core primitive of the whole program.

**Decision relevance.** Structure is scored by its effect on decisions, not its truth alone:

$$
V(H) = \alpha\,\Delta C_{\text{model}} + \beta\,\Delta E_{\text{pred}} + \gamma\,\Delta C_{\text{solve}} + \delta\,\Delta J_{\text{design}}
$$

— change in model compactness, prediction error, solve cost, and realized design quality. A true-but-useless symmetry scores $\approx 0$ and is dropped.

## 4. Bounds before search: the Bound Certificate

The red–blue pebble game [1] gives I/O lower bounds against fast-memory size $M$. For $n \times n$ matrix multiply:

$$
Q(n, M) \;\ge\; \frac{n^3}{2\sqrt{2}\,\sqrt{M}} - M ,
$$

attained (up to constants) by $\sqrt{M}$-tiling. Consequences the AI must derive, not memorize:

1. Attainable intensity for GEMM scales as $\sqrt{M}$ — SRAM capacity, not FLOPs, sets the roofline position for compute-bound kernels.
2. Standard attention over context $s$ has $Q = \Omega(s d)$ per query *regardless of tiling* — the KV cache must be read; only algorithmic change (sparse/linear attention, KV compression) moves this bound.
3. For a $p$-way tensor-parallel GEMM, per-step all-reduce volume is $\Theta(B d)$ per device independent of $p$ [2] — scaling out does not amortize activation communication, which is why the interconnect term in §5 saturates.

The bound's role is triage: it separates *"a schedule can fix this"* from *"only an algorithmic or architectural change can fix this."* An agent that spends $10^4$ rollouts trying to tile away an $\Omega(sd)$ cost has failed a test the bound answers in closed form.

Bounds therefore ship as first-class artifacts parallel to Structure Certificates — a **Bound Certificate**: `{quantity, bound, status, proof technique or citation, assumptions, gap to best-known-achievable}`, with epistemic status $\in$ {exact, proven, empirical, conjectured, unknown}. The rule is **bound-aware search**, not search-forbidden-without-proof: every cost enters search carrying its certificate, and status `unknown` opens a genuine meta-decision — spend compute on **Search Design** (improve $x$ under current understanding) or on **Search Bound** (understand the problem: derive, or conjecture-and-test, a bound). The most interesting architectures live precisely where no bound is known, and *discovering* one is itself a scored artifact; allocating between the two searches is a value-of-information problem, priced by how often certified bounds have redirected search away from provably futile regions — as the $\Omega(sd)$ example above does in closed form. If a certified bound and a surrogate disagree by more than the surrogate's certified error, the surrogate is wrong.

## 5. The performance model: bounds, overlap, and contention

Let $T_i$ be the isolated service time of resource $i \in \{\text{compute}, \text{mem}, \text{net}, \text{sync}\}$, e.g. $T_c = F/(\pi u_c)$, $T_m = Q_{\text{mem}}/(\beta u_m)$. Two analytically exact envelopes bracket reality:

$$
T_{\text{LB}} = \max_i T_i \qquad\text{(perfect overlap: the roofline bound [3])}, \qquad
T_{\text{UB}} = \sum_i T_i \qquad\text{(fully serialized)}.
$$

For two resources an overlap coefficient interpolates exactly: $T = T_1 + T_2 - \alpha \min(T_1, T_2)$, $\alpha \in [0,1]$. For $n \ge 3$ resources, pairwise corrections double-count — three unit-time phases with all $\alpha_{ij} = 1$ would give $T = 0 < T_{\text{LB}}$ — so the general model is not a closed-form correction but a small **resource-constrained schedule**, which is truer to this paper's own doctrine: find the structure, don't fit a convenient formula. Phases $i$ with durations $d_i$ and start times $s_i$, precedence from the dataflow order, and per-resource capacities:

$$
T = \min\; \max_i (s_i + d_i)
\quad \text{s.t.} \quad
s_j \ge s_i + d_i \;\; \forall (i \to j), \qquad
\sum_{i:\, t \in [s_i,\, s_i + d_i]} r_{ik} \le C_k \;\; \forall k, t,
$$

solved or bounded by a small LP/flow surrogate. Overlap, serialization, and contention now *emerge* from precedence and capacity rather than being pasted on as corrections; queueing at a contended resource enters as an effective capacity or an M/G/1 delay term $\mathbb{E}[T_q] = \frac{\rho}{1-\rho}\cdot\frac{\bar{S}(1+C_v^2)}{2}$; and $T_{\text{LB}}, T_{\text{UB}}$ remain exact envelopes of every feasible schedule.

Workloads are modeled as a regime mixture $W = \bigcup_k W_k$ (compute-, memory-, network-, sync-bound), and the design target is $\min_x \mathbb{E}_{w \sim P(W)} J(x, w)$ — never a single benchmark point.

The structure makes residuals diagnostic rather than merely wrong: measured $T$ near $T_{\text{LB}}$ certifies the overlap machinery; drift toward $T_{\text{UB}}$ localizes a serialization — a missing overlap or a false precedence edge in the schedule model; $T > T_{\text{UB}}$, or per-resource times inflated above their isolated values, is contention or resource coupling and forces a capacity/queueing upgrade. A bare $\max$ model can only report "wrong"; this one reports *where*.

## 6. Constraints are typed; only some are laws

$$
C = C_{\text{logical}} \cup C_{\text{physical}} \cup C_{\text{resource}} \cup C_{\text{probabilistic}} \cup C_{\text{conventional}}
$$

- **Logical** ($\text{RTL} \models \text{Spec}$): never traded, never softened.
- **Physical** (timing, thermal, IR drop): softenable only against a physics model, e.g. chance constraint $\Pr[t_{\text{path}} > t_{\text{clk}}] \le \epsilon$ under process variation.
- **Resource** (power, area, BW): carry constraint values; primary innovation signals (§8).
- **Probabilistic / robust**: $\Pr[g(x,\xi) > 0] \le \epsilon$ or $g(x,\xi) \le 0\ \forall \xi \in \Xi$.
- **Conventional**: cache-line width, page size, coherence protocol, number formats. For each, the AI must answer: *physics or convention?* Conventions are the natural targets of the reformulation operators that follow.

## 7. The Problem Reformulation Operator: making invention searchable

This is the program's central algorithmic object. AlphaEvolve-style systems [17] have *program → mutation operator*; we need *problem formulation → mutation operator*. Constraint transmutation stops being an instruction and becomes a typed edit on the Problem IR of §1.

**Operator library** (initial set; each maps $\mathcal{P} \mapsto \mathcal{P}'$, is applicable only when its precondition on $(C, H, B)$ holds, and must emit the new certificates its output needs):

| | Operator | Formal edit | Canonical instance |
|---|---|---|---|
| $R_1$ | parameter → design variable | move a symbol from fixed data into $X$ (edit $B$) | HBM stack count, voltage domain, number format |
| $R_2$ | hard → chance constraint | $g \le 0 \Rightarrow \Pr[g > 0] \le \epsilon$, requires a $U$-model | timing margin under variation |
| $R_3$ | global → partitioned resource | one capacity $b$ → per-partition $b_k$ + an allocation variable | banked memory, power domains |
| $R_4$ | storage → recompute | delete capacity demand, add FLOPs: $Q \downarrow, F \uparrow$ | activation checkpointing |
| $R_5$ | dynamic → static decision | move a runtime choice to compile time; deletes state from $C_{\text{model}}$ | static scheduling vs. arbitration |
| $R_6$ | monolithic → graph partition | object → $\{G_1..G_k\}$ + cut-cost terms | chiplets (§11) |
| $R_7$ | exact → approximate computation | admit error $\epsilon$ as a variable with a quality constraint | quantization, sparsification, lossy KV |
| $R_8$ | hardware ↔ software responsibility | move an obligation across the HW/compiler/runtime boundary in $B$ | software-managed scratchpad vs. cache |
| $R_9$ | scarcity → redundancy/substitution | replace a single resource by a qualified set with a selection variable | multi-vendor memory, spare rows |
| $R_{10}$ | algorithm family change | replace the computation itself, subject to task-level quality constraints | attention → sparse/linear attention |

**The semantic contract.** Operators like $R_7$ and $R_{10}$ change the computation itself, so the IR's contract $S$ (§1) is what blocks the cheapest hack available to a formulation-editing agent — quietly changing the task. Every operator application must be a refinement: $\mathcal{P}' \models S$ exactly, or approximately, $d(\text{Behavior}(\mathcal{P}'), S) \le \epsilon_S$, with the behavior metric and tolerance declared in $S$ itself. A reformulation that cannot exhibit its refinement witness is rejected regardless of objective value.

**The search.** Reformulation is a **tree search over formulations**, not greedy hill climbing: an operator application can lower value at depth one and open a dominant design space at depth two. Nodes are formulations, edges are operator applications proposed by a policy $R_\theta(\mathcal{P}, C_i, \text{evidence})$ — *evidence* = the constraint values of §8, residuals of §9, and certificates in force — and expansion is guided by a *predicted* value $\hat{V}(\mathcal{P})$ with uncertainty, because the true $V(\mathcal{P})$ of §10 requires the expensive downstream pipeline and is measured only at selected leaves. Selection uses $\Pr[V(\mathcal{P}') > V(\mathcal{P}) \mid \mathcal{D}]$ or expected improvement, as in Bayesian optimization; MCTS and population variants are natural here [16, 17]. Operator applications are logged as derivation trees, so a discovered architecture carries its formulation lineage — the training data for both $\hat{V}$ and $R_\theta$.

**Operator induction.** A fixed library invites the objection that "architectural creativity" is just selection among ten human-written templates. The program is therefore two-phase. *Phase I (fixed grammar):* establish that search over $R_1..R_{10}$ beats parameter search inside a fixed $\mathcal{P}$ — hypothesis D (§22). *Phase II (induction):* mine recurring transformation patterns from successful derivation trees — e.g. many derivations independently composing *global dynamic resource → partitioned static resource + offline coordination* — and promote them to new operators with induced preconditions:

$$
\mathcal{R}_{t+1} = \mathcal{R}_t \cup \operatorname{Induce}(\text{successful derivations}),
$$

so the operator library itself evolves. Induced operators enter on probation: they must earn certificate-grade evidence of their applicability conditions before the proposal policy may weight them.

## 8. Constraint value: the invention signal, generalized

The signal that triggers reformulation is the **marginal value of constraint relaxation**:

$$
MV_i \;=\; \frac{V^*(b_i + \Delta) - V^*(b_i)}{\Delta},
$$

the finite-difference change in achievable objective when constraint $i$'s budget moves. This is the primitive; the convex dual is its cheapest estimator, not its definition. In convex problems $MV_i \to \lambda_i = -\partial V^*/\partial b_i$ exactly. In the mixed-integer, topology-changing, simulator-defined problems that dominate architecture, classical duals may not exist, may be non-unique, or may carry a duality gap — so the system maintains an estimator ladder, ordered by cost: (i) exact duals where convexity is certified (§3); (ii) LP-relaxation duals and reduced costs for MIPs; (iii) surrogate-gradient estimates $\partial \hat{V}/\partial b_i$ from a fitted model; (iv) ground truth by perturb-and-re-solve. Estimates from cheap rungs are periodically audited against rung (iv), like any other model in §9.

Every estimate carries uncertainty: the system maintains $(\widehat{MV}_i, \sigma_i)$, and the trigger for invention is a confidence statement, not a noisy point estimate — open a **reformulation episode** on constraint $i$ when $\Pr[MV_i > \tau \mid \mathcal{D}]$ is high (equivalently, when the lower confidence bound clears $\tau$), and rank competing episodes by expected value of reformulation per unit exploration cost. A large $\widehat{MV}_i$ with large $\sigma_i$ is first an *experiment* (tighten the estimate on a cheaper-rung audit), not an expensive episode. The episode itself asks: (a) why does $g_i$ exist — physics or convention (§6)? (b) which operators $R_j$ have their precondition satisfied on it? (c) is the value of relaxation, $MV_i \cdot \Delta b_i$, worth the operator's cost in complexity and verification?

Concrete instance: at small-batch decode the binding constraint is HBM bandwidth, with $MV_{\beta}$ orders of magnitude above $MV_\pi \approx 0$. The mechanical response is "buy more bandwidth"; the operator responses are $R_7$ (quantize weights/KV — cuts $Q$), $R_4$/speculative decoding (raises effective batch, hence intensity [14]), and $R_8$ + near-memory compute (moves work across the constraint boundary). A trained system must produce all three *from the signal*, not from a menu. The full loop:

$$
\text{optimize} \;\to\; \text{measure } MV \;\to\; \text{scientific question} \;\to\; \text{operator application}.
$$

## 9. Three simulator roles: train, falsify, calibrate

A simulator may be a learning environment; it must never be the sole definition of reality. The oracle set is partitioned by role:

$$
\mathcal{S} = \mathcal{S}_{\text{opt}} \;\cup\; \mathcal{S}_{\text{fals}} \;\cup\; \mathcal{S}_{\text{calib}}.
$$

- $\mathcal{S}_{\text{opt}}$ — **training environments**: cheap, differentiable or fast, safe to overfit *tactically*; this is where AlphaChip/AlphaEvolve-style loops [16, 17] run.
- $\mathcal{S}_{\text{fals}}$ — **hidden falsifiers**: held-out model families the designer never queries during search; they exist to attack the model and the design, and their residuals are the reward for the falsifier agent below.
- $\mathcal{S}_{\text{calib}}$ — **calibration oracles**: the expensive rungs (RTL, synthesis, physical design, emulation, silicon) that anchor all upstream models to reality.

The model predicts $\hat{y} = M(x)$; oracle $k$ returns $y_k = S_k(x)$; the object of interest is the residual $r_k = y_k - M(x)$, interpreted against a taxonomy:

| Residual signature | Diagnosis | Action |
|---|---|---|
| bias growing with utilization $\rho$ | missing contention state | add $Q_{\text{cont}}$ term (§5) |
| drift from $T_{\text{LB}}$ toward $T_{\text{UB}}$ | serialization: missing overlap or false precedence in the schedule model | fix the overlap machinery, or the precedence/capacity constraints |
| $T > T_{\text{UB}}$ or inflated isolated $T_i$ | interference / resource coupling | model the shared resource explicitly |
| bias growing with reuse distance | wrong locality model | re-fit reuse distribution |
| heteroscedastic noise | unmodeled nondeterminism | model it, or *remove* it ($R_5$; legibility, §10) |
| sign flip across regimes | wrong regime boundary | re-cluster $W_k$ |
| violation of a certificate | structural hypothesis false | revoke; flag all citing models |

**Designer/falsifier alternation.** Maintain three co-evolving archives — designs $\mathcal{D}$, models $\mathcal{M}$, counterexamples $\mathcal{C}$ — and iterate

$$
x_k = \arg\min_x J(x; \Xi_k), \qquad \xi_k^* = \arg\max_{\xi} \text{Violation}(x_k, \xi), \qquad \Xi_{k+1} = \Xi_k \cup \{\xi_k^*\},
$$

with the falsifier drawing from $\mathcal{S}_{\text{fals}}$ across model families — distinguishing workload, parameter, model-form, and implementation uncertainty. A large Monte Carlo sweep of one simulator is not epistemic robustness. Cross-level feedback is mandatory: a synthesis-reported $f_{\max}$ miss updates the architecture-level timing prior; a physical-design wiring blowup updates the communication cost model upstream.

**Model discrepancy is not noise.** Observations decompose as $y(x) = M(x) + \delta(x) + \epsilon$ [22]: mechanistic model, structured discrepancy, measurement noise. A small, structureless $\delta$ certifies $M$ at its current fidelity; a $\delta$ with systematic dependence on identifiable variables is a bug report against $M$, and the taxonomy above names the fix. Keeping $\delta$ explicit protects against both failure modes: complexifying $M$ to chase noise, and averaging away real inadequacy as if it were noise.

**Experiment selection: information gain, not only falsification.** The falsifier finds worst cases; a scientist also designs the experiment that best *discriminates competing models*. When the archive $\mathcal{M}$ holds rivals $M_1, M_2$ — say a bandwidth-saturation memory model against a bank-conflict-dominated one — select

$$
e^* = \arg\max_e\; I(M;\, Y_e)
$$

under the oracle budget: the Bayesian-optimal experiment [23]. Often one specially constructed workload separates the rival predictions more than any number of production traces. This **Experiment Selection Operator** stands beside the reformulation operators of §7 as the program's second first-class move: certificates, $\hat{V}$, and $\widehat{MV}$ all sharpen fastest per oracle dollar when queries are chosen for information, not convenience.

**Abstraction error budget.** Each level — workload IR, architecture model, RTL, physical design — introduces approximation $\epsilon_\ell$, composing into a decision-relevant budget $\epsilon_{\text{decision}} \le \sum_\ell \epsilon_\ell$ (or a task-relevant divergence between reality and the model stack). Oracle spending is then itself an allocation problem: minimize simulation cost subject to $\epsilon_{\text{decision}} \le \epsilon$ — spend the expensive rungs where the *binding* abstraction error lives. This is the quantitative form of "simulators are experimental instruments."

## 10. Legibility as an objective; formulation quality as a metric

Two designs with equal PPA can differ by orders of magnitude in predictability. Define legibility as the complexity of the *simplest adequate predictive model* — an MDL-style quantity:

$$
C_{\text{legible}}(x; \epsilon) = \min_{M} \left\{ C(M) \;:\; \mathbb{E}\big[\ell(M(x), Y(x))\big] \le \epsilon \right\},
$$

the minimum model complexity required to predict the architecture's behavior to tolerance $\epsilon$. A legible architecture is one whose behavior admits a compact predictive model. In practice $C_{\text{legible}}$ is estimated through a proxy vector — hidden dynamic state count, nondeterminism sources, calibration sample complexity, surrogate error at fixed budget — reported as separate terms, since their units differ, and the design objective is $J = J_{\text{PPA}} + \lambda\, \hat{C}_{\text{legible}}$. A legible design is cheaper to optimize, verify, compile to, and *redesign by AI in the next iteration* — a bet on compounding returns, and the objective that makes $R_5$ profitable.

**Formulation quality** $V(\mathcal{P})$ (used by §7's acceptance test): given a fixed solver budget, score (i) regret of $x^*_{\mathcal{P}}$ under the *true* downstream evaluator ($\mathcal{S}_{\text{calib}}$), (ii) certificate survival rate at higher-fidelity oracles, (iii) solve + verification cost. This makes "did it pick the right problem" measurable.

## 11. Worked integration: chiplet partitioning

Yield follows a negative-binomial defect model [4], $Y(A) = (1 + AD/\alpha)^{-\alpha}$, with representative reported leading-edge values $D \approx 0.1$–$0.2\ \text{cm}^{-2}$ and clustering $\alpha \approx 2$–$4$. Cost per good monolithic die of area $A$ scales as $C(A) \propto A / Y(A)$ — superlinear, which is the economic engine behind $R_6$. Partition the block graph $G$ into $k$ chiplets $\{G_1..G_k\}$ with cut traffic $q(\mathcal{P})$ bytes/s, and minimize

$$
\min_{k,\ \mathcal{P}(G)} \;\; J_{\text{si}} + J_{\text{pkg}} + J_{\text{comm}}
\qquad \text{s.t. } A_j \le A_{\max},\quad q(\mathcal{P}) \le B_{\text{D2D}},
$$

$$
J_{\text{si}} = \sum_{j=1}^{k} \frac{C_{\text{wafer}}\, A_j / A_{\text{wafer}}}{Y(A_j)},
\qquad
J_{\text{pkg}} = C_{\text{assembly+test}}(k),
\qquad
J_{\text{comm}} = \lambda_E\, e_{\text{D2D}}\, q(\mathcal{P}) + \lambda_T\, \Delta T_{\text{D2D}}(\mathcal{P}),
$$

with $e_{\text{D2D}} \approx 0.25$–$0.5$ pJ/bit for UCIe-class links [5] versus $\approx 0.05$–$0.1$ pJ/bit on-die [6]. Everything in Part I appears here: a Bound Certificate (min-cut of $G$ bounds $q$), a Structure Certificate (is $G$ nearly separable?), typed constraints (reticle limit = physics; D2D protocol = convention), constraint values ($MV$ on $B_{\text{D2D}}$ prices the next packaging generation), operators ($R_6$ created the problem; $R_9$ adds link redundancy), and falsification (the linear D2D energy model is attacked by a thermal-coupling simulator in $\mathcal{S}_{\text{fals}}$). Solved as ILP/graph partitioning at fixed $k$, outer loop on $k$. This is also the hinge to Part II: $Y$, $C_{\text{pkg}}$, and $B_{\text{D2D}}$ are time-varying, uncertain, and supply-constrained.

## 12. Solver selection follows structure

| Discovered structure | Solver class |
|---|---|
| posynomial delay/energy models (gate sizing, pipelines) | geometric programming [7] |
| convex/conic surrogate | interior point; exact duals for §8 |
| discrete + coupling (partition, mapping, tiling) | ILP / CP-SAT / branch-and-bound + decomposition |
| expensive black-box oracle | Bayesian optimization, active learning |
| sequential structure (placement, scheduling games) | DP / RL / tree search [16] |
| runtime resource control | MPC, stochastic control |
| heavy uncertainty | robust / distributionally robust optimization |

RL is one row, chosen when its preconditions hold — not the default.

---

# Part II — The Outer Loop: Model the Evolving World

## 13. A partially observed technology world

The feasible set moves: $x_t \in \mathcal{X}(t, \omega, g)$, the intersection of silicon, memory, packaging, network, power, cooling, manufacturing, supply, and facility feasibility, indexed by time $t$, scenario $\omega$, geography $g$. For each technology $j$, the states *lab → qualified → volume → economic* are real and distinct — conflating them is the canonical roadmap failure — but the true state is **not observed**. Nobody outside (and often inside) the supplier knows true HBM-generation yield, actual ramp rates, packaging capacity allocations, or grid-interconnection outcomes; what arrives are noisy signals: announcements, samples, qualification results, allocation offers.

So the correct formulation is belief-state control [9], not a fully observed Markov chain:

$$
z_t = \text{true technology/supply state}, \qquad
o_t \sim p(o_t \mid z_t), \qquad
b_t(z) = \Pr[z_t = z \mid o_{0:t}, u_{0:t-1}],
$$

with roadmap policy $u_t = \pi(b_t)$. Readiness time $\tau_j$ is a first-passage time *of the belief-updated process*; forecast calibration (§19) is scored on $b_t$, and information itself acquires value: a qualification program is partly an *experiment purchased to sharpen $b_t$*. Crucially, actions enter the transition kernel — $\Pr[z_{t+1} \mid z_t, u_t]$ — which is what §18 makes causal.

## 14. The objective: useful delivered service

Peak chip performance is two abstractions away from value. Define delivered service:

$$
U_t = \int_w v(w, t)\; y(w; x_t)\; dP_t(w),
\qquad
\max_\pi\; \mathbb{E}\left[ \sum_{t=0}^{T} \beta^t\, U_t \right],
$$

where $P_t$ is the workload demand distribution, $v(w,t)$ the value of serving request class $w$, and $y(w; x_t)$ the fraction served within SLA — a function of throughput, latency, availability, and model quality on the deployed fleet $x_t$. The valuation must not become an unaudited free parameter — chosen freely, $v$ can rationalize any roadmap. It is split into $v_{\text{exogenous}}$ (contracted prices, committed demand) and $v_{\text{estimated}}$ (demand and price forecasts), the latter carrying uncertainty that propagates into $U$; and the underlying performance vector (served requests, tail latency, availability, energy, revenue) stays multi-objective as long as possible — scalarization is a logged, revisable decision subject to the same audit as any other model, because the objective is itself part of the formulation. Deployed volume × per-system performance ($N_t P_t$) is the first-order surrogate, but the service form is what prevents pathologies: a 10× FP8-throughput system that misses latency SLAs or serves no demanded workload scores what it should — nothing. Two consequences survive either form: a system with $0.8\times$ performance at $10\times$ volume dominates, and a 12-month-earlier system beats a superior later one whenever the discounted ramp says so. **Best chip $\ne$ best compute system.**

## 15. Roadmapping as stochastic control; deployment as stopping

State $x_t$ (fleet, in-flight designs, beliefs $b_t$, capital), decisions $u_t$ (design starts, qualification experiments, volume commitments, deployments), scenario tree with nonanticipativity [10]:

$$
\min_{\pi}\; \rho\!\left[ \sum_t \beta^t J(x_t, u_t, \omega_t) \right]
\quad \text{s.t. } x_{t+1} = F(x_t, u_t, \omega_t),\ x_t \in \mathcal{X}(t,\omega),
$$

with the risk functional $\rho$ chosen per decision: expectation for repeated small bets; $\text{CVaR}_{0.95}(L) = \min_\eta \eta + \frac{1}{0.05}\mathbb{E}[(L-\eta)^+]$ [11] for ruin-relevant exposure (single-source components); minimax regret $\min_\pi \max_\omega [J(\pi,\omega) - J^*(\omega)]$ for technology-timing bets where priors are unreliable.

**Deploy-or-wait is optimal stopping** [8]. With a candidate improvement arriving stochastically:

$$
V_t(b) = \max\Big\{ \underbrace{\textstyle\sum_{t' \ge t} \beta^{t'-t}\, U_{t'}^{\text{now}}}_{\text{deploy}},\;\; \underbrace{\mathbb{E}\big[V_{t+1}(b') \mid b\big] - c_{\text{delay}}}_{\text{wait}} \Big\},
$$

a threshold policy on the belief: wait only while expected technology gain exceeds the delay cost, which is dominated by the deployment ramp foregone — why "wait for the better interconnect" so often loses. **Optionality** is the architectural dual of waiting: a design holding an upgrade path (socket, protocol, power envelope compatible with the successor) truncates the regret distribution without paying the delay, and its value is computable as the difference of the two stopping problems.

## 16. Supply, power, and facilities: capacity constraints with prices

- **Supply:** $N_t\, d_j(x) \le Q_j(t,\omega)$ per component $j$ (HBM stacks, advanced-packaging interposer area, optics). $MV_j \gg 0$ triggers the same reformulation episode as §8, with industrial operators added: substitute, redesign to cut $d_j$ ($R_7$, $R_4$), qualify a second source ($R_9$), or commit volume — which *changes* $Q_j$ (§18).
- **Power:** $P_{\text{sys}} = P_{\text{compute}} + P_{\text{mem}} + P_{\text{net}} + P_{\text{conv}} + P_{\text{cool}}$; facility constraint $\text{PUE} \cdot \sum P_{\text{sys}} \le P_{\text{grid}}(g, t)$, with fleet PUE $\approx 1.1$–$1.3$ [18]. At $\sim 1$ kW+/accelerator, a 1 GW campus caps near $\sim 7\times10^5$ accelerators — power, not silicon, is the binding constraint of the current buildout, and $MV_{\text{power}}$ backpropagates into Part I as pressure on data movement and on voltage architecture ($I = P/V$: higher distribution voltage cuts conductor mass but demands facility conversion stages with their own lead times).
- **Facilities:** grid interconnection requests currently take on the order of $4$–$7$ years to commercial operation [19]; substations $2$–$4$ y; buildings $1$–$2$ y — all longer than a silicon design cycle. A roadmap whose critical path ignores $\mathcal{X}_{\text{facility}}(t)$ is infeasible regardless of PPA.
- **Geography** indexes power price and availability, cooling, latency to demand, import constraints; the optimal fleet is heterogeneous in $g$ — homogeneity is a convention, not a law.

Migration paths are explicit design objects: optimize $\sum_t J(x_t) + C_{\text{mig}}(x_t, x_{t+1})$ over sequences $A \to H_1 \to \cdots \to B$; a transitional $H_i$ with worse steady-state efficiency can strictly dominate by de-risking schedule and stranded capital.

## 17. Coupling the loops: prices, cuts, and counterexamples

This is the program's central picture. The **inner loop** (Part I) produces an architecture menu $\mathcal{A} = \{a_1..a_N\}$ with resource vectors $d(a) = (\text{HBM stacks}, \text{interposer mm}^2, \text{W}, \text{network BW}, \ldots)$. The **outer loop** solves the roadmap over $\mathcal{A}$ and returns constraint values $\lambda = (\lambda_{\text{HBM}}, \lambda_{\text{pkg}}, \lambda_{\text{power}}, \ldots)$. The inner loop then re-optimizes the *priced* objective

$$
\min_x\; J_{\text{PPA}}(x) + \lambda^\top d(x)
$$

and returns new candidates. The coordination is *Benders/Lagrangian-inspired*, stated with the care the inner loop's structure demands: where the coupling is convex, classical optimality and feasibility cuts $\theta \ge \alpha + \beta^\top y$ apply [20]; where inner decisions are discrete or simulator-defined, logic-based Benders and no-good cuts take their place; and the cleanest reading of the menu itself is **column generation** — architectures are columns, the outer master problem prices resources, and the inner loop is the pricing subproblem, generating a new column exactly when $\min_x J_{\text{PPA}}(x) + \lambda^\top d(x)$ finds negative reduced cost. Three message types flow between the loops: **prices** (scarcity → design incentives), **cuts** (roadmap infeasibilities that prune the menu), and **counterexamples** (deployed-world residuals that revoke inner-loop certificates). Signature behavior: $\lambda_{\text{HBM}} \gg 0$ → the inner loop re-derives the memory hierarchy (more SRAM, compression, recomputation — §2's operators, now economically motivated); $\lambda_{\text{power}} \gg 0$ → it attacks movement and voltage. Bottleneck economics, not taste, steers architecture search:

$$
\boxed{\ \text{industrial scarcity} \;\to\; \text{architecture incentives}.\ }
$$

## 18. Frontier expansion is causal intervention, not forecasting

The outer loop may act so that tomorrow's feasible set is larger: $\mathcal{X}_{t+1} = F(\mathcal{X}_t, u_t, \omega_t)$. But "HBM supply will grow next year" (a forecast, $\Pr[Q_{t+1} \mid \text{data}]$) and "committing to 5M stacks will grow next year's supply by $\Delta$" (an intervention, $\Pr[Q_{t+1} \mid do(u)]$) are different objects [21], and an agent trained only on observational histories will systematically confuse them — reading correlation between commitments and later supply as intervention value. Frontier expansion therefore requires a **structural causal model** over the technology world: supplier capacity responds to committed demand with lag and elasticity; standardization shifts entrant rates; co-investment moves a transition intensity in §13's kernel. Action classes and their causal effect:

| Action | Effect on the world model | Valuation |
|---|---|---|
| second-source qualification | $Q_j \to Q_j^{(1)} + Q_j^{(2)}$; cuts CVaR tail | risk functional delta |
| volume commitment | shifts $P(\tau_j)$, raises $do$-response of $Q_j$ | $\approx MV_j \cdot \mathbb{E}[\Delta Q_j \mid do(u)]$ vs. capital at risk |
| interface standardization | bespoke dependency edge → market; collapses $\lambda_j$ | price path under entry model |
| co-investment | raises a transition intensity in $\Pr[z_{t+1} \mid z_t, u_t]$ | option value on earlier $\tau_j$ |

Identification comes from historical *interventions* (past commitments, past standards) and deliberate small experiments, not from passive curves; where the causal effect is unidentified, the system must report that, and the roadmap must not lean on it.

One limitation is stated rather than hidden: suppliers, competitors, hyperscalers, utilities, and governments are themselves optimizing agents, so the world is strictly a partially observable stochastic *game*, and a volume commitment's effect is an equilibrium response $v^*(u) = \arg\max_v U_{\text{supplier}}(v; u)$ — shaped by the supplier's opportunity costs, rival demand, and capex constraints — not a fixed causal curve. The SCM is the first approximation; where a counterparty's strategic response materially changes an action's value, the model must be upgraded to an endogenous-response (bilevel or equilibrium) form, and until then the Roadmap Certificate must flag those valuations as game-theoretically naive. The composed primitive of the whole program:

$$
\text{optimize} \to \text{measure constraint value} \to \text{apply reformulation operator} \to \text{intervene on the frontier}.
$$

## 19. Artifacts and backtesting

Three machine-checkable artifacts make the outer loop auditable:

1. **Technology Dependency Graph** $G_T$: nodes = technologies/suppliers/facilities; edges = dependencies carrying $(\tau, Q, C, Y)$ belief distributions and, where identified, causal response functions. The system must surface critical paths, single points of failure, and high-$\lambda$ edges.
2. **Roadmap Certificate**: nominal path $x_0 \to x_1 \to x_2$, fallback branches, trigger conditions on *beliefs* ("if $b_t(\text{HBM-next at volume by Q3}) < 0.4$, switch to branch B"), frontier-expansion actions with their causal valuation, and preserved options with computed option value.
3. **Calibration record**: at historical date $t$, hide the future; have the system form beliefs over $(\tau_j, Q_j, C_j)$ and design the $N$-year roadmap; score belief calibration (CRPS) and roadmap regret against the realized world. The DRAM, NAND, node-cadence, and interconnect histories are the training set; realized past interventions are the identification data for §18.

## 20. Stochastic semantics, deterministic replay

Robustness and trainability must be won without a claim the program cannot honestly make: that the world is deterministic. The semantics stay stochastic — transitions are $s_{t+1} \sim P(\cdot \mid s_t, a_t)$, expectations are expectations, posteriors are posteriors. What is engineered is **replay**: every episode records its complete draw — seeds, scenario ids, oracle versions, dataset versions — so that

$$
\text{same recorded experiment} \;\Rightarrow\; \text{exactly reproducible outcome},
$$

and every reward is exactly recomputable as $R(\text{trajectory}, \text{recorded environment})$. That — not a deterministic world — is what verifiable-reward training and clean credit assignment require.

The finite objects below are accordingly *estimators with quantified error*, never the quantities themselves: a sample average $\frac{1}{N}\sum_i f(x, \xi_i)$ estimates $\mathbb{E}_\xi[f(x,\xi)]$ with known convergence [10]; a particle set approximates the posterior $b_t$; a scenario tree approximates the stochastic program — and each approximation gap is an entry in the abstraction error budget of §9. Every instrument carries a version, so refreshing a scenario set or re-drawing particles is an explicit, logged state change: exactly where distribution shift becomes visible and attributable instead of being laundered through unlogged environment randomness.

| Stochastic object | Replayable finite instrument |
|---|---|
| workload distribution $P(W)$ | pinned, versioned scenario set $\hat{W} = \{w_1..w_N\}$ — a sample-average estimator of the expectation [10], not the expectation |
| chance constraint $\Pr[g(x,\xi) > 0] \le \epsilon$ | scenario counterpart on the pinned set (at most $\lfloor \epsilon N \rfloor$ violations), with scenario-approximation guarantees quoted alongside |
| robust constraint $\forall \xi \in \Xi$ | finite $\Xi_k$ grown by the falsifier's cutting-plane loop (§9) — the counterexample archive $\mathcal{C}$ *is* the working uncertainty set; each attack runs under a recorded seed |
| simulator nondeterminism | pinned seeds and versioned oracle builds; *irreproducible* residual variance is a model defect (§9) and a legibility cost (§10), to be modeled or removed ($R_5$) — never silently averaged |
| belief $b_t$ over technology state (§13) | particle filter over recorded observations: the update is a replayable map given the logged $o_t$, approximating — not equaling — the posterior |
| multi-stage roadmap (§15) | deterministic equivalent over a versioned scenario tree with nonanticipativity [10] |
| risk functionals $\text{CVaR}, \max$-regret | exact finite formulas over the tree's leaves (CVaR is an LP over scenarios [11]) |

The training problem is then an MDP whose *recorded episodes* are deterministic functions of (state, action, logged draw): exploration randomness lives in the policy and is logged; fresh information from the world — a new workload trace, a silicon measurement, a supply signal — enters only through versioned dataset transitions. The robustness payoff, honestly stated: because $\Xi_k$ and $\hat{W}$ are grown adversarially rather than sampled i.i.d., "robust" means *robust against the strongest recorded counterexamples, re-checkable forever* — a monotone guarantee about the archive, complementing (not replacing) the statistical guarantees of the estimators above.

## 21. The algorithm, in one place

$$
\boxed{
\begin{array}{c}
\textbf{Inner loop (per design iteration)}\\[2pt]
\text{construct Problem IR } \mathcal{P} \to \text{discover + certify structure and bounds} \to \text{solve} \\
\to\ \text{estimate constraint values } (\widehat{MV}, \sigma) \to \text{tree-search reformulations under } \hat{V} \to \text{new } \mathcal{P} \\
\to\ \text{predict} \to \text{select experiments (falsify + discriminate)} \to \text{refine domains, revise models} \to \text{induce operators}
\end{array}
}
$$

$$
\boxed{
\begin{array}{c}
\textbf{Outer loop (per planning epoch)}\\[2pt]
\text{update technology beliefs } b_t \to \text{optimize roadmap (stopping, risk, migration)} \\
\to\ \text{emit scarcity prices } \lambda \text{, cuts, counterexamples to inner loop} \\
\to\ \text{select frontier interventions } do(u) \to \text{deploy} \to \text{measure delivered service} \to \text{recalibrate}
\end{array}
}
$$

## 22. Hypotheses and experimental design

| # | Hypothesis | Test |
|---|---|---|
| A | Explicit models transfer: structure-first agents beat fixed-environment RL after workload/technology shift | train on workload family 1, evaluate zero/few-shot on family 2 at **equal total compute and equal oracle-query budget** (else structure-first wins unfairly by spending more on solvers and high-fidelity queries); metric: regret vs oracle |
| B | Hidden-falsifier ensembles beat Monte Carlo of one simulator | design under §9's role split vs single-sim; the held-out family must be **independently constructed** (different codebase, different modeling idiom) — a re-parameterized copy tests nothing about model-form generalization |
| C | Constraint-value analysis finds bottleneck-breaking designs that reward search misses | four arms: random reformulation; unguided proposal (no $MV$ signal); $\widehat{MV}$-guided; oracle-$MV$-guided — separating the value of the signal from the value of the operator library; metric: dominating designs outside the initial $\mathcal{X}$ |
| D | Reformulation-operator training increases design novelty at equal quality | train $R_\theta$ on derivation trees vs parameter search in fixed $\mathcal{P}$; novelty **conditioned on quality**: archive distance counted only for designs with $J(x) \le J_{\text{baseline}} - \epsilon$; headline metric: Pareto-improving designs unreachable in the initial $\mathcal{X}_0$ |
| E | Supply/availability modeling changes designs materially | compare $x^*$ with and without §16 constraints; measure decision divergence, not just objective |
| F | Optionality-preserving roadmaps win under realized uncertainty | backtest (§19.3): regret distribution of adaptive vs committed roadmaps |
| G | Causal frontier-expansion improves long-horizon delivered value | simulated world with a ground-truth SCM; compare policies with the causal model ablated to observational forecasting |
| H | **Formulation transfer**: the system constructs useful formulations in an unseen domain | train on chiplet partitioning, memory hierarchy, and scheduling; test on a domain never seen — NoC topology, power delivery, or cooling — requiring discovery of $Z, H, C, B$ and operator applications, not just optimization; compare learned vs fixed formulation on downstream regret, oracle-query count, and certificate survival |

Minimal viable program, in order: (1) inner loop §§2–5 + §9 on open accelerator simulators, with the residual taxonomy live; (2) the chiplet problem of §11 end-to-end, including certificate localization and revocation; (3) the operator library of §7 on that same testbed (hypothesis D); (4) the backtest of §19 on public technology histories; (5) the coupled loops of §17 in a simulated world with a ground-truth causal model (hypothesis G); (6) the cross-domain transfer test (hypothesis H) — the direct measurement of whether an applied mathematician, rather than a chip-design heuristic engine, has been trained. Hypothesis H is the program's most fundamental claim; the others are instrumental to it.

## 23. Thesis

The inner loop asks: *what is the right mathematical model of this engineered system?* The outer loop asks: *what is the right model — predictive and causal — of the technological world in which it must exist?* One scientific method runs at both levels: hypothesize structure, commit to a model, let reality attack it — simulators and silicon below, markets and roadmaps above — revise, and, where the causal model licenses it, act to change what reality permits. The central intellectual move is that the AI searches not only design space but model space and formulation space; the machinery is what makes that claim testable — certificates whose domains shrink before they die, bounds with epistemic status that steer compute between designing and understanding, operators that make reformulation a move and can themselves be induced, experiments chosen for information and not only for failure, prices that couple engineering to industry, and interventions distinguished from forecasts. The target is an AI that moves between equations and transistors, and between what is physically optimal and what can actually be built, at the level of rigor this document demands of its own claims: stated so that they can be refuted.

---

## References

[1] J.-W. Hong and H. T. Kung. "I/O complexity: The red–blue pebble game." *STOC*, 1981.
[2] D. Irony, S. Toledo, A. Tiskin. "Communication lower bounds for distributed-memory matrix multiplication." *J. Parallel Distrib. Comput.* 64(9), 2004.
[3] S. Williams, A. Waterman, D. Patterson. "Roofline: an insightful visual performance model for multicore architectures." *CACM* 52(4), 2009.
[4] C. H. Stapper. "Defect density distribution for LSI yield calculations." *IEEE Trans. Electron Devices* 20(7), 1973.
[5] Universal Chiplet Interconnect Express (UCIe) Consortium. *UCIe Specification 1.1*, 2023 — energy-efficiency targets per bit for standard and advanced packaging.
[6] M. Horowitz. "Computing's energy problem (and what we can do about it)." *ISSCC*, 2014.
[7] S. Boyd, S.-J. Kim, D. Patil, M. Horowitz. "Digital circuit optimization via geometric programming." *Operations Research* 53(6), 2005.
[8] A. Dixit and R. Pindyck. *Investment under Uncertainty.* Princeton University Press, 1994.
[9] L. Kaelbling, M. Littman, A. Cassandra. "Planning and acting in partially observable stochastic domains." *Artificial Intelligence* 101, 1998.
[10] J. Birge and F. Louveaux. *Introduction to Stochastic Programming.* 2nd ed., Springer, 2011.
[11] R. T. Rockafellar and S. Uryasev. "Optimization of conditional value-at-risk." *Journal of Risk* 2(3), 2000.
[12] R. Pope et al. "Efficiently scaling transformer inference." *MLSys*, 2023.
[13] J. Ainslie et al. "GQA: Training generalized multi-query transformer models from multi-head checkpoints." *EMNLP*, 2023.
[14] Y. Leviathan, M. Kalman, Y. Matias. "Fast inference from transformers via speculative decoding." *ICML*, 2023.
[15] NVIDIA Corporation. *Blackwell architecture datasheet* (B200-class: dense FP8 throughput and HBM3e bandwidth), 2024 — used as a representative operating point, not an endorsement.
[16] A. Mirhoseini et al. "A graph placement methodology for fast chip design." *Nature* 594, 2021.
[17] A. Novikov et al. "AlphaEvolve: a coding agent for scientific and algorithmic discovery." Google DeepMind, 2025.
[18] Google. *Environmental Report* (fleet-wide trailing PUE ≈ 1.10), 2024; Uptime Institute *Global Data Center Survey* (industry average ≈ 1.5), 2024.
[19] J. Rand et al. "Queued Up: Characteristics of power plants seeking transmission interconnection." Lawrence Berkeley National Laboratory, 2024 — typical request-to-operation durations of ~4–7 years.
[20] J. F. Benders. "Partitioning procedures for solving mixed-variables programming problems." *Numerische Mathematik* 4, 1962; A. M. Geoffrion. "Generalized Benders decomposition." *JOTA* 10, 1972.
[21] J. Pearl. *Causality: Models, Reasoning, and Inference.* 2nd ed., Cambridge University Press, 2009.
[22] M. C. Kennedy and A. O'Hagan. "Bayesian calibration of computer models." *Journal of the Royal Statistical Society B* 63(3), 2001.
[23] K. Chaloner and I. Verdinelli. "Bayesian experimental design: a review." *Statistical Science* 10(3), 1995.
