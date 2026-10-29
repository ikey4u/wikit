import { Canvas, keymap, cleanup, colorScheme, effect, Keyboard, platform, Scroll, setColorScheme, signal, Text, View, WebView, windowSize } from "nsh";
import { Input } from "nsh/base";
import { BackHandler } from "nsh/mobile";
import { Wikit } from "wikit/native";

interface Dictionary { id: string; name: string }
interface Bookmark { dictionary: string; word: string }
interface Snapshot {
  dictionaries: Dictionary[]; errors: string[]; sources: string[]; selected: string;
  history: Bookmark[]; favorites: Bookmark[]; notes: string; appearance: string; dataDirectory: string;
}
interface Definition { html: string; baseUrl: string; word: string; dictionary: string; count: number }
type Page = "Search" | "Favorites" | "History" | "Notes" | "Library" | "Preview";
const empty: Snapshot = { dictionaries: [], errors: [], sources: [], selected: "", history: [], favorites: [], notes: "", appearance: "system", dataDirectory: "" };
const colors = {
  background: () => colorScheme() === "dark" ? "#1d2221" : "#fbfcfa",
  surface: () => colorScheme() === "dark" ? "#272e2b" : "#ffffff",
  text: () => colorScheme() === "dark" ? "#edf1ed" : "#222d27",
  muted: () => colorScheme() === "dark" ? "#aebbb2" : "#637168",
  accent: "#33644d",
  sidebar: () => colorScheme() === "dark" ? "#232a26" : "#f0f3ee",
  line: () => colorScheme() === "dark" ? "#39463d" : "#dfe5dd",
  selection: () => colorScheme() === "dark" ? "#344b3c" : "#e0e9de",
};

/** Original monoline glyphs, painted by the native canvas at every scale. */
function NavIcon(props: { page: Page; selected: boolean }) {
  return <Canvas style={{ width: 18, height: 18, pointerEvents: "none" }} draw={(cx) => {
    cx.strokeStyle = props.selected ? (colorScheme() === "dark" ? "#b9dbc2" : "#315b40") : colors.muted();
    cx.lineWidth = 1.5; cx.lineCap = "round"; cx.lineJoin = "round";
    cx.beginPath();
    if (props.page === "Search") { cx.arc(7.5, 7.5, 5, 0, Math.PI * 2); cx.moveTo(11, 11); cx.lineTo(16, 16); }
    else if (props.page === "Favorites") {
      for (let i = 0; i < 10; i++) { const a = i * Math.PI / 5 - Math.PI / 2; const r = i % 2 ? 3.5 : 7.5;
        const x = 9 + Math.cos(a) * r, y = 9 + Math.sin(a) * r; if (i) cx.lineTo(x, y); else cx.moveTo(x, y); } cx.closePath();
    } else if (props.page === "History") { cx.arc(9, 9, 6.5, 0, Math.PI * 2); cx.moveTo(9, 5); cx.lineTo(9, 9); cx.lineTo(12, 11); }
    else if (props.page === "Notes") { cx.rect(4, 2.5, 10, 13); cx.moveTo(7, 6); cx.lineTo(11, 6); cx.moveTo(7, 9); cx.lineTo(11, 9); cx.moveTo(7, 12); cx.lineTo(10, 12); }
    else if (props.page === "Library") { cx.rect(2, 3, 5, 12); cx.rect(10, 3, 5, 12); cx.moveTo(2, 12); cx.lineTo(7, 12); cx.moveTo(10, 12); cx.lineTo(15, 12); }
    else { cx.rect(2, 3, 14, 12); cx.moveTo(2, 7); cx.lineTo(16, 7); cx.moveTo(7, 7); cx.lineTo(7, 15); }
    cx.stroke();
  }} />;
}

