<script lang="ts">
  import { onMount } from 'svelte';
  import { compact, csvEscape, diffWords } from './lib';

  type Source = { id:string; name:string; url:string; selector:string; extract_mode:string; threshold:number; interval_minutes:number; enabled:number; baseline:string|null; last_checked:string|null; last_status:string; last_error:string|null; next_check:string|null; created_at:string };
  type Change = { id:string; source_id:string; source_name:string; source_url:string; selector:string; previous_text:string; current_text:string; change_ratio:number; summary:string; review_state:string; useful:number|null; created_at:string };
  type Stats = { sources:number; unread:number; useful:number; rated:number };
  type FormData = { name:string; url:string; selector:string; extract_mode:string; threshold:number; interval_minutes:number };

  const emptyForm: FormData = { name:'', url:'', selector:'main', extract_mode:'selector', threshold:3, interval_minutes:1440 };
  const SLUG = 'change-diff-inbox';
  const licenseKey = `sb_license:${SLUG}`;
  const verdictKey = `${licenseKey}:verdict`;
  let page: 'app'|'privacy'|'terms' = location.pathname === '/privacy' ? 'privacy' : location.pathname === '/terms' ? 'terms' : 'app';
  let tab: 'inbox'|'sources'|'pro' = 'inbox';
  let sources: Source[] = [];
  let changes: Change[] = [];
  let stats: Stats = { sources:0, unread:0, useful:0, rated:0 };
  let stateFilter = 'all';
  let sourceFilter = 'all';
  let loading = true;
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

  $: displayedChanges = changes.filter(c => (stateFilter === 'all' || c.review_state === stateFilter) && (sourceFilter === 'all' || c.source_id === sourceFilter));
  $: usefulness = stats.rated ? Math.round(stats.useful / stats.rated * 100) : 0;
  $: billingBase = location.hostname === 'change-diff-inbox.sociobot.in' ? 'https://api.sociobot.in' : 'https://pilot-api.sociobot.in';
  $: buyUrl = `${billingBase}/api/v1/products/${SLUG}/checkout`;

  onMount(() => {
    handleLicense();
    loadAll();
    const onOnline = () => { online = true; loadAll(); };
    const onOffline = () => online = false;
    window.addEventListener('online', onOnline); window.addEventListener('offline', onOffline);
    return () => { window.removeEventListener('online', onOnline); window.removeEventListener('offline', onOffline); };
  });

  async function api<T>(path:string, init?:RequestInit):Promise<T> {
    const response = await fetch(path, { ...init, headers: { 'content-type':'application/json', ...(init?.headers || {}) } });
    if (!response.ok) {
      const body = await response.json().catch(() => ({}));
      throw new Error(body.error || `Request failed (${response.status})`);
    }
    return response.status === 204 ? undefined as T : response.json();
  }

  async function loadAll() {
    loading = true; loadError = '';
    try {
      [sources, changes, stats] = await Promise.all([api('/api/sources'), api('/api/changes'), api('/api/stats')]);
    } catch (error) { loadError = error instanceof Error ? error.message : 'Could not load the inbox'; }
    finally { loading = false; }
  }

  function notify(message:string) {
    toast = message; window.clearTimeout(toastTimer); toastTimer = window.setTimeout(() => toast = '', 4500);
  }

  function openNew() {
    editing = ''; form = {...emptyForm}; formError = ''; formOpen = true; tab = 'sources';
    requestAnimationFrame(() => document.getElementById('source-name')?.focus());
  }

  function openEdit(source:Source) {
    editing = source.id; form = { name:source.name, url:source.url, selector:source.selector, extract_mode:source.extract_mode, threshold:source.threshold*100, interval_minutes:source.interval_minutes };
    formError=''; formOpen=true; requestAnimationFrame(() => document.getElementById('source-name')?.focus());
  }

  async function saveSource(event:SubmitEvent) {
    event.preventDefault(); formError=''; saving=true;
    if (!licensed && !editing && sources.length >= 5) { formError='The free tier supports five sources. Unlock Pro for unlimited sources.'; saving=false; return; }
    const payload = {...form, threshold:form.threshold/100, interval_minutes:(!licensed && form.interval_minutes < 1440) ? 1440 : form.interval_minutes};
    try {
      const saved = await api<Source>(editing ? `/api/sources/${editing}` : '/api/sources', {method:editing?'PUT':'POST', body:JSON.stringify(payload)});
      formOpen=false; await loadAll(); notify(editing ? 'Source settings saved.' : 'Source added. Capturing its first baseline…');
      if (!editing) await runCheck(saved.id);
    } catch(error) { formError = error instanceof Error ? error.message : 'Could not save this source'; }
    finally { saving=false; }
  }

  async function runCheck(id:string) {
    checking=id;
    try { const result=await api<{message:string}>(`/api/sources/${id}/check`,{method:'POST'}); notify(result.message); await loadAll(); }
    catch(error) { notify(error instanceof Error ? error.message : 'Check failed'); }
    finally { checking=''; }
  }

  async function removeSource(source:Source) {
    if (!confirm(`Remove “${source.name}” and all of its saved changes? This cannot be undone.`)) return;
    try { await api(`/api/sources/${source.id}`,{method:'DELETE'}); await loadAll(); notify('Source and its change history removed.'); }
    catch(error) { notify(error instanceof Error ? error.message : 'Could not remove source'); }
  }

  async function review(change:Change, review_state?:string, useful?:boolean) {
    try { await api(`/api/changes/${change.id}`, {method:'PATCH',body:JSON.stringify({review_state,useful})}); await loadAll(); notify(useful === true ? 'Marked useful.' : useful === false ? 'Marked as noise.' : 'Review state updated.'); }
    catch(error) { notify(error instanceof Error ? error.message : 'Could not update change'); }
  }

  function exportCsv() {
    const rows = [['source','url','detected','change_percent','summary','review_state','useful'], ...displayedChanges.map(c => [c.source_name,c.source_url,c.created_at,(c.change_ratio*100).toFixed(1),c.summary,c.review_state,c.useful===1?'yes':c.useful===0?'no':''])];
    const blob = new Blob([rows.map(row=>row.map(csvEscape).join(',')).join('\n')],{type:'text/csv'});
    const a=document.createElement('a'); a.href=URL.createObjectURL(blob); a.download='change-diff-inbox.csv'; a.click(); URL.revokeObjectURL(a.href);
    notify('Inbox exported as CSV.');
  }

  function navigate(next:'app'|'privacy'|'terms') {
    page=next; const path=next==='app'?'/':`/${next}`; history.pushState({},'',path); window.scrollTo({top:0,behavior:'smooth'});
  }

  async function handleLicense() {
    const params=new URLSearchParams(location.search); const returned=params.get('license');
    if (returned) { localStorage.setItem(licenseKey,returned); params.delete('license'); history.replaceState({},'',`${location.pathname}${params.size?'?'+params:''}`); }
    const token=returned || localStorage.getItem(licenseKey); if (!token) return;
    const cached=JSON.parse(localStorage.getItem(verdictKey)||'null');
    if (cached?.valid) licensed=true;
    if (!cached || Date.now()-cached.checkedAt > 86400000 || returned) await verifyLicense(token);
  }

  async function verifyLicense(token:string) {
    verifying=true; licenseNotice='';
    try {
      const response=await fetch(`${billingBase}/api/v1/products/${SLUG}/verify?license=${encodeURIComponent(token)}`);
      const verdict=await response.json(); licensed=Boolean(verdict.valid); localStorage.setItem(verdictKey,JSON.stringify({...verdict,checkedAt:Date.now()}));
      if (!verdict.valid) licenseNotice='This license is no longer active. Free monitoring remains available.';
      else notify('Pro license active on this device.');
    } catch { licenseNotice='License could not be rechecked while offline. Your cached access is unchanged.'; }
    finally { verifying=false; }
  }

  function restoreLicense(event:SubmitEvent) {
    event.preventDefault(); const token=licenseInput.trim(); if (!token) return;
    localStorage.setItem(licenseKey,token); verifyLicense(token);
  }

  function formatTime(value:string|null) {
    if (!value) return 'Not checked yet'; const date=new Date(value); return date.toLocaleString(undefined,{dateStyle:'medium',timeStyle:'short'});
  }
