# The Applied Mathematician's Approach to Chip and Compute-System Design

**A Concrete Research Design in Two Parts**

*Part I — Model the engineered system. Part II — Model the evolving world in which it must be built.*

---

## Abstract

Existing AI-for-chip-design systems optimize inside a problem someone else formulated: the design variables, objectives, constraints, and simulators are given. This paper specifies, concretely, a system that *constructs* the formulation. Part I defines the design problem as a tuple $\mathcal{P} = (W, Z, H, X, J, C, U)$ and gives explicit, falsifiable mathematics for each component: communication lower bounds from red–blue pebbling, roofline-style bottleneck decompositions, yield-driven chiplet partitioning, geometric-programming circuit models, and a designer/falsifier adversarial loop with machine-checkable structure certificates. Part II lifts the formulation to a multi-stage stochastic program over a time-varying feasible set $\mathcal{X}(t,\omega,g)$: technology readiness as a Markov chain, deployment as an optimal-stopping problem, supply and power as capacity constraints with shadow prices, and risk via CVaR and minimax regret. The two levels are coupled by Lagrangian price coordination: the industrial layer prices scarce resources; the technical layer redesigns against those prices. Every model in this paper is stated in a form that a simulator, a dataset, or a historical backtest can refute.

---

# Part I — Model the Engineered System

## 1. The design problem is the primary object

A conventional optimizer solves $\min_{x \in \mathcal{X}} J(x)$ s.t. $g_i(x) \le 0$. The architect's real output is the problem itself:

$$
\mathcal{P} = (W, Z, H, X, J, C, U)
$$

| Symbol | Meaning | Concrete instance (running example: LLM serving) |
|---|---|---|
| $W$ | workload distribution | mixture of prefill and decode requests; context length $s \sim \text{LogNormal}$; batch process $B_t$ |
| $Z$ | representation | dataflow graph with per-edge byte counts, per-node FLOPs, reuse distances |
| $H$ | structural hypotheses | "decode is bandwidth-bound", "attention is low-rank-compressible", "KV reuse is temporally local" |
| $X$ | design variables | SRAM/HBM capacities, NoC topology, tile sizes, chiplet partition, precision |
| $J$ | objectives | tokens/s/W, tokens/s/\$, tail latency |
| $C$ | constraints | typed: logical, physical, resource, probabilistic, conventional (§6) |
| $U$ | uncertainty | workload drift, process variation, simulator model-form error |

The AI's competence is measured on $\mathcal{P}$-construction quality, not only on $x^*$ quality. §9 gives the metric.

## 2. Workload representation with numbers attached

Represent a workload phase as a computation on which we can compute two quantities exactly: FLOPs $F$ and minimum data movement $Q$.

**Transformer decode, per token, per layer** (model dim $d$, batch $B$, context $s$, bytes/element $b$):

- GEMM (QKVO + MLP): $F \approx 24 B d^2$ FLOPs against $12 d^2 b$ weight bytes → arithmetic intensity $I = 2B/b$. At $B=1$, FP8: $I = 2$ FLOP/byte.
- Attention score/value: reads $2 s d b$ KV bytes for $\approx 4 B s d$ FLOPs → $I = 2B/b$ again.

A representative 2026 accelerator has peak $\pi \approx 5\times10^{15}$ FLOP/s (dense FP8) and HBM bandwidth $\beta \approx 8\times10^{12}$ B/s, so the ridge point is $\pi/\beta \approx 600$ FLOP/byte. Decode at small batch sits **two to three orders of magnitude** below the ridge: the machine is a memory system with an arithmetic unit attached. Prefill GEMMs ($I \sim 10^2$–$10^3$) sit above it. This single computation dictates the architecture conversation, and the AI must be able to produce it unprompted from a workload trace.

