state: ideation
authors: Forrest Akin
discussion:

# RFD 0005: System architecture and collaboration

<!-- The system-level RFD. Expect it to split into several as phases 2–4 start. -->

## Components (proposed)

| Component | Phase | Responsibility | Notes |
| --- | --- | --- | --- |
| Core library (`rhythm-tree`) | 0 | Model, codec, validation, event compiler | Shared by the client (wasm) and every service (native), so all of them agree on what a valid groove is. |
| Web client | 1 | Editing, playback, local-first state | Static hosting. |
| API service | 2 | Accounts, groove library, revisions, short links | OpenAPI spec; generated TypeScript client. |
| Identity and permissions | 2 | Sign-in, workspaces, sharing | Write the authorization model down before building it. |
| Sync service | 3 | Real-time sessions per document | WebSockets; server orders and validates operations. |
| Control plane | 3 | Places document sessions on sync nodes; drains and recovers | Leases, rebalancing, failure detection. |
| Export workers | 4 | PDF and audio rendering | Durable, retried, idempotent jobs; object storage. |

## Questions to answer

- **Operations.** What does an edit look like on the wire: set a hit, replace a
  subtree, change a meter, insert a bar? How do concurrent edits to the same
  bar resolve? Options: server-ordered operations with last-writer-wins per
  slot; operational transformation over paths; a CRDT keyed by stable node IDs.
- **Identity of slots.** Leaf indices shift when a subdivision changes. Do we
  give nodes stable IDs, or transform operations through structural edits?
- **Source of truth.** Operation log plus snapshots? How often do we snapshot,
  and how is a link related to a revision?
- **Placement.** How does a client find the node that owns a document's
  session? What happens to connected clients when that node drains or dies?
- **Failure modes.** Network partitions, slow clients, replay after reconnect,
  duplicate operations.

## Notes

- Hosting options trade control-plane work you own against managed
  primitives that do it for you (for example Durable Objects); see RFD 0006.
- Keep the core free of I/O so services can embed it without surprises.

## Sources

<!-- Add as you research: papers, prior art (Figma multiplayer, Automerge, Yjs), and the specific failure stories you design against. -->
