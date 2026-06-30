# Codex Delivery Roadmap

This repo uses a staged delivery model so future prompts can stay short and still remain aligned with the product boundary.

## V1

Local workflow MVP and artifact UX without LLM prose.

Core flow:

1. Source import
2. Extract
3. Chunk
4. Build retrieval index
5. Build Evidence Pack
6. Build Answer Draft
7. Build Grounded Answer
8. Build Final Answer contract
9. Refresh Answer Artifact overview
10. Review export preview gate
11. Run explicit export

V1 goal:

- make the local source-to-artifact workflow understandable and safe
- keep every mutation explicit
- keep normal UI cards path-free
- keep retrieval and artifact contracts inspectable
- avoid LLM prose claims

## V2

Consent-gated local LLM answer generation with grounding and review gates.

Indicative flow:

1. Prompt pack
2. Local runtime invocation gate
3. Local LLM draft
4. Grounding inspection
5. Citation candidate mapping
6. User review
7. Final grounded prose

V2 boundaries:

- no black-box answer
- no answer without an Evidence Pack
- no citation without a locator-backed source trail
- no automatic model execution without consent
- no cloud LLM as the default runtime

## V3

Agentic local scholar workflow with plan/act/verify loops and approval gates.

Indicative flow:

1. User goal
2. Agent plan
3. Tool selection
4. Local retrieval
5. Evidence assembly
6. Local model reasoning
7. Verification
8. User approval gates
9. Artifact output

Expected components:

- tool registry
- task planner
- execution gates
- audit trail
- evaluation harness
- citation verifier
- local project context
- external adapter boundary

## Recommended sequencing

1. Stabilize V1 first.
2. Start V2 only after the V1 happy path is reliable.
3. Start V3 only after V2 grounding and review gates are trustworthy.
