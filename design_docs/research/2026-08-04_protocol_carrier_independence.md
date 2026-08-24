# Carrier independence — how many smolweb protocols run over Reticulum, and what all 17 would cost

**Date:** 2026-08-04
**Status:** analysis, answering Mark's question. No work scheduled here.
**Companion to:** the
[smolweb home decision](../technical_architecture/2026-08-03_smolweb_home_decision.md) (the grouping
rule and the three moves landed so far) and the
Reticulum browsing plan (`turnstone/design_docs/2026-08-03_reticulum_browsing_plan.md`)
(the lane this would feed).

## The short answer

**Thirteen of the seventeen port to Reticulum unchanged**, because the
compatibility test is not about the protocol at all. It is one question:

> Does this protocol need a bidirectional byte stream, or does it need
> something else?

Retinue's `LinkStream` implements `AsyncRead` and `AsyncWrite`
(`crates/retinue/src/endpoint.rs:208,218`). Every smolweb protocol whose whole
shape is *open a connection, write a request line, read a response, close*
therefore runs over an encrypted Reticulum link with **no protocol changes at
all**. That is most of the small web, because that shape is the small web's
defining aesthetic.

This is not speculation. It is already proven for gemini twice over: retinue's
`gemini_over_reticulum` example serves and fetches a capsule over a real link,
and `gemini_protocol::exchange` has a test that runs the exchange over an
in-memory duplex with no TCP and no TLS.

## The categories

| | Protocols | Count | Over Reticulum |
|---|---|---|---|
| **A. Clean stream port** | gemini, gopher, Gopher+, finger, spartan, nex, titan, text, SuperText, mercury, scroll, scorpion, molerat | 13 | Works unchanged. A link is a byte stream. |
| **B. Encryption made redundant** | gophers | 1 | Degenerates: `gophers` is gopher-over-TLS, and a link is already encrypted, so the honest answer is *gopher over Reticulum*. |
| **C. Datagram-shaped** | guppy, fsp | 2 | Possible but architecturally silly: both are UDP with their own chunking, acks, and retransmission, which Reticulum already provides. Carrying them means running two reliability layers. |
| **D. Not a protocol** | terse | 1 | Corrected below: there is no wire protocol to be compatible with. |

Separately, **misfin** (mail, which we also implement) is the one genuinely
interesting case, and it belongs to none of these. See below.

