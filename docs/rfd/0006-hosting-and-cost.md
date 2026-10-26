state: ideation
authors: Forrest Akin
discussion:

# RFD 0006: Hosting, cost and business model

<!-- Skeleton: questions, facts and sources. Prices change; re-check before deciding. -->

## Questions to answer

- What is the cheapest credible way to host each phase, for friends-only use
  and for a few thousand monthly users?
- Which parts should run on managed primitives, and which should run as
  services we operate ourselves (where the control-plane work is the point)?
- What would people pay for, and how does the free tier stay generous enough
  to spread by link?

## Notes: hosting (checked 2026-10-25)

- **Static (phase 1).** Cloudflare Pages / Workers static assets: unlimited
  bandwidth and static requests, no commercial-use restriction found.
  GitHub Pages forbids commercial use; Vercel Hobby is non-commercial only;
  Netlify Free is a 300-credit hard cap (about 15 GB of bandwidth).
- **Exports can be client-side.** MIDI, SVG/PDF and audio (OfflineAudioContext)
  from the wasm core cost nothing to serve; server export jobs become optional.
- **Cloudflare Durable Objects** (SQLite-backed) run on the free Workers plan
  (100k requests/day, 13,000 GB-s/day, 5 GB storage; operations fail past the
  daily caps). Workers Paid is $5/month with generous included usage.
  Hibernated WebSockets bill no duration. Rust works via `workers-rs` 0.8.7.
  Placement is automatic and objects can't move: little control-plane work
  left to own.
- **Fly Machines.** shared-cpu-1x: 256 MB $2.19, 512 MB $3.69, 1 GB $6.70 per
  month, billed per second; no free tier (trial, then card). `fly-replay` with
  `instance=<id>` lets our own placement logic pin a document's requests to a
  machine. Managed Postgres from $38/month (use Neon instead).
- **VPS.** DigitalOcean $4/$6/$12 droplets (IPv4 included, per-second
  billing). Vultr $2.50–$6. Hetzner raised prices twice in 2026; its cheapest
  US plan is €17.99/month and every cloud plan showed "currently unavailable"
  on 2026-10-25. Oracle Always Free halved its ARM allowance to 2 OCPU / 12 GB
  around 2026-06-15 and reclaims idle free VMs.
- **Postgres.** Neon Free (100 CU-hours per project, 1 GB, scale-to-zero cold
  starts); Supabase Free (500 MB, pauses after a week idle); CockroachDB has
  no free tier for new organisations since 2026-09-15 (30-day $400 trial).
- **Object storage.** Cloudflare R2: 10 GB-month free, free egress.

### Stacks and rough monthly cost

| Stack | Control-plane work owned | Friends only | ~3,000 MAU |
| --- | --- | --- | --- |
| All Cloudflare (static, API Worker, a Durable Object per document, Queues, R2, Neon) | Little | $0 | ~$5–25 |
| Fly Machines (static on Cloudflare; API/placement + 2 session machines; Neon; R2) | Placement, registry, draining, failover | ~$5–8 | ~$12–35 |
| Own nodes on VPSes (gateway/placement + 2 session nodes + Postgres) | Everything | $0 (Oracle, with caveats) or ~$15–18 | ~$30–40 |

## Notes: market (checked 2026-10-25)

- **Closest competitors.** Groove Scribe (free, GPL v2, active again: mobile
  builds July 2026, commits through 2026-10-03; one meter per groove, fixed
  subdivisions). Beam (beamdrums.com, launched ~June 2026: web step grid,
  live view links, looping; free for 10 charts, Pro £4/mo, Teacher £8/mo;
  mixed meters and nesting not documented). Drum Notation by Beat Note (Apple
  platforms only, $59.99–119.99/yr).
- **General editors cover parts of it.** Dorico has a percussion grid editor
  and nested tuplets ($99.99–579.99, desktop, no links). MuseScore Studio
  (free) nests tuplets deeply; Soundslice added nested tuplets 2026-10-07 and
  leads on practice tools; Flat has real-time co-editing but light drum input.
- **Gap.** No product found combines a drum-first grid in the browser,
  per-bar meters, mixed subdivisions within a bar, nested tuplets, link
  sharing, practice playback and real-time co-editing. The simple-groove end
  is crowded (a free incumbent plus Beam); the complex-rhythm end is not.
- **What people pay for.** Tool subscriptions are cheap ($4–10/month,
  $40–60/year). Teacher tiers (student folders, assignments) are a consistent
  upsell. Schools license per seat (Flat for Education, Noteflight Learn).
  Platforms pay for embedding (Soundslice Licensing $100/month for 200 users).
  Transcriptions sell through marketplaces (ArrangeMe, mySongBook, Gumroad).
  Lesson content commands the highest prices (Drumeo, Melodics ~$30/month).

## Sources

- Cloudflare: https://developers.cloudflare.com/durable-objects/platform/pricing/ ·
  https://developers.cloudflare.com/workers/platform/pricing/ ·
  https://developers.cloudflare.com/pages/platform/limits/ ·
  https://developers.cloudflare.com/r2/pricing/ ·
  https://github.com/cloudflare/workers-rs
- Static hosts: https://docs.github.com/en/pages/getting-started-with-github-pages/github-pages-limits ·
  https://vercel.com/docs/plans/hobby · https://www.netlify.com/pricing/
- Fly.io: https://docs.fly.io/about/pricing · https://docs.fly.io/networking/dynamic-request-routing/
- VPS: https://www.digitalocean.com/pricing/droplets · https://www.vultr.com/pricing/ ·
  https://www.hetzner.com/cloud/ · https://docs.hetzner.com/general/infrastructure-and-availability/price-adjustment/ ·
  https://docs.oracle.com/en-us/iaas/Content/FreeTier/freetier_topic-Always_Free_Resources.htm
- Databases: https://neon.com/pricing · https://supabase.com/pricing · https://www.cockroachlabs.com/pricing/
- Market: https://beamdrums.com/ · https://github.com/montulli/GrooveScribe ·
  https://beat-note.app/ · https://www.soundslice.com/plans/ · https://www.soundslice.com/licensing ·
  https://flat.io/pricing · https://www.noteflight.com/premium ·
  https://handbook.musescore.org/notation/rhythm-meter-and-measures/tuplets.md ·
  https://www.scoringnotes.com/news/dorico-4-2/ · https://www.drumeo.com/ ·
  https://www.arrangeme.com/faq