**KV-cache capacity**, per token: $2 L d b$ bytes ($L$ layers). For $L=80$, $d=8192$, FP16: $2.6$ MB/token → an 8k-context sequence holds $21$ GB; a batch of 32 holds $687$ GB, far beyond one package's HBM. Constraint transmutations that dissolve this (each changes the *formula*, not the operating point): grouped-query attention divides by group count $g$; quantization divides by $b_{16}/b_q$; paging converts a capacity constraint into a bandwidth+latency cost; recomputation trades $Q \to F$.

## 3. Structure discovery: each hypothesis is a test, not a sentence

Every structural hypothesis $H$ ships as a triple $(H, \text{statistic}, \text{refutation search})$:

| Structure | Formal claim | Test statistic | Refutation search |
|---|---|---|---|
| Low rank | $\|A - A_r\|_F / \|A\|_F \le \epsilon$ at rank $r$ | SVD tail energy on held-out workloads | maximize tail energy over workload perturbations |
| Sparsity | $\|x\|_0 \le k$, support $S$ stable | Jaccard$(S_{w}, S_{w'})$ across distribution shift | adversarial input search for support churn |
| Monotonicity | $\partial T / \partial B_{\text{mem}} \le 0$ | finite differences on a sampled grid | gradient ascent on the violation $\max(0, \Delta T)$ |
| Convexity | $f(\lambda x + (1{-}\lambda) y) \le \lambda f(x) + (1{-}\lambda) f(y)$ | random-chord violation rate | maximize the chord gap over $(x, y, \lambda)$ |
| Separability | $J(x) \approx \sum_i J_i(x_i)$ | $R^2$ of additive fit; interaction ANOVA | search for high-order interaction terms |
| Conservation | flow balance $A f = 0$ on the traffic graph | residual $\|Af\|$ on simulator traces | trace mining for buffering/drop events |
| Invariance | $f(gx) = f(x)$ for group $G$ | orbit variance | symmetry-breaking instance search |

A hypothesis that survives yields a **Structure Certificate**: a machine-checkable record `{claim, domain of validity, statistic, threshold, counterexample budget spent, expiry condition}`. Certificates are cited by downstream models; when a certificate is revoked (a counterexample is found), every model citing it is flagged for revision (§8).

**Decision relevance.** Structure is scored by its effect on decisions, not its truth alone:

$$
V(H) = \alpha\,\Delta C_{\text{model}} + \beta\,\Delta E_{\text{pred}} + \gamma\,\Delta C_{\text{solve}} + \delta\,\Delta J_{\text{design}}
$$

— change in model compactness, prediction error, solve cost, and realized design quality. A true-but-useless symmetry scores $\approx 0$ and is dropped.

## 4. Communication lower bounds before search

The red–blue pebble game (Hong–Kung) gives I/O lower bounds against fast-memory size $M$. For $n \times n$ matrix multiply:

$$
Q(n, M) \;\ge\; \frac{n^3}{2\sqrt{2}\,\sqrt{M}} - M ,
$$

attained (up to constants) by $\sqrt{M}$-tiling. Consequences the AI must derive, not memorize:

1. Attainable intensity for GEMM scales as $\sqrt{M}$ — SRAM capacity, not FLOPs, sets the roofline position for compute-bound kernels.
2. Standard attention over context $s$ has $Q = \Omega(s d)$ per query *regardless of tiling* — the KV cache must be read; only algorithmic change (sparse/linear attention, KV compression) moves this bound. Distinguishing "schedule can fix it" from "only algorithm can fix it" is exactly what a lower bound is for.
3. For a $p$-way tensor-parallel GEMM, per-step all-reduce volume is $\Theta(B d)$ per device independent of $p$ — scaling out does not amortize activation communication, which is why the interconnect term in §5 saturates.

Rule: **no search is launched on a cost the AI has not bounded.** If the bound and the surrogate disagree by more than the surrogate's certified error, the surrogate is wrong.

## 5. The bottleneck model and its falsification

First model, four terms:

$$
T = \max\left( \frac{F}{\pi\,u_c},\; \frac{Q_{\text{mem}}}{\beta\,u_m},\; \frac{Q_{\text{net}}}{\beta_{\text{net}}\,u_n},\; T_{\text{sync}} \right)
$$

with utilization factors $u \in (0,1]$ calibrated per regime. Workloads are modeled as a regime mixture $W = \bigcup_k W_k$ (compute-, memory-, network-, sync-bound), and the design target is $\min_x \mathbb{E}_{w \sim P(W)} J(x, w)$ — never a single benchmark point.

This model is deliberately falsifiable: it predicts no overlap between compute and movement. Measured $T$ below the max (overlap) or above it (contention, e.g., NoC congestion inflating effective $\beta$) are *structured residuals* that force specific model upgrades — an overlap coefficient, or a queueing term $\mathbb{E}[T_q] = \frac{\rho}{1-\rho}\cdot\frac{\bar{S}(1+C_v^2)}{2}$ (M/G/1) on the contended resource. The residual taxonomy in §8 makes this loop mechanical.

## 6. Constraints are typed; only some are laws

$$
C = C_{\text{logical}} \cup C_{\text{physical}} \cup C_{\text{resource}} \cup C_{\text{probabilistic}} \cup C_{\text{conventional}}
$$

- **Logical** ($\text{RTL} \models \text{Spec}$): never traded, never softened.
- **Physical** (timing, thermal, IR drop): softenable only against a physics model, e.g. chance constraint $\Pr[t_{\text{path}} > t_{\text{clk}}] \le \epsilon$ under process variation.
- **Resource** (power, area, BW): carry duals; primary innovation signals (§7).
- **Probabilistic / robust**: $\Pr[g(x,\xi) > 0] \le \epsilon$ or $g(x,\xi) \le 0\ \forall \xi \in \Xi$.
- **Conventional**: cache-line width, page size, coherence protocol, IEEE-754 formats. For each, the AI must answer: *physics or convention?* Conventions are candidates for **constraint transmutation** — replacing the constraint with new variables: defect rate → redundancy allocation (yield constraint becomes a spare-row/column sizing problem); monolithic yield → partition + D2D links (§10); DRAM capacity → compression + recomputation (capacity → bandwidth/compute); runtime nondeterminism → static scheduling (a verification constraint becomes a compiler problem).

## 7. Duals are the invention signal

Solve the design problem, then read the multipliers. If constraint $i$ is active with $\lambda_i \gg 0$, the system does not just report $x^*$; it opens a reformulation episode on constraint $i$: (a) why does $g_i$ exist (physics/convention)? (b) can transmutation introduce variables $z$ making $g_i(x, z)$ slack? (c) what is the *option value* of relaxing it, $\lambda_i \cdot \Delta g_i$, versus the cost of the transmutation?

Concrete instance: at small-batch decode the binding constraint is HBM bandwidth with $\lambda_{\beta} \approx \partial(\text{tokens/s})/\partial \beta \approx I_{\text{decode}}/1$ tokens per B/s — enormous relative to $\lambda_\pi \approx 0$. The mechanical response is "buy more bandwidth"; the transmutation responses are speculative decoding (raises effective $B$, hence $I$), weight/KV quantization (cuts $Q$), and near-memory compute (moves work across the constraint boundary). A trained system must produce all three *from the dual*, not from a menu.

## 8. Simulation as falsification, not as environment

Oracles form a hierarchy of increasing cost and rigor: analytical bound → surrogate → cycle-approximate sim → RTL sim → formal → synthesis → physical design → emulation → silicon. The model predicts $\hat{y} = M(x)$; oracle $k$ returns $y_k = S_k(x)$; the object of interest is the residual $r_k = y_k - M(x)$, interpreted against a taxonomy:

| Residual signature | Diagnosis | Action |
|---|---|---|
| bias, grows with contention $\rho$ | missing queueing state | add contention term |
| bias, grows with $s$ or reuse distance | wrong locality model | re-fit reuse distribution |
| heteroscedastic noise | missing nondeterminism source | model or *remove* it (make the design more legible, §9) |
| sign flip across regimes | wrong regime boundary | re-cluster $W_k$ |
| violation of a certificate | structural hypothesis false | revoke certificate; flag all citing models |

**Designer/falsifier alternation.** Maintain three co-evolving archives: designs $\mathcal{D}$, models $\mathcal{M}$, counterexamples $\mathcal{C}$. Iterate

$$
x_k = \arg\min_x J(x; \Xi_k), \qquad \xi_k^* = \arg\max_{\xi} \text{Violation}(x_k, \xi), \qquad \Xi_{k+1} = \Xi_k \cup \{\xi_k^*\},
$$

with the falsifier drawing from simulator *ensembles* $S_m(x,\xi)$ across model families — distinguishing workload, parameter, model-form, and implementation uncertainty. A large Monte Carlo sweep of one simulator is not epistemic robustness. Cross-level feedback is mandatory: a synthesis-reported $f_{\max}$ miss updates the architecture-level timing prior; a physical-design wiring blowup updates the communication cost $c_e$ upstream.

## 9. Legibility as an objective; formulation quality as a metric

Two designs with equal PPA can differ by orders of magnitude in predictability. Define $C_{\text{model}}(x)$ = (hidden dynamic state count, nondeterminism sources, calibration sample complexity, surrogate error at fixed budget) and optimize $J = J_{\text{PPA}} + \lambda\, C_{\text{model}}$. A legible design is cheaper to optimize, verify, compile to, and *redesign by AI in the next iteration* — the term is a bet on compounding returns.

**Formulation quality metric** (for §1): given the AI's problem $\mathcal{P}$ and a reference solver budget, score $\mathcal{P}$ by (i) regret of $x^*_{\mathcal{P}}$ under the *true* downstream evaluator, (ii) certificate survival rate at higher-fidelity oracles, (iii) solve cost. This makes "did it pick the right problem" measurable.

## 10. Worked integration: chiplet partitioning

Yield follows a negative-binomial defect model, $Y(A) = (1 + AD/\alpha)^{-\alpha}$ (defect density $D \approx 0.1\text{–}0.2\,\text{cm}^{-2}$ at a mature leading-edge node, clustering $\alpha \approx 2\text{–}4$). Cost per good monolithic die of area $A$ scales as $C(A) \propto A / Y(A)$ — superlinear. Partition the block graph $G$ into $k$ chiplets $\{G_1..G_k\}$, cut traffic $q(\mathcal{P})$ bytes/s:

$$
\min_{k,\ \mathcal{P}(G)} \;\; \underbrace{\sum_{j=1}^{k} \frac{C_{\text{wafer}} A_j / A_{\text{wafer}}}{Y(A_j)}}_{\text{silicon}} \;+\; \underbrace{C_{\text{pkg}}(k)}_{\text{assembly, test, interposer}} \;+\; \underbrace{\lambda_E\, e_{\text{D2D}}\, q(\mathcal{P}) + \lambda_T\, \Delta T_{\text{D2D}}(\mathcal{P})}_{\text{communication}} \quad \text{s.t. } A_j \le A_{\max},\ q(\mathcal{P}) \le B_{\text{D2D}}
$$

with $e_{\text{D2D}} \approx 0.25\text{–}0.5$ pJ/bit (UCIe-class) versus $\approx 0.05\text{–}0.1$ pJ/bit on-die. Everything in Part I appears here: a lower bound (min-cut of $G$ bounds $q$), a structure certificate (is $G$ nearly separable?), typed constraints (reticle limit = physics; D2D protocol = convention), duals ($\lambda$ on $B_{\text{D2D}}$ prices the next packaging generation), and falsification (the linear D2D energy model is attacked by a thermal-coupling simulator). Solved as ILP/graph partitioning at fixed $k$, outer loop on $k$. This is also the hinge to Part II: $Y$, $C_{\text{pkg}}$, and $B_{\text{D2D}}$ are time-varying, uncertain, and supply-constrained.

## 11. Solver selection follows structure

| Discovered structure | Solver class |
|---|---|
| posynomial delay/energy models (gate sizing, pipelines) | geometric programming |
| convex/conic surrogate | interior point; exploit duals directly |
| discrete + coupling (partition, mapping, tiling) | ILP / CP-SAT / branch-and-bound + decomposition |
| expensive black-box oracle | Bayesian optimization, active learning |
| sequential structure (placement, scheduling games) | DP / RL / tree search |
| runtime resource control | MPC, stochastic control |
| heavy uncertainty | robust / distributionally robust optimization |

RL is one row, chosen when its preconditions hold — not the default.

---

# Part II — Model the Evolving World

## 12. The feasible set moves

$$
x_t \in \mathcal{X}(t, \omega, g) = \bigcap_{r \in \{\text{silicon, memory, packaging, network, power, cooling, mfg, supply, facility}\}} \mathcal{X}_r(t, \omega, g)
$$

where $t$ is time, $\omega$ a scenario (technology and supply realization), $g$ geography. Each technology $j$ occupies a readiness state in $\{\text{lab}, \text{qualified}, \text{volume}, \text{economic}\}$, modeled as a Markov chain with transition intensities calibrated from historical roadmaps; availability $\tau_j$ is a first-passage time with distribution $P_j(\tau)$, and volume, cost, and yield are stochastic processes $Q_j(t,\omega),\ C_j(t,\omega),\ Y_j(t,\omega)$. *Possible* $\ne$ *qualified* $\ne$ *at volume* $\ne$ *economic*; conflating these states is the canonical roadmap failure.

## 13. The objective: discounted useful deployed compute

$$
\max_\pi\; \mathbb{E}\left[ \sum_{t=0}^{T} \beta^t\, U\big(N_t, P_t, W_t\big) \right], \qquad U = N_t \cdot P_t \cdot v(W_t)
$$

deployed volume × useful per-system performance × workload value. Two consequences worth stating as inequalities: a system with $0.8 P$ at $10N$ dominates; and for discount/growth conditions typical of the current buildout, a 12-month-earlier system at $P_B < P_A$ wins whenever $\sum_t \beta^t N_B(t) P_B > \sum_t \beta^t N_A(t) P_A$ — which holds for surprisingly large $P_A/P_B$ when $N(t)$ ramps over years. **Best chip $\ne$ best compute system.**

## 14. Roadmapping as multi-stage stochastic control

State $x_t$ (deployed fleet, in-flight designs, qualified technologies, capital), decisions $u_t$ (design starts, qualification investments, volume commitments, deployment), scenario tree $\{\omega\}$ with nonanticipativity ($u_t$ measurable w.r.t. information at $t$):

$$
\min_{\pi}\; \rho\!\left[ \sum_t \beta^t J(x_t, u_t, \omega_t) \right] \quad \text{s.t. } x_{t+1} = F(x_t, u_t, \omega_t),\ x_t \in \mathcal{X}(t,\omega)
$$

where $\rho$ is a risk functional chosen per decision: expectation for repeated small bets; $\text{CVaR}_{0.95}(L) = \min_\eta \eta + \frac{1}{0.05}\mathbb{E}[(L-\eta)^+]$ for ruin-relevant exposure (single-source components); minimax regret $\min_\pi \max_\omega [J(\pi,\omega) - J^*(\omega)]$ for technology-timing bets where priors are unreliable.

**Deploy-or-wait is optimal stopping.** With a candidate technology arriving as a stochastic improvement $\Delta$:

$$
V_t(s) = \max\Big\{ \underbrace{\textstyle\sum_{t' \ge t} \beta^{t'-t} N(t') P_{\text{now}}}_{\text{deploy}},\;\; \underbrace{\mathbb{E}\big[V_{t+1}(s') \mid s\big] - c_{\text{delay}}}_{\text{wait}} \Big\}
$$

yielding a threshold policy: wait only while $\mathbb{E}[\Delta V_{\text{tech}}] > c_{\text{delay}}$. The delay cost is dominated by the deployment ramp foregone, which is why "wait for the better interconnect" so often loses. **Optionality** is the architectural dual of waiting: a design holding an upgrade path (socket, protocol, power envelope compatible with the successor) truncates the regret distribution without paying the delay — and its value is computable as the difference of the two stopping problems.

## 15. Supply, power, and facilities are constraints with prices

- **Supply:** $N_t\, d_j(x) \le Q_j(t,\omega)$ per component $j$ (HBM stacks, CoWoS-class interposer area, optics). Dual $\lambda_j \gg 0$ triggers the same reformulation episode as §7, now with industrial moves added: substitute, redesign to reduce $d_j$, qualify second source, sign volume commitment (which *changes* $Q_j$ — see §17).
- **Power:** $P_{\text{sys}} = P_{\text{compute}} + P_{\text{mem}} + P_{\text{net}} + P_{\text{conv}} + P_{\text{cool}}$; facility constraint $\text{PUE} \cdot \sum P_{\text{sys}} \le P_{\text{grid}}(g, t)$. At $\sim 1$ kW+/accelerator and PUE 1.1–1.3, a 1 GW campus caps near $\sim 7\times10^5$ accelerators — power, not silicon, is the binding constraint of the current buildout, and $\lambda_{\text{power}}$ backpropagates into Part I as pressure on data movement and voltage architecture ($I = P/V$: higher distribution voltage cuts conductor mass but demands facility conversion stages with their own $\tau$).
- **Facilities:** lead times $\tau_{\text{grid}} \sim 3\text{–}7$ y, $\tau_{\text{substation}} \sim 2\text{–}4$ y, $\tau_{\text{building}} \sim 1\text{–}2$ y dominate silicon design cycles. Any roadmap whose critical path ignores $\mathcal{X}_{\text{facility}}(t)$ is infeasible regardless of PPA.
- **Geography:** $g$ indexes power price and availability, cooling, network latency to demand, equipment import constraints; the optimal fleet is heterogeneous in $g$ — homogeneity is a convention, not a law.

Migration paths are explicit design objects: optimize $\sum_t J(x_t) + C_{\text{mig}}(x_t, x_{t+1})$ over sequences $A \to H_1 \to \cdots \to B$, where a transitional $H_i$ with worse steady-state efficiency can strictly dominate by de-risking schedule and stranded capital.

## 16. Coupling the levels: price coordination

Part I produces an architecture menu $\mathcal{A} = \{a_1..a_N\}$ with resource vectors $d(a) = (\text{HBM stacks}, \text{interposer mm}^2, \text{W}, \text{network BW}, \ldots)$. Part II solves the roadmap over $\mathcal{A}$ and returns duals $\lambda = (\lambda_{\text{HBM}}, \lambda_{\text{pkg}}, \lambda_{\text{power}}, \ldots)$. Part I then re-optimizes the *priced* objective

$$
\min_x\; J_{\text{PPA}}(x) + \lambda^\top d(x)
$$

and returns new candidates; iterate (Lagrangian / Benders-style decomposition of the joint problem). The loop's signature behavior: $\lambda_{\text{HBM}} \gg 0$ → Part I re-derives the memory hierarchy (more SRAM, compression, recomputation — §2's transmutations, now economically motivated); $\lambda_{\text{power}} \gg 0$ → Part I attacks $Q$ and voltage. Bottleneck economics, not taste, steers architecture search.

