<script lang="ts">
  import { onMount, tick } from 'svelte';
  import { compact, csvEscape, diffWords } from './lib';

  type Source = { id:string; name:string; url:string; selector:string; extract_mode:string; threshold:number; interval_minutes:number; enabled:number; last_checked:string|null; last_status:string; last_error:string|null; next_check:string|null; created_at:string };
  type Change = { id:string; source_id:string; source_name:string; source_url:string; selector:string; previous_text:string; current_text:string; change_ratio:number; summary:string; review_state:string; useful:number|null; created_at:string };
  type Stats = { sources:number; unread:number; useful:number; rated:number };
  type FormData = { name:string; url:string; selector:string; extract_mode:string; threshold:number; interval_minutes:number };
  type Page = 'home'|'demo'|'privacy'|'terms';

  const emptyForm: FormData = { name:'', url:'', selector:'main', extract_mode:'selector', threshold:3, interval_minutes:1440 };
  const SLUG = 'change-diff-inbox';
  const licenseKey = `sb_license:${SLUG}`;
  const verdictKey = `${licenseKey}:verdict`;
  let page: Page = routeFor(location.pathname);
  let tab: 'inbox'|'sources'|'pro' = 'inbox';
  let sources: Source[] = [];
  let changes: Change[] = [];
  let stats: Stats = { sources:0, unread:0, useful:0, rated:0 };
  let stateFilter = 'all';
  let sourceFilter = 'all';
  let loading = page === 'demo';
  let loadError = '';
  let online = navigator.onLine;
  let expanded = '';
  let formOpen = false;
  let editing = '';
  let form: FormData = {...emptyForm};
  let formError = '';
  let saving = false;
  let checking = '';
  let toast = '';
  let toastTimer = 0;
  let licensed = false;
  let licenseNotice = '';
  let licenseInput = '';
  let verifying = false;
  let build = 'dev';
  let routeAnnouncement = '';

  $: isDemo = page === 'demo';
  $: apiBase = isDemo ? '/api/demo' : '/api';
  $: displayedChanges = changes.filter(c => (stateFilter === 'all' || c.review_state === stateFilter) && (sourceFilter === 'all' || c.source_id === sourceFilter));
  $: usefulness = stats.rated ? Math.round(stats.useful / stats.rated * 100) : 0;

  onMount(() => {
    setMetadata();
    initialize(page === 'demo');
    fetch('/health').then(response => response.json()).then(value => build = value.build || 'dev').catch(() => {});
    const onOnline = () => { online = true; initialize(false); };
    const onOffline = () => online = false;
    const onPop = () => changeRoute(routeFor(location.pathname), false);
    window.addEventListener('online', onOnline);
    window.addEventListener('offline', onOffline);
    window.addEventListener('popstate', onPop);
    return () => {
      window.removeEventListener('online', onOnline);
      window.removeEventListener('offline', onOffline);
      window.removeEventListener('popstate', onPop);
    };
  });

  function routeFor(path:string): Page {
    if (path === '/demo') return 'demo';
    if (path === '/privacy') return 'privacy';
    if (path === '/terms') return 'terms';
    return 'home';
  }

  function setMetadata() {
    document.title = page === 'privacy' ? 'Privacy — Change Diff Inbox'
      : page === 'terms' ? 'Terms — Change Diff Inbox'
      : page === 'demo' ? 'Demo — Change Diff Inbox'
      : 'Change Diff Inbox — Review meaningful page changes';
    const canonical = document.querySelector<HTMLLinkElement>('link[rel="canonical"]');
    if (canonical) canonical.href = `https://change-diff-inbox.sociobot.in${page === 'home' ? '/' : `/${page}`}`;
    const socialUrl = document.querySelector<HTMLMetaElement>('meta[property="og:url"]');
    if (socialUrl) socialUrl.content = canonical?.href || location.href;
  }

  async function changeRoute(next:Page, push=true) {
    page = next;
    tab = 'inbox';
    formOpen = false;
    const path = next === 'home' ? '/' : `/${next}`;
    if (push) history.pushState({}, '', path);
    setMetadata();
    await tick();
    if (next === 'home' || next === 'demo') await initialize(true);
    window.scrollTo({top:0, behavior:'auto'});
    requestAnimationFrame(() => {
      const heading = document.querySelector<HTMLElement>('main h1');
      routeAnnouncement = heading?.textContent || '';
      heading?.focus();
    });
  }

  async function initialize(showLoading:boolean) {
    if (page === 'privacy' || page === 'terms') return;
    loading = showLoading;
    loadError = '';
    try {
      await fetch(`${apiBase}/session`, {method:'POST'}).then(async response => {
        if (!response.ok) throw new Error((await response.json().catch(() => ({}))).error || 'Could not start a workspace');
      });
      if (!isDemo) await handleLicense();
      await loadAll(false);
    } catch (error) {
      loadError = error instanceof Error ? error.message : 'Could not load the inbox';
    } finally { loading = false; }
  }

  async function api<T>(path:string, init?:RequestInit):Promise<T> {
    const response = await fetch(`${apiBase}${path}`, { ...init, headers: { 'content-type':'application/json', ...(init?.headers || {}) } });
    if (!response.ok) {
      const body = await response.json().catch(() => ({}));
      throw new Error(body.error || `Request failed (${response.status})`);
    }
    return response.status === 204 ? undefined as T : response.json();
  }

  async function loadAll(showLoading = false) {
    loading = showLoading; loadError = '';
    try {
      [sources, changes, stats] = await Promise.all([api('/sources'), api('/changes'), api('/stats')]);
    } catch (error) { loadError = error instanceof Error ? error.message : 'Could not load the inbox'; }
    finally { loading = false; }
  }

  function notify(message:string) {
    toast = message;
    window.clearTimeout(toastTimer);
    toastTimer = window.setTimeout(() => toast = '', 4500);
  }

  function selectTab(next:'inbox'|'sources'|'pro') {
    tab = next;
    formOpen = false;
    requestAnimationFrame(() => document.querySelector<HTMLElement>('main h1')?.focus());
  }

  function openNew() {
    editing = '';
    form = {...emptyForm};
    formError = '';
    formOpen = true;
    tab = 'sources';
    requestAnimationFrame(() => document.getElementById('source-name')?.focus());
  }

  function openEdit(source:Source) {
    editing = source.id;
    form = { name:source.name, url:source.url, selector:source.selector, extract_mode:source.extract_mode, threshold:source.threshold*100, interval_minutes:source.interval_minutes };
    formError = '';
    formOpen = true;
    requestAnimationFrame(() => document.getElementById('source-name')?.focus());
  }

  async function saveSource(event:SubmitEvent) {
    event.preventDefault();
    formError = '';
    saving = true;
    const payload = {...form, threshold:form.threshold/100};
    try {
      const saved = await api<Source>(editing ? `/sources/${editing}` : '/sources', {method:editing?'PUT':'POST', body:JSON.stringify(payload)});
      formOpen = false;
      await loadAll();
      notify(editing ? 'Source settings saved.' : 'Source added. Capturing its first baseline.');
      if (!editing) await runCheck(saved.id);
    } catch(error) { formError = error instanceof Error ? error.message : 'Could not save this source'; }
    finally { saving = false; }
  }

  async function runCheck(id:string) {
    checking = id;
    try {
      const result = await api<{message:string}>(`/sources/${id}/check`, {method:'POST'});
      notify(result.message);
      await loadAll();
    } catch(error) { notify(error instanceof Error ? error.message : 'Check failed'); }
    finally { checking = ''; }
  }

  async function removeSource(source:Source) {
    if (!confirm(`Remove “${source.name}” and all of its saved changes? This cannot be undone.`)) return;
    try {
      await api(`/sources/${source.id}`, {method:'DELETE'});
      await loadAll();
      notify('Source and its change history removed.');
    } catch(error) { notify(error instanceof Error ? error.message : 'Could not remove source'); }
  }

  async function review(change:Change, review_state?:string, useful?:boolean) {
    try {
      await api(`/changes/${change.id}`, {method:'PATCH', body:JSON.stringify({review_state,useful})});
      await loadAll();
      notify(useful === true ? 'Marked useful.' : useful === false ? 'Marked as noise.' : 'Review state updated.');
    } catch(error) { notify(error instanceof Error ? error.message : 'Could not update change'); }
  }

  function exportCsv() {
    const rows = [['source','url','detected','change_percent','summary','review_state','useful'], ...displayedChanges.map(c => [c.source_name,c.source_url,c.created_at,(c.change_ratio*100).toFixed(1),c.summary,c.review_state,c.useful===1?'yes':c.useful===0?'no':''])];
    const blob = new Blob([rows.map(row=>row.map(csvEscape).join(',')).join('\n')], {type:'text/csv'});
    const anchor = document.createElement('a');
    anchor.href = URL.createObjectURL(blob);
    anchor.download = 'change-diff-inbox.csv';
    anchor.click();
    URL.revokeObjectURL(anchor.href);
    notify('Inbox exported as CSV.');
  }

  async function resetDemo() {
    try {
      await api('/reset', {method:'POST'});
      expanded = '';
      stateFilter = 'all';
      sourceFilter = 'all';
      await loadAll();
      notify('Sample data reset.');
    } catch (error) { notify(error instanceof Error ? error.message : 'Could not reset sample data'); }
  }

  async function handleLicense() {
    const params = new URLSearchParams(location.search);
    const returned = params.get('license');
    if (returned) {
      localStorage.setItem(licenseKey, returned);
      params.delete('license');
      history.replaceState({}, '', `${location.pathname}${params.size ? `?${params}` : ''}`);
    }
    const token = returned || localStorage.getItem(licenseKey);
    if (!token) return;
    let cached: {valid?:boolean; checkedAt?:number}|null = null;
    try { cached = JSON.parse(localStorage.getItem(verdictKey) || 'null'); } catch { localStorage.removeItem(verdictKey); }
    if (cached?.valid) licensed = true;
    if (!cached?.checkedAt || Date.now() - cached.checkedAt > 86_400_000 || returned) await verifyLicense(token);
  }

  async function verifyLicense(token:string) {
    verifying = true;
    licenseNotice = '';
    try {
      const verdict = await api<{valid:boolean;reason:string}>('/license', {method:'POST', body:JSON.stringify({license:token})});
      licensed = verdict.valid;
      localStorage.setItem(verdictKey, JSON.stringify({...verdict, checkedAt:Date.now()}));
      if (!verdict.valid) licenseNotice = 'This license is not active. Free monitoring remains available.';
      else notify('Pro license active on this device.');
    } catch (error) {
      licenseNotice = error instanceof Error ? error.message : 'The license could not be checked. Free monitoring remains available.';
    } finally { verifying = false; }
  }

  function restoreLicense(event:SubmitEvent) {
    event.preventDefault();
    const token = licenseInput.trim();
    if (!token) { licenseNotice = 'Paste the license token from your receipt.'; return; }
    localStorage.setItem(licenseKey, token);
    verifyLicense(token);
  }

  function formatTime(value:string|null) {
    if (!value) return 'Not checked yet';
    return new Date(value).toLocaleString(undefined, {dateStyle:'medium', timeStyle:'short'});
  }
