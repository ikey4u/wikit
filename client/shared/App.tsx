import { cleanup, colorScheme, effect, Keyboard, platform, Scroll, setColorScheme, signal, Text, View, WebView, windowSize } from "nsh";
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
type Page = "查词" | "收藏" | "历史" | "笔记" | "词典" | "预览";
const empty: Snapshot = { dictionaries: [], errors: [], sources: [], selected: "", history: [], favorites: [], notes: "", appearance: "system", dataDirectory: "" };
const colors = {
  background: () => colorScheme() === "dark" ? "#101820" : "#f4f6f8",
  surface: () => colorScheme() === "dark" ? "#1c2732" : "#ffffff",
  text: () => colorScheme() === "dark" ? "#edf3f8" : "#182d3e",
  muted: () => colorScheme() === "dark" ? "#a4b7c7" : "#617589",
  accent: "#087f8c",
};

function Button(props: { label: string; onPress: () => void; active?: boolean; disabled?: boolean }) {
  return <View role="button" accessibilityLabel={props.label} accessibilityState={{ disabled: !!props.disabled }}
    onPress={() => { if (!props.disabled) props.onPress(); }}
    style={{ paddingHorizontal: 14, paddingVertical: 12, borderRadius: 10, minHeight: 44,
      backgroundColor: props.active ? colors.accent : colors.surface(), opacity: props.disabled ? 0.45 : 1 }}>
    <Text style={{ fontSize: 14, fontWeight: "600", color: props.active ? "#ffffff" : colors.text() }}>{props.label}</Text>
  </View>;
}
function Heading(props: { text: string }) {
  return <Text style={{ fontSize: 20, fontWeight: "700", color: colors.text() }}>{props.text}</Text>;
}
function Hint(props: { text: string }) {
  return <Text style={{ fontSize: 13, lineHeight: 20, color: colors.muted() }}>{props.text}</Text>;
}
function Field(props: { label: string; value: string; onChange: (value: string) => void; onSubmit?: () => void; multiline?: boolean }) {
  return <Input value={props.value} placeholder={props.label} accessibilityLabel={props.label}
    autoCapitalize="none" autoCorrect={false} multiline={props.multiline}
    onChange={({ value }) => props.onChange(value)} onSubmit={() => props.onSubmit?.()}
    style={{ backgroundColor: colors.surface(), color: colors.text(), borderRadius: 10,
      paddingHorizontal: 14, paddingVertical: 12, minHeight: props.multiline ? 180 : 48, fontSize: 16 }} />;
}