## 17. Frontier expansion: the feasible set is a design object

Beyond transmuting constraints (Part I) the system may act on the world so tomorrow's $\mathcal{X}$ is larger: $\mathcal{X}_{t+1} = F(\mathcal{X}_t, u_t, \omega_t)$. Concrete action classes with their mathematical effect: second-source qualification (replaces $Q_j$ by $Q_j^{(1)}+Q_j^{(2)}$ and cuts $\text{CVaR}$ tail), volume commitment (shifts $P_j(\tau)$ left in exchange for capital risk), interface standardization (turns a bespoke dependency edge into a market, collapsing $\lambda_j$), co-investment in an enabling technology (raises a Markov transition intensity). Each action is valued exactly as a constraint relaxation: worth $\approx \lambda_j \Delta Q_j$ (or the option-value analogue) against its cost. The composed primitive is:

$$
\text{optimize} \to \text{read duals} \to \text{transmute the constraint} \to \text{expand the frontier}.
$$

## 18. Artifacts and backtesting

Three machine-checkable artifacts make Part II auditable:

1. **Technology Dependency Graph** $G_T$: nodes = technologies/suppliers/facilities, edges = dependencies with $(\tau, Q, C, Y)$ distributions; the system must surface critical paths, single points of failure, and high-$\lambda$ edges.
2. **Roadmap Certificate**: nominal path $x_0 \to x_1 \to x_2$, fallback branches, trigger conditions ("if $\tau_{\text{HBM,next}} > $ Q3, switch to branch B"), frontier-expansion actions with their $\lambda\Delta Q$ justification, and preserved options with their computed option value.
3. **Calibration record**: at historical date $t$, hide the future, have the system forecast $(\tau_j, Q_j, C_j)$ and design the $N$-year roadmap; score forecast calibration (CRPS) and roadmap regret against the realized world. Technology forecasting and roadmap design become supervised, backtestable tasks — the DRAM, NAND, node-cadence, and interconnect histories are the training set.

