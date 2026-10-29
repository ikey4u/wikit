import { expect, open, test } from "nsh/test";

test("native navigation works at phone width", async () => {
  const app = await open("./App.tsx", { width: 390, height: 844 });
  await app.waitFor("Your dictionary starts here");
  expect(app.texts()).toContain("Wikit / Search");
  await app.press("label:Library");
  await app.waitFor("Dictionary library");
  expect(app.shows("Import dictionary")).toBe(true);
  await app.press("label:Notes");
  await app.waitFor("Study notes");
  await app.press("label:Search");
  await app.waitFor("Your dictionary starts here");
  expect(app.shows("Look up")).toBe(false);
});

test("desktop sidebar opens favorites and preview", async () => {
  const app = await open("./App.tsx", { width: 1100, height: 760 });
  await app.waitFor("Your dictionary starts here");
  await app.press("label:Favorites");
  await app.waitFor("Saved words");
  await app.press("label:History");
  await app.waitFor("Recently looked up");
  await app.press("label:Preview");
  await app.waitFor("Dictionary preview");
  expect(app.shows("Refresh preview")).toBe(true);
});


test("desktop keyboard navigation works in dark mode with a Chinese system locale", async () => {
  const app = await open("./App.tsx", { width: 1100, height: 760, platform: "macos", colorScheme: "dark", locale: "zh-CN" });
  await app.waitFor("Your dictionary starts here");
  await app.keyboard.press("mod+4");
  await app.waitFor("Study notes");
  expect(app.shows("Save notes")).toBe(true);
  await app.keyboard.press("mod+5");
  await app.waitFor("Dictionary library");
  await app.keyboard.press("mod+1");
  await app.waitFor("Your dictionary starts here");
});


test("library separates file import from server connection", async () => {
  const app = await open("./App.tsx", { width: 390, height: 844 });
  await app.waitFor("Your dictionary starts here");
  await app.press("label:Add dictionary");
  await app.waitFor("Dictionary library");
  expect(app.shows("Download URL or file path")).toBe(true);
  await app.press("label:Connect a server");
  await app.waitFor("Wikit server URL");
  expect(app.shows("Connect source")).toBe(true);
  expect(app.shows("Import dictionary")).toBe(false);
  await app.press("label:Import a file");
  await app.waitFor("Download URL or file path");
  await app.press("label:Preview");
  await app.waitFor("Dictionary preview");
  await app.press("label:Library");
  await app.waitFor("Dictionary library");
});


test("empty collections guide setup and navigation adapts on resize", async () => {
  const app = await open("./App.tsx", { width: 320, height: 640, safeArea: { top: 20, bottom: 24 } });
  await app.waitFor("Your dictionary starts here");
  await app.press("label:Favorites");
  await app.waitFor("Keep a word close");
  await app.press("label:Add dictionary");
  await app.waitFor("Dictionary library");
  await app.press("label:Notes");
  await app.waitFor("Study notes");
  await app.setViewport(1100, 760);
  await app.press("label:Preview");
  await app.waitFor("Dictionary preview");
  await app.setViewport(320, 640);
  await app.press("label:Search");
  await app.waitFor("Your dictionary starts here");
});


test("desktop sidebar expands and search shortcut returns to lookup", async () => {
  const app = await open("./App.tsx", { width: 1100, height: 760, platform: "macos" });
  await app.waitFor("Your dictionary starts here");
  await app.press("label:Expand sidebar");
  expect(app.shows("Wikit")).toBe(true);
  await app.press("label:Notes");
  await app.waitFor("Study notes");
  await app.keyboard.press("mod+f");
  await app.waitFor("Your dictionary starts here");
  await app.press("label:Collapse sidebar");
  expect(app.shows("Expand sidebar")).toBe(true);
});
