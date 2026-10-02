# Lux Design crate inventory (197 crates with Cargo.toml)

Source: /Users/dcharlot/vibe-coding/lux-design/crates


## 01-pipeline-publish (12)

- `lux-cms` (`crates/cms`)
- `lux-compiler` (`crates/compiler`)
- `lux-copy` (`crates/copy`)
- `lux-deploy` (`crates/deploy`)
- `lux-director` (`crates/director`)
- `lux-embed` (`crates/embed`)
- `lux-export` (`crates/export`)
- `lux-ingestion` (`crates/ingestion`)
- `lux-pipeline` (`crates/pipeline`)
- `lux-scroll-engine` (`crates/scroll-engine`)
- `lux-tier-embed` (`crates/tier_embed`)
- `lux-video-export` (`crates/video_export`)

## 02-desktop-ui (9)

- `lux-canvas` (`crates/canvas`)
- `lux-code-tui` (`crates/code_tui`) — Full-screen terminal UI for the lux-code coding agent (ratatui)
- `lux-desktop` (`crates/desktop`) — Lux Design as a native desktop app via the Photon UI framework.
- `lux-desktop-host` (`crates/desktop_host`) — Standalone native window host for the Lux Design Photon app.
- `lux-engine` (`crates/engine`)
- `lux-prompt2ui` (`crates/prompt2ui`)
- `lux-studioq` (`crates/studioq`) — Pure-Rust HuggingFace-safetensors → .studioq quantizer + tokenizer→vocab (no Python)
- `lux-surface` (`crates/surface`) — Per-surface adapters for AI-native fates. Binds a LocalRuntime to each surface in the bundle's manifest (web HTTP, PWA, agent-as-MCP, native shell binding, voice webhook, ambient device-class). v1 ships three full adapters (Web/Pwa/Agent) + three binding stubs (Native/Voice/Ambient) where production wiring needs external platform integration.
- `lux-themes` (`crates/themes`)

## 03-coding-agent (12)

- `lux-agent-code` (`crates/agent_code`)
- `lux-agents` (`crates/agents`)
- `lux-code-acp` (`crates/code_acp`) — ACP (Agent Client Protocol) agent exposing lux-code to editors like Zed
- `lux-code-agent` (`crates/code_agent`) — The brain for lux-code: drives a CompletionModel through the tool loop over a live repo (plan/build modes)
- `lux-code-mcp` (`crates/code_mcp`) — MCP server exposing the lux-code repo tools (read/write/edit/grep/glob/bash/git) to any MCP client
- `lux-code-parallel` (`crates/code_parallel`) — Build-mode parallelism for lux-code: N agents in isolated git worktrees (via lux-burst)
- `lux-code-route` (`crates/code_route`) — Route → run → verify → escalate: cheapest-tier-first coding gated by a deterministic verifier
- `lux-code-tools` (`crates/code_tools`) — Agent-callable repo tools (read/write/edit/grep/glob/bash/git) for the lux-code coding agent
- `lux-framework-codegen` (`crates/framework_codegen`) — Deterministic codegen of runnable web-framework projects from a Lux SceneGraph (vanilla-ts, Next, SvelteKit, Astro, Nuxt, SolidStart, React Router, Qwik, TanStack Start).
- `lux-orchestra` (`crates/orchestra`)
- `lux-screenshot2code` (`crates/screenshot2code`)
- `lux-spec-decode` (`crates/spec_decode`) — Pure-Rust speculative-decoding harness (draft + verifier loop) for Lux Design's AI runtime

## 04-ai-models (19)