## 19. Hypotheses and experimental design

Falsifiable claims, each with its measurement:

| # | Hypothesis | Test |
|---|---|---|
| A | Explicit models transfer: structure-first agents beat fixed-environment RL after workload/technology shift | train on workload family 1, evaluate zero/few-shot on family 2; metric: regret vs oracle |
| B | Simulator ensembles beat Monte Carlo of one simulator | design under ensemble vs single-sim; evaluate on held-out simulator family |
| C | Dual-guided reformulation finds bottleneck-breaking designs parameter search misses | count designs outside the initial $\mathcal{X}$ that dominate; ablate the dual signal |
| D | Constraint-transmutation training increases design novelty at equal quality | novelty = distance from training archive under a fixed design embedding |
| E | Supply/availability modeling changes designs materially | compare $x^*$ with and without §15 constraints; measure decision divergence, not just objective |
| F | Optionality-preserving roadmaps win under realized uncertainty | backtest (§18.3): regret distribution of adaptive vs committed roadmaps |
| G | Frontier-expansion actions improve long-horizon deployed value | simulated world model with endogenous $\mathcal{X}_t$; compare with expansion actions ablated |

Minimal viable program, in order: (1) Part I §§2–5 on open accelerator simulators with the residual taxonomy of §8; (2) the chiplet problem of §10 end-to-end, including certificate revocation; (3) the backtest of §18 on public technology histories; (4) the coupled loop of §16 in a simulated world.

## 20. Thesis

Part I asks: *what is the right mathematical model of this engineered system?* Part II asks: *what is the right model of the technological world in which it must exist?* The unifying loop is one scientific method applied at two levels — hypothesize structure, commit to a model, let reality (simulators below, markets and roadmaps above) attack it, revise, and occasionally act to change what reality permits. The target capability is an AI that moves between equations and transistors, and between what is physically optimal and what can actually be built, at the level of rigor this document demands of every one of its own claims: stated so that it can be refuted.
