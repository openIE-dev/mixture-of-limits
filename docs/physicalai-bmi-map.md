# Physical AI BMI → Mixture of Limits / OpenIE map

**Date:** 2026-09-30 (America/New_York)  
**Parent:** [BLUEPRINT.md](../BLUEPRINT.md) §4 and §14 · collapse gears: [quantum-thermo-collapse.md](./quantum-thermo-collapse.md)  
**Sources fetched:** [physicalai-bmi.org](https://physicalai-bmi.org/) · [efa.physicalai-bmi.org](https://efa.physicalai-bmi.org/) · [energy.physicalai-bmi.org](https://energy.physicalai-bmi.org/) · [/thermo](https://energy.physicalai-bmi.org/thermo) · [/neuromorphic](https://energy.physicalai-bmi.org/neuromorphic)  
**Scope:** Read-only field map. No crate API change. No claim that MoL v0.1 runs Ferric, ferrotherm, p-bits, or a robot. Landauer figures below are **the sites’ numbers**, kept labelled; MoL receipts still treat `landauer_floor_J` as an **estimate ≠ measured joules**.

---

## One-paragraph MoL read

Physical AI BMI (Charlot Lab, Institute for Physical AI at the John Bailey Institute) is the embodied cousin of MoL **collapse intelligence**: one scalar energy is descended to act **and** is a machine-checked proof the closed loop will not diverge, before commit, on the device. That is WCA’s propose → certify → commit|refuse with the neural policy demoted to a wrapped residual — Model LAST — not a bigger scorer. Thermodynamic sampling and neuromorphic pages price the same Landauer gap MoL already annotates (arithmetic is not the bill; erase and especially **data movement** are), and they refuse to average modelled, simulated, and measured joules. The sites do **not** name MathGround, OpenIE, compute.openie.dev, or JouleDB; joules-per-task and the hub’s OER/1 receipt schema are **parallel vocabularies**, not shared types.

---

## What each site claims

### Hub — [physicalai-bmi.org](https://physicalai-bmi.org/)

**Thesis.** Machines that see, think, and act are an economics question whose speed is physical: energy, compute, and the loop between them. The Institute for Physical AI is a division of the John Bailey Institute, a 501(c)(3) aviation-education nonprofit (est. 2001). Work is taught, measured, and built in the open (Rust, browser, MuJoCo).

**Loop.** Perceive → simulate → act, many times a second, on the device. Before commit, the action is checked against “the one energy the body descends, a proof it won’t diverge.” The hub calls that closed loop **agency**; **intelligence** is named as the other axis.

**Nearby artifacts (titles only, not fetched in full).** Energy Lab; “Reading an Energy Claim”; “What a decision costs” (moving a decision off the generative path saves at most `1/(1−p)`); “The Two Operating Systems” (a body runs a substrate that acts before any signal is processed, and an information system that rides on top — data thesis scales only the second); hardware lottery for energy-based models; “Where Is the Energy Reporting?” (embodied energy undisclosed to buyers/regulators); OER/1 and DRIFT/1 (**a receipt schema for energy claims**, and a drift-learning benchmark); RELAX/1 (ten operations, an exchange form, a lowering matrix across substrates); “Joules per Punch”; global energy database (“eighty-five claims, each with its verification grade”); “What Do You Type?” (toolchain of thermodynamic computing — hardware funded, **interface** now the binding constraint).

### EFA — [efa.physicalai-bmi.org](https://efa.physicalai-bmi.org/)

**Byline on the page:** Energy First Architecture, Charlot Lab · Institute for Physical AI @ JBI.

**Thesis (narrow on purpose).** One scalar energy over a sparse-positive latent is at once (1) the policy a body descends to act, (2) the world model it predicts consequences in, and (3) a **machine-checked certificate** that the action will not diverge, before it commits. The proof is the objective, not a bolted-on verifier. Joules per task is the cost of carrying the proof, not a throughput contest. The on-device sparse substrate (pure Rust **Ferric**, WebGPU, browser tab) is called an **adjacent**, not part of the certificate claim.

**Not the claim.** Energy-as-score is table stakes and crowded: softmax as Boltzmann, JEPA as latent distance (kills Z), energy-based transformers that think by descent, concepts by summing energies. EFA says it does that and does **not** lead with it, and does **not** claim to beat frontier systems at scale.

**Certificate vs score.**

| | Energy-as-score (the field) | Energy-as-certificate (EFA) |
|---|---|---|
| Asks | Is this output a good answer? | Will this action keep the closed loop from diverging? |
| Is | Density / distance / compatibility, after the fact | Lyapunov guarantee, \(\dot V \le 0\) by construction |
| Holds | After the answer | Before commit, at the real dt, on the device |

**Key measured claims (their ledger; simulation, not yet hardware).**

- Port-Hamiltonian structure ⇒ Lyapunov certificate by construction, \(\dot V \le 0\) to machine precision at a **24-dimensional** state, where a box/SMT proof dies near 8D. A learned residual in the skew coupling cancels out of \(\dot V\) identically.
- 7-DOF arm: aggressive policy leaves the safe envelope 6126/12000 steps; certificate vetoes to **0/12000**, 143 µs/step, on-device. MuJoCo arm (real inertia, torque limits, energy from MuJoCo’s mass matrix): structural identity to 1.5e-9; naive 4263/6000 vs certificate **0/6000**.
- Compression rule: shrinking a learned certificate after training destroys coverage (median low-sixties, about the trivial \(V=|x|^2\) at 61%). Training factored from the start holds 100% of the domain at 16× fewer parameters. Compress only what the guarantee does not depend on.
- Untrusted observation (60,000 trials): scattered faults → trust-weighted fuser ~0% false-safe at ~20% conservatism. Coordinated spoof of 5/9 sensors: fuser locks the self-consistent lie (36% vs 28% false-safe). **Refusal** (rival internally-consistent explanation → decline to certify) cuts false-safe under 2% at 95% abstention. Past a coordinated majority, readings no longer contain the state; needs an unforgeable anchor. Refusal is part of the certificate.
- Seatbelt is light (microjoules, O(1)) and wraps a frozen policy **it did not train** (a VLA of any size). The AI being certified need not be small. Bounds they price: exhaustive worst-case proof of a *learned* energy is 2–4D (hence structure, not proof); underactuated bodies re-introduce the hard part; results in simulation; contact/manipulation certification open; external released VLA in the loop still the remaining integration step.
- “Intelligence = economy of effort”: as the model learns, descent steps 8→4; the landscape recodes so the hard problem becomes easy — fewer watts, not a bigger model. Pendulum swing-up ≈ 110 kFLOP vs ≈ 14 GFLOP for one token of a 7B model (~10⁵×), and the cloud model cannot run the continuous loop.
- Adjacent, **kept distinct**: energy-minimization as law discovery (oscillator, Lorenz, Burgers, Fisher–KPP, pendulum invariant corr 0.99, lynx–hare Lotka–Volterra). Not the certificate claim.
- Walk-back they publish: an earlier “feedforward scores 0%” claim was wrong and withdrawn. A field-scale energy-conserving surrogate did **not** beat a naive force net even with exact second-order gradients — structural negative. AI-for-math verifier is mechanism-sound but gated on a Lean-task encoder; general embeddings at chance.
- Krakauer / SFI: physics → adaptation → agency least-action ladder; “materiality does computation” (physical descent packs discs 100% where abstract search collapses to 0%).

**Landauer, as written on EFA.** Landauer floor for erasing one bit is \(k_B T \ln 2\), about \(2.9 \times 10^{-21}\,\mathrm{J}\) at room temperature. Contemporary digital arithmetic spends on the order of picojoules per operation, five to six orders above that floor, and most of it is **moving operands**, not the arithmetic. Incumbent hardware (von Neumann bottleneck, Dennard, x86, CUDA) is an investment artifact, not a physical optimality proof. Target metric: **joules per completed task** plus acquisition cost of the hardware — total cost of operation, not a benchmark score.

**Field they say they build on, not precede.** EBT-Policy (2510.27545, energy is a scorer not a Lyapunov certificate); ECD (2606.21646); V-JEPA 2 (proto-EFA, frozen, no certificate); Certified World Models (2606.13092, certificate and action-objective stay two scalars); port-Hamiltonian certificates (2604.26172, 2512.24493); runtime VLA safety (SafeVLA, Pre-VLA, AEGIS); measurement-robust barriers under spoofing.

### Energy Lab — [energy.physicalai-bmi.org](https://energy.physicalai-bmi.org/)

**Thesis.** Teaching apparatus for a claim you should not take on trust: **electricity, not transistors, now limits computing.** Two simulations and two calculations, on your own device. Every number labelled by kind. Leave able to: derive why moving data costs more than computing it; price a transition and name which assumption does the work; explain compute-by-settling; **refuse to average a modelled figure with a measured one.**

**Memory wall (labelled modelled, order-of-magnitude).** MAC ≈ 0.3 pJ, SRAM ≈ 5 pJ, DRAM ≈ 100 pJ, Landauer floor ≈ 3 zJ at 300 K. At 20% operands from off-chip DRAM: arithmetic 0.30 nJ, movement 24.0 nJ, movement share 98.8%, “above Landauer floor” \(8 \times 10^9\). Dial fetch to zero and the arithmetic is still nearly ten billion times the physics floor. Response they teach: not a faster multiplier — stop fetching, settle in place, compute in memory, or spend no multiply at all.

**Settle demo.** 100 coupled variables, \(2^{100}\) arrangements; a few dozen flips, no enumeration. Settling finds a **good** answer quickly and an **optimal** one rarely — useful under time pressure, unsuitable where only the exact best will do.

**Transition workbook (assumptions, labelled).** AI data-centre electricity 190 TWh in 2026 (IEA anchor), +22%/yr to 2030, +12%/yr to 2035. Scenarios vary amenable share (10/20/30%), gain (10/100/1000×), adoption. Amenable share is the least defensible input. Savings arrive late; cheaper compute historically becomes more compute. Case rests on **capability per joule**, not falling demand.

**Substrate × model-class map.** 84 organisations. Diagonal matters: a substrate that executes relaxation **and** a model class that computes by relaxing. Asterisk = learns at inference (still largely unclaimed). Clusters they count: 12 neuromorphic & spiking (largest, real neuron counts); 11 photonic (capital-rich, integration-poor); 17 model classes that learn at inference (algorithms before silicon); 6 graveyard rows (unverifiable claims, missing toolchain, or no model class that needed the hardware).

| Substrate | Energy-based | Hopfield / associative | Learns at inference |
|---|---|---|---|
| Thermodynamic / p-bit | Extropic DTM; Normal Computing | sampling-native candidate | p-bit Boltzmann (UCSB, Tohoku) |
| Ising / annealer | OIM-as-sampler (2026) | Toshiba SBM; Fujitsu DA; D-Wave; NTT CIM | — |
| Neuromorphic SNN | Darwin Monkey attractor nets | Loihi 2 / Akida on-chip plasticity * | PKU-CAS memristor |
| Photonic | photonic EBM proposals | photonic Ising (NTT, Lightelligence) | optical matmul, ternary |
| In-memory / CIM | memristor Hopfield crossbars | Hebbian outer-product native | CIM ternary (Witmem, Houmo) |
| FPGA settling + accounting | ternary settling fabric | settle ×2 * | **EFA on Ferric → fabric target *** |
| Digital GPU (incumbent) | EBT on GPUs | AXIOM (VERSES) * | Titans / TTT / DeltaNet *; BitNet b1.58 |

### Thermo — [energy.physicalai-bmi.org/thermo](https://energy.physicalai-bmi.org/thermo)

**Thesis.** Sampling as a substrate. Transistors draw random samples from their own thermal noise instead of computing them with arithmetic. Physics is public: Ising 1925, Glauber 1963, Gibbs 1984. Instrument must reproduce **Onsager’s 1944** 2D Ising solution before it reports throughput. Stack: **ferrotherm**, Apache-2.0, zero-dependency Rust, CPU → Wasm unchanged; joules ledger travels with the code. Sample certificates: temperature actually sampled, independent draws, and (when the model is small) distance from the exact distribution beside the noise floor — computed from the samples, so a broken sampler cannot certify itself.

**What a thermodynamic sampling unit charges for (vendor pre-silicon, arXiv:2608.01615 Table IV — their label).**

| Event | Figure | Label |
|---|---|---|
| Per node update | 7.09 fJ | Pre-silicon circuit simulation, vendor appendix |
| Reading everything, every step | 1.692 pJ ≈ 239 updates | Same source; “reading everything is already the wrong regime” |
| Per node written | 153.6 pJ ≈ 21,700 updates | Reprogramming capped near once per second |

Design sentence: the architecture wins where **many local updates happen between rare crossings** of the chip edge.

**How to read the 10,000× headline (they decompose it; do not average).**

- Measured on silicon: subthreshold-CMOS probabilistic bit, programmable bias, ~100 ns decorrelation, femtojoule-class randomness. That is the complete silicon list they accept.
- Simulated: ML image-generation on a software model of the proposed chip, parity with small GPU baselines.
- Projected: energy multipliers; analytic model with one of four terms measured; I/O excluded; refined vendor appendix ~10× more expensive than the headline’s coarse model.
- Measured, peer-reviewed: **5 to 18×** over optimised GPU/TPU samplers on sparse-graph FPGA probabilistic computers (Aadit et al., Nature Electronics 2022).

**Browser throughputs (labelled measured on their machines, idle).** Portable Rust: 73e6 updates/s one thread, 380e6 on eighteen threads; laptop GPU 9.35e9 in a browser tab. An earlier 11.6e6/thread was a busy-machine artifact and is corrected on the page.

**Embodied boundary (measured in simulation, on the vendor’s published degree-16 topology).** A robot reaching controller compiles onto that fabric. Discretised controller still reaches 90% of targets; compiled kernels plateau near **45%**. Missing piece: **multiplicative structure** — the reaching law multiplies state variables, and sparse local pairwise couplings plus a few hidden spins cannot route those products at this scale. They state no published work demonstrates a control workload on this hardware class. What would move it: published energy price of clamping an input (100 Hz loop vs ~1 Hz reprogramming cap), and capacity for products. Below roughly **one watt** of compute, the arm’s motors, not the model, set the bill.

**Compiler honesty.** Inequality → slack variable → equality → squared energy penalty, because squaring “at most three” would punish two as hard as four. Slack costs spins and must not appear in the answer. Values stay in the range you wrote.

### Neuromorphic — [energy.physicalai-bmi.org/neuromorphic](https://energy.physicalai-bmi.org/neuromorphic)

**Thesis.** Machines that wake up. A brain at about twenty watts keeps almost every neuron silent and pays only for spikes. No clock driving a dense matrix; nothing happens until a signal arrives. Instrument must reproduce Lapicque’s 1907 closed-form rate, and emit nothing below threshold, before it reports a count.

**What decides the win.** A synaptic event’s arithmetic is cheap; fetching the weight is not — same memory wall. The field’s line is **spikes per synapse per inference**: cross it and you re-read weights more often than a dense pass, and dense wins. Cited bounds, all below two: Davidson & Furber ≈ **1.72** (Front. Neurosci. 2021); Dampfhoffer et al. 0.15–1.38 (memory accesses put back in); Yan et al. arXiv:2409.08290 gives **0.78×** once data movement is counted.

**Provenance sting.** TrueNorth 26 pJ per synaptic event at 28 nm is measured silicon (Merolla et al., Science 2014). The oft-quoted Loihi **23.6 pJ/synaptic op** comes from a table captioned **pre-silicon** and was cited as a measurement for years. This review did not locate a published fetch-energy figure for any buyable part, so the instrument counts fetches and does not price them at zero. The missing experiment they want is a meter on a memory rail while a spiking net runs — a protocol, not a new chip.

---

## Relation to Landauer

| Site | What they do with Landauer | MoL hook |
|---|---|---|
| EFA | Floor \(k_B T \ln 2 \approx 2.9\times 10^{-21}\,\mathrm{J}/\mathrm{bit}\); digital ops ~pJ, 5–6 orders up, mostly operand motion. Headroom is an empirical architecture question, not a slogan. | Same constant MoL annotates as `landauer_floor_J`. **Do not** copy their pJ/op rhetoric onto a MoL receipt as `measured_j`. |
| Energy Lab | ≈ 3 zJ at 300 K; even a zero-DRAM MAC is ~\(10^{9}\)–\(10^{10}\) above the floor (modelled). Movement share ~99% once DRAM is in the path. | Explains why MoL’s coin-cell move is **don’t fetch / don’t erase / don’t open the model**, not a faster MAC. ThermoClass L0 (Lookup/Formula) is the “compute where the data already is” gear. |
| Thermo | The interesting cost is not Landauer-per-flip in the abstract; it is fJ-class noise updates vs pJ-class **reads and writes** at the chip edge, plus a ~1 Hz reprogramming cap. | Solver-tier settle is cheap only inside a fabric. Crossing the edge (I/O, reprogramming) is the joule floor that actually binds — cousin of MoL `energy` budget, still an estimate until metered. |
| Neuromorphic | Landauer is not the headline. The binding floor is **events × weight fetches**. A synaptic-op count that ignores fetches prices the wall at zero. | Activity sparsity is a VoI-like floor (pay only for spikes that clear threshold), not a Landauer derivation. Still label modelled vs measured. |

Shared discipline, strongest alignment with MoL §8: **a modelled figure, a stand-in, and a metered joule are different provenance and are never averaged.** EFA’s own walk-back (feedforward-0% withdrawn; energy surrogate loses to a force net) is the same honesty posture as “no fake RAPL.”

---

## Collapse intelligence

MoL collapse ([quantum-thermo-collapse.md](./quantum-thermo-collapse.md)): the answer comes from **settling, certifying, or rewriting** structure — not from residual tokens.

| BMI move | Collapse reading | Not this |
|---|---|---|
| Energy descent / Gibbs / Ising relax into an answer | **Settle = answer.** Energy Lab’s “any physics that falls downhill can compute.” Reserved MoL inspiration id `physical_settle`. | Not optimality. They state settle finds good, rarely best. Exact grammar still belongs to Formula. |
| EFA: the objective **is** the Lyapunov certificate, \(\dot V \le 0\) before commit | **Certify = the same object as the controller.** Stronger than “score then check.” Port-Hamiltonian identity is structural, dimension-free past the 24D demo. | Not a post-hoc verifier, not energy-as-score (EBT, JEPA). |
| Refusal under spoofed sensors | Certificate includes **decline**. A confident \(\dot V\) on a lie is not a noisy proof. | Not “more inference fixes a coordinated majority.” |
| Law discovery by fitting an energy | Collapse of data onto a governing law (Formula-adjacent). EFA **separates** this from the certificate claim. | Not automatic coverage of MathGround. |
| Neuromorphic silence | Collapse of **work**: most neurons do nothing; the machine pays the spike. | Not an energy-landscape ground state. Different gear (activity floor). |
| “Intelligence = economy of effort” (steps 8→4) | Past a floor, more descent does not buy the outcome; the landscape recodes. | Not “scale the model.” |
| Krakauer ladder (physics → adaptation → agency) | Agency = the certified loop on its own power; intelligence is the other axis (hub). | Not a ToE. MoL does not import SFI as a crate law. |

---

## MoL cascade gears

```text
Lookup → Formula → Solver → Model LAST
```

| Gear | BMI object that sits here | Replay / honesty |
|---|---|---|
| **Lookup** | Onsager / Lapicque closed forms used as **instrument self-checks** before a number is shown. Allow/refuse under sensor spoof (decline to certify). Not a big LUT of actions. | Deterministic check. WCA LUT certify is a cousin of the *veto*, not of EFA’s energy object. |
| **Formula** | Port-Hamiltonian structural identity (\(\dot V\) independent of the skew residual). Closed-form law recovery (ODE/PDE/invariant) as an **adjacent** suite. Slack-variable rewrite so an inequality is an equality you may square. | Deterministic / Composed when the identity holds. EFA: compressing a learned net after the fact destroys the certificate — train the form small. MoL rhyme: formula beats excess parameters. |
| **Solver** | Energy descent (EBT unrolled 2nd-order autograd; MPPI latent planning 39%→69%). Gibbs / checkerboard Ising on GPU or p-bit. ferrotherm: constraints + objective compile to spins. FPGA “settling fabric” row, with **EFA on Ferric** marked learns-at-inference and still a target. | Composed **if certified**. Uncertified samples stay stochastic. Thermo page: products (multiplies) do not route on degree-16 pairwise couplings — a real `primitive_gap` for control-on-p-bits, not a reason to open a VLA. |
| **Model LAST** | Frozen VLA / diffusion / next-token policy that EFA **wraps and can veto**. GPU column of the substrate map (EBT, Titans, BitNet) is the incumbent, not the certificate. | ModelGenerated. Cannot coerce to Deterministic. `allow_model=false` remains the MoL default. The seatbelt is O(1); the policy may be any size. |

**Zone rhyme (OpenIE), not a citation on their pages.** Certificate-before-commit is Z1/WCA spirit (cheap, on-device, refuse over escalate). Consequence prediction by energy descent is Z1/Z2 Solver. A released VLA in the loop is Z3 residual and must not bypass the gate. JouleDB’s Lookup → Formula → Extract → Aggregate → Reason is the DB twin of the same order; BMI has no database cascade.

**Where BMI is stricter than a naive MoL reading.** A score is not a proof. Best-of-N energy selection is verification-after-candidates, which they measure, and they explicitly say that is **not** the certificate (Lyapunov, before commit, on the body). MoL must not file EFA under “energy model = Solver and done.”

**Where MoL is stricter than BMI’s teaching demos.** Settle-then-act without a certify step is still a proposal. Motors dominating below ~1 W means the `energy` floor for an embodied task is often **not** the compute Landauer gap. EFA’s 143 µs and µJ figures are their measurements on their machines; MoL receipts do not import them as `measured_j`.

---

## MathGround, OpenIE, joules — link check

Searched the five fetched pages for MathGround, OpenIE, compute.openie.dev, proof.openie.dev, JouleDB, Mixture of Limits, Periodic Stack, and the proof law \(E(x)\ge\theta(D)\cdot\mu(S,V)\).

**No hits.** These sites do not cite that stack. Overlap is conceptual and, on the EFA byline, **institutional** (Charlot Lab). Do not write a fake cross-link into either tree.

| Their word | MoL / OpenIE word | Relation |
|---|---|---|
| Joules per task; µJ on device; ferrotherm joules ledger | `estimated_j` / `measured_j`, proof law, Landauer annotation | Same **unit of account**. Different schemas. Their ledger labels provenance in prose; MoL types it (`EstimateKind`, `MeasureSource`, `board_synth_claimed=false`). |
| OER/1 “receipt schema for energy claims” (hub title only) | `MolReceipt` | Cousin. Not read in this pass; do not equate fields. |
| RELAX/1 lowering across substrates | CascadeTier × ThermoClass L0/L1/L2max | Cousin: one IR, many fabrics. MoL v0.1 has no MLIR/NIR. |
| “What a decision costs”, save at most \(1/(1-p)\) | VoI stop | Same instinct: routing off the generative path has a ceiling. Their algebra was not derived here. |
| Verification grade; modelled ≠ measured ≠ projected | Honesty §8 | Directly adoptable as wording. Already MoL policy. |
| EFA Ferric / ferrotherm / browser instruments | `mol-*` crates | **Not dependencies.** Pattern-only, same rule as leapfrog / jouledb / wca. |
| Learns-at-inference asterisk (Hebbian fast weights, TTT, EFA-on-fabric) | Model leaf or a future primitive | Still unclaimed silicon, by their count (17 algorithm rows ahead of the chips). Do not mark Periodic Stack cells filled. |

---

## Borrow vs refuse

| Borrow (docs only) | Refuse |
|---|---|
| Provenance rule: never average modelled / simulated / projected / measured | Pasting EFA µJ, thermo fJ, or TrueNorth 26 pJ into MoL receipts as measured |
| Certificate-before-commit; refusal is part of the certificate | Treating energy-as-score (EBT, JEPA, best-of-N) as a Lyapunov guarantee |
| Structural identity over post-hoc compression of a learned net | Shipping Ferric, ferrotherm, or a p-bit SDK in v0.1 |
| Settle finds good-not-best; exact work stays Formula | Using the Ising demo to skip Lookup/Formula when the grammar is covered |
| Spikes-per-synapse floor; unpublished fetch ≠ zero | Citing Loihi 23.6 pJ as silicon |
| Motors dominate below ~1 W; control-on-degree-16 fabric plateaus ~45% for lack of products | Claiming thermodynamic chips close embodied MoL tasks |
| Walk-backs they already printed (feedforward 0%; force-net negative) | Re-quoting withdrawn claims, or claiming Charlot Lab *is* the MoL crate |

Empty Periodic Stack cells stay empty. If a task coordinate is “control workload on a thermodynamic sampler,” the honest MoL flag is `primitive_gap`, with this page as the evidence the products do not route — not as coverage.

---

## One-line citation

> Physical AI BMI collapses the controller into its own pre-commit energy certificate and prices Landauer as a movement-and-provenance problem; MoL navigates that as Formula then Solver, model last, without treating their joule figures as MoL measurements or their pages as OpenIE citations.