</script>

<a class="skip-link" href="#main">Skip to main content</a>
<header class="site-header">
  <button class="brand" on:click={() => navigate('app')} aria-label="Change Diff Inbox home">
    <span class="brand-mark" aria-hidden="true"><i></i><i></i><i></i><b></b></span>
    <span>Change Diff <em>Inbox</em></span>
  </button>
  {#if page === 'app'}
    <nav aria-label="Main navigation">
      <button class:active={tab==='inbox'} on:click={()=>tab='inbox'}>Inbox {#if stats.unread}<span class="count">{stats.unread}</span>{/if}</button>
      <button class:active={tab==='sources'} on:click={()=>tab='sources'}>Sources</button>
      <button class:active={tab==='pro'} on:click={()=>tab='pro'}>Pro {#if licensed}<span class="licensed-dot" title="Licensed">✓</span>{/if}</button>
    </nav>
    <button class="primary compact" on:click={openNew}><span aria-hidden="true">＋</span> Add source</button>
  {:else}
    <button class="text-button" on:click={()=>navigate('app')}>← Back to inbox</button>
  {/if}
</header>

{#if !online}<div class="offline" role="status">Offline — showing the last loaded inbox. Checks will resume when this device reconnects.</div>{/if}

<main id="main">
{#if page === 'privacy'}
  <article class="legal">
    <p class="eyebrow">Legal / plain language</p><h1>Privacy</h1><p class="lede">Your monitored content stays in this deployment. We do not sell, profile, or track you.</p>
    <h2>What is stored</h2><p>Change Diff Inbox stores source names, public URLs, CSS selectors, extracted text snapshots, review decisions, and check errors in its local SQLite database. If you add a Pro license, its token and daily verification result are stored in your browser.</p>
    <h2>What leaves the service</h2><p>The server requests the public pages you ask it to monitor and each site’s robots.txt. License verification sends only the license token to Sociobot’s billing API. There are no analytics, advertising cookies, third-party fonts, or tracking scripts.</p>
    <h2>Control and deletion</h2><p>Removing a source deletes its snapshots and changes from this deployment. Clearing site data removes the license token on this browser. Your deployment operator controls database retention and backups.</p>
    <h2>Contact</h2><p>For a hosted deployment, contact the operator shown in its service documentation. Last updated 27 August 2026.</p>
  </article>
{:else if page === 'terms'}
  <article class="legal">
    <p class="eyebrow">Legal / plain language</p><h1>Terms</h1><p class="lede">Use Change Diff Inbox to monitor public material you are allowed to access—never to bypass controls.</p>
    <h2>Acceptable use</h2><p>You are responsible for the URLs and selectors you configure. Do not use the service to evade authentication, access private networks, overwhelm sites, or violate applicable terms. The product respects robots.txt, limits fetch size, and enforces a minimum check interval.</p>
    <h2>Service and accuracy</h2><p>Extraction and semantic thresholds reduce noise but cannot guarantee that every important change is detected. Checks may fail because a site moves, blocks automated requests, or changes its markup. Review important provider changes at their source.</p>
    <h2>Pro purchase</h2><p>Pro is a $39 one-time license for this product’s enhanced scheduling controls. Sociobot/Dodo is the merchant of record and handles checkout and refunds. A refund or revocation deactivates the license. The free tier and data export remain available.</p>
    <h2>Warranty</h2><p>The software is provided “as is,” subject to the MIT License. Last updated 27 August 2026.</p>
  </article>
{:else}
  {#if loadError}
    <section class="state-panel" role="alert"><span class="state-icon">!</span><h1>The inbox could not connect</h1><p>{loadError}. Check that the server is running, then try again.</p><button class="primary" on:click={loadAll}>Try again</button></section>
  {:else if loading}
    <section class="loading-state" aria-live="polite"><div class="pulse-line wide"></div><div class="pulse-line"></div><div class="pulse-card"></div><span>Opening your observation desk…</span></section>
  {:else if tab === 'inbox'}
    {#if sources.length === 0}
      <section class="hero">
        <div class="hero-copy"><p class="eyebrow"><span></span> Semantic change monitoring</p><h1>Watch the parts that matter. Ignore the rest.</h1><p class="lede">Select a docs section, pricing table, code block, or structured record. We turn meaningful changes into one quiet, reviewable inbox.</p><div class="hero-actions"><button class="primary" on:click={openNew}>Add your first source <span aria-hidden="true">→</span></button><span>Self-hosted core · no tracking</span></div><ul class="signal-list"><li><b>01</b><span><strong>Extract</strong> one stable semantic region</span></li><li><b>02</b><span><strong>Filter</strong> changes below your noise threshold</span></li><li><b>03</b><span><strong>Review</strong> and score every alert</span></li></ul></div>
        <figure class="hero-art"><picture><source type="image/avif" srcset="/assets/hero-640.avif 640w, /assets/hero-960.avif 960w" sizes="(max-width: 800px) 92vw, 52vw"><source type="image/webp" srcset="/assets/hero-640.webp 640w, /assets/hero-960.webp 960w" sizes="(max-width: 800px) 92vw, 52vw"><img src="/assets/hero-960.webp" width="960" height="640" alt="Translucent data sheets pass beneath a glass lens that isolates one amber change" fetchpriority="high" decoding="async"></picture><figcaption><span>Observe / isolate / decide</span><b>Semantic signal</b></figcaption></figure>
      </section>
    {:else}
      <section class="workspace-head"><div><p class="eyebrow"><span></span> Observation desk</p><h1>Changes worth your attention.</h1><p>{stats.unread ? `${stats.unread} item${stats.unread===1?'':'s'} waiting for review.` : 'You are caught up. Watched sources are still checking quietly.'}</p></div><div class="score"><strong>{stats.rated ? `${usefulness}%` : '—'}</strong><span>useful alerts</span><small>{stats.rated} rated</small></div></section>
      <section class="toolbar" aria-label="Inbox filters"><label>Review state<select bind:value={stateFilter}><option value="all">All changes</option><option value="unread">Unread</option><option value="reviewed">Reviewed</option><option value="archived">Archived</option></select></label><label>Source<select bind:value={sourceFilter}><option value="all">Every source</option>{#each sources as source}<option value={source.id}>{source.name}</option>{/each}</select></label><button class="secondary export" on:click={exportCsv}>↓ Export CSV</button></section>
      {#if displayedChanges.length === 0}
        <section class="quiet-state"><div class="radar" aria-hidden="true"><i></i><b></b></div><h2>{changes.length ? 'No changes match these filters' : 'Baseline set. Listening for signal.'}</h2><p>{changes.length ? 'Try a different review state or source.' : 'Run a source check after its page changes, or let the scheduler watch in the background.'}</p><button class="secondary" on:click={()=>tab='sources'}>View monitored sources</button></section>
      {:else}
        <section class="change-list" aria-label="Detected changes">
          {#each displayedChanges as change}
            <article class:unread={change.review_state==='unread'} class="change-card">
              <button class="change-toggle" aria-expanded={expanded===change.id} on:click={()=>expanded=expanded===change.id?'':change.id}>
                <span class="change-signal" aria-hidden="true"></span><span class="change-main"><span class="change-meta"><b>{change.source_name}</b><time datetime={change.created_at}>{formatTime(change.created_at)}</time><em>{(change.change_ratio*100).toFixed(1)}% changed</em></span><strong>{compact(change.summary,220)}</strong><code>{change.selector}</code></span><span class="chevron" aria-hidden="true">⌄</span>
              </button>
              {#if expanded===change.id}
                {@const pieces=diffWords(change.previous_text,change.current_text)}
                <div class="change-detail"><div class="diff-grid"><div><h3><span class="minus">−</span> Previous</h3><pre>{#each pieces.old as part}<span class:removed={part.type==='removed'}>{part.value}</span>{/each}</pre></div><div><h3><span class="plus">＋</span> Current</h3><pre>{#each pieces.next as part}<span class:added={part.type==='added'}>{part.value}</span>{/each}</pre></div></div><div class="review-bar"><a href={change.source_url} target="_blank" rel="noreferrer">Open source ↗</a><span>Was this alert useful?</span><button class:chosen={change.useful===1} on:click={()=>review(change,'reviewed',true)} aria-label="Mark this alert useful">Yes</button><button class:chosen={change.useful===0} on:click={()=>review(change,'reviewed',false)} aria-label="Mark this alert as noise">No, noise</button><button on:click={()=>review(change,'archived')}>Archive</button></div></div>
              {/if}
            </article>
          {/each}
        </section>
      {/if}
    {/if}
  {:else if tab === 'sources'}
    <section class="workspace-head"><div><p class="eyebrow"><span></span> Source registry</p><h1>Monitored sources</h1><p>Public pages only. Each check reads one semantic region and honors robots.txt.</p></div><button class="primary" on:click={openNew}>＋ Add source</button></section>
    {#if formOpen}
      <section class="source-form-wrap" aria-labelledby="form-title"><div class="form-heading"><div><p class="eyebrow">{editing?'Edit watch':'New watch'}</p><h2 id="form-title">{editing?'Tune this source':'Choose the signal'}</h2></div><button class="icon-button" on:click={()=>formOpen=false} aria-label="Close source form">×</button></div>
        <form on:submit={saveSource} novalidate><div class="field-grid"><label>Name<input id="source-name" bind:value={form.name} required minlength="2" maxlength="80" autocomplete="off"><small>e.g. Stripe API versioning</small></label><label>Public page URL<input type="url" bind:value={form.url} required placeholder="https://docs.example.com/changelog" autocomplete="url"><small>No logins, local URLs, or bypasses</small></label></div><div class="field-grid thirds"><label>Extract<select bind:value={form.extract_mode}><option value="selector">CSS-selected section</option><option value="table">Table</option><option value="code">Code block</option><option value="jsonld">JSON-LD</option></select></label><label>CSS selector<input class="mono" bind:value={form.selector} maxlength="200" placeholder="main .release-notes"><small>Leave blank for the mode default</small></label><label>Noise threshold<div class="unit-input"><input type="number" bind:value={form.threshold} min="0" max="100" step="0.5"><span>%</span></div><small>Ignore smaller changes</small></label></div><fieldset><legend>Check interval</legend><div class="intervals">{#each [{v:15,l:'15 min',pro:true},{v:60,l:'Hourly',pro:true},{v:1440,l:'Daily',pro:false},{v:10080,l:'Weekly',pro:false}] as option}<label class:locked={option.pro&&!licensed}><input type="radio" bind:group={form.interval_minutes} value={option.v} disabled={option.pro&&!licensed}><span>{option.l}{#if option.pro&&!licensed}<small>PRO</small>{/if}</span></label>{/each}</div></fieldset>{#if formError}<p class="form-error" role="alert">{formError}</p>{/if}<div class="form-actions"><button type="button" class="text-button" on:click={()=>formOpen=false}>Cancel</button><button class="primary" disabled={saving}>{saving?'Saving…':editing?'Save settings':'Add and capture baseline'}</button></div></form>
      </section>
    {/if}
    {#if sources.length===0 && !formOpen}<section class="quiet-state compact-state"><h2>No sources yet</h2><p>Add a public docs page, vendor status section, pricing table, or code block to begin.</p><button class="primary" on:click={openNew}>Add your first source</button></section>{/if}
    <section class="source-list" aria-label="Monitored sources">{#each sources as source}<article class="source-row"><span class="status-dot {source.last_status}" aria-hidden="true"></span><div class="source-main"><div><h2>{source.name}</h2><span class="status-label">{source.last_status==='new'?'Needs baseline':source.last_status==='error'?'Check failed':source.last_status==='changed'?'Change detected':source.last_status==='quiet'?'Below threshold':'Watching'}</span></div><a href={source.url} target="_blank" rel="noreferrer">{source.url}</a><p><code>{source.extract_mode==='jsonld'?'JSON-LD':source.selector||source.extract_mode}</code><span>{(source.threshold*100).toFixed(1)}% threshold</span><span>Every {source.interval_minutes<60?`${source.interval_minutes} min`:source.interval_minutes===60?'hour':source.interval_minutes===1440?'day':`${Math.round(source.interval_minutes/1440)} days`}</span><span>{formatTime(source.last_checked)}</span></p>{#if source.last_error}<div class="source-error" role="status">{source.last_error}</div>{/if}</div><div class="row-actions"><button class="secondary" on:click={()=>runCheck(source.id)} disabled={checking===source.id}>{checking===source.id?'Checking…':'Check now'}</button><button class="icon-button" on:click={()=>openEdit(source)} aria-label={`Edit ${source.name}`}>✎</button><button class="icon-button danger" on:click={()=>removeSource(source)} aria-label={`Remove ${source.name}`}>×</button></div></article>{/each}</section>
  {:else}
    <section class="pro-hero"><div><p class="eyebrow"><span></span> One-time unlock</p><h1>Turn quiet watching into a tighter feedback loop.</h1><p class="lede">Pro unlocks unlimited sources and 15-minute or hourly schedules on this deployment. The free five-source daily watcher remains yours.</p><div class="price"><strong>$39</strong><span>one time<br>one product license</span></div>{#if licensed}<div class="license-active">✓ Pro is active on this device</div>{:else}<a class="primary buy" href={buyUrl}>Buy Pro securely ↗</a>{/if}<p class="merchant">Checkout and refunds are handled by Sociobot/Dodo, the merchant of record.</p></div><div class="glass-spec"><span class="spec-label">PRO / CONTROL LAYER</span><ul><li><b>∞</b><span><strong>Unlimited sources</strong>Monitor every engineering dependency.</span></li><li><b>15</b><span><strong>Minute checks</strong>Choose 15-minute, hourly, daily, or weekly.</span></li><li><b>24h</b><span><strong>Cached verification</strong>Your free experience never waits on billing.</span></li></ul></div></section>
    <section class="restore"><div><h2>Restore a purchase</h2><p>Paste the license token from your receipt to use Pro on this device.</p></div><form on:submit={restoreLicense}><label for="license">License token</label><div><input id="license" type="password" bind:value={licenseInput} autocomplete="off" spellcheck="false"><button class="secondary" disabled={verifying}>{verifying?'Checking…':'Verify license'}</button></div>{#if licenseNotice}<p role="status">{licenseNotice}</p>{/if}</form></section>
  {/if}
{/if}
</main>

<footer><span>Change Diff Inbox · self-hostable semantic monitoring</span><nav aria-label="Legal"><button on:click={()=>navigate('privacy')}>Privacy</button><button on:click={()=>navigate('terms')}>Terms</button><a href="https://github.com/B-Divyesh/sf-change-diff-inbox" rel="noreferrer">Source ↗</a></nav><small>Hero illustration generated for this product; no stock assets.</small></footer>
{#if toast}<div class="toast" role="status">{toast}</div>{/if}