Two footnotes on category A. **Mercury and scorpion are the best fits in the
set**: mercury is gemini without TLS, and scorpion makes TLS optional, so
neither has an encryption layer to make redundant. And **molerat mandates TLS**
and uses request *and response* headers with a markdown-like markup
([spec](https://molerat.trinket.icu/)), so it ports as a stream but is the most
HTTP-shaped of the group.

## Why it is nearly free, and what "nearly" costs

The mechanism is the three-layer split already shipped in `gemini-protocol`
and `gopher-protocol`:

```
<format>   always compiled, zero dependencies    the grammar
client     feature: tokio                        the exchange over ANY stream
tls / tcp  feature: rustls, ring                 the ordinary internet carrier
```

If that shape is house style, **Reticulum support is not per-protocol work**.
Thirteen protocols do not need thirteen ports; they need their `client` layer,
which they need anyway, plus one shared piece of glue.

That shared piece is the real cost, and it is **addressing**, not transport.
Every one of these protocols puts a hostname in its request line, and Reticulum
has no DNS and no host:port. A destination is a hash, optionally reached by an
announced name. So there is exactly one decision to make, once, for all of
them: how a smolweb URL's authority names a Reticulum destination. The retinue
example already demonstrates the named form, resolving `gemini://capsule/` by
recompute-and-match against announces, which is how Nomad Network addresses
nodes. That decision is N0 in the Reticulum browsing
plan (`turnstone/design_docs/2026-08-03_reticulum_browsing_plan.md`)
and it is a prerequisite for all thirteen, not a per-protocol tax.

The second shared piece is **trust vocabulary**. Over TLS the identity is a
pinned certificate fingerprint; over Reticulum it is a destination hash the
link handshake proves. Both are "trust the key you actually talked to", so the
ladder maps, but the posture needs its own name rather than borrowing TOFU's,
and the fidelity plan's trust descriptor is where it lands.

## The semantics: when a protocol is defined by its medium (Mark, 2026-08-04)

Carrier independence is a claim about **code**, not about **meaning**. The
question is never "can this run over carrier X"; it is "what does it *mean*
over carrier X". Getting that wrong is how broad protocol support turns into
misinformation, because a client that reports the same posture regardless of
what carried the bytes is lying about at least one of them.

A protocol can depend on its medium at four separate layers, and they behave
differently:

| Layer | Medium-dependent? | Consequence |
|---|---|---|
| **Syntax** — the bytes on the wire | Almost never | This is what carrier independence makes shareable. A request line is a request line. |
| **Security** — confidentiality, authentication | Almost always | Must be re-described per carrier. See the correction below. |
| **Identity** — who the peer is, how they are named | Sometimes protocol-defined, sometimes carrier-defined | When both define it, they conflict. Bridge, do not port. |
| **Purpose** — why the protocol exists at all | Sometimes | When the carrier negates the purpose, the port is possible and pointless. Do not offer it. |

So the rule, stated once:

> **A protocol's implementation is carrier-independent. Its description is
> not.** Posture, identity, and addressing are properties of the
> *(protocol, carrier)* pair, and must be computed from the carrier that
> actually carried the bytes, never looked up from the scheme.

### The defect this exposes, and it is ours

The fidelity plan (`mere/design_docs/nematic_docs/implementation_strategy/2026-07-01_smolweb_fidelity_plan.md`)'s
WS2 maps **protocol to posture**: gemini to TOFU, gopher and finger and nex and
spartan to Insecure, misfin to Trusted-with-signer. Under carrier independence
that table is simply **wrong**, and it would ship a lie:

- **gopher over Reticulum is not Insecure.** The link is encrypted and the peer
  is proven by its destination key. Reporting "unauthenticated by design"
  because the scheme is `gopher://` would understate the actual security by a
  wide margin.
- **gemini over Reticulum is not TOFU.** There is no certificate and no pin.
  Reporting a pin state would be reporting a thing that does not exist.

The fix is small and structural: the trust descriptor is produced by the
**carrier**, and the protocol contributes only what it adds on top (misfin's
signed sender, gemini's client certificate). This is the same shape as the
existing WS2 finding that trust originates at the transport; it just has to
mean *the actual transport*, not the one the scheme implies.

### Identity, when both sides define it

Misfin is the clean example. Its sender identity *is* an X.509 client
certificate, and a Reticulum sender's identity *is* its destination key. Both
are complete identity systems, so carrying misfin over Reticulum does not
compose them; it forces a choice, and either choice produces something that is
no longer misfin. That is the signature of a case that wants a **bridge**
rather than a port.

Gemini's `6x` client-certificate flow is the same case in miniature, which is
why the "unchanged" claim covers gemini's ordinary path and not that one.

### Purpose, when the carrier negates it

Guppy exists *because* it is UDP: it is meant to run on a microcontroller with
no TCP stack, which is why it carries its own chunking, acks, and
retransmission. Over a Reticulum link, all three are already provided, and the
one property that justified the protocol is gone. It would still function.
It would no longer be for anything. The honest response is not to offer that
combination rather than to offer it and let someone discover the pointlessness
themselves.

### Addressing has the same shape

A URL's authority means "DNS name and port" over TCP and "destination hash, or
a name resolved against announces" over Reticulum. Identical syntax, different
referent. So resolution is also a *(protocol, carrier)* function, and it is the
single shared adapter named earlier rather than per-protocol work.

### What this actually costs retinue

Small, and now quantified. Retinue already supplies the hard part, a link that
is `AsyncRead + AsyncWrite`, so it inherits the whole stream family at once.
What is left is not per-protocol:

1. the addressing adapter (how an authority names a destination);
2. a posture vocabulary for what a link proves, distinct from TOFU's;
3. optionally, Resources for bulk bodies, which is how Nomad Network already
   serves pages.

Three pieces, once, for thirteen protocols.

## The distinction that matters: two different bridges

Conflating these is the trap, and keeping them apart is what stops broad
protocol support from turning into mush.

**1. The carrier bridge — cheap, already proven.** A smolweb protocol running
over a Reticulum link. Nothing about the protocol changes; only what carries
the bytes. Gemini over Reticulum is still gemini, and it is still gemtext at
the other end.

**2. The format bridge — expensive, and mostly a bad idea.** Translating
between a smolweb format and a Reticulum-native one. This is *not* a carrier
question, and the protocol-faithfulness doctrine says not to do it: render
micron as micron, not as lowered gemtext.

The useful observation is that the Reticulum-native formats are **analogues,
not gaps**:

| Smolweb | Reticulum-native | Relationship |
|---|---|---|
| gemtext | micron | Both are the line-oriented markup their side reads. |
| misfin | LXMF | Both are the peer-to-peer mail their side sends. |

So "misfin over Reticulum" is a question that mostly answers itself: Reticulum
already has LXMF. Porting misfin there would mean rebinding its identity model,
because misfin's sender identity *is* an X.509 client certificate, while a
Reticulum sender's identity is its destination key. That is not a carrier swap;
it is a new protocol wearing misfin's name. The same reasoning applies to
gemini's `6x` client-certificate flow, which is why category A's "unchanged"
claim covers gemini's ordinary path and not its client-cert path.

The right move where the two worlds meet is a bridge at the *message* level
(misfin ↔ LXMF), deliberately, with provenance preserved, and not a silent
transcoding.

## Revision: a better catalogue, and two protocols we missed (2026-08-04)

Mark surfaced
[zzo38computer.org's small-web catalogue](https://dbohdan.com/archive/scorpion/zzo38computer.org/smallweb.txt/),
which carries real wire detail where the earlier sources carried names. It
supersedes the seventeen-item list above, and it adds two protocols that were
not on it at all:

| Protocol | Port | Encryption | Notes |
|---|---|---|---|
| **Kepler** | 2009 / 10009 | optional | Gemini plus **caching**: the request carries `last_cached` and a language, the response carries content length, last-updated and expires. |
| **Demarkus** | 6309 (**UDP**) | mandatory | Markdown-only, **capability tokens** for auth, and **version control**. |

And it corrects several entries above:

- **Scorpion** (port 1517, optional TLS) is a **binary** format with **range
  requests and uploads**, not merely a richer document format.
- **Scroll** (port 5699, TLS mandatory) does **language negotiation** and
  carries **Universal Decimal Classification**.
- **Text Protocol** (ports 1961/1965/1968) makes TLS *or* **Noise** optional
  and supports **DNS Service Discovery**.
- **SuperTXT** runs over **SSH** (port 22) and executes **WebAssembly**
  ("WA-Nine"), which puts it in a category of its own rather than with the
  plain-stream group.
- **Molerat** the catalogue describes, in its own words, as "badly designed in
  many ways" — worth knowing before investing in it.
- **FSP** is port 21 **UDP**, FTP-like, transferring unformatted files.

**Effect on the categories.** Kepler joins category A (clean stream port).
Demarkus joins category C with guppy and fsp: it is UDP, so a Reticulum link
would duplicate reliability it already arranges. SuperTXT is neither, since
SSH is a stream but brings its own authentication, so carrying it over a link
would mean two identity systems, which is the misfin problem in a different
costume.

**Two of these are interesting to us beyond compatibility.** Kepler's caching
metadata is the only cache model in the whole family, and a browser that
stores fetched documents as graph nodes has an obvious use for expiry and
last-updated. Demarkus's **capability tokens** rhyme directly with the
participant gate and tessera, which is worth a look even though the protocol
itself is UDP and awkward for us.

## Correction: terse is a design sketch, not a protocol (2026-08-04)

Listing `terse` as "unsurveyed, probably category A" was a guess, and it was
wrong. Reading [TerseNet's README](https://github.com/runvnc/tersenet)
directly: it specifies **no transport, no port, no request or response
format**. The author says plainly that after two days the prototype is "a
janky RST viewer that only knows about headings and paragraphs. Not useable
for anything," and that everything beyond that "is just an idea -- not
actually coded yet."

So terse is not a protocol we could implement faithfully even if we wanted to.
It is a document-format sketch (a restricted reStructuredText subset), a URI
scheme, and a set of design intentions. It should come out of any protocol
count and be read as design kin instead.

## Design kin: what TerseNet independently arrived at

Worth recording because it is a convergence signal rather than a borrowing
opportunity. TerseNet's sketch reaches, from a standing start, most of the
architecture this workspace has been building:

| TerseNet | Ours |
|---|---|
| "Unlike Chrome or Firefox, which incorporate a full bloated operating system", a relatively simple system | The W3C knockout strategy and genet's decomposition |
| Multiple implementations by individuals or small teams, so no corporation controls the platform | Merely's posture, and why the spec crates are published separately |
| Three browser tiers: Info, Media Application (wasm), Extended Application (wasm plus device-driver-like extensions) | The engine picker's rung ladder (engine adoption plan (`turnstone/design_docs/2026-08-03_turnstone_engine_adoption_plan.md`)) |
| Attachments and applications never load without explicit user selection; separate Media and Applications tabs | The participant gate, and the UA taxonomy plan's downloads and permissions rows |
| Peer-to-peer distribution with full-text search across the peer network | Turnstone's places, plus the Knot search fusion that is the intel family's first real consumer |
| Bounded pages: 5KiB text, 1KiB attachment listings, 10KB default image cap | Bounded documents suit a graph canvas, where every node is a document |
| Links only to other terse pages | A closed link graph, which is a fully navigable bounded graph for a graph browser |

None of this is adoptable, because none of it is built. It is corroboration
that the shape is a natural attractor, which is a different and cheaper kind
of useful.

## The one actionable thing: IconVG

TerseNet embeds [IconVG](https://github.com/google/iconvg) between paragraphs,
and IconVG is the part that is real, specified, and independent of TerseNet
entirely: a compact binary format for icons, logos, glyphs and emoji by Nigel
Tao, deliberately far simpler than SVG (no text, multimedia, interactivity,
scripting, animation, DOM).

**It maps onto our painting stack exactly, which is verified rather than
hoped.** IconVG's model is "define geometry from linear, quadratic and cubic
Bézier segments; define paint as flat colors or gradients; fill the geometry
with the paint." Our `paint_list_api` already carries a `Path` item built from
a Bézier command sequence plus `LinearGradient`, `RadialGradient` and
`ConicGradient` payloads, and sprigging's custom-paint `Path` leaf already has
`move_to` / `line_to` / `quad_to` / `cubic_to` / `close` / `arc`. So an IconVG
decoder needs **no new rendering capability at all**; it decodes into paint
items we already emit.

`iconvg` is available on crates.io and there appears to be no Rust
implementation (the known ones are Go).

Where it would actually earn its place is **node faces in the graph canvas**.
Nodes are content-type-coded shapes carrying favicons, and IconVG is purpose-built
for exactly that payload: vector, tiny, resolution-independent, no scripting
surface, and a fraction of the attack surface of SVG. That is a better fit for
our canvas than for any smolweb protocol, terse included.

Not scheduled. Recorded because it is the rare case where an outside format
lands precisely on capability we already have.

## Homes

The rule stays the one already recorded, and carrier independence does not
disturb it:

> **A crate's home follows who defines the spec, not what carries the bytes.**

- **smolweb workspace**: the internet-native small protocols and the formats
  their specs define. A gemini crate that can run over a Reticulum link is
  still a gemini crate.
- **retinue**: the Reticulum-native protocols and formats — micron, LXMF, node
  page and file serving. These are Reticulum facts and belong with the trunk.
- **errand / turnstone**: the composition. "Fetch this URL, over that carrier"
  is a client-integration decision, which is ours, and it is where the
  addressing adapter and the carrier choice live.

One open question worth naming rather than deciding here: errand is a
published, deliberately light crate, and retinue is a large dependency. So the
Reticulum lane is either a feature on errand or a separate bridge that
turnstone composes. The second is likelier right, but it is a real decision and
it is not urgent.

## Is broad, esoteric protocol support possible?

Yes, and the honest limiting factor is not code.

**Code volume is small and measured, not guessed.** From the three crates
landed so far: `gopher-protocol`'s client is about 130 lines and its menu
parser 245; `finger-protocol`'s client is about 180. A simple stream protocol
costs roughly 100 to 200 lines of client plus its format, with tests. Formats
are consistently more expensive than transports, which is why scroll,
scorpion, and molerat (each with a richer document format than gemtext) are the
expensive three and the remaining transports are nearly free.

**The real constraints are two.**

*Specs.* Several of these have thin, single-author, or moving specifications,
and one (terse) I could not characterise at all from public sources. A crate
published as spec-accurate cannot be written from a summary; the gopher work
already made that concrete, where an introductory page had to be abandoned for
the actual 1993 document before the wire format could be implemented.

*Testability.* There are few live servers for the obscure end of this list, so
"it compiles and round-trips against my own fixture" is most of what is
available. That is worth stating in each crate rather than implying field
verification we do not have.

**Sequencing that follows from all this**, if it is ever picked up:

1. `gophers` and `mercury` — the cheapest real wins, both nearly free beside
   crates that exist.
2. `scroll` — not for expansion but because it is a live fidelity bug: its
   nematic engine delegates to the gemtext engine although scroll defines a
   richer format, and it has no errand transport at all. It is the one item on
   this page that fixes something broken rather than adding surface.
3. `text` / SuperText — one small crate, trivially A.
4. `scorpion`, `molerat` — real work, because each brings a document format.
5. `fsp` — read the spec first; it is category C and may simply not be worth
   carrying. `terse` is off the list entirely, per the correction above.

Reticulum support, on this analysis, is not step six. It is a property the
`client` layer already gives every one of them, plus the addressing decision,
which is worth making once and early.
