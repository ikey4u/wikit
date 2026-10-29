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
  background: () => colorScheme() === "dark" ? "#202124" : "#fafaf9",
  surface: () => colorScheme() === "dark" ? "#2b2d31" : "#ffffff",
  text: () => colorScheme() === "dark" ? "#f3f3f1" : "#242629",
  muted: () => colorScheme() === "dark" ? "#b0b2b8" : "#666b73",
  accent: "#3468c0",
  sidebar: () => colorScheme() === "dark" ? "#26282c" : "#eeefed",
  line: () => colorScheme() === "dark" ? "#3c3f44" : "#dddeda",
  selection: () => colorScheme() === "dark" ? "#35435b" : "#dde6f5",
};

/** Original monoline glyphs, painted by the native canvas at every scale. */
function NavIcon(props: { page: Page; selected: boolean }) {
  return <Canvas style={{ width: 18, height: 18, pointerEvents: "none" }} draw={(cx) => {
    cx.strokeStyle = props.selected ? (colorScheme() === "dark" ? "#b9d2ff" : "#244e91") : colors.muted();
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
  return <View role="button" accessibilityLabel={props.label} accessibilityState={{ disabled: !!props.disabled }}
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

/** Shared native screens: NSH renders platform views, with only dictionary HTML in a WebView. */
export function App() {
  const page = signal<Page>("Search");
  const data = signal<Snapshot>(empty);
  const selected = signal("");
  const word = signal("");
  const source = signal("");
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

  return <View style={{ flex: 1, backgroundColor: colors.background(), paddingTop: "env(safe-area-inset-top)",
    paddingBottom: "env(safe-area-inset-bottom)", paddingLeft: "env(safe-area-inset-left)", paddingRight: "env(safe-area-inset-right)" }}>
    <View style={{ flex: 1, flexDirection: wide() ? "row" : "column" }}>
      <View style={{ width: wide() ? 224 : "100%", backgroundColor: colors.sidebar(),
        borderRightWidth: wide() ? 1 : 0, borderBottomWidth: wide() ? 0 : 1, borderColor: colors.line(),
        padding: wide() ? 16 : 12, gap: 24 }}>
        <View style={{ flexDirection: "row", alignItems: "center", gap: 10, paddingHorizontal: 8, paddingTop: 8 }}>
          <View style={{ width: 32, height: 32, borderRadius: 8, backgroundColor: colors.accent, alignItems: "center", justifyContent: "center" }}>
            <Text style={{ fontSize: 20, fontWeight: "600", color: "#ffffff" }}>W</Text>
          </View>
          <View><Text style={{ fontSize: 18, fontWeight: "600", color: colors.text() }}>Wikit</Text>
            <If when={wide()}><Hint text="Your personal dictionary" /></If></View>
        </View>
        <Scroll horizontal={!wide()} style={{ flexGrow: wide() ? 1 : 0 }}>
          <View style={{ flexDirection: wide() ? "column" : "row", gap: 4 }}>
            <If when={wide()}><Text style={{ fontSize: 11, fontWeight: "600", color: colors.muted(), padding: 8 }}>WORKSPACE</Text></If>
            <Each of={pages}>{(item, index) =>
              <View role="button" accessibilityLabel={item()} accessibilityState={{ selected: page() === item() }} onPress={() => page.set(item())}
                style={{ flexDirection: "row", alignItems: "center", gap: 10, paddingHorizontal: 12,
                  minHeight: wide() ? 38 : 44, borderRadius: 6, backgroundColor: page() === item() ? colors.selection() : "transparent" }}>
                <NavIcon page={item()} selected={page() === item()} />
                <Text style={{ fontSize: 14, fontWeight: page() === item() ? "600" : "400", color: page() === item() ? (colorScheme() === "dark" ? "#b9d2ff" : "#244e91") : colors.text(), flex: 1 }}>{item()}</Text>
                <If when={wide() && (item() === "Favorites" || item() === "History")}><Hint text={String(item() === "Favorites" ? data().favorites.length : data().history.length)} /></If>
              </View>
            }</Each>
          </View>
        </Scroll>
        <If when={wide()}><View style={{ gap: 6, borderTopWidth: 1, borderColor: colors.line(), paddingTop: 16, paddingHorizontal: 8 }}>
          <Hint text={`${data().dictionaries.length} dictionaries available`} />
          <Hint text="Made for curious minds." /><Hint text={`${keymap.label("mod+1")}–${keymap.label("mod+6")} to navigate`} />
        </View></If>
      </View>
      <View style={{ flex: 1 }}>
        <View style={{ minHeight: 56, paddingHorizontal: 24, flexDirection: "row", alignItems: "center", borderBottomWidth: 1, borderColor: colors.line(), gap: 12 }}>
          <Text style={{ fontSize: 14, fontWeight: "600", color: colors.text(), flex: 1 }}>{page()}</Text>
          <Hint text={busy() ? "Working…" : ""} />
          <If when={page() !== "Library" && data().dictionaries.length > 0}><Button label="Add dictionary" onPress={() => page.set("Library")} /></If>
        </View>
        <View style={{ flex: 1, gap: 16, padding: wide() ? 28 : 16 }}>
        <If when={!!error()}><Text role="alert" style={{ color: "#c94747", fontSize: 14 }}>{error()}</Text></If>
        <If when={!!status()}><Hint text={status()} /></If>
        <If when={page() === "Search"}>
          <View style={{ gap: 10 }}>
            <Scroll horizontal style={{ flexGrow: 0 }}><View style={{ flexDirection: "row", gap: 8 }}>
              <Each of={data().dictionaries}>{item => <Button label={item().name} active={selected() === item().id} disabled={busy()} onPress={() => void choose(item().id)} />}</Each>
            </View></Scroll>
            <View style={{ flexDirection: "row", gap: 8 }}>
              <View style={{ flex: 1 }}><Field label="Search for a word" value={word()} onChange={v => word.set(v)} onSubmit={() => void search()} /></View>
              <Button label="Look up" active disabled={busy()} onPress={() => void search()} />
            </View>
          </View>
          <If when={!data().dictionaries.length}>
            <View style={{ flex: 1, padding: wide() ? 48 : 24, gap: 20, alignItems: "center", justifyContent: "center" }}>
              <Text style={{ fontSize: 48, color: colors.accent, fontWeight: "300" }}>Aa</Text><Heading text="A world of words awaits" /><Hint text="Add a dictionary to start exploring. Import a .wikit file for offline access, or connect to a server." />
              <Button label="Add dictionary" active onPress={() => page.set("Library")} />
            </View>
          </If>
          <If when={data().dictionaries.length > 0 && !definition()}>
            <View style={{ flex: 1, alignItems: "center", justifyContent: "center", gap: 12 }}>
              <Heading text="Follow your curiosity" /><Hint text="Type a word above and press Return to explore its meaning." />
            </View>
          </If>
          <If when={!!definition()}>
            <View style={{ flexDirection: "row", alignItems: "center", gap: 12 }}>
              <View style={{ flex: 1 }}><Heading text={definition()?.word ?? ""} /><Hint text={dictionaryName(definition()?.dictionary ?? "")} /></View>
              <Button label={isFavorite() ? "Remove favorite" : "Save word"} disabled={busy() || !definition()?.count} onPress={() => void favorite()} />
            </View>
            <WebView html={definition()?.html ?? ""} baseUrl={definition()?.baseUrl} style={{ flex: 1, minHeight: 240, borderRadius: 8, backgroundColor: "#ffffff" }} />
          </If>
        </If>
        <If when={page() === "Favorites" || page() === "History"}>
          <View style={{ flexDirection: "row", alignItems: "center", gap: 12 }}>
            <View style={{ flex: 1 }}><Heading text={page() === "Favorites" ? "Saved words" : "Recently looked up"} /></View>
            <If when={page() === "History"}><Button label="Clear history" disabled={busy()} onPress={() => void run(async () => {
              await request({ action: "clearHistory" }); data.update(d => ({ ...d, history: [] }));
            })} /></If>
          </View>
          <Scroll style={{ flex: 1 }}><View style={{ gap: 8 }}>
            <If when={!(page() === "Favorites" ? data().favorites : data().history).length}><Hint text="Nothing here yet. Look up a word to start your collection." /></If>
            <Each of={page() === "Favorites" ? data().favorites : data().history}>{item =>
              <View style={{ gap: 4, padding: 12, borderBottomWidth: 1, borderColor: colors.line() }}>
                <Button label={item().word} disabled={busy()} onPress={() => void search(item())} /><Hint text={dictionaryName(item().dictionary)} />
              </View>
            }</Each>
          </View></Scroll>
        </If>
        <If when={page() === "Notes"}>
          <Heading text="Study notes" /><Hint text="A quiet place for examples, ideas, and words worth remembering." />
          <Field label="Write an example or a note…" value={notes()} onChange={v => notes.set(v)} multiline />
          <View style={{ flexDirection: "row", alignItems: "center", gap: 12 }}><Button label="Save notes" active disabled={busy()} onPress={() => void saveNotes()} /><If when={wide()}><Hint text={keymap.label("mod+s")} /></If></View>
        </If>
        <If when={page() === "Library"}>
          <Scroll style={{ flex: 1 }}><View style={{ gap: 16, paddingBottom: 24 }}>
            <Heading text="Dictionary library" />
            <Hint text="Connect a Wikit server or import a .wikit file from a URL or an accessible device path." />
            <Field label="Server URL, download URL, or file path" value={source()} onChange={v => source.set(v)} />
            <View style={{ flexDirection: "row", flexWrap: "wrap", gap: 8 }}>
              <Button label="Connect source" disabled={busy()} onPress={() => void add(false)} />
              <Button label="Import dictionary" active disabled={busy()} onPress={() => void add(true)} />
              <If when={platform.idiom === "desktop"}><Button label="Browse files…" disabled={busy()} onPress={() => void run(async () => {
                const path = await Wikit.pickDictionary(); if (path) source.set(path);
              })} /></If>
              <Button label="Refresh library" disabled={busy()} onPress={() => void run(refresh)} />
            </View>
            <Each of={data().errors}>{item => <Text style={{ color: "#c94747", fontSize: 13 }}>{item()}</Text>}</Each>
            <Each of={data().sources}>{(item, index) => <View style={{ padding: 16, gap: 8, borderWidth: 1, borderColor: colors.line(), borderRadius: 8, backgroundColor: colors.surface() }}>
              <Hint text={item()} /><Button label="Remove source" disabled={busy()} onPress={() => void run(async () => {
                apply(await request<Snapshot>({ action: "removeSource", index: index() })); definition.set(null);
              })} />
            </View>}</Each>
            <Heading text="Appearance" />
            <View style={{ flexDirection: "row", gap: 8 }}><Each of={[{ label: "System", value: "system" }, { label: "Light", value: "light" }, { label: "Dark", value: "dark" }] }>{item =>
              <Button label={item().label} active={data().appearance === item().value} disabled={busy()} onPress={() => void run(async () => {
                await request({ action: "appearance", appearance: item().value });
                data.update(d => ({ ...d, appearance: item().value }));
                setColorScheme(item().value as "system" | "light" | "dark");
              })} />
            }</Each></View>
            <Hint text={`Local data: ${data().dataDirectory}`} />
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
    </View>
  </View>;
}