- `lux-ai` (`crates/ai`)
- `lux-ai-bundle` (`crates/ai_bundle`) — AI-native bundle format — the signed, content-addressed primitive that defines a Lux application: intent (prompt + scene graph + brand) + capabilities (MCP refs) + surface manifest + runtime preference graph. See docs/AI_NATIVE_FATES.md.
- `lux-ai-sdk` (`crates/ai_sdk`) — The ergonomic, provider-agnostic AI SDK for Lux — generate_text / stream_text / generate_object / tool-calling over any OpenAI-compatible model, local-first (defaults to the on-device Lux server), with a Mock provider for tests.
- `lux-ai-sdk-tier` (`crates/ai_sdk_tier`) — Run the lux-ai-sdk against an in-process lux-runtime Tier — the local model with NO server. Wrap any Tier (on-device LFM2, stub, cloud) as a LanguageModel.
- `lux-email` (`crates/email`)
- `lux-forecast` (`crates/forecast`)
- `lux-generative` (`crates/generative`)
- `lux-lfm2` (`crates/lux-lfm2`) — Hand-ported LFM2 device runtime: pure-Rust GGUF loader + hybrid short-conv/GQA-attention forward + GPT-2 byte-level BPE tokenizer. No candle, no torch — the DS4-style owned-kernel counterpart to `lux-llm-gguf`'s mistral.rs backend.
- `lux-lfm2-wgpu` (`crates/lux-lfm2-wgpu`) — wgpu / WGSL LFM2 kernels — one codebase runs on Metal (Mac), Vulkan (Linux any GPU + browser via WebGPU), DX12 (Windows). Walking skeleton: ships the argmax kernel + WgpuContext today; the LFM2 forward-pass kernels (RMSNorm, matmul, attention, short-conv, SwiGLU) land incrementally on top.
- `lux-liveportrait` (`crates/liveportrait`)
- `lux-llm-gguf` (`crates/llm_gguf`) — Model-agnostic desktop GGUF inference backend (mistral.rs) for lux-code — the pure-Rust, SOTA-tracking local brain. Default target: Qwen3-Coder-Next. Off-by-default heavy engine, mirroring local-nemotron.
- `lux-mt` (`crates/mt`) — Real local machine translation (Marian / opus-mt via candle).
- `lux-nllb-convert` (`crates/nllb_convert`) — Offline converter: HuggingFace NLLB-200 safetensors → our on-disk .nllb format.
- `lux-runtime` (`crates/runtime`) — The host for an AI-native SignedBundle. Loads + verifies the bundle, resolves capabilities, dispatches requests through the runtime preference graph (device → edge → cloud tiers). v1 ships LocalRuntime + three stub tiers; real model integration lands in subsequent sessions.
- `lux-skills-runtime` (`crates/skills_runtime`) — Registry of markdown-defined Skills with YAML front-matter and intent dispatch
- `lux-translate` (`crates/translate`)
- `lux-voice` (`crates/voice`)
- `lux-wasm-runtime` (`crates/wasm_runtime`) — WASM-Component-Model-compatible shape for the AI-native runtime. Compiles to wasm32-wasip2 as a WASM Component exporting the lux:runtime/runtime interface, OR to a native rlib for in-process polymorphism tests. See README + docs/AI_NATIVE_RUNTIME.wit.
- `lux-webcontainer` (`crates/webcontainer`) — Server-side ECMAScript sandbox with a virtual filesystem, powering Lux Design's WebContainer-style live preview slice.

## 05-energy-compute (2)

- `lux-cost` (`crates/cost`)
- `lux-energy` (`crates/energy`) — Energy observability for Lux — a universal estimation model (J/token, J/GPU-s, J/CPU-s, J/byte) plus a measurement-backend trait and a process-global ledger, so every unit of work can report joules. Estimation is the wasm-safe floor; native hardware counters refine it.

## 06-design-media (18)

