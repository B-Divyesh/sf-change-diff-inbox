# Change Diff Inbox — visual thesis

## Direction

**Luminous glass data landscape.** The interface feels like a quiet observation room at night: monitored sources sit on a deep mineral-blue field, semantic changes glow like specimens under glass, and cyan/amber signals reveal what deserves an engineer's attention. The glass is functional—translucent layers distinguish source context, changed content, and review state—rather than decorative dashboard chrome.

The product is explicitly single-mode (dark). Monitoring is an ambient, long-running task and the dark field makes sparse, high-value change signals legible without turning the inbox into alarm-red noise.

## Tokens

- Background `#07111D` (deep night), elevated background `#0B1827`
- Glass surface `rgba(17, 37, 55, .76)`, solid surface `#102538`, hairline `#315069`
- Primary text `#F5FAFF`, secondary text `#ADC2D3`, quiet text `#8FA8BB`
- Signal cyan `#62E7E1`, cyan ink `#062728`
- Change amber `#FFC86A`, amber ink `#322006`
- Success mint `#6DE3A4`, danger coral `#FF8C84`

All body-text combinations meet 4.5:1. State is always paired with a word or icon, never color alone.

## Type and spacing

- Display and interface: self-hosted **Manrope**, weights 500–700, geometric but warm enough for review work.
- Extracted data and diffs: self-hosted **IBM Plex Mono**, weights 400–500; tabular figures make thresholds and timestamps scan cleanly.
- Scale: 14 / 16 / 18 / 24 / 34 / clamp(44–68) px. Body never drops below 16px in reading surfaces.
- Spacing follows a 4px base with primary steps 8, 12, 16, 24, 32, 48, 64. Reading measures top out at 72ch.
- Corners are 10px for controls, 16px for independent records, and 24px only for the hero specimen window.

## Layout and interaction grammar

The public introduction uses a split landscape: a concise promise beside an original floating semantic-diff specimen. Inside the product, a narrow left rail holds stable source context while the inbox gets the visual weight. Review items are independent records, so they may use glass cards; forms and secondary settings use proximity and ruled groups instead of nested cards.

Actions depress by 1px and brighten at the edge. Opening a change expands from its row. Keyboard focus is a two-layer cyan/ink ring. On phones, navigation becomes a compact top strip, metadata stacks, and secondary counts disappear before core actions do. All targets are at least 44px.

## Motion

UI transitions last 160–240ms and use opacity/transform only. Inbox details disclose from their source row; toast notices rise from the action that caused them. The hero's three luminous contour layers drift once on entrance and then remain still—nothing loops. Under `prefers-reduced-motion: reduce`, transforms and smooth scrolling are removed and state changes are instant or opacity-only.

## Original asset plan and prompt sheet

One generated hero illustration depicts an abstract monitored data landscape: translucent sheets containing table cells, code lines, and structured nodes, with one amber change moving through a cyan observation aperture. It explains the product's semantic extraction without implying browser automation or a screenshot service.

Art direction prompt: “Abstract nocturnal data landscape for a developer change-monitoring product, oblique isometric view, translucent mineral glass sheets floating above a deep navy field, crisp cyan code-line filaments and table-grid fragments, a single warm amber changed segment passing through a circular observation lens, sparse editorial composition, tactile frosted glass, controlled bloom, high contrast, sophisticated product illustration, generous negative space, no people, no browser UI, no readable text, no watermark, no logos, no brand symbols.”

Negative list: generic SaaS dashboard, purple gradient blob, neon city, people, robots, padlocks, literal email, legible pseudo-code, stock icons, excessive bloom.

Generation provenance: created for this product with the factory `factory-image` model on 2026-08-27. Original generated asset; no third-party source material. The exact prompt and generation metadata are stored beside the source image in `assets/src/hero-landscape.json`. Generated imagery is disclosed in the footer.

## Performance treatment

The hero ships as responsive AVIF/WebP with explicit dimensions, with the mobile candidate under 300KB. It is the sole high-priority image. The product uses at most two self-hosted WOFF2 files and targets initial JS under 200KB and CSS under 50KB. Glass blur is reduced on narrow/low-power layouts and never required for hierarchy.
