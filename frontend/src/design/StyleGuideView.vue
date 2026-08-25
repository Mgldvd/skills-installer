<template>
  <div class="style-guide">
    <header class="style-guide__intro">
      <h1>Skills Installer — Style Guide</h1>
      <p>
        A visual inventory of the components, tokens, and patterns that already exist in this codebase. Everything
        below is the real application UI — either the actual Vue components, or (where a fragment like the footer bar
        isn't its own component) the exact existing styles reapplied. Nothing here is new, redesigned, or invented.
      </p>
      <nav class="style-guide__toc" aria-label="Sections">
        <a v-for="section in sections" :key="section.id" :href="`#${section.id}`">{{ section.title }}</a>
      </nav>
    </header>

    <!-- ============================================================= -->
    <section id="colors" class="sg-section">
      <h2>Colors</h2>
      <p class="sg-section__intro">Every color token defined in <code>tokens.scss</code>, read live from the page.</p>

      <div v-for="group in colorGroups" :key="group.title" class="sg-block">
        <h3>{{ group.title }}</h3>
        <div class="sg-swatch-grid">
          <div v-for="swatch in group.swatches" :key="swatch.cssVar" class="sg-swatch">
            <span class="sg-swatch__color" :style="{ background: `var(${swatch.cssVar})` }" aria-hidden="true" />
            <span class="sg-swatch__label">{{ swatch.label }}</span>
            <code class="sg-swatch__var">{{ swatch.cssVar }}</code>
            <code class="sg-swatch__hex">{{ tokenValues[swatch.cssVar] }}</code>
          </div>
        </div>
      </div>
    </section>

    <!-- ============================================================= -->
    <section id="typography" class="sg-section">
      <h2>Typography</h2>
      <p class="sg-section__intro">The font-size scale, line-heights, and base element defaults from the tokens and global styles.</p>

      <div class="sg-block">
        <h3>Font-size scale</h3>
        <div class="sg-type-scale">
          <div v-for="size in fontSizeTokens" :key="size.cssVar" class="sg-type-scale__row">
            <span class="sg-type-scale__sample" :style="{ fontSize: `var(${size.cssVar})` }">Install Selected</span>
            <code>{{ size.cssVar }} — {{ tokenValues[size.cssVar] }}</code>
          </div>
        </div>
      </div>

      <div class="sg-block">
        <h3>Base elements &amp; roles</h3>
        <div class="sg-stack">
          <h1 class="sg-demo-heading">Heading level 1</h1>
          <h2 class="sg-demo-heading">Heading level 2</h2>
          <h3 class="sg-demo-heading">Heading level 3</h3>
          <p>
            Paragraph text uses <code>--font-family-base</code> at <code>--font-size-md</code> with
            <code>--line-height-normal</code> — the same defaults every dialog and card inherits from
            <code>base.scss</code>.
          </p>
          <p :style="{ fontSize: 'var(--font-size-xs)', color: 'var(--text-muted)', textTransform: 'uppercase', letterSpacing: '0.06em' }">
            Eyebrow / label style (as in AppHeader's "Install to" label)
          </p>
          <p :style="{ fontSize: 'var(--font-size-xs)', color: 'var(--text-secondary)' }">
            Caption / help text style (as in dialog sub-headings)
          </p>
          <a href="#colors" class="sg-demo-link">A default inline link</a>
        </div>
      </div>

      <div class="sg-block">
        <h3>Line height</h3>
        <div class="sg-row">
          <p class="sg-line-height-demo" :style="{ lineHeight: 'var(--line-height-tight)' }">
            Tight line-height (1.25) — used for card and dialog titles that need to stay compact across two lines.
          </p>
          <p class="sg-line-height-demo" :style="{ lineHeight: 'var(--line-height-normal)' }">
            Normal line-height (1.5) — the default for body copy and descriptions throughout the app.
          </p>
        </div>
      </div>
    </section>

    <!-- ============================================================= -->
    <section id="spacing-radius-shadow" class="sg-section">
      <h2>Spacing, Radius, Shadow &amp; Motion</h2>
      <p class="sg-section__intro">The remaining layout tokens: spacing rhythm, corner radii, elevation, and transition durations.</p>

      <div class="sg-block">
        <h3>Spacing scale</h3>
        <div class="sg-stack">
          <div v-for="space in spacingTokens" :key="space.cssVar" class="sg-spacing-row">
            <code>{{ space.cssVar }} — {{ tokenValues[space.cssVar] }}</code>
            <span class="sg-spacing-row__bar" :style="{ width: `var(${space.cssVar})` }" />
          </div>
        </div>
      </div>

      <div class="sg-row">
        <div class="sg-block">
          <h3>Radius</h3>
          <div class="sg-row">
            <div v-for="radius in radiusTokens" :key="radius.cssVar" class="sg-token-box">
              <span class="sg-token-box__preview" :style="{ borderRadius: `var(${radius.cssVar})` }" />
              <code>{{ radius.cssVar }} — {{ tokenValues[radius.cssVar] }}</code>
            </div>
          </div>
        </div>

        <div class="sg-block">
          <h3>Shadow</h3>
          <div class="sg-row">
            <div v-for="shadow in shadowTokens" :key="shadow.cssVar" class="sg-token-box">
              <span class="sg-token-box__preview" :style="{ boxShadow: `var(${shadow.cssVar})` }" />
              <code>{{ shadow.cssVar }}</code>
            </div>
          </div>
        </div>
      </div>

      <div class="sg-block">
        <h3>Motion</h3>
        <div class="sg-row">
          <div v-for="motion in motionTokens" :key="motion.cssVar" class="sg-motion-demo">
            <span class="sg-motion-demo__box" :style="{ transitionDuration: `var(${motion.cssVar})` }" />
            <code>{{ motion.cssVar }} — {{ tokenValues[motion.cssVar] }}</code>
          </div>
        </div>
        <p class="sg-hint">Hover the boxes above — they use the app's actual <code>--transition-fast</code> / <code>--transition-base</code> durations.</p>
      </div>
    </section>

    <!-- ============================================================= -->
    <section id="icons" class="sg-section">
      <h2>Icons &amp; Basic Visual Elements</h2>
      <p class="sg-section__intro">The reusable icon components. Every other icon in the app is an inline SVG local to its own component — those appear in context below.</p>

      <div class="sg-block">
        <h3>AgentIcon</h3>
        <div class="sg-icon-grid">
          <div v-for="id in agentIconDemoIds" :key="id" class="sg-icon-cell">
            <span class="sg-icon-cell__frame"><AgentIcon :agent-id="id" /></span>
            <span>{{ id === "custom-agent" ? "Unknown id (fallback letter)" : agentLabel(id) }}</span>
          </div>
        </div>
      </div>

      <div class="sg-block">
        <h3>SourceIcon</h3>
        <div class="sg-icon-grid">
          <div class="sg-icon-cell">
            <span class="sg-icon-cell__frame"><SourceIcon local /></span>
            <span>Local</span>
          </div>
          <div class="sg-icon-cell">
            <span class="sg-icon-cell__frame"><SourceIcon :local="false" /></span>
            <span>Remote</span>
          </div>
          <div class="sg-icon-cell">
            <span class="sg-icon-cell__frame"><SourceIcon local decorative /></span>
            <span>Decorative (no label)</span>
          </div>
        </div>
      </div>

      <div class="sg-block">
        <h3>Utility classes</h3>
        <div class="sg-stack">
          <p><code>.truncate</code> keeps long text on one line with an ellipsis:</p>
          <p class="truncate sg-truncate-demo">
            This is a deliberately long line of text that demonstrates the .truncate utility class from utilities.scss
          </p>
          <p>
            <code>.scroll-x</code> adds a horizontally scrollable overflow container (used by GroupEditBar and the
            install output log). <code>.visually-hidden</code> hides content visually while keeping it available to
            screen readers — used throughout for accessible labels like "Sort by".
          </p>
        </div>
      </div>
    </section>

    <!-- ============================================================= -->
    <section id="buttons" class="sg-section">
      <h2>Buttons</h2>
      <p class="sg-section__intro">
        There is no single shared Button component — every dialog builds its buttons from the same
        <code>dialog-base.scss</code> mixins. The one standalone button component is CloseButton; the rest of the
        button treatments (primary / danger / segmented / toolbar) are shown in the sections below, in the real
        components that own them.
      </p>
      <div class="sg-block">
        <h3>CloseButton</h3>
        <div class="sg-row sg-row--center">
          <CloseButton />
          <CloseButton aria-label="Close description" />
        </div>
      </div>
    </section>

    <!-- ============================================================= -->
    <section id="badges" class="sg-section">
      <h2>Badges, Chips &amp; Tags</h2>
      <p class="sg-section__intro">Group and Pack identifiers used throughout the catalog.</p>

      <div class="sg-block">
        <h3>GroupBadge</h3>
        <div class="sg-row sg-row--center">
          <GroupBadge :name="mockGroups[0].name" :color="mockGroups[0].color" size="md" />
          <GroupBadge :name="mockGroups[1].name" :color="mockGroups[1].color" size="sm" />
          <GroupBadge :name="mockGroups[2].name" :color="mockGroups[2].color" size="md" />
        </div>
      </div>

      <div class="sg-block">
        <h3>PackBadge</h3>
        <div class="sg-badge-grid">
          <div v-for="variant in packBadgeVariants" :key="variant.label" class="sg-badge-grid__cell">
            <PackBadge v-bind="variant.props" />
            <span>{{ variant.label }}</span>
          </div>
          <div class="sg-badge-grid__cell">
            <PackBadge :name="mockTags[1].name" :color="mockTags[1].color" interactive>
              <template #trailing>
                <span class="sg-badge-count-demo">3</span>
              </template>
            </PackBadge>
            <span>With trailing count (Pack select)</span>
          </div>
          <div class="sg-badge-grid__cell">
            <PackBadge :name="mockTags[2].name" :color="mockTags[2].color" interactive selected>
              <template #trailing>
                <svg class="sg-badge-edit-demo" viewBox="0 0 14 14" aria-hidden="true">
                  <path d="M2.5 10.4V12h1.6l6.6-6.6-1.6-1.6-6.6 6.6Zm7.4-7.4 1-1 1.6 1.6-1 1L9.9 3Z" fill="currentColor" />
                </svg>
              </template>
            </PackBadge>
            <span>With trailing edit icon (Packs catalog)</span>
          </div>
        </div>
      </div>
    </section>

    <!-- ============================================================= -->
    <section id="forms" class="sg-section">
      <h2>Form Controls</h2>
      <p class="sg-section__intro">
        Checkboxes and the toggle switch are the app's global, always-available controls. Text inputs, textareas,
        selects, and radio buttons are dialog-scoped (built from shared mixins) — see them live in the Overlays
        section below.
      </p>

      <div class="sg-block">
        <h3>Checkbox</h3>
        <div class="sg-row sg-row--center">
          <label class="sg-inline-label"><input type="checkbox" /> Unchecked</label>
          <label class="sg-inline-label"><input type="checkbox" checked /> Checked</label>
          <label class="sg-inline-label"><input type="checkbox" disabled /> Disabled</label>
          <label class="sg-inline-label"><input type="checkbox" checked disabled /> Checked + disabled</label>
        </div>
      </div>

      <div class="sg-block">
        <h3>Toggle switch</h3>
        <div class="app-shell sg-flat-shell">
          <label class="app-shell__toggle">
            <input type="checkbox" class="app-shell__toggle-input" />
            <span class="app-shell__toggle-track" aria-hidden="true"><span class="app-shell__toggle-thumb" /></span>
            <span class="app-shell__toggle-label">Off</span>
          </label>
          <label class="app-shell__toggle">
            <input type="checkbox" class="app-shell__toggle-input" checked />
            <span class="app-shell__toggle-track" aria-hidden="true"><span class="app-shell__toggle-thumb" /></span>
            <span class="app-shell__toggle-label">Skip confirmation</span>
          </label>
        </div>
      </div>

      <div class="sg-block">
        <h3>ColorPalettePicker</h3>
        <div class="sg-row sg-row--center">
          <div>
            <ColorPalettePicker v-model="pickerPreset" aria-label="Preset color" />
            <p class="sg-hint">Preset selected</p>
          </div>
          <div>
            <ColorPalettePicker v-model="pickerCustom" aria-label="Custom color" />
            <p class="sg-hint">Custom color selected</p>
          </div>
        </div>
      </div>

      <div class="sg-block">
        <h3>FontScaleControl</h3>
        <div class="sg-row sg-row--center">
          <div>
            <FontScaleControl v-model="fontScaleCompact" variant="compact" />
            <p class="sg-hint">Compact variant</p>
          </div>
          <div>
            <FontScaleControl v-model="fontScaleFull" variant="full" />
            <p class="sg-hint">Full variant</p>
          </div>
        </div>
      </div>

      <div class="sg-block">
        <h3>GroupSelector</h3>
        <GroupSelector v-model="selectedGroupId" :groups="mockGroups" @create-new="() => {}" />
      </div>
    </section>

    <!-- ============================================================= -->
    <section id="validation-states" class="sg-section">
      <h2>Validation &amp; States</h2>
      <p class="sg-section__intro">Error, disabled, and focus treatments as they actually occur in the app.</p>

      <div class="sg-block">
        <h3>Inline validation error</h3>
        <button type="button" class="sg-trigger" @click="agentsEmptyOpen = true">
          Open Agents dialog with nothing selected
        </button>
        <button type="button" class="sg-trigger" @click="groupEditorErrorOpen = true">
          Open Group Editor with a name-conflict error
        </button>
      </div>
      <AgentsDialog
        v-model:open="agentsEmptyOpen"
        v-model="agentsEmptySelection"
        :agent-order="[]"
        :agent-scopes="{}"
      />
      <GroupEditor
        v-model:open="groupEditorErrorOpen"
        mode="create"
        error-message="A group named &quot;Frontend&quot; already exists."
        @submit="() => (groupEditorErrorOpen = false)"
      />

      <div class="sg-block">
        <h3>Disabled state</h3>
        <p>
          Disabled controls appear throughout: the App footer buttons, PackBadge (see Badges above), form fields, and
          a SkillCard whose selection surface is disabled once fully installed (see Cards &amp; Panels below).
        </p>
      </div>

      <div class="sg-block">
        <h3>Focus-visible outline</h3>
        <p class="sg-hint">
          Static preview of the app's keyboard-focus treatment (only ever shown after Tab navigation, via
          <code>body.is-keyboard-navigation :focus-visible</code>):
        </p>
        <span class="sg-focus-demo">Focused element</span>
      </div>
    </section>

    <!-- ============================================================= -->
    <section id="filtering-toolbar" class="sg-section">
      <h2>Filtering &amp; Toolbar</h2>
      <p class="sg-section__intro">SkillToolbar (Pack selection) and SkillFilterBar (search, sort, view, source filters) — fully interactive, live component instances.</p>

      <div class="sg-block">
        <h3>SkillToolbar</h3>
        <SkillToolbar
          :tags="mockTags"
          :skills="mockSkills"
          :selected-ids="toolbarSelectedIds"
          :needs-agents-count="2"
          @toggle-tag="() => {}"
          @reorder-tags="() => {}"
          @clear-selection="toolbarSelectedIds = []"
          @select-missing="() => {}"
          @open-packs="() => {}"
        />
      </div>

      <div class="sg-block">
        <h3>SkillFilterBar</h3>
        <SkillFilterBar
          :query="filterQuery"
          :sort-by="filterSort"
          :view="filterView"
          :source-filter="filterSourceFilter"
          :pack-filter="filterPack"
          :tags="mockTags"
          :needs-agents-only="filterNeedsAgentsOnly"
          :needs-agents-count="2"
          @update:query="filterQuery = $event"
          @update:sort-by="filterSort = $event"
          @update:view="filterView = $event"
          @update:source-filter="filterSourceFilter = $event"
          @update:pack-filter="filterPack = $event"
          @update:needs-agents-only="filterNeedsAgentsOnly = $event"
        />
      </div>
    </section>

    <!-- ============================================================= -->
    <section id="cards-panels" class="sg-section">
      <h2>Cards &amp; Panels</h2>
      <p class="sg-section__intro">SkillCard states and the InstallProgressPanel.</p>

      <div class="sg-block">
        <h3>SkillCard states (grid mode)</h3>
        <div class="sg-card-grid">
          <div v-for="demo in skillCardDemos" :key="demo.title" class="sg-card-demo">
            <span class="sg-card-demo__label">{{ demo.title }}</span>
            <div class="sg-card-demo__frame">
              <SkillCard
                :skill="demo.skill"
                :selected="demo.selected"
                :tags="mockTags"
                :target-agents="targetAgents"
                :installing="demo.installing ?? false"
                :has-update="demo.hasUpdate ?? false"
                @toggle="() => {}"
                @edit="() => {}"
                @update="() => {}"
              />
            </div>
          </div>
        </div>
      </div>

      <div class="sg-block">
        <h3>SkillCard layout variants</h3>
        <div class="sg-row sg-row--top">
          <div>
            <p class="sg-hint">Compact</p>
            <div class="sg-card-demo__frame sg-card-demo__frame--compact">
              <SkillCard :skill="mockSkills[0]" :selected="false" :tags="mockTags" :target-agents="targetAgents" compact @toggle="() => {}" @edit="() => {}" @update="() => {}" />
            </div>
          </div>
          <div class="sg-card-demo__frame--list-wrap">
            <p class="sg-hint">List</p>
            <div class="sg-card-demo__frame sg-card-demo__frame--list">
              <SkillCard :skill="mockSkills[0]" :selected="false" :tags="mockTags" :target-agents="targetAgents" list @toggle="() => {}" @edit="() => {}" @update="() => {}" />
            </div>
          </div>
        </div>
      </div>

      <div class="sg-block">
        <h3>InstallProgressPanel</h3>
        <div class="sg-stack">
          <div>
            <p class="sg-hint">Installing</p>
            <InstallProgressPanel :installation="mockInstallationInstalling" :skills="mockSkills" @cancel="() => {}" @dismiss="() => {}" />
          </div>
          <div>
            <p class="sg-hint">Completed successfully</p>
            <InstallProgressPanel :installation="mockInstallationSuccess" :skills="mockSkills" @cancel="() => {}" @dismiss="() => {}" />
          </div>
          <div>
            <p class="sg-hint">Failed</p>
            <InstallProgressPanel :installation="mockInstallationError" :skills="mockSkills" @cancel="() => {}" @dismiss="() => {}" />
          </div>
        </div>
      </div>
    </section>

    <!-- ============================================================= -->
    <section id="navigation" class="sg-section">
      <h2>Navigation &amp; App Shell</h2>
      <p class="sg-section__intro">AppHeader and the footer bar (recreated here from App.scss, since the footer isn't its own component).</p>

      <div class="sg-block">
        <h3>AppHeader</h3>
        <div class="sg-stack">
          <AppHeader v-model:project-path="demoProjectPath" v-model:scope="demoScope" />
          <AppHeader project-path="" scope="global" />
        </div>
      </div>

      <div class="sg-block">
        <h3>App footer bar</h3>
        <div class="app-shell sg-flat-shell">
          <footer class="app-shell__footer">
            <div class="app-shell__footer-row">
              <div class="app-shell__footer-left">
                <button type="button" class="app-shell__footer-btn app-shell__footer-btn--add">Add Skill</button>
                <button type="button" class="app-shell__footer-btn">Projects</button>
                <button type="button" class="app-shell__footer-btn" aria-pressed="true">Preferences (pressed)</button>
                <button type="button" class="app-shell__footer-btn app-shell__footer-btn--icon">
                  <svg viewBox="0 0 24 24" aria-hidden="true">
                    <polyline points="23 4 23 10 17 10" />
                    <polyline points="1 20 1 14 7 14" />
                    <path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15" />
                  </svg>
                </button>
                <button type="button" class="app-shell__footer-btn app-shell__footer-btn--icon is-loading">
                  <svg viewBox="0 0 24 24" aria-hidden="true">
                    <polyline points="23 4 23 10 17 10" />
                    <polyline points="1 20 1 14 7 14" />
                    <path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15" />
                  </svg>
                </button>
              </div>
              <div class="app-shell__footer-actions">
                <label class="app-shell__toggle">
                  <input type="checkbox" class="app-shell__toggle-input" checked />
                  <span class="app-shell__toggle-track" aria-hidden="true"><span class="app-shell__toggle-thumb" /></span>
                  <span class="app-shell__toggle-label">Skip confirmation</span>
                </label>
                <span class="app-shell__selected-count">2 selected</span>
                <button type="button" class="app-shell__footer-btn app-shell__footer-btn--primary">Install Selected</button>
                <button type="button" class="app-shell__footer-btn app-shell__footer-btn--primary" disabled>Install Selected</button>
              </div>
            </div>
            <div class="app-shell__footer-status">
              <span class="app-shell__dependency is-ready">
                <span class="app-shell__dependency-dot" aria-hidden="true" />
                Skills CLI ready
              </span>
              <span class="app-shell__dependency is-missing">
                <span class="app-shell__dependency-dot" aria-hidden="true" />
                Skills CLI not found
              </span>
              <button type="button" class="app-shell__agents-summary">
                <span class="app-shell__manage-agents" aria-hidden="true">
                  <svg viewBox="0 0 16 16" aria-hidden="true">
                    <path d="M8 2v12M2 8h12" fill="none" stroke="currentColor" stroke-width="3.2" stroke-linecap="round" />
                  </svg>
                </span>
                Agents
                <span class="app-shell__agent-icons">
                  <AgentIcon v-for="id in targetAgents" :key="id" :agent-id="id" />
                </span>
              </button>
            </div>
          </footer>
        </div>
      </div>
    </section>

    <!-- ============================================================= -->
    <section id="lists-tables" class="sg-section">
      <h2>Lists &amp; Tables</h2>
      <p class="sg-section__intro">GroupEditBar's chip list. The Add Skill catalog table and the Packs assignment list appear inside their dialogs below.</p>

      <div class="sg-block">
        <h3>GroupEditBar</h3>
        <GroupEditBar :groups="groupChips" @edit-group="() => {}" />
      </div>
    </section>

    <!-- ============================================================= -->
    <section id="feedback" class="sg-section">
      <h2>Loaders, Toasts &amp; Empty States</h2>
      <p class="sg-section__intro">Spinners appear inline on SkillCard and the footer refresh button (see Cards and Navigation above).</p>

      <div class="sg-block">
        <h3>Toasts</h3>
        <div class="sg-row">
          <button type="button" class="sg-trigger" @click="demoToast('info')">Push info toast</button>
          <button type="button" class="sg-trigger" @click="demoToast('success')">Push success toast</button>
          <button type="button" class="sg-trigger" @click="demoToast('error')">Push error toast</button>
        </div>
        <ToastHost />
      </div>

      <div class="sg-block">
        <h3>Empty state</h3>
        <SkillGrid :skills="[]" :tags="mockTags" :selected-ids="new Set()" :target-agents="targetAgents" />
      </div>
    </section>

    <!-- ============================================================= -->
    <section id="overlays" class="sg-section">
      <h2>Modals, Dialogs &amp; Overlays</h2>
      <p class="sg-section__intro">
        Every dialog in the app is a native <code>&lt;dialog&gt;</code> opened modally, exactly as it behaves in the
        real app. Click a trigger to preview it.
      </p>

      <div class="sg-block">
        <h3>Confirmation</h3>
        <div class="sg-row">
          <button type="button" class="sg-trigger" @click="confirmOpen = true">Open ConfirmDialog</button>
          <button type="button" class="sg-trigger" @click="confirmDestructiveOpen = true">Open ConfirmDialog (destructive)</button>
          <button type="button" class="sg-trigger" @click="descriptionOpen = true">Open SkillDescriptionDialog</button>
        </div>
      </div>

      <div class="sg-block">
        <h3>Groups</h3>
        <div class="sg-row">
          <button type="button" class="sg-trigger" @click="groupEditorCreateOpen = true">Open GroupEditor (create)</button>
          <button type="button" class="sg-trigger" @click="groupEditorEditOpen = true">Open GroupEditor (edit)</button>
          <button type="button" class="sg-trigger" @click="deleteGroupEmptyOpen = true">Open DeleteGroupDialog (empty group)</button>
          <button type="button" class="sg-trigger" @click="deleteGroupWithSkillsOpen = true">Open DeleteGroupDialog (has skills)</button>
        </div>
      </div>

      <div class="sg-block">
        <h3>Catalog management</h3>
        <div class="sg-row">
          <button type="button" class="sg-trigger" @click="skillsDialogOpen = true">Open SkillsDialog</button>
          <button type="button" class="sg-trigger" @click="tagsOpen = true">Open TagsDialog</button>
          <button type="button" class="sg-trigger" @click="projectsOpen = true">Open ProjectsDialog</button>
          <button type="button" class="sg-trigger" @click="agentsDialogOpen = true">Open AgentsDialog</button>
          <button type="button" class="sg-trigger" @click="preferencesOpen = true">Open PreferencesDialog</button>
        </div>
      </div>

      <div class="sg-block">
        <h3>Skill editing &amp; installation</h3>
        <div class="sg-row">
          <button type="button" class="sg-trigger" @click="editSkillOpen = true">Open EditSkillDialog</button>
          <button type="button" class="sg-trigger" @click="installConfirmOpen = true">Open InstallConfirmDialog</button>
          <button type="button" class="sg-trigger" @click="addSkillOpen = true">Open AddSkillDialog</button>
        </div>
      </div>

      <ConfirmDialog
        v-model:open="confirmOpen"
        title="Remove Skill?"
        message="This removes the Skill from your current selection. You can add it again later."
        confirm-label="Remove"
      />
      <ConfirmDialog
        v-model:open="confirmDestructiveOpen"
        title="Delete Pack?"
        message="This Pack will be permanently removed. Skills assigned to it are not deleted."
        confirm-label="Delete Pack"
        destructive
      />
      <SkillDescriptionDialog v-model:open="descriptionOpen" :skill="mockSkills[3]" />

      <GroupEditor v-model:open="groupEditorCreateOpen" mode="create" @submit="() => (groupEditorCreateOpen = false)" />
      <GroupEditor v-model:open="groupEditorEditOpen" mode="edit" :group="mockGroups[0]" @submit="() => (groupEditorEditOpen = false)" />
      <DeleteGroupDialog v-model:open="deleteGroupEmptyOpen" :group="emptyGroup" :skill-count="0" :other-groups="mockGroups" />
      <DeleteGroupDialog
        v-model:open="deleteGroupWithSkillsOpen"
        :group="mockGroups[1]"
        :skill-count="backendSkillCount"
        :other-groups="otherGroupsForBackend"
      />

      <SkillsDialog v-model:open="skillsDialogOpen" :skills="mockSkills" local-source-path="~/.control/skill" />
      <TagsDialog v-model:open="tagsOpen" :skills="mockSkills" :tags="mockTags" />
      <ProjectsDialog v-model:open="projectsOpen" :projects="mockProjects" :installed-skill-names="['code-review', 'test-driven-development']" />
      <AgentsDialog
        v-model:open="agentsDialogOpen"
        v-model="agentsSelection"
        :agent-order="[]"
        :agent-scopes="{}"
      />
      <PreferencesDialog v-model:open="preferencesOpen" :preferences="preferencesDemo" @update="handlePreferencesUpdate" />

      <EditSkillDialog v-model:open="editSkillOpen" :skill="mockSkills[1]" :tags="mockTags" />
      <InstallConfirmDialog v-model:open="installConfirmOpen" :skills="mockSkills.slice(0, 3)" :options="mockInstallOptions" project-path="/home/dev/acme/marketing-site" />
      <AddSkillDialog v-model:open="addSkillOpen" :groups="mockGroups" :skills="mockSkills" default-group-id="frontend" />
    </section>

    <!-- ============================================================= -->
    <section id="composite" class="sg-section">
      <h2>Composite: Catalog Screen</h2>
      <p class="sg-section__intro">
        The app's primary screen, assembled from the components above: AppHeader, SkillToolbar, SkillFilterBar, and
        SkillGrid, sharing the same live filter/selection state.
      </p>
      <div class="sg-composite-frame">
        <AppHeader v-model:project-path="demoProjectPath" v-model:scope="demoScope" />
        <SkillToolbar
          :tags="mockTags"
          :skills="mockSkills"
          :selected-ids="toolbarSelectedIds"
          :needs-agents-count="2"
          @toggle-tag="() => {}"
          @reorder-tags="() => {}"
          @clear-selection="toolbarSelectedIds = []"
          @select-missing="() => {}"
          @open-packs="() => {}"
        />
        <SkillFilterBar
          :query="filterQuery"
          :sort-by="filterSort"
          :view="filterView"
          :source-filter="filterSourceFilter"
          :pack-filter="filterPack"
          :tags="mockTags"
          :needs-agents-only="filterNeedsAgentsOnly"
          :needs-agents-count="2"
          @update:query="filterQuery = $event"
          @update:sort-by="filterSort = $event"
          @update:view="filterView = $event"
          @update:source-filter="filterSourceFilter = $event"
          @update:pack-filter="filterPack = $event"
          @update:needs-agents-only="filterNeedsAgentsOnly = $event"
        />
        <SkillGrid
          :skills="filteredSkills"
          :tags="mockTags"
          :selected-ids="selectedIdSet"
          :compact="gridCompact"
          :view="gridView"
          :skills-with-updates="mockSkillsWithUpdates"
          :target-agents="targetAgents"
          @toggle="toggleFilterSelection"
          @edit="() => {}"
          @update="() => {}"
        />
      </div>
    </section>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref } from "vue";

import AddSkillDialog from "../components/AddSkillDialog/AddSkillDialog.vue";
import AgentIcon from "../components/AgentIcon/AgentIcon.vue";
import AgentsDialog from "../components/AgentsDialog/AgentsDialog.vue";
import AppHeader from "../components/AppHeader/AppHeader.vue";
import CloseButton from "../components/CloseButton/CloseButton.vue";
import ColorPalettePicker from "../components/ColorPalettePicker/ColorPalettePicker.vue";
import ConfirmDialog from "../components/ConfirmDialog/ConfirmDialog.vue";
import DeleteGroupDialog from "../components/DeleteGroupDialog/DeleteGroupDialog.vue";
import EditSkillDialog from "../components/EditSkillDialog/EditSkillDialog.vue";
import FontScaleControl from "../components/FontScaleControl/FontScaleControl.vue";
import GroupBadge from "../components/GroupBadge/GroupBadge.vue";
import GroupEditBar from "../components/GroupEditBar/GroupEditBar.vue";
import GroupEditor from "../components/GroupEditor/GroupEditor.vue";
import GroupSelector from "../components/GroupSelector/GroupSelector.vue";
import InstallConfirmDialog from "../components/InstallConfirmDialog/InstallConfirmDialog.vue";
import InstallProgressPanel from "../components/InstallProgressPanel/InstallProgressPanel.vue";
import PackBadge from "../components/PackBadge/PackBadge.vue";
import PreferencesDialog from "../components/PreferencesDialog/PreferencesDialog.vue";
import ProjectsDialog from "../components/ProjectsDialog/ProjectsDialog.vue";
import SkillCard from "../components/SkillCard/SkillCard.vue";
import SkillDescriptionDialog from "../components/SkillDescriptionDialog/SkillDescriptionDialog.vue";
import SkillFilterBar from "../components/SkillFilterBar/SkillFilterBar.vue";
import SkillGrid from "../components/SkillGrid/SkillGrid.vue";
import SkillsDialog from "../components/SkillsDialog/SkillsDialog.vue";
import SkillToolbar from "../components/SkillToolbar/SkillToolbar.vue";
import SourceIcon from "../components/SourceIcon/SourceIcon.vue";
import TagsDialog from "../components/TagsDialog/TagsDialog.vue";
import ToastHost from "../components/ToastHost/ToastHost.vue";
import { useToasts } from "../composables/useToasts";
import { SUPPORTED_AGENTS, type InstallScope, type Skill, type SkillGroup, type UiPreferences } from "../types";
import { agentLabel } from "../utils/agents";
import {
  mockGroups,
  mockInstallOptions,
  mockInstallationError,
  mockInstallationInstalling,
  mockInstallationSuccess,
  mockPreferences,
  mockProjects,
  mockSkills,
  mockSkillsWithUpdates,
  mockTags,
  targetAgents,
} from "./mockData";

const sections = [
  { id: "colors", title: "Colors" },
  { id: "typography", title: "Typography" },
  { id: "spacing-radius-shadow", title: "Spacing, Radius, Shadow & Motion" },
  { id: "icons", title: "Icons & Utilities" },
  { id: "buttons", title: "Buttons" },
  { id: "badges", title: "Badges, Chips & Tags" },
  { id: "forms", title: "Form Controls" },
  { id: "validation-states", title: "Validation & States" },
  { id: "filtering-toolbar", title: "Filtering & Toolbar" },
  { id: "cards-panels", title: "Cards & Panels" },
  { id: "navigation", title: "Navigation & App Shell" },
  { id: "lists-tables", title: "Lists & Tables" },
  { id: "feedback", title: "Loaders, Toasts & Empty States" },
  { id: "overlays", title: "Modals & Overlays" },
  { id: "composite", title: "Composite: Catalog Screen" },
];

// --- Foundations: tokens ---------------------------------------------------
interface TokenRef {
  label: string;
  cssVar: string;
}

const backgroundSwatches: TokenRef[] = [
  { label: "App background", cssVar: "--bg-app" },
  { label: "Panel", cssVar: "--bg-panel" },
  { label: "Card", cssVar: "--bg-card" },
  { label: "Card hover", cssVar: "--bg-card-hover" },
  { label: "Muted", cssVar: "--bg-muted" },
];
const textSwatches: TokenRef[] = [
  { label: "Primary text", cssVar: "--text-primary" },
  { label: "Secondary text", cssVar: "--text-secondary" },
  { label: "Muted text", cssVar: "--text-muted" },
];
const borderSwatches: TokenRef[] = [
  { label: "Soft border", cssVar: "--border-soft" },
  { label: "Strong border", cssVar: "--border-strong" },
];
const accentSwatches: TokenRef[] = [
  { label: "Accent", cssVar: "--accent" },
  { label: "Accent hover", cssVar: "--accent-hover" },
  { label: "Accent soft", cssVar: "--accent-soft" },
  { label: "Accent soft strong", cssVar: "--accent-soft-strong" },
];
const semanticSwatches: TokenRef[] = [
  { label: "Success", cssVar: "--success" },
  { label: "Success soft", cssVar: "--success-soft" },
  { label: "Warning", cssVar: "--warning" },
  { label: "Warning soft", cssVar: "--warning-soft" },
  { label: "Danger", cssVar: "--danger" },
  { label: "Danger soft", cssVar: "--danger-soft" },
  { label: "Info", cssVar: "--info" },
  { label: "Info soft", cssVar: "--info-soft" },
];
const paletteSwatches: TokenRef[] = [
  { label: "Pink", cssVar: "--palette-pink" },
  { label: "Coral", cssVar: "--palette-coral" },
  { label: "Orange", cssVar: "--palette-orange" },
  { label: "Amber", cssVar: "--palette-amber" },
  { label: "Green", cssVar: "--palette-green" },
  { label: "Teal", cssVar: "--palette-teal" },
  { label: "Cyan", cssVar: "--palette-cyan" },
  { label: "Blue", cssVar: "--palette-blue" },
  { label: "Indigo", cssVar: "--palette-indigo" },
  { label: "Violet", cssVar: "--palette-violet" },
];
const colorGroups = [
  { title: "Backgrounds", swatches: backgroundSwatches },
  { title: "Text", swatches: textSwatches },
  { title: "Borders", swatches: borderSwatches },
  { title: "Accent", swatches: accentSwatches },
  { title: "Semantic", swatches: semanticSwatches },
  { title: "Curated group & pack palette", swatches: paletteSwatches },
];

const radiusTokens: TokenRef[] = [
  { label: "Small", cssVar: "--radius-sm" },
  { label: "Medium", cssVar: "--radius-md" },
  { label: "Large", cssVar: "--radius-lg" },
];
const shadowTokens: TokenRef[] = [
  { label: "Small", cssVar: "--shadow-sm" },
  { label: "Medium", cssVar: "--shadow-md" },
];
const spacingTokens: TokenRef[] = [
  { label: "2xs", cssVar: "--space-2xs" },
  { label: "xs", cssVar: "--space-xs" },
  { label: "sm", cssVar: "--space-sm" },
  { label: "md", cssVar: "--space-md" },
  { label: "lg", cssVar: "--space-lg" },
  { label: "xl", cssVar: "--space-xl" },
];
const motionTokens: TokenRef[] = [
  { label: "Fast", cssVar: "--transition-fast" },
  { label: "Base", cssVar: "--transition-base" },
];
const fontSizeTokens: TokenRef[] = [
  { label: "xs", cssVar: "--font-size-xs" },
  { label: "sm", cssVar: "--font-size-sm" },
  { label: "md", cssVar: "--font-size-md" },
  { label: "lg", cssVar: "--font-size-lg" },
  { label: "xl", cssVar: "--font-size-xl" },
  { label: "2xl", cssVar: "--font-size-2xl" },
];

const allTokenVars = [
  ...backgroundSwatches,
  ...textSwatches,
  ...borderSwatches,
  ...accentSwatches,
  ...semanticSwatches,
  ...paletteSwatches,
  ...radiusTokens,
  ...shadowTokens,
  ...spacingTokens,
  ...motionTokens,
  ...fontSizeTokens,
].map((t) => t.cssVar);

const tokenValues = reactive<Record<string, string>>({});
onMounted(() => {
  const styles = window.getComputedStyle(document.documentElement);
  for (const name of allTokenVars) tokenValues[name] = styles.getPropertyValue(name).trim();
});

// --- Icons -------------------------------------------------------------
const agentIconDemoIds = [...SUPPORTED_AGENTS.map((agent) => agent.id), "custom-agent"];

// --- Badges --------------------------------------------------------------
const packBadgeVariants = [
  { label: "Static (non-interactive)", props: { name: mockTags[2].name, color: mockTags[2].color } },
  { label: "Interactive, default", props: { name: mockTags[0].name, color: mockTags[0].color, interactive: true } },
  { label: "Selected", props: { name: mockTags[0].name, color: mockTags[0].color, interactive: true, selected: true } },
  { label: "Partially selected", props: { name: mockTags[1].name, color: mockTags[1].color, interactive: true, partial: true } },
  { label: "Muted (unassigned)", props: { name: mockTags[1].name, color: mockTags[1].color, interactive: true, muted: true } },
  { label: "Compact", props: { name: mockTags[2].name, color: mockTags[2].color, interactive: true, compact: true, selected: true } },
  { label: "Disabled", props: { name: mockTags[3].name, color: mockTags[3].color, interactive: true, disabled: true } },
];

// --- Forms -----------------------------------------------------------------
const pickerPreset = ref(mockPreferences.accent);
const pickerCustom = ref("#123ABC");
const fontScaleCompact = ref(1);
const fontScaleFull = ref(1.1);
const selectedGroupId = ref<string | null>(mockGroups[0].id);

// --- Validation & states ----------------------------------------------------
const agentsEmptyOpen = ref(false);
const agentsEmptySelection = ref<string[]>([]);
const groupEditorErrorOpen = ref(false);

// --- Filtering & toolbar / composite ---------------------------------------
const filterQuery = ref("");
const filterSort = ref<"name" | "pack" | "local" | "remote">("name");
const filterView = ref<"grid" | "compact" | "list">("grid");
const filterSourceFilter = ref<"all" | "local" | "remote">("all");
const filterPack = ref<string | null>(null);
const filterNeedsAgentsOnly = ref(false);
const toolbarSelectedIds = ref<string[]>([mockSkills[0].id]);

const filteredSkills = computed<Skill[]>(() => {
  const q = filterQuery.value.trim().toLowerCase();
  return mockSkills.filter((skill) => !q || skill.displayName.toLowerCase().includes(q));
});
const gridCompact = computed(() => filterView.value === "compact");
const gridView = computed<"grid" | "list">(() => (filterView.value === "list" ? "list" : "grid"));
const selectedIdSet = computed(() => new Set(toolbarSelectedIds.value));

function toggleFilterSelection(id: string) {
  toolbarSelectedIds.value = toolbarSelectedIds.value.includes(id)
    ? toolbarSelectedIds.value.filter((existing) => existing !== id)
    : [...toolbarSelectedIds.value, id];
}

// --- Cards & Panels ----------------------------------------------------------
const skillCardDemos: { title: string; skill: Skill; selected: boolean; installing?: boolean; hasUpdate?: boolean }[] = [
  { title: "Default", skill: mockSkills[3], selected: false },
  { title: "Selected", skill: mockSkills[3], selected: true },
  { title: "Disabled (enabled: false)", skill: mockSkills[6], selected: false },
  { title: "Installing", skill: mockSkills[1], selected: false, installing: true },
  { title: "Partially installed", skill: mockSkills[1], selected: false },
  { title: "Fully installed", skill: mockSkills[0], selected: false },
  { title: "Has an update available", skill: mockSkills[2], selected: false, hasUpdate: true },
];

// --- Navigation --------------------------------------------------------------
const demoProjectPath = ref("/home/dev/acme/marketing-site");
const demoScope = ref<InstallScope>("project");

// --- Lists & tables ------------------------------------------------------------
const groupChips = mockGroups.map((group) => ({
  id: group.id,
  name: group.name,
  color: group.color,
  count: mockSkills.filter((skill) => skill.groupId === group.id).length,
}));

// --- Feedback ------------------------------------------------------------------
const { push: pushToast } = useToasts();
function demoToast(variant: "info" | "success" | "error") {
  const messages: Record<typeof variant, string> = {
    info: "Preferences saved.",
    success: "3 Skills installed.",
    error: "Installation failed: permission denied.",
  };
  pushToast(messages[variant], variant);
}

// --- Overlays ------------------------------------------------------------------
const confirmOpen = ref(false);
const confirmDestructiveOpen = ref(false);
const descriptionOpen = ref(false);
const skillsDialogOpen = ref(false);
const agentsDialogOpen = ref(false);
const agentsSelection = ref<string[]>([...targetAgents]);
const deleteGroupEmptyOpen = ref(false);
const deleteGroupWithSkillsOpen = ref(false);
const installConfirmOpen = ref(false);
const groupEditorCreateOpen = ref(false);
const groupEditorEditOpen = ref(false);
const projectsOpen = ref(false);
const tagsOpen = ref(false);
const editSkillOpen = ref(false);
const preferencesOpen = ref(false);
const addSkillOpen = ref(false);

const emptyGroup: SkillGroup = { id: "empty-group", name: "Prototyping", color: "#06B6D4", order: 4, enabled: true };
const backendSkillCount = mockSkills.filter((skill) => skill.groupId === "backend").length;
const otherGroupsForBackend = mockGroups.filter((group) => group.id !== "backend");

const preferencesDemo = reactive<UiPreferences>({ ...mockPreferences });
function handlePreferencesUpdate(partial: Partial<UiPreferences>) {
  Object.assign(preferencesDemo, partial);
}
</script>

<style scoped lang="scss" src="./StyleGuideView.scss"></style>
<style lang="scss" src="../App.scss"></style>