function Button(props: { label: string; onPress: () => void; active?: boolean; disabled?: boolean }) {
  return <View role="button" accessibilityLabel={props.label} accessibilityState={{ disabled: !!props.disabled, selected: !!props.active }}
    onPress={() => { if (!props.disabled) props.onPress(); }}
    style={{ paddingHorizontal: 12, paddingVertical: platform.idiom === "desktop" ? 7 : 12, borderRadius: 6, minHeight: platform.idiom === "desktop" ? 32 : 44,
      borderWidth: 1, borderColor: props.active ? colors.accent : colors.line(),
      backgroundColor: props.active ? colors.accent : colors.surface(), opacity: props.disabled ? 0.45 : 1 }}>
    <Text style={{ fontSize: 13, fontWeight: "500", color: props.active ? "#ffffff" : colors.text() }}>{props.label}</Text>
  </View>;
}
function Heading(props: { text: string }) {
  return <Text style={{ fontSize: 24, fontWeight: "600", color: colors.text() }}>{props.text}</Text>;
}
function Hint(props: { text: string }) {
  return <Text style={{ fontSize: 13, lineHeight: 20, color: colors.muted() }}>{props.text}</Text>;
}
function Field(props: { label: string; value: string; onChange: (value: string) => void; onSubmit?: () => void; multiline?: boolean }) {
  return <Input value={props.value} placeholder={props.label} accessibilityLabel={props.label}
    autoCapitalize="none" autoCorrect={false} multiline={props.multiline}
    onChange={({ value }) => props.onChange(value)} onSubmit={() => props.onSubmit?.()}
    style={{ backgroundColor: colors.surface(), color: colors.text(), borderRadius: 7, borderWidth: 1, borderColor: colors.line(),
      paddingHorizontal: 14, paddingVertical: 10, minHeight: props.multiline ? 240 : 44, fontSize: 15 }} />;
}

function EmptyState(props: { title: string; detail: string; action: string; onPress: () => void; page: Page }) {
  return <View style={{ flex: 1, justifyContent: "center", alignItems: "center", padding: 24, gap: 18 }}>
    <NavIcon page={props.page} selected={false} />
    <Text style={{ fontSize: 26, fontWeight: "500", color: colors.text(), textAlign: "center" }}>{props.title}</Text>
    <Text style={{ maxWidth: 340, fontSize: 15, lineHeight: 24, color: colors.muted(), textAlign: "center" }}>{props.detail}</Text>
    <Button label={props.action} active onPress={props.onPress} />
  </View>;
}