</script>

<a class="skip-link" href="#main">Skip to main content</a>
<div class="route-announcement" aria-live="polite">{routeAnnouncement}</div>
<header class="site-header">
  <a class="brand" href="/" on:click|preventDefault={() => changeRoute('home')} aria-label="Change Diff Inbox home">
    <span class="brand-mark" aria-hidden="true"><i></i><i></i><i></i><b></b></span>
    <span>Change Diff <em>Inbox</em></span>
  </a>
  <nav aria-label="Main navigation">
    <a href="/demo" on:click|preventDefault={() => changeRoute('demo')}>Demo</a>
    <a href="/privacy" on:click|preventDefault={() => changeRoute('privacy')}>Privacy</a>
    <a href="/terms" on:click|preventDefault={() => changeRoute('terms')}>Terms</a>
  </nav>
  {#if page === 'home'}<button class="primary compact" on:click={openNew}><span aria-hidden="true">＋</span> Add source</button>{/if}
</header>

{#if isDemo}
  <aside class="demo-banner" aria-label="Sample workspace">
    <strong>Demo — sample data, nothing is saved to your workspace</strong>
    <span>Changes remain in this temporary demo for up to 24 hours.</span>
    <button on:click={resetDemo}>Reset demo</button>
    <a href="/" on:click|preventDefault={() => changeRoute('home')}>Start for real</a>
  </aside>
{/if}
{#if !online}<div class="offline" role="status">Offline — the app shell is available. Reconnect to load or change inbox data.</div>{/if}

<main id="main">
{#if page === 'privacy'}
  <article class="legal">
    <p class="eyebrow">Privacy</p>
    <h1 tabindex="-1">Privacy for your monitored pages</h1>
    <p class="lede">Each browser gets a separate workspace. Other visitors cannot read or change it.</p>
    <h2>What this service stores</h2>
    <p>The service stores source names, public URLs, selectors, extracted text, review decisions, and errors in SQLite.</p>
    <p>A signed browser cookie identifies your workspace. It does not contain your saved content.</p>
    <h2>What leaves this service</h2>
    <p>The server requests each public page you add and its robots.txt file.</p>
    <p>License checks send only the license token to the Sociobot billing API.</p>
    <p>This site has no analytics, advertising cookies, remote fonts, or tracking scripts.</p>
    <h2>Demo and deletion</h2>
    <p>The demo uses a separate temporary workspace. It never reads or writes your regular workspace.</p>
    <p>Removing a source also removes its snapshots and changes. Demo workspaces expire after 24 hours.</p>
    <h2>Operator contact</h2>
    <p>Email <a href="mailto:privacy@sociobot.in">privacy@sociobot.in</a> for access or deletion requests. Last updated 5 September 2026.</p>
  </article>
{:else if page === 'terms'}
  <article class="legal">
    <p class="eyebrow">Terms</p>
    <h1 tabindex="-1">Terms for monitoring public pages</h1>
    <p class="lede">Monitor only public material that you are allowed to access.</p>
    <h2>Acceptable use</h2>
    <p>Do not evade authentication, access private networks, overwhelm sites, or break their terms.</p>
    <p>The service checks robots.txt, limits response size, and enforces minimum check intervals.</p>
    <h2>Checks and accuracy</h2>
    <p>Selectors and thresholds reduce noise. They cannot guarantee detection of every important change.</p>
    <p>Review important provider changes at the original source.</p>
    <h2>Free and Pro plans</h2>
    <p>The free plan includes five sources with daily or weekly checks.</p>
    <p>Pro is a $39 one-time license. It includes unlimited sources and 15-minute or hourly checks.</p>
    <p>Sales are unavailable until the billing offer is registered. The license field remains ready for issued tokens.</p>
    <p>Sociobot/Dodo is the merchant of record. Refunds revoke the related license.</p>
    <h2>Warranty</h2>
    <p>The software is provided “as is” under the MIT License. Last updated 5 September 2026.</p>
  </article>
{:else if loadError}
  <section class="state-panel" role="alert">
    <span class="state-icon" aria-hidden="true">!</span>
    <h1 tabindex="-1">The inbox could not connect</h1>
    <p>{loadError}. Check the connection, then try again.</p>
    <button class="primary" on:click={() => initialize(true)}>Try again</button>
  </section>
{:else if loading}
  <section class="loading-state" aria-live="polite">
    <h1 tabindex="-1">Loading the change inbox</h1>
    <div class="pulse-line wide"></div><div class="pulse-line"></div><div class="pulse-card"></div>
    <span>Preparing this workspace.</span>
  </section>
{:else if page === 'home' && tab === 'inbox' && sources.length === 0}
  <section class="hero">
    <div class="hero-copy">
      <p class="eyebrow"><span></span>Page change monitoring</p>
      <h1 tabindex="-1">Review meaningful page changes</h1>
      <p class="lede">For developers monitoring documentation, status pages, and dashboards who need useful text changes instead of screenshot noise.</p>
      <div class="hero-actions">
        <a class="primary" href="/demo" on:click|preventDefault={() => changeRoute('demo')}>Try it with sample data</a>
        <button class="secondary" on:click={openNew}>Add a source</button>
      </div>
      <p class="action-note">The sample opens a separate inbox with three realistic page changes.</p>
      <ul class="plain-facts">
        <li><strong>Free:</strong> five sources with daily or weekly checks.</li>
        <li><strong>Private:</strong> each browser has an isolated workspace.</li>
        <li><strong>Offline:</strong> the app shell reloads after your first visit.</li>
      </ul>
    </div>
    <figure class="hero-art">
      <picture>
        <source type="image/avif" srcset="/assets/hero-640.avif 640w, /assets/hero-960.avif 960w" sizes="(max-width: 800px) 92vw, 52vw">
        <source type="image/webp" srcset="/assets/hero-640.webp 640w, /assets/hero-960.webp 960w" sizes="(max-width: 800px) 92vw, 52vw">
        <img src="/assets/hero-960.jpg" width="960" height="640" alt="Translucent data sheets pass beneath a lens that isolates one changed line" fetchpriority="high" decoding="async">
      </picture>
      <figcaption>Illustration of a selected structured change</figcaption>
    </figure>
  </section>
  <section class="preview-section" aria-labelledby="preview-title">
    <div class="section-intro"><p class="eyebrow">Sample output</p><h2 id="preview-title">See the changed words first</h2><p>The inbox keeps the selected text, change size, time, and review state together.</p></div>
    <article class="preview-change">
      <span class="change-signal" aria-hidden="true"></span>
      <div><small>Northstar API limits · 18 minutes ago</small><h3>Standard limit changed from 1,000 to 1,500 requests per minute</h3><code>#limits · table · 16.7% changed</code></div>
      <strong>Unread</strong>
    </article>
  </section>
  <section class="how-section" aria-labelledby="how-title">
    <p class="eyebrow">How it works</p><h2 id="how-title">Turn a selected section into a review queue</h2>
    <ol><li><strong>Select a page section</strong><span>Add a public URL and choose a CSS selector, table, code block, or JSON-LD.</span></li><li><strong>Set a noise threshold</strong><span>The first check saves a baseline. Later checks ignore changes below your threshold.</span></li><li><strong>Review each change</strong><span>Compare changed words, mark useful alerts, archive noise, or export the displayed inbox.</span></li></ol>
  </section>
  <section class="boundaries-section" aria-labelledby="boundaries-title">
    <div><p class="eyebrow">Limits and privacy</p><h2 id="boundaries-title">Public HTML only</h2></div>
    <ul><li>No logins, browser automation, or anti-bot bypasses.</li><li>Private and local network addresses are blocked.</li><li>Responses stop at 2 MB. Extracted text stops at 250 KB.</li><li>The server checks robots.txt before each page request.</li></ul>
  </section>
  <section class="pricing-section" aria-labelledby="pricing-title">
    <div><p class="eyebrow">Plans</p><h2 id="pricing-title">Use five sources for free</h2><p>Daily and weekly checks are included. CSV export stays free.</p></div>
    <div class="price-panel"><strong>$39 <small>one time</small></strong><p>Pro includes unlimited sources and 15-minute or hourly checks.</p><span>Sales are not available yet. Billing registration is pending.</span><button class="secondary" on:click={() => selectTab('pro')}>View license details</button></div>
  </section>
{:else}
  <section class="workspace-head">
    <div>
      <p class="eyebrow"><span></span>{isDemo ? 'Sample inbox' : tab === 'sources' ? 'Sources' : tab === 'pro' ? 'Plans' : 'Change inbox'}</p>
      <h1 tabindex="-1">{isDemo ? 'Review sample page changes' : tab === 'sources' ? 'Manage monitored sources' : tab === 'pro' ? 'Compare free and Pro plans' : 'Review detected page changes'}</h1>
      {#if isDemo}<p>Three sample sources show table, code, and structured-data changes.</p>
      {:else if tab === 'inbox'}<p>{stats.unread ? `${stats.unread} item${stats.unread===1?'':'s'} waiting for review.` : 'No unread changes. Scheduled checks continue in the background.'}</p>
      {:else if tab === 'sources'}<p>Add public pages and choose the exact content to compare.</p>
      {:else}<p>The free plan works without a license. The license field is ready for issued tokens.</p>{/if}
    </div>
    {#if tab === 'inbox' || isDemo}<div class="score"><strong>{stats.rated ? `${usefulness}%` : '—'}</strong><span>useful alerts</span><small>{stats.rated} rated</small></div>{/if}
  </section>

  {#if !isDemo}
    <nav class="app-tabs" aria-label="Workspace sections">
      <button class:active={tab==='inbox'} on:click={() => selectTab('inbox')}>Inbox {#if stats.unread}<span class="count">{stats.unread}</span>{/if}</button>
      <button class:active={tab==='sources'} on:click={() => selectTab('sources')}>Sources</button>
      <button class:active={tab==='pro'} on:click={() => selectTab('pro')}>Plans {#if licensed}<span class="licensed-dot" title="Licensed">✓</span>{/if}</button>
    </nav>
  {/if}

  {#if tab === 'inbox' || isDemo}
    <section class="toolbar" aria-label="Inbox filters">
      <label>Review state<select bind:value={stateFilter}><option value="all">All changes</option><option value="unread">Unread</option><option value="reviewed">Reviewed</option><option value="archived">Archived</option></select></label>
      <label>Source<select bind:value={sourceFilter}><option value="all">Every source</option>{#each sources as source}<option value={source.id}>{source.name}</option>{/each}</select></label>
      <button class="secondary export" on:click={exportCsv}>Export CSV</button>
    </section>
    {#if displayedChanges.length === 0}
      <section class="quiet-state">
        <div class="radar" aria-hidden="true"><i></i><b></b></div>
        <h2>{changes.length ? 'No changes match these filters' : 'No changes detected yet'}</h2>
        <p>{changes.length ? 'Choose another review state or source.' : 'Run a source check after its selected page content changes.'}</p>
        {#if !isDemo}<button class="secondary" on:click={() => selectTab('sources')}>View monitored sources</button>{/if}
      </section>
    {:else}
      <section class="change-list" aria-label="Detected changes">
        {#each displayedChanges as change}
          <article class:unread={change.review_state==='unread'} class="change-card">
            <button class="change-toggle" aria-expanded={expanded===change.id} on:click={() => expanded=expanded===change.id?'':change.id}>
              <span class="change-signal" aria-hidden="true"></span>
              <span class="change-main"><span class="change-meta"><b>{change.source_name}</b><time datetime={change.created_at}>{formatTime(change.created_at)}</time><em>{(change.change_ratio*100).toFixed(1)}% changed</em></span><strong>{compact(change.summary,220)}</strong><code>{change.selector}</code></span>
              <span class="chevron" aria-hidden="true">⌄</span>
            </button>
            {#if expanded===change.id}
              {@const pieces=diffWords(change.previous_text,change.current_text)}
              <div class="change-detail">
                <div class="diff-grid"><div><h3><span class="minus">−</span> Previous</h3><pre>{#each pieces.old as part}<span class:removed={part.type==='removed'}>{part.value}</span>{/each}</pre></div><div><h3><span class="plus">＋</span> Current</h3><pre>{#each pieces.next as part}<span class:added={part.type==='added'}>{part.value}</span>{/each}</pre></div></div>
                <div class="review-bar">{#if isDemo}<em>Sample source — no external request</em>{:else}<a href={change.source_url} target="_blank" rel="noreferrer">Open source <span class="sr-only">in a new tab</span> ↗</a>{/if}<span>Was this alert useful?</span><button class:chosen={change.useful===1} on:click={() => review(change,'reviewed',true)} aria-label="Mark this alert useful">Yes</button><button class:chosen={change.useful===0} on:click={() => review(change,'reviewed',false)} aria-label="Mark this alert as noise">No, noise</button><button on:click={() => review(change,'archived')}>Archive</button></div>
              </div>
            {/if}
          </article>
        {/each}
      </section>
    {/if}
  {:else if tab === 'sources'}
    <div class="source-command"><button class="primary" on:click={openNew}>Add source</button></div>
    {#if formOpen}
      <section class="source-form-wrap" aria-labelledby="form-title">
        <div class="form-heading"><div><p class="eyebrow">{editing?'Edit source':'New source'}</p><h2 id="form-title">{editing?'Change source settings':'Choose page content'}</h2></div><button class="icon-button" on:click={() => formOpen=false} aria-label="Close source form">×</button></div>
        <form on:submit={saveSource} novalidate>
          <div class="field-grid">
            <label for="source-name">Name<input id="source-name" bind:value={form.name} required minlength="2" maxlength="80" autocomplete="off" aria-invalid={formError ? 'true' : undefined} aria-describedby={formError ? 'source-form-error' : 'source-name-help'}><small id="source-name-help">Example: Vendor API versions</small></label>
            <label for="source-url">Public page URL<input id="source-url" type="url" bind:value={form.url} required placeholder="https://docs.example.com/changelog" autocomplete="url" aria-invalid={formError ? 'true' : undefined} aria-describedby={formError ? 'source-form-error' : 'source-url-help'}><small id="source-url-help">No logins, local URLs, or bypasses</small></label>
          </div>
          <div class="field-grid thirds">
            <label for="extract-mode">Extract<select id="extract-mode" bind:value={form.extract_mode}><option value="selector">CSS-selected section</option><option value="table">Table</option><option value="code">Code block</option><option value="jsonld">JSON-LD</option></select></label>
            <label for="selector">CSS selector<input id="selector" class="mono" bind:value={form.selector} maxlength="200" placeholder="main .release-notes"><small>Leave blank for the mode default</small></label>
            <label for="threshold">Noise threshold<div class="unit-input"><input id="threshold" type="number" bind:value={form.threshold} min="0" max="100" step="0.5"><span>%</span></div><small>Ignore smaller changes</small></label>
          </div>
          <fieldset><legend>Check interval</legend><div class="intervals">{#each [{v:15,l:'15 min',pro:true},{v:60,l:'Hourly',pro:true},{v:1440,l:'Daily',pro:false},{v:10080,l:'Weekly',pro:false}] as option}<label class:locked={option.pro&&!licensed}><input type="radio" bind:group={form.interval_minutes} value={option.v} disabled={option.pro&&!licensed}><span>{option.l}{#if option.pro&&!licensed}<small>PRO</small>{/if}</span></label>{/each}</div></fieldset>
          {#if formError}<p id="source-form-error" class="form-error" role="alert">{formError}</p>{/if}
          <div class="form-actions"><button type="button" class="text-button" on:click={() => formOpen=false}>Cancel</button><button class="primary" disabled={saving}>{saving?'Saving…':editing?'Save settings':'Add and capture baseline'}</button></div>
        </form>
      </section>
    {/if}
    {#if sources.length===0 && !formOpen}<section class="quiet-state compact-state"><h2>No sources yet</h2><p>Add a public docs page, status section, pricing table, or code block.</p><button class="primary" on:click={openNew}>Add your first source</button></section>{/if}
    <section class="source-list" aria-label="Monitored sources">
      {#each sources as source}
        <article class="source-row"><span class="status-dot {source.last_status}" aria-hidden="true"></span><div class="source-main"><div><h2>{source.name}</h2><span class="status-label">{source.last_status==='new'?'Needs baseline':source.last_status==='error'?'Check failed':source.last_status==='changed'?'Change detected':source.last_status==='quiet'?'Below threshold':'Watching'}</span></div><a href={source.url} target="_blank" rel="noreferrer">{source.url}<span class="sr-only"> opens in a new tab</span></a><p><code>{source.extract_mode==='jsonld'?'JSON-LD':source.selector||source.extract_mode}</code><span>{(source.threshold*100).toFixed(1)}% threshold</span><span>Every {source.interval_minutes<60?`${source.interval_minutes} min`:source.interval_minutes===60?'hour':source.interval_minutes===1440?'day':`${Math.round(source.interval_minutes/1440)} days`}</span><span>{formatTime(source.last_checked)}</span></p>{#if source.last_error}<div class="source-error" role="status">{source.last_error}</div>{/if}</div><div class="row-actions"><button class="secondary" on:click={() => runCheck(source.id)} disabled={checking===source.id}>{checking===source.id?'Checking…':'Check now'}</button><button class="icon-button" on:click={() => openEdit(source)} aria-label={`Edit ${source.name}`}>✎</button><button class="icon-button danger" on:click={() => removeSource(source)} aria-label={`Remove ${source.name}`}>×</button></div></article>
      {/each}
    </section>
  {:else}
    <section class="pro-hero">
      <div><p class="eyebrow"><span></span>One-time license</p><h2>Pro adds capacity and shorter schedules</h2><p class="lede">Pro includes unlimited sources and 15-minute or hourly schedules. The free plan keeps five daily or weekly sources.</p><div class="price"><strong>$39</strong><span>one time<br>one product license</span></div>{#if licensed}<div class="license-active">✓ Pro is active on this device</div>{:else}<p class="billing-pending" role="status">Sales are not available yet because billing registration is pending.</p>{/if}<p class="merchant">Sociobot/Dodo handles checkout and refunds when sales are available.</p></div>
      <div class="glass-spec"><span class="spec-label">PRO DETAILS</span><ul><li><b>∞</b><span><strong>Unlimited sources</strong>Add more than five monitored pages.</span></li><li><b>15</b><span><strong>Minute checks</strong>Choose 15-minute, hourly, daily, or weekly checks.</span></li><li><b>24h</b><span><strong>Cached verification</strong>License results are checked at most once each day.</span></li></ul></div>
    </section>
    <section class="restore"><div><h2>Restore a purchase</h2><p>Paste the license token from your receipt to use Pro on this device.</p></div><form on:submit={restoreLicense}><label for="license">License token</label><div><input id="license" type="password" bind:value={licenseInput} autocomplete="off" spellcheck="false"><button class="secondary" disabled={verifying}>{verifying?'Checking…':'Verify license'}</button></div>{#if licenseNotice}<p role="status">{licenseNotice}</p>{/if}</form></section>
  {/if}
{/if}
</main>

<footer>
  <span>Review selected page changes in one private workspace.</span>
  <nav aria-label="Footer"><a href="/privacy" on:click|preventDefault={() => changeRoute('privacy')}>Privacy</a><a href="/terms" on:click|preventDefault={() => changeRoute('terms')}>Terms</a><a href="https://github.com/B-Divyesh/sf-change-diff-inbox" rel="noreferrer">Source <span class="sr-only">on GitHub</span> ↗</a></nav>
  <small>Built by Param Factory · Build {build.slice(0,12)} · Original generated illustration, no stock assets.</small>
</footer>
{#if toast}<div class="toast" role="status">{toast}</div>{/if}
