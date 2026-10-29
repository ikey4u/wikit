import { expect, open, test } from "nsh/test";

test("native navigation works at phone width", async () => {
  const app = await open("./App.tsx", { width: 390, height: 844 });
  await app.waitFor("A world of words awaits");
  expect(app.texts()).toContain("Wikit");
  await app.press("label:Library");
  await app.waitFor("Dictionary library");
  expect(app.shows("Import dictionary")).toBe(true);
  await app.press("label:Notes");
  await app.waitFor("Study notes");
  await app.press("label:Search");
  await app.press("label:Look up");
  await app.waitFor("Enter a word and select a dictionary first.");
});

test("desktop sidebar opens favorites and preview", async () => {
  const app = await open("./App.tsx", { width: 1100, height: 760 });
  await app.waitFor("A world of words awaits");
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
  await app.waitFor("A world of words awaits");
  await app.keyboard.press("mod+4");
  await app.waitFor("Study notes");
  expect(app.shows("Save notes")).toBe(true);
  await app.keyboard.press("mod+5");
  await app.waitFor("Dictionary library");
  await app.keyboard.press("mod+1");
  await app.waitFor("A world of words awaits");
});