/** Shared native screens: NSH renders platform views, with only dictionary HTML in a WebView. */
export function App() {
  const page = signal<Page>("查词");
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
    if (!query || !dictionary) { error.set("请输入单词，并先添加、选择一本词典。"); return; }
    const version = ++searchVersion;
    await run(async () => {
      word.set(query); selected.set(dictionary); page.set("查词"); definition.set(null);
      Keyboard.dismiss();
      const next = await request<Definition>({ action: "lookup", dictionary, word: query });
      if (alive && version === searchVersion) {
        definition.set(next); status.set(next.count ? `找到 ${next.count} 个词条` : "没有匹配词条，请尝试其他拼写。");
        // Only the history changed: avoid another network catalog request.
        const entry = { dictionary, word: query };
        data.update(d => ({ ...d, history: [entry, ...d.history.filter(b => b.dictionary !== dictionary || b.word !== query)].slice(0, 200) }));
      }
    });
  }
  async function choose(id: string) {
    await run(async () => {
      await request({ action: "select", dictionary: id });
      selected.set(id); definition.set(null); status.set("已切换词典");
    });
  }
  async function add(importFile: boolean) {
    const value = source().trim();
    if (!value) { error.set("请输入词典服务器地址、文件下载地址或本地绝对路径。"); return; }
    await run(async () => {
      apply(await request<Snapshot>({ action: importFile ? "import" : "addSource", source: value }));
      source.set(""); status.set("词典来源已保存");
    });
  }
  async function favorite() {
    const entry = definition(); if (!entry || !entry.count) return;
    await run(async () => {
      const next = await request<{ favorites: Bookmark[] }>({ action: "favorite", dictionary: entry.dictionary, word: entry.word });
      data.update(d => ({ ...d, favorites: next.favorites })); status.set("收藏已更新");
    });
  }
  async function renderPreview() {
    await run(async () => {
      const next = await request<{ html: string; baseUrl: string }>({ action: "preview", path: previewPath() });
      preview.set({ ...next, html: next.html + `<!-- refresh ${Date.now()} -->` });
    });
  }
  void run(async () => {
    const next = await request<Snapshot>({ action: "snapshot" });
    if (!alive) return;
    apply(next); notes.set(next.notes);
    setColorScheme(next.appearance === "dark" ? "dark" : next.appearance === "light" ? "light" : "system");
  });
  const back = BackHandler.on("press", () => {
    if (page() === "查词") return false;
    page.set("查词"); return true;
  });
  cleanup(back);

  effect(() => {
    const path = previewPath();
    if (page() !== "预览" || !livePreview() || !path) return;
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
  const dictionaryName = (id: string) => data().dictionaries.find(d => d.id === id)?.name ?? "词典不可用";
  const isFavorite = () => !!definition() && data().favorites.some(b => b.dictionary === definition()!.dictionary && b.word === definition()!.word);

  return <View style={{ flex: 1, backgroundColor: colors.background(), paddingTop: "env(safe-area-inset-top)",
    paddingBottom: "env(safe-area-inset-bottom)", paddingLeft: "env(safe-area-inset-left)", paddingRight: "env(safe-area-inset-right)" }}>
    <View style={{ padding: 20, flexDirection: "row", alignItems: "center", gap: 12 }}>
      <Text style={{ fontSize: 28, fontWeight: "800", color: colors.text(), flex: 1 }}>Wikit</Text>
      <Hint text={busy() ? "处理中…" : "你的随身词典"} />
    </View>
    <View style={{ flex: 1, flexDirection: wide() ? "row" : "column", gap: 12, paddingHorizontal: 16, paddingBottom: 12 }}>
      <Scroll horizontal={!wide()} style={{ flexGrow: 0, width: wide() ? 144 : "100%" }}>
        <View style={{ flexDirection: wide() ? "column" : "row", gap: 8, paddingBottom: 8 }}>
          <Each of={["查词", "收藏", "历史", "笔记", "词典", "预览"] as Page[]}>{item =>
            <Button label={item()} active={page() === item()} onPress={() => page.set(item())} />
          }</Each>
        </View>
      </Scroll>
      <View style={{ flex: 1, gap: 12 }}>
        <If when={!!error()}><Text role="alert" style={{ color: "#c94747", fontSize: 14 }}>{error()}</Text></If>
        <If when={!!status()}><Hint text={status()} /></If>
        <If when={page() === "查词"}>
          <View style={{ gap: 10 }}>
            <Scroll horizontal style={{ flexGrow: 0 }}><View style={{ flexDirection: "row", gap: 8 }}>
              <Each of={data().dictionaries}>{item => <Button label={item().name} active={selected() === item().id} disabled={busy()} onPress={() => void choose(item().id)} />}</Each>
            </View></Scroll>
            <View style={{ flexDirection: "row", gap: 8 }}>
              <View style={{ flex: 1 }}><Field label="输入单词" value={word()} onChange={v => word.set(v)} onSubmit={() => void search()} /></View>
              <Button label="查询" active disabled={busy()} onPress={() => void search()} />
            </View>
          </View>
          <If when={!data().dictionaries.length}>
            <View style={{ padding: 24, gap: 14, backgroundColor: colors.surface(), borderRadius: 14 }}>
              <Heading text="从第一本词典开始" /><Hint text="连接 Wikit 词典服务器，或导入 .wikit 文件；导入后可离线查词。" />
              <Button label="添加词典" active onPress={() => page.set("词典")} />
            </View>
          </If>
          <If when={!!definition()}>
            <View style={{ flexDirection: "row", alignItems: "center", gap: 12 }}>
              <View style={{ flex: 1 }}><Heading text={definition()?.word ?? ""} /><Hint text={dictionaryName(definition()?.dictionary ?? "")} /></View>
              <Button label={isFavorite() ? "取消收藏" : "收藏"} disabled={busy() || !definition()?.count} onPress={() => void favorite()} />
            </View>
            <WebView html={definition()?.html ?? ""} baseUrl={definition()?.baseUrl} style={{ flex: 1, minHeight: 240, borderRadius: 12, backgroundColor: "#ffffff" }} />
          </If>
        </If>
        <If when={page() === "收藏" || page() === "历史"}>
          <View style={{ flexDirection: "row", alignItems: "center", gap: 12 }}>
            <View style={{ flex: 1 }}><Heading text={page() === "收藏" ? "收藏的单词" : "最近查询"} /></View>
            <If when={page() === "历史"}><Button label="清空历史" disabled={busy()} onPress={() => void run(async () => {
              await request({ action: "clearHistory" }); data.update(d => ({ ...d, history: [] }));
            })} /></If>
          </View>
          <Scroll style={{ flex: 1 }}><View style={{ gap: 8 }}>
            <If when={!(page() === "收藏" ? data().favorites : data().history).length}><Hint text="还没有记录。查询一个单词后，它会出现在这里。" /></If>
            <Each of={page() === "收藏" ? data().favorites : data().history}>{item =>
              <View style={{ gap: 4, padding: 14, borderRadius: 12, backgroundColor: colors.surface() }}>
                <Button label={item().word} disabled={busy()} onPress={() => void search(item())} /><Hint text={dictionaryName(item().dictionary)} />
              </View>
            }</Each>
          </View></Scroll>
        </If>
        <If when={page() === "笔记"}>
          <Heading text="学习笔记" /><Hint text="记录例句与学习心得，保存在当前设备。" />
          <Field label="写下例句或笔记" value={notes()} onChange={v => notes.set(v)} multiline />
          <Button label="保存笔记" active disabled={busy()} onPress={() => void run(async () => {
            await request({ action: "saveNotes", notes: notes() }); status.set("笔记已保存");
          })} />
        </If>
        <If when={page() === "词典"}>
          <Scroll style={{ flex: 1 }}><View style={{ gap: 16, paddingBottom: 24 }}>
            <Heading text="词典与设置" />
            <Hint text="服务器地址如 https://dict.example.com；导入文件可填写 .wikit 下载地址或设备内可访问的绝对路径。" />
            <Field label="服务器、文件地址或本地路径" value={source()} onChange={v => source.set(v)} />
            <View style={{ flexDirection: "row", flexWrap: "wrap", gap: 8 }}>
              <Button label="连接服务器 / 本地文件" disabled={busy()} onPress={() => void add(false)} />
              <Button label="导入到设备" active disabled={busy()} onPress={() => void add(true)} />
              <If when={platform.idiom === "desktop"}><Button label="选择文件" disabled={busy()} onPress={() => void run(async () => {
                const path = await Wikit.pickDictionary(); if (path) source.set(path);
              })} /></If>
              <Button label="刷新词典" disabled={busy()} onPress={() => void run(refresh)} />
            </View>
            <Each of={data().errors}>{item => <Text style={{ color: "#c94747", fontSize: 13 }}>{item()}</Text>}</Each>
            <Each of={data().sources}>{(item, index) => <View style={{ padding: 14, gap: 8, borderRadius: 12, backgroundColor: colors.surface() }}>
              <Hint text={item()} /><Button label="移除来源" disabled={busy()} onPress={() => void run(async () => {
                apply(await request<Snapshot>({ action: "removeSource", index: index() })); definition.set(null);
              })} />
            </View>}</Each>
            <Heading text="外观" />
            <View style={{ flexDirection: "row", gap: 8 }}><Each of={[{ label: "跟随系统", value: "system" }, { label: "浅色", value: "light" }, { label: "深色", value: "dark" }] }>{item =>
              <Button label={item().label} active={data().appearance === item().value} disabled={busy()} onPress={() => void run(async () => {
                await request({ action: "appearance", appearance: item().value });
                data.update(d => ({ ...d, appearance: item().value }));
                setColorScheme(item().value as "system" | "light" | "dark");
              })} />
            }</Each></View>
            <Hint text={`设备数据目录：${data().dataDirectory}`} />
          </View></Scroll>
        </If>
        <If when={page() === "预览"}>
          <Heading text="词典页面预览" /><Hint text="填写 HTML 文件或包含 index.html 的目录。修改文件后点刷新预览。" />
          <Field label="HTML 文件或目录的绝对路径" value={previewPath()} onChange={v => previewPath.set(v)} onSubmit={() => void renderPreview()} />
          <View style={{ flexDirection: "row", gap: 8 }}>
            <Button label="刷新预览" active disabled={busy()} onPress={() => void renderPreview()} />
            <Button label={livePreview() ? "停止自动刷新" : "自动刷新"} active={livePreview()} onPress={() => livePreview.set(!livePreview())} />
          </View>
          <If when={!!preview().html}><WebView html={preview().html} baseUrl={preview().baseUrl} style={{ flex: 1, minHeight: 240, backgroundColor: "#ffffff" }} /></If>
        </If>
      </View>
    </View>
  </View>;
}