- `lux-blender` (`crates/blender`) — Blender as a process-isolated render/sim Tier for AI-native fates. Drives headless Blender (blender -b -P driver.py) with a canonical params.json; never links Blender (GPL stays at the process boundary). Implements lux_runtime::Tier so the orchestrator routes 'render'/'simulate'/'bake' requests with fall-through and PrivacyMode filtering.
- `lux-brand` (`crates/brand`)
- `lux-comfy` (`crates/comfy`) — Native Rust runtime for ComfyUI-style diffusion node graphs: loads ComfyUI workflow JSON into a typed DAG, topo-sorts + evaluates it through a pluggable node executor (stub by default; real diffusion behind a feature). Content-addressed like the rest of Lux.
- `lux-dam` (`crates/dam`)
- `lux-datavis` (`crates/datavis`)
- `lux-design-system-extractor` (`crates/design_system_extractor`)
- `lux-design-verify` (`crates/design_verify`) — Computational verification for generated design — the deterministic gate that lets UI output ride the same route→verify→escalate loop as code. Backs onto lux-a11y's WCAG checks and adds design-token resolution and component-reference integrity.
- `lux-diagram` (`crates/diagram`)
- `lux-figma` (`crates/figma`)
- `lux-geometry` (`crates/geometry`) — Deterministic, seedable procedural-geometry node-graph evaluator. Byte-identical mesh output across native, wasm32-unknown-unknown, and wasm32-wasip2.
- `lux-image` (`crates/image`)
- `lux-ink` (`crates/ink`)
- `lux-media` (`crates/media`)
- `lux-model3d` (`crates/model3d`)
- `lux-tier-google-media` (`crates/tier_google_media`)
- `lux-tier-nodegraph` (`crates/tier_nodegraph`) — Node-graph runtimes as lux-runtime tiers: ComfyTier (lux-comfy diffusion DAG) + TdTier (lux-td realtime procedural graph), so ComfyUI/TouchDesigner-style workflows are orchestrator-routable Lux capabilities that return PNGs.
- `lux-vector` (`crates/vector`)
- `lux-video` (`crates/video`)

## 07-agents-state (13)

- `lux-actor` (`crates/actor`)
- `lux-agent` (`crates/agent`) — A folder is an agent — load an agent from a directory (agent.toml + instructions.md + skills/ + tools) and run it on the bounded tool-use loop, driven by any lux-ai-sdk model, with a durable transcript.
- `lux-agent-computer` (`crates/agent_computer`)
- `lux-agent-loop` (`crates/agent_loop`) — Bounded tool-use loop primitive — the actual scaffold behind 'agentic AI'
- `lux-agent-mcp` (`crates/agent_mcp`)
- `lux-agent-web` (`crates/agent_web`)
- `lux-asl` (`crates/asl`)
- `lux-bt` (`crates/bt`)
- `lux-fate-runner` (`crates/fate_runner`) — Generic Photon-based native shell that renders a Lux artifact as a native desktop window. Used by the v1.2 swap for Fate::MacOSApp / WindowsApp / LinuxAppImage.
- `lux-fate-statechart` (`crates/fate_statechart`)
- `lux-refactor` (`crates/refactor`)
- `lux-scxml` (`crates/scxml`)
- `lux-statechart` (`crates/statechart`)

## 08-storage (8)

- `hyperdb` (`crates/hyperdb`) — Hybrid graph + vector database engine — HNSW ANN, property graph, hybrid query pipeline, SIMD distance kernels, zero dependencies on external services
- `lux-adr` (`crates/adr`) — Architecture Decision Record loader: parse markdown ADRs with YAML front-matter from a directory and return them sorted by id.
- `lux-db` (`crates/db`)
- `lux-feedback` (`crates/feedback`)
- `lux-livesandbox` (`crates/livesandbox`) — Client-side WebContainer-style instant preview via a hardened srcdoc iframe sandbox
- `lux-sandbox-exec` (`crates/sandbox_exec`)
- `lux-storage` (`crates/storage`)
- `lux-workers` (`crates/workers`)

## 09-api-cli-mcp (6)

- `lux-api` (`crates/api`)
- `lux-cli` (`crates/cli`)
- `lux-dev-server` (`crates/dev-server`)
- `lux-mcp` (`crates/mcp`)
- `lux-mcp-bundle` (`crates/mcp_bundle`) — Capability resolution for AI-native fates. Takes a CapabilityBundle (MCP server refs), produces ResolvedCapabilities with live endpoints + authorized scopes + handshake config. Pluggable provisioners (local storage, echo, future: Stripe/Twilio/etc.).
- `lux-sdk` (`crates/sdk`)

## 10-product-collab (9)

- `lux-a11y` (`crates/a11y`)
- `lux-ads` (`crates/ads`)
- `lux-analytics` (`crates/analytics`)
- `lux-auth` (`crates/auth`)
- `lux-collab` (`crates/collab`)
- `lux-comments` (`crates/comments`)
- `lux-community` (`crates/community`)
- `lux-compliance` (`crates/compliance`)
- `lux-forms` (`crates/forms`)

## 11-infra-tooling (11)