/** Shared native screens: NSH renders platform views, with only dictionary HTML in a WebView. */
export function App() {
  const page = signal<Page>("Search");
  const data = signal<Snapshot>(empty);
  const selected = signal("");
  const word = signal("");
  const source = signal("");
  const sourceMode = signal<"file" | "server">("file");
  const notes = signal("");
  const previewPath = signal("");
  const livePreview = signal(false);
  const preview = signal({ html: "", baseUrl: "" });
  const definition = signal<Definition | null>(null);
  const busy = signal(false);
  const status = signal("");
  const error = signal("");
  let alive = true;
  let searchVersion = 0;
  cleanup(() => { alive = false; searchVersion++; });

  async function request<T>(operation: object): Promise<T> {
    return JSON.parse(await Wikit.request(JSON.stringify(operation))) as T;
  }
  function apply(next: Snapshot) {
    data.set(next);
    if (!next.dictionaries.some(d => d.id === selected())) {
      selected.set(next.dictionaries.find(d => d.id === next.selected)?.id ?? next.dictionaries[0]?.id ?? "");
    }
  }
  async function run(work: () => Promise<void>) {
    if (busy()) return;
    busy.set(true); error.set(""); status.set("");
    try { await work(); }
    catch (e) { if (alive) error.set(String(e)); }
    finally { if (alive) busy.set(false); }
  }
  async function refresh() { const next = await request<Snapshot>({ action: "snapshot" }); if (alive) apply(next); }
  async function search(bookmark?: Bookmark) {
    if (busy()) return;
    const query = bookmark?.word ?? word().trim();
    const dictionary = bookmark?.dictionary ?? selected();
    if (!query || !dictionary) { error.set("Enter a word and select a dictionary first."); return; }
    const version = ++searchVersion;
    await run(async () => {
      word.set(query); selected.set(dictionary); page.set("Search"); definition.set(null);
      Keyboard.dismiss();
      const next = await request<Definition>({ action: "lookup", dictionary, word: query });
      if (alive && version === searchVersion) {
        definition.set(next); status.set(next.count ? `${next.count} entries found` : "No entries found. Try another spelling.");
        // Only the history changed: avoid another network catalog request.
        const entry = { dictionary, word: query };
        data.update(d => ({ ...d, history: [entry, ...d.history.filter(b => b.dictionary !== dictionary || b.word !== query)].slice(0, 200) }));
      }
    });
  }
  async function choose(id: string) {
    await run(async () => {
      await request({ action: "select", dictionary: id });
      selected.set(id); definition.set(null); status.set("Dictionary selected");
    });
  }
  async function add(importFile: boolean) {
    const value = source().trim();
    if (!value) { error.set("Enter a server URL, download URL, or absolute file path."); return; }
    await run(async () => {
      apply(await request<Snapshot>({ action: importFile ? "import" : "addSource", source: value }));
      source.set(""); status.set("Dictionary source saved");
    });
  }
  async function favorite() {
    const entry = definition(); if (!entry || !entry.count) return;
    await run(async () => {
      const next = await request<{ favorites: Bookmark[] }>({ action: "favorite", dictionary: entry.dictionary, word: entry.word });
      data.update(d => ({ ...d, favorites: next.favorites })); status.set("Favorites updated");
    });
  }
  async function renderPreview() {
    await run(async () => {
      const next = await request<{ html: string; baseUrl: string }>({ action: "preview", path: previewPath() });
      preview.set({ ...next, html: next.html + `<!-- refresh ${Date.now()} -->` });
    });
  }
  async function saveNotes() {
    await run(async () => { await request({ action: "saveNotes", notes: notes() }); status.set("Notes saved"); });
  }
  const pages: Page[] = ["Search", "Favorites", "History", "Notes", "Library", "Preview"];
  pages.forEach((destination, index) => keymap.bind(`mod+${index + 1}`, () => page.set(destination)));
  keymap.bind("mod+s", () => { if (page() === "Notes") void saveNotes(); });
  effect(() => {
    if (!status()) return;
    const timer = setTimeout(() => status.set(""), 3500);
    cleanup(() => clearTimeout(timer));
  });
  void run(async () => {
    const next = await request<Snapshot>({ action: "snapshot" });
    if (!alive) return;
    apply(next); notes.set(next.notes);
    setColorScheme(next.appearance === "dark" ? "dark" : next.appearance === "light" ? "light" : "system");
  });
  const back = BackHandler.on("press", () => {
    if (page() === "Search") return false;
    page.set("Search"); return true;
  });
  cleanup(back);

  effect(() => {
    const path = previewPath();
    if (page() !== "Preview" || !livePreview() || !path) return;
    let disposed = false;
    let pending = false;
    const interval = setInterval(async () => {
      if (pending || busy()) return;
      pending = true;
      try {
        const next = await request<{ html: string; baseUrl: string }>({ action: "preview", path });
        if (!disposed) preview.set({ ...next, html: next.html + `<!-- refresh ${Date.now()} -->` });
      } catch (e) {
        if (!disposed) { error.set(String(e)); livePreview.set(false); }
      } finally { pending = false; }
    }, 1500);
    cleanup(() => { disposed = true; clearInterval(interval); });
  });

  // Adapt the native navigation to the available window width.
  const wide = () => windowSize().width >= 800;
  const dictionaryName = (id: string) => data().dictionaries.find(d => d.id === id)?.name ?? "Dictionary unavailable";
  const isFavorite = () => !!definition() && data().favorites.some(b => b.dictionary === definition()!.dictionary && b.word === definition()!.word);

  const navigationSelected = (destination: Page, mobile?: boolean) => page() === destination || !!mobile && destination === "Library" && (page() === "Notes" || page() === "Preview");
  function Navigation(props: { mobile?: boolean }) {
    return <View style={{ flexDirection: props.mobile ? "row" : "column", gap: props.mobile ? 0 : 4 }}>
      <Each of={props.mobile ? pages.filter(p => p !== "Notes" && p !== "Preview") : pages}>{item =>
        <View role="button" accessibilityLabel={item()} accessibilityState={{ selected: navigationSelected(item(), props.mobile) }} onPress={() => page.set(item())}
          style={{ flex: props.mobile ? 1 : undefined, flexDirection: props.mobile ? "column" : "row", alignItems: "center",
            justifyContent: props.mobile ? "center" : "flex-start", gap: props.mobile ? 5 : 12,
            paddingHorizontal: props.mobile ? 4 : 12, minHeight: props.mobile ? 60 : 42, borderRadius: 7,
            marginTop: !props.mobile && item() === "Library" ? 28 : 0,
            backgroundColor: navigationSelected(item(), props.mobile) ? colors.selection() : "transparent" }}>
          <NavIcon page={item()} selected={navigationSelected(item(), props.mobile)} />
          <Text style={{ fontSize: props.mobile ? 11 : 14, fontWeight: navigationSelected(item(), props.mobile) ? "600" : "400", color: colors.text(), flex: props.mobile ? undefined : 1 }}>{item()}</Text>
          <If when={!props.mobile && (item() === "Favorites" || item() === "History")}>
            <Hint text={String(item() === "Favorites" ? data().favorites.length : data().history.length)} />
          </If>
        </View>
      }</Each>
    </View>;
  }
  function WordRow(props: { entry: Bookmark }) {
    return <View role="button" accessibilityLabel={props.entry.word} accessibilityState={{ disabled: busy() }}
      onPress={() => void search(props.entry)} style={{ flexDirection: "row", alignItems: "center", gap: 16,
        minHeight: 72, paddingVertical: 14, borderBottomWidth: 1, borderColor: colors.line(), opacity: busy() ? 0.5 : 1 }}>
      <View style={{ flex: 1, gap: 5 }}>
        <Text style={{ fontSize: 18, fontWeight: "500", color: colors.text() }}>{props.entry.word}</Text>
        <Hint text={dictionaryName(props.entry.dictionary)} />
      </View>
      <NavIcon page="Search" selected={false} />
    </View>;
  }
  return <View style={{ flex: 1, backgroundColor: colors.background(), paddingTop: "env(safe-area-inset-top)",
    paddingBottom: "env(safe-area-inset-bottom)", paddingLeft: "env(safe-area-inset-left)", paddingRight: "env(safe-area-inset-right)" }}>
    <View style={{ flex: 1, flexDirection: wide() ? "row" : "column" }}>
      <If when={wide()}>
        <View style={{ width: 208, backgroundColor: colors.sidebar(), padding: 16, gap: 32 }}>
          <View style={{ paddingHorizontal: 12, paddingTop: 16, paddingBottom: 8, gap: 4 }}>
            <Text style={{ fontSize: 25, fontWeight: "600", color: colors.text() }}>Wikit</Text>
            <Hint text="A little more understanding." />
          </View>
          <Navigation />
          <View style={{ flex: 1 }} />
          <View style={{ gap: 8, padding: 12 }}>
            <Hint text={`${data().dictionaries.length} dictionaries`} />
            <Hint text={`${keymap.label("mod+1")}–${keymap.label("mod+6")} to navigate`} />
          </View>
        </View>
      </If>
      <View style={{ flex: 1 }}>
        <View style={{ minHeight: 56, paddingHorizontal: wide() ? 32 : 20, flexDirection: "row", alignItems: "center", gap: 12 }}>
          <Text style={{ fontSize: 14, fontWeight: "500", color: colors.muted(), flex: 1 }}>{wide() ? page() : `Wikit / ${page()}`}</Text>
          <Hint text={busy() ? (page() === "Search" ? "Looking up…" : "Working…") : ""} />
        </View>
        <View style={{ flex: 1, width: "100%", maxWidth: page() === "Search" && !!definition() ? 1200 : 880,
          alignSelf: "center", gap: 20, paddingHorizontal: wide() ? 40 : 20, paddingBottom: wide() ? 32 : 12 }}>
        <If when={!!error()}><Text role="alert" style={{ color: colorScheme() === "dark" ? "#ffb4a9" : "#a8352b", fontSize: 14, lineHeight: 22 }}>{error()}</Text></If>
        <If when={!!status()}><Hint text={status()} /></If>
        <If when={page() === "Search"}>
          <If when={data().dictionaries.length > 0}>
            <View style={{ gap: 16, paddingTop: wide() ? 20 : 8 }}>
              <If when={!definition()}><Text style={{ fontSize: wide() ? 32 : 28, fontWeight: "500", color: colors.text() }}>Every word opens a door.</Text></If>
              <View style={{ flexDirection: "row", gap: 8, alignItems: "center" }}>
                <View style={{ flex: 1 }}><Field label="Search for a word" value={word()} onChange={v => word.set(v)} onSubmit={() => void search()} /></View>
                <Button label="Look up" active disabled={busy() || !word().trim() || !selected()} onPress={() => void search()} />
              </View>
              <Scroll horizontal style={{ flexGrow: 0 }}><View style={{ flexDirection: "row", gap: 8 }}>
                <Each of={data().dictionaries}>{item =>
                  <View role="button" accessibilityLabel={item().name} accessibilityState={{ selected: selected() === item().id, disabled: busy() }}
                    onPress={() => { if (!busy()) void choose(item().id); }}
                    style={{ paddingHorizontal: 12, minHeight: 44, justifyContent: "center", borderBottomWidth: selected() === item().id ? 2 : 0,
                      borderColor: colors.accent, opacity: busy() ? 0.5 : 1 }}>
                    <Text style={{ fontSize: 13, fontWeight: selected() === item().id ? "600" : "400", color: selected() === item().id ? colors.text() : colors.muted() }}>{item().name}</Text>
                  </View>
                }</Each>
              </View></Scroll>
            </View>
          </If>
          <If when={!data().dictionaries.length}>
            <EmptyState title="Your dictionary starts here" detail="Bring a dictionary you love. Import a .wikit file or connect to a Wikit server." action="Add dictionary" page="Library" onPress={() => page.set("Library")} />
          </If>
          <If when={data().dictionaries.length > 0 && !definition()}>
            <Scroll style={{ flex: 1 }}><View style={{ gap: 8, paddingTop: 20 }}>
              <If when={!data().history.length}><Hint text="Type a word and press Return. Its meaning will appear here." /></If>
              <If when={data().history.length > 0}><Hint text="Pick up where you left off" />
                <Each of={data().history.slice(0, 5)}>{item => <WordRow entry={item()} />}</Each>
              </If>
            </View></Scroll>
          </If>
          <If when={!!definition() && !!definition()?.count}>
            <View style={{ flexDirection: "row", alignItems: "center", gap: 12, paddingTop: 8 }}>
              <View style={{ flex: 1, gap: 6 }}><Text style={{ fontSize: 32, fontWeight: "500", color: colors.text() }}>{definition()?.word ?? ""}</Text><Hint text={dictionaryName(definition()?.dictionary ?? "")} /></View>
              <Button label={isFavorite() ? "Remove favorite" : "Save word"} disabled={busy()} onPress={() => void favorite()} />
            </View>
            <WebView html={definition()?.html ?? ""} baseUrl={definition()?.baseUrl} style={{ flex: 1, minHeight: 200, borderRadius: 6, backgroundColor: "#ffffff" }} />
          </If>
          <If when={!!definition() && !definition()?.count}>
            <EmptyState title="No matching entry" detail="Try another spelling, or choose a different dictionary above." action="Browse dictionaries" page="Search" onPress={() => page.set("Library")} />
          </If>
        </If>
        <If when={page() === "Favorites" || page() === "History"}>
          <View style={{ flexDirection: "row", alignItems: "center", gap: 12, paddingTop: 16 }}>
            <View style={{ flex: 1, gap: 8 }}><Heading text={page() === "Favorites" ? "Saved words" : "Recently looked up"} />
              <Hint text={page() === "Favorites" ? "Keep the words you want to return to." : "Continue a thought. Rediscover a word."} />
            </View>
            <If when={page() === "History" && data().history.length > 0}><Button label="Clear history" disabled={busy()} onPress={() => void run(async () => {
              await request({ action: "clearHistory" }); data.update(d => ({ ...d, history: [] }));
            })} /></If>
          </View>
          <If when={!(page() === "Favorites" ? data().favorites : data().history).length}>
            <EmptyState title={page() === "Favorites" ? "Keep a word close" : "A fresh page"}
              detail={page() === "Favorites" ? "Save a word while reading its definition. You can return to it here anytime." : "Words you look up will appear here, ready to revisit."}
              action={data().dictionaries.length ? "Look up a word" : "Add dictionary"} page={page()}
              onPress={() => page.set(data().dictionaries.length ? "Search" : "Library")} />
          </If>
          <If when={(page() === "Favorites" ? data().favorites : data().history).length > 0}>
            <Scroll style={{ flex: 1 }}><View>
              <Each of={page() === "Favorites" ? data().favorites : data().history}>{item => <WordRow entry={item()} />}</Each>
            </View></Scroll>
          </If>
        </If>
        <If when={page() === "Notes"}>
          <Heading text="Study notes" /><Hint text="A quiet place for examples, ideas, and words worth remembering." />
          <Scroll style={{ flex: 1 }}><Field label="Write an example or a note…" value={notes()} onChange={v => notes.set(v)} multiline /></Scroll>
          <View style={{ flexDirection: "row", alignItems: "center", gap: 12 }}><Button label="Save notes" active disabled={busy()} onPress={() => void saveNotes()} /><If when={wide()}><Hint text={keymap.label("mod+s")} /></If></View>
        </If>
        <If when={page() === "Library"}>
          <Scroll style={{ flex: 1 }}><View style={{ gap: 16, paddingBottom: 24 }}>
            <Heading text="Dictionary library" />
            <Hint text="Your dictionaries, ready when you need them." />
            <View style={{ flexDirection: "row", flexWrap: "wrap", gap: 8 }}>
              <Button label="Import a file" active={sourceMode() === "file"} onPress={() => sourceMode.set("file")} />
              <Button label="Connect a server" active={sourceMode() === "server"} onPress={() => sourceMode.set("server")} />
            </View>
            <View style={{ gap: 12, paddingVertical: 8 }}>
              <Hint text={sourceMode() === "file" ? "Use a .wikit download URL or an absolute file path." : "Enter the base URL of your Wikit server."} />
              <Field label={sourceMode() === "file" ? "Download URL or file path" : "Wikit server URL"} value={source()} onChange={v => source.set(v)} onSubmit={() => { if (source().trim()) void add(sourceMode() === "file"); }} />
              <View style={{ flexDirection: "row", flexWrap: "wrap", gap: 8 }}>
                <Button label={sourceMode() === "file" ? "Import dictionary" : "Connect source"} active disabled={busy() || !source().trim()} onPress={() => void add(sourceMode() === "file")} />
                <If when={platform.idiom === "desktop" && sourceMode() === "file"}><Button label="Browse files…" disabled={busy()} onPress={() => void run(async () => {
                  const path = await Wikit.pickDictionary(); if (path) source.set(path);
                })} /></If>
              </View>
            </View>
            <If when={data().sources.length > 0}><View style={{ flexDirection: "row", alignItems: "center", gap: 12, paddingTop: 16 }}>
              <View style={{ flex: 1 }}><Heading text="Your sources" /></View>
              <Button label="Refresh library" disabled={busy()} onPress={() => void run(refresh)} />
            </View></If>
            <Each of={data().errors}>{item => <Text style={{ color: "#c94747", fontSize: 13 }}>{item()}</Text>}</Each>
            <Each of={data().sources}>{(item, index) => <View style={{ paddingVertical: 16, gap: 12, borderBottomWidth: 1, borderColor: colors.line() }}>
              <Hint text={item()} /><Button label="Remove source" disabled={busy()} onPress={() => void run(async () => {
                apply(await request<Snapshot>({ action: "removeSource", index: index() })); definition.set(null);
              })} />
            </View>}</Each>
            <If when={!wide()}><View style={{ gap: 12, paddingTop: 16 }}>
              <Heading text="More tools" />
              <View style={{ flexDirection: "row", flexWrap: "wrap", gap: 8 }}>
                <Button label="Notes" onPress={() => page.set("Notes")} />
                <Button label="Preview" onPress={() => page.set("Preview")} />
              </View>
            </View></If>
            <View style={{ paddingTop: 24 }}><Heading text="Appearance" /></View>
            <View style={{ flexDirection: "row", gap: 8 }}><Each of={[{ label: "System", value: "system" }, { label: "Light", value: "light" }, { label: "Dark", value: "dark" }] }>{item =>
              <Button label={item().label} active={data().appearance === item().value} disabled={busy()} onPress={() => void run(async () => {
                await request({ action: "appearance", appearance: item().value });
                data.update(d => ({ ...d, appearance: item().value }));
                setColorScheme(item().value as "system" | "light" | "dark");
              })} />
            }</Each></View>

          </View></Scroll>
        </If>
        <If when={page() === "Preview"}>
          <Heading text="Dictionary preview" /><Hint text="Open an HTML file or a folder containing index.html to preview your dictionary layout." />
          <Field label="Absolute path to HTML file or folder" value={previewPath()} onChange={v => previewPath.set(v)} onSubmit={() => void renderPreview()} />
          <View style={{ flexDirection: "row", gap: 8 }}>
            <Button label="Refresh preview" active disabled={busy()} onPress={() => void renderPreview()} />
            <Button label={livePreview() ? "Stop live preview" : "Live preview"} active={livePreview()} onPress={() => livePreview.set(!livePreview())} />
          </View>
          <If when={!!preview().html}><WebView html={preview().html} baseUrl={preview().baseUrl} style={{ flex: 1, minHeight: 240, backgroundColor: "#ffffff" }} /></If>
        </If>
      </View>
      </View>
      <If when={!wide()}><View style={{ borderTopWidth: 1, borderColor: colors.line(), backgroundColor: colors.surface(), paddingHorizontal: 8, paddingTop: 4 }}><Navigation mobile /></View></If>
    </View>
  </View>;
}