- `lux-broadcast` (`crates/broadcast`)
- `lux-burst` (`crates/burst`) — Specialist fan-out + merge orchestrator: run N specialists in parallel against one prompt and merge their outputs.
- `lux-changelog` (`crates/changelog`) — Conventional-commit changelog generator: bucket commits by feat/fix/docs prefix into semver-grouped release notes.
- `lux-component-states` (`crates/component_states`)
- `lux-crash` (`crates/crash`) — Crash reporting for the Lux Design desktop binary — panic hook + on-disk crash log + next-launch dialog.
- `lux-docs` (`crates/docs`)
- `lux-e2e` (`crates/e2e`)
- `lux-events` (`crates/events`)
- `lux-fuzz` (`crates/fuzz`) — Adversarial-input harness for Lux Design's user-input parsers.
- `lux-model-install` (`crates/model_install`) — Model-weight installer: check-installed / download / hash-verify for the on-device model files consumed by lux-runtime tiers. Independent of wgpu.
- `lux-update` (`crates/update`) — Check GitHub Releases for a newer Lux Design version and surface it to the UI.

## 12-other (78)

- `lux-app-gen` (`crates/app_gen`)
- `lux-artifact` (`crates/artifact`) — Live artifact primitive: stateful canvas elements bundling template, persistent state, and named API bindings.
- `lux-automation` (`crates/automation`)
- `lux-backend-gen` (`crates/backend_gen`)
- `lux-compute-fabric` (`crates/compute_fabric`) — Layer 2 of routing — pick the optimal local execution path (kernel + accelerator) for the model on THIS box; never force CPU when a GPU can serve it
- `lux-core` (`crates/core`)
- `lux-grant-integrations` (`crates/grant-integrations`)
- `lux-grants` (`crates/grants`)
- `lux-hooks` (`crates/hooks`)
- `lux-hosting` (`crates/hosting`)
- `lux-i18n` (`crates/i18n`)
- `lux-iac` (`crates/iac`)
- `lux-integrations` (`crates/integrations`)
- `lux-intent` (`crates/intent`)
- `lux-intent-tier` (`crates/intent_tier`)
- `lux-labs` (`crates/labs`)
- `lux-landing` (`crates/landing`)
- `lux-live` (`crates/live`)
- `lux-lmm` (`crates/lmm`)
- `lux-loadtest` (`crates/loadtest`)
- `lux-lsp` (`crates/lsp`)
- `lux-marketplace` (`crates/marketplace`)
- `lux-mashup` (`crates/mashup`)
- `lux-migration` (`crates/migration`)
- `lux-model-select` (`crates/model_select`) — The routed, work-dependent model slot — pick a concrete model from the installed catalog by task + hardware + whether constrained decoding is needed. No fixed default.
- `lux-monitor` (`crates/monitor`)
- `lux-motion` (`crates/motion`)
- `lux-multimodal-intent` (`crates/multimodal_intent`) — Multimodal intent fusion — combine text + image refs + CSS refs into a structured FusedIntent for downstream generators
- `lux-mutation` (`crates/mutation`)
- `lux-name` (`crates/name`) — Federated name registry for AI-native fates. Binds human-readable lux://owner/name to a signed-snapshot-chain of bundle hashes. v1 ships in-memory + JSON-file persistence; federation gossip is the next session.
- `lux-narrative` (`crates/narrative`)
- `lux-notebook` (`crates/notebook`)
- `lux-notifications` (`crates/notifications`)
- `lux-onboarding` (`crates/onboarding`)
- `lux-paged` (`crates/paged`) — PagedAttention block allocator — the KV-cache memory manager (fixed-size blocks, ref-counted copy-on-write sharing, prefix forking). Zero deps; shared by lux-serve and the model backends so one BlockAllocator can be the real KV store.
- `lux-payments` (`crates/payments`)
- `lux-perceive` (`crates/perceive`)
- `lux-perf` (`crates/perf`)
- `lux-plugins` (`crates/plugins`)
- `lux-pr` (`crates/pr`)
- `lux-presenter` (`crates/presenter`)
- `lux-principles` (`crates/principles`)
- `lux-proposal` (`crates/proposal`)
- `lux-puppet` (`crates/puppet`)
- `lux-repo` (`crates/repo`)
- `lux-research` (`crates/research`)
- `lux-review` (`crates/review`)
- `lux-rig2d` (`crates/rig2d`)
- `lux-risk` (`crates/risk`)
- `lux-scene3d` (`crates/scene3d`)
- `lux-scheduling` (`crates/scheduling`)
- `lux-scivis` (`crates/scivis`)
- `lux-security-verify` (`crates/security_verify`) — Deterministic security gate — committed-credential detection and dependency licence auditing, shaped as a lux_code_agent::verify::Check so security rides the same route→verify→escalate loop as code and design.
- `lux-seo` (`crates/seo`)
- `lux-serve` (`crates/serve`) — Serving-runtime primitives ported from vLLM/SGLang to pure Rust — paged KV-cache (PagedAttention), prefix-cache radix tree (RadixAttention), and FSM-constrained decoding. Model-agnostic; the heavy tensor work is injected.
- `lux-simli` (`crates/simli`)
- `lux-sketch` (`crates/sketch`)
- `lux-skills` (`crates/skills`)
- `lux-smoke` (`crates/smoke`) — End-to-end smoke tests for the four core Lux Design user journeys.
- `lux-snapshot` (`crates/snapshot`)
- `lux-social` (`crates/social`)
- `lux-ssr` (`crates/ssr`) — Speculative parallel scaling reasoning: run N independent generators and pick the modal output.
- `lux-stream` (`crates/stream`) — Cancellation token + progress-event MPSC channel + cancellable rayon helper — the harness behind streaming AI UX.
- `lux-support` (`crates/support`)
- `lux-td` (`crates/td`) — Native Rust runtime for TouchDesigner-style realtime procedural node graphs: TOP (texture) + CHOP (signal) operators, cooked per-frame against a first-class time input, with feedback (stateful previous-frame) and CHOP-drives-TOP-param cross-family flow. CPU-reference ops now; wgpu acceleration behind a feature.
- `lux-templates` (`crates/templates`) — Starter-project template loader: scans templates/<slug>/manifest.json + files/ and scaffolds into a Vec of (path, contents).
- `lux-tier-asset` (`crates/tier_asset`) — Asset-generation Tiers for AI-native fates: wraps game-studio's 3D/texture/video providers (TRELLIS, Hunyuan3D, TripoSG, Meshy, Rodin, OpenSCAD, Wan2.1) as lux_runtime::Tier so the orchestrator routes mesh_3d/texture/cad/video_synthesis requests with fall-through + PrivacyMode. Default build ships stub tiers; the real providers are behind the `ai-media` feature (path-dep on studio-asset-gen).
- `lux-tier-browser` (`crates/tier_browser`) — Browser-tier Runtime adapter for AI-native fates. Runs LFM-2.5 1.2B-Instruct in-browser via WebGPU with OPFS-cached weights and SHA-256 manifest verification. Implements lux_runtime::Tier; the orchestrator routes `device:lfm-2.5-1.2b-instruct` requests to this when the client supports WebGPU. Server-side targets compile a sentinel that's never registered; the wasm32 build is what actually runs.
- `lux-tier-cloud` (`crates/tier_cloud`) — Cloud-tier Runtime adapters for AI-native fates. OpenRouter (300+ models via OpenAI-compatible API), Anthropic Messages, Google Gemini, OpenAI direct. Each implements lux_runtime::Tier; the orchestrator routes requests through them based on bundle preferences.
- `lux-tier-edge` (`crates/tier_edge`) — Edge-tier Runtime adapters for AI-native fates. Cloudflare Workers AI (free 10k requests/day), Replicate, HuggingFace Inference Endpoints. Each implements lux_runtime::Tier; the orchestrator routes requests through them after device candidates exhaust and before cloud.
- `lux-tier-music` (`crates/tier_music`)
- `lux-tier-shapes` (`crates/tier_shapes`)
- `lux-tier-tts` (`crates/tier_tts`)
- `lux-tier-vision` (`crates/tier_vision`)
- `lux-tier-whisper` (`crates/tier_whisper`)
- `lux-vision` (`crates/vision`)
- `lux-wave8-handlers` (`crates/wave8_handlers`)
- `lux-workflow` (`crates/workflow`)
